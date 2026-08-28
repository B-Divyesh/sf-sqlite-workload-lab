use std::fmt::Write;

use serde::{Deserialize, Serialize};

use crate::manifest::Environment;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub schema_version: u32,
    pub tool_version: String,
    pub workload: WorkloadEvidence,
    pub environment: EnvironmentEvidence,
    pub sqlite: SqliteEvidence,
    pub pragmas: Vec<PragmaEvidence>,
    pub queries: Vec<QueryEvidence>,
    pub methodology: Methodology,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadEvidence {
    pub name: String,
    pub manifest_sha256: String,
    pub fixture_path: String,
    pub fixture_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentEvidence {
    pub profile: String,
    pub evidence_kind: Environment,
    pub architecture: String,
    pub operating_system: String,
    pub cpu_features: Vec<String>,
    pub profile_match: bool,
    pub mismatches: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqliteEvidence {
    pub version: String,
    pub source_id: String,
    pub compile_options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PragmaEvidence {
    pub name: String,
    pub requested: String,
    pub observed: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryEvidence {
    pub name: String,
    pub sql: String,
    pub plan: Vec<String>,
    pub row_count: usize,
    pub result_digest: String,
    pub timing_us: Timing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timing {
    pub samples: Vec<u64>,
    pub min: u64,
    pub median: u64,
    pub mean: u64,
    pub p95: u64,
    pub max: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Methodology {
    pub warmups: u32,
    pub repetitions: u32,
    pub clock: String,
    pub statistical_claim: String,
}

impl Report {
    pub fn markdown(&self) -> String {
        let mut out = String::new();
        writeln!(out, "# SQLite workload report: {}\n", self.workload.name).unwrap();
        writeln!(out, "> **{} evidence.** Timings from this report must not be represented as hardware measurements from another environment.\n", title_case(&self.environment.evidence_kind.to_string())).unwrap();
        writeln!(out, "## Reproduction record\n").unwrap();
        writeln!(out, "| Field | Captured value |").unwrap();
        writeln!(out, "| --- | --- |").unwrap();
        writeln!(
            out,
            "| Profile | `{}` ({}) |",
            escape(&self.environment.profile),
            self.environment.evidence_kind
        )
        .unwrap();
        writeln!(
            out,
            "| Profile check | {} |",
            if self.environment.profile_match {
                "Pass"
            } else {
                "Mismatch—see below"
            }
        )
        .unwrap();
        writeln!(
            out,
            "| Platform | `{}` / `{}` |",
            self.environment.architecture, self.environment.operating_system
        )
        .unwrap();
        writeln!(out, "| SQLite | `{}` |", self.sqlite.version).unwrap();
        writeln!(
            out,
            "| Fixture SHA-256 | `{}` |",
            self.workload.fixture_sha256
        )
        .unwrap();
        writeln!(
            out,
            "| Manifest SHA-256 | `{}` |",
            self.workload.manifest_sha256
        )
        .unwrap();
        writeln!(
            out,
            "| Method | {} warmups, {} measured repetitions, monotonic clock |\n",
            self.methodology.warmups, self.methodology.repetitions
        )
        .unwrap();

        if !self.environment.mismatches.is_empty() {
            writeln!(out, "### Profile mismatches\n").unwrap();
            for mismatch in &self.environment.mismatches {
                writeln!(out, "- {}", escape(mismatch)).unwrap();
            }
            writeln!(out).unwrap();
        }

        writeln!(out, "## Query timings\n").unwrap();
        writeln!(out, "| Query | Median | p95 | Min–max | Rows | Digest |").unwrap();
        writeln!(out, "| --- | ---: | ---: | ---: | ---: | --- |").unwrap();
        for query in &self.queries {
            writeln!(
                out,
                "| {} | {} µs | {} µs | {}–{} µs | {} | `{}` |",
                escape(&query.name),
                query.timing_us.median,
                query.timing_us.p95,
                query.timing_us.min,
                query.timing_us.max,
                query.row_count,
                &query.result_digest[..12]
            )
            .unwrap();
        }
        writeln!(out, "\n> {}\n", self.methodology.statistical_claim).unwrap();

        writeln!(out, "## PRAGMAs\n").unwrap();
        writeln!(out, "| Name | Requested | Observed |").unwrap();
        writeln!(out, "| --- | --- | --- |").unwrap();
        for pragma in &self.pragmas {
            writeln!(
                out,
                "| `{}` | `{}` | `{}` |",
                pragma.name,
                escape(&pragma.requested),
                escape(&pragma.observed)
            )
            .unwrap();
        }

        writeln!(out, "\n## Query plans\n").unwrap();
        for query in &self.queries {
            writeln!(out, "### {}\n", escape(&query.name)).unwrap();
            writeln!(out, "```sql\n{}\n```\n", query.sql.trim()).unwrap();
            if query.plan.is_empty() {
                writeln!(out, "Plan capture disabled.\n").unwrap();
            } else {
                writeln!(out, "```text").unwrap();
                for line in &query.plan {
                    writeln!(out, "{line}").unwrap();
                }
                writeln!(out, "```\n").unwrap();
            }
        }

        writeln!(out, "## SQLite build\n").unwrap();
        writeln!(out, "Source ID: `{}`\n", escape(&self.sqlite.source_id)).unwrap();
        writeln!(
            out,
            "Compile options: {}\n",
            self.sqlite
                .compile_options
                .iter()
                .map(|option| format!("`{}`", escape(option)))
                .collect::<Vec<_>>()
                .join(", ")
        )
        .unwrap();
        writeln!(
            out,
            "CPU features: {}\n",
            self.environment
                .cpu_features
                .iter()
                .map(|feature| format!("`{}`", escape(feature)))
                .collect::<Vec<_>>()
                .join(", ")
        )
        .unwrap();
        writeln!(
            out,
            "---\nGenerated by SQLite Workload Lab {}. No telemetry was sent.",
            self.tool_version
        )
        .unwrap();
        out
    }
}

fn escape(value: &str) -> String {
    value.replace('|', "\\|").replace('`', "\\`")
}

fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => value.to_string(),
    }
}
