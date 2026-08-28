use std::fs;
use std::path::Path;
use std::time::Instant;

use anyhow::{Context, Result, anyhow, bail, ensure};
use rusqlite::types::ValueRef;
use rusqlite::{Connection, OpenFlags};
use sha2::{Digest, Sha256};

use crate::manifest::{Manifest, Profile, normalize_feature};
use crate::report::{
    EnvironmentEvidence, Methodology, PragmaEvidence, QueryEvidence, Report, SqliteEvidence,
    Timing, WorkloadEvidence,
};
use crate::{VERSION, fixture_path, sha256_bytes};

pub fn run(
    manifest_path: &Path,
    manifest: &Manifest,
    profile: &Profile,
    allow_profile_mismatch: bool,
) -> Result<Report> {
    let manifest_bytes = fs::read(manifest_path)
        .with_context(|| format!("could not read {}", manifest_path.display()))?;
    let fixture = fixture_path(manifest_path, manifest);
    let fixture_sql = fs::read_to_string(&fixture)
        .with_context(|| format!("fixture {} is not valid UTF-8 SQL", fixture.display()))?;

    let cpu_features = cpu_features();
    let mismatches = profile_mismatches(profile, &cpu_features);
    if !mismatches.is_empty() && !allow_profile_mismatch {
        bail!(
            "CPU does not match profile {}: {} (pass --allow-profile-mismatch to record investigative evidence)",
            profile.id,
            mismatches.join("; ")
        );
    }

    let temp = tempfile::Builder::new()
        .prefix("sqlite-workload-lab-")
        .tempdir()?;
    let database_name = Path::new(&manifest.lab.database)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("workload.db");
    let database_path = temp.path().join(database_name);
    let connection = Connection::open_with_flags(
        &database_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
    )
    .with_context(|| {
        format!(
            "could not create isolated database {}",
            database_path.display()
        )
    })?;

    let mut pragmas = Vec::new();
    for pragma in &manifest.pragmas {
        connection
            .pragma_update(None, &pragma.name, &pragma.value)
            .with_context(|| format!("could not apply PRAGMA {}={}", pragma.name, pragma.value))?;
        let observed = pragma_value(&connection, &pragma.name)?;
        pragmas.push(PragmaEvidence {
            name: pragma.name.clone(),
            requested: pragma.value.clone(),
            observed,
        });
    }
    connection
        .execute_batch(&fixture_sql)
        .with_context(|| format!("could not apply fixture {}", fixture.display()))?;

    let sqlite = SqliteEvidence {
        version: connection.query_row("SELECT sqlite_version()", [], |row| row.get(0))?,
        source_id: connection.query_row("SELECT sqlite_source_id()", [], |row| row.get(0))?,
        compile_options: query_strings(&connection, "PRAGMA compile_options")?,
    };

    let mut queries = Vec::new();
    for query in &manifest.queries {
        let statement = connection
            .prepare(&query.sql)
            .with_context(|| format!("could not prepare query {}", query.name))?;
        ensure!(
            statement.readonly(),
            "query {} mutates the database; measured queries must be read-only",
            query.name
        );
        drop(statement);

        let plan = if query.capture_plan {
            explain_plan(&connection, &query.sql)?
        } else {
            Vec::new()
        };
        for _ in 0..manifest.lab.warmups {
            execute_and_digest(&connection, &query.sql)?;
        }
        let mut samples = Vec::with_capacity(manifest.lab.repetitions as usize);
        let mut expected_result: Option<(usize, String)> = None;
        for _ in 0..manifest.lab.repetitions {
            let started = Instant::now();
            let result = execute_and_digest(&connection, &query.sql)?;
            let elapsed = started.elapsed().as_micros().max(1) as u64;
            if let Some(expected) = &expected_result {
                ensure!(
                    &result == expected,
                    "query {} returned inconsistent results between repetitions",
                    query.name
                );
            } else {
                expected_result = Some(result.clone());
            }
            samples.push(elapsed);
        }
        let (row_count, result_digest) = expected_result
            .ok_or_else(|| anyhow!("query {} produced no measurement", query.name))?;
        queries.push(QueryEvidence {
            name: query.name.clone(),
            sql: query.sql.clone(),
            plan,
            row_count,
            result_digest,
            timing_us: summarize(samples),
        });
    }

    Ok(Report {
        schema_version: 1,
        tool_version: VERSION.to_string(),
        workload: WorkloadEvidence {
            name: manifest.lab.name.clone(),
            manifest_sha256: sha256_bytes(&manifest_bytes),
            fixture_path: manifest.lab.fixture.to_string_lossy().into_owned(),
            fixture_sha256: manifest.lab.fixture_sha256.to_ascii_lowercase(),
        },
        environment: EnvironmentEvidence {
            profile: profile.id.clone(),
            evidence_kind: profile.environment,
            architecture: std::env::consts::ARCH.to_string(),
            operating_system: std::env::consts::OS.to_string(),
            cpu_features,
            profile_match: mismatches.is_empty(),
            mismatches,
        },
        sqlite,
        pragmas,
        queries,
        methodology: Methodology {
            warmups: manifest.lab.warmups,
            repetitions: manifest.lab.repetitions,
            clock: "monotonic std::time::Instant; microsecond samples".into(),
            statistical_claim: "Descriptive timings only. Repeated runs reduce noise but do not establish statistical significance.".into(),
        },
    })
}

fn pragma_value(connection: &Connection, name: &str) -> Result<String> {
    let sql = format!("PRAGMA {name}");
    let mut statement = connection.prepare(&sql)?;
    let columns = statement.column_count();
    if columns == 0 {
        return Ok("(no value)".into());
    }
    let value = statement.query_row([], |row| display_value(row.get_ref(0)?))?;
    Ok(value)
}

fn query_strings(connection: &Connection, sql: &str) -> Result<Vec<String>> {
    let mut statement = connection.prepare(sql)?;
    let values = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(values)
}

fn explain_plan(connection: &Connection, sql: &str) -> Result<Vec<String>> {
    let mut statement = connection.prepare(&format!("EXPLAIN QUERY PLAN {sql}"))?;
    let rows = statement.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let parent: i64 = row.get(1)?;
        let detail: String = row.get(3)?;
        Ok(format!("{id}:{parent} {detail}"))
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn execute_and_digest(connection: &Connection, sql: &str) -> Result<(usize, String)> {
    let mut statement = connection.prepare_cached(sql)?;
    let columns = statement.column_count();
    let mut rows = statement.query([])?;
    let mut count = 0usize;
    let mut digest = Sha256::new();
    while let Some(row) = rows.next()? {
        count += 1;
        for index in 0..columns {
            match row.get_ref(index)? {
                ValueRef::Null => digest.update([0]),
                ValueRef::Integer(value) => {
                    digest.update([1]);
                    digest.update(value.to_le_bytes());
                }
                ValueRef::Real(value) => {
                    digest.update([2]);
                    digest.update(value.to_bits().to_le_bytes());
                }
                ValueRef::Text(value) => {
                    digest.update([3]);
                    digest.update((value.len() as u64).to_le_bytes());
                    digest.update(value);
                }
                ValueRef::Blob(value) => {
                    digest.update([4]);
                    digest.update((value.len() as u64).to_le_bytes());
                    digest.update(value);
                }
            }
        }
    }
    Ok((count, format!("{:x}", digest.finalize())))
}

fn display_value(value: ValueRef<'_>) -> rusqlite::Result<String> {
    Ok(match value {
        ValueRef::Null => "NULL".into(),
        ValueRef::Integer(value) => value.to_string(),
        ValueRef::Real(value) => value.to_string(),
        ValueRef::Text(value) => String::from_utf8_lossy(value).into_owned(),
        ValueRef::Blob(value) => format!("<{} byte blob>", value.len()),
    })
}

fn summarize(mut samples: Vec<u64>) -> Timing {
    samples.sort_unstable();
    let len = samples.len();
    let mean = samples.iter().sum::<u64>() / len as u64;
    let p95_index = ((len as f64 * 0.95).ceil() as usize)
        .saturating_sub(1)
        .min(len - 1);
    Timing {
        min: samples[0],
        median: samples[len / 2],
        mean,
        p95: samples[p95_index],
        max: samples[len - 1],
        samples,
    }
}

fn cpu_features() -> Vec<String> {
    let mut features = Vec::new();
    if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
        for line in cpuinfo.lines() {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            if key.trim().eq_ignore_ascii_case("flags")
                || key.trim().eq_ignore_ascii_case("features")
            {
                features.extend(value.split_whitespace().map(normalize_feature));
                break;
            }
        }
    }
    features.sort();
    features.dedup();
    features
}

fn profile_mismatches(profile: &Profile, available: &[String]) -> Vec<String> {
    let available: std::collections::HashSet<_> = available.iter().map(String::as_str).collect();
    let mut mismatches = Vec::new();
    for feature in &profile.required_cpu_features {
        let normalized = normalize_feature(feature);
        if !available.contains(normalized.as_str()) {
            mismatches.push(format!("required feature {normalized} is absent"));
        }
    }
    for feature in &profile.forbidden_cpu_features {
        let normalized = normalize_feature(feature);
        if available.contains(normalized.as_str()) {
            mismatches.push(format!("forbidden feature {normalized} is present"));
        }
    }
    mismatches
}

#[cfg(test)]
mod tests {
    use super::summarize;

    #[test]
    fn timing_summary_is_stable() {
        let timing = summarize(vec![10, 1, 5, 20, 8]);
        assert_eq!(timing.samples, vec![1, 5, 8, 10, 20]);
        assert_eq!(timing.median, 8);
        assert_eq!(timing.p95, 20);
        assert_eq!(timing.mean, 8);
    }
}
