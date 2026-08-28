use std::collections::BTreeMap;
use std::fmt::Write;

use anyhow::{Result, bail, ensure};
use serde::{Deserialize, Serialize};

use crate::Report;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comparison {
    pub schema_version: u32,
    pub profile: String,
    pub threshold_percent: f64,
    pub context_match: bool,
    pub context_warnings: Vec<String>,
    pub regressions: usize,
    pub queries: Vec<QueryComparison>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryComparison {
    pub name: String,
    pub baseline_median_us: u64,
    pub candidate_median_us: u64,
    pub change_percent: f64,
    pub status: ComparisonStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ComparisonStatus {
    Improved,
    Stable,
    Regression,
}

impl std::fmt::Display for ComparisonStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Improved => "improved",
            Self::Stable => "stable",
            Self::Regression => "regression",
        })
    }
}

pub fn compare_reports(
    baseline: &Report,
    candidate: &Report,
    threshold: f64,
    allow_context_mismatch: bool,
) -> Result<Comparison> {
    ensure!(
        threshold >= 0.0 && threshold.is_finite(),
        "threshold must be a finite non-negative percentage"
    );
    let mut warnings = Vec::new();
    compare_context(
        &mut warnings,
        "profile",
        &baseline.environment.profile,
        &candidate.environment.profile,
    );
    compare_context(
        &mut warnings,
        "evidence kind",
        &baseline.environment.evidence_kind.to_string(),
        &candidate.environment.evidence_kind.to_string(),
    );
    compare_context(
        &mut warnings,
        "SQLite version",
        &baseline.sqlite.version,
        &candidate.sqlite.version,
    );
    compare_context(
        &mut warnings,
        "fixture SHA-256",
        &baseline.workload.fixture_sha256,
        &candidate.workload.fixture_sha256,
    );
    compare_context(
        &mut warnings,
        "manifest SHA-256",
        &baseline.workload.manifest_sha256,
        &candidate.workload.manifest_sha256,
    );
    if !warnings.is_empty() && !allow_context_mismatch {
        bail!(
            "reports are not comparable: {} (pass --allow-context-mismatch only for exploratory analysis)",
            warnings.join("; ")
        );
    }

    let baseline_queries: BTreeMap<_, _> = baseline
        .queries
        .iter()
        .map(|query| (query.name.as_str(), query))
        .collect();
    let candidate_queries: BTreeMap<_, _> = candidate
        .queries
        .iter()
        .map(|query| (query.name.as_str(), query))
        .collect();
    ensure!(
        baseline_queries.keys().eq(candidate_queries.keys()),
        "reports contain different query sets"
    );

    let mut regressions = 0;
    let mut queries = Vec::new();
    for (name, before) in baseline_queries {
        let after = candidate_queries[name];
        ensure!(
            before.result_digest == after.result_digest,
            "query {name} result digest changed; performance comparison would hide a behavior change"
        );
        let change = if before.timing_us.median == 0 {
            0.0
        } else {
            (after.timing_us.median as f64 - before.timing_us.median as f64)
                / before.timing_us.median as f64
                * 100.0
        };
        let status = if change > threshold {
            regressions += 1;
            ComparisonStatus::Regression
        } else if change < -threshold {
            ComparisonStatus::Improved
        } else {
            ComparisonStatus::Stable
        };
        queries.push(QueryComparison {
            name: name.to_string(),
            baseline_median_us: before.timing_us.median,
            candidate_median_us: after.timing_us.median,
            change_percent: round_two(change),
            status,
        });
    }

    Ok(Comparison {
        schema_version: 1,
        profile: candidate.environment.profile.clone(),
        threshold_percent: threshold,
        context_match: warnings.is_empty(),
        context_warnings: warnings,
        regressions,
        queries,
    })
}

impl Comparison {
    pub fn markdown(&self) -> String {
        let mut out = String::new();
        writeln!(out, "# SQLite workload comparison: {}\n", self.profile).unwrap();
        writeln!(
            out,
            "**Gate: {}** — {} regression(s) above {:.2}%\n",
            if self.regressions == 0 {
                "PASS"
            } else {
                "FAIL"
            },
            self.regressions,
            self.threshold_percent
        )
        .unwrap();
        if !self.context_warnings.is_empty() {
            writeln!(out, "> Exploratory comparison: report context differs.\n").unwrap();
            for warning in &self.context_warnings {
                writeln!(out, "- {warning}").unwrap();
            }
            writeln!(out).unwrap();
        }
        writeln!(
            out,
            "| Query | Baseline median | Candidate median | Change | Status |"
        )
        .unwrap();
        writeln!(out, "| --- | ---: | ---: | ---: | --- |").unwrap();
        for query in &self.queries {
            writeln!(
                out,
                "| {} | {} µs | {} µs | {:+.2}% | {} |",
                query.name.replace('|', "\\|"),
                query.baseline_median_us,
                query.candidate_median_us,
                query.change_percent,
                query.status
            )
            .unwrap();
        }
        writeln!(out, "\nDescriptive median comparison only; this gate does not assert statistical significance.\n").unwrap();
        out
    }
}

fn compare_context(warnings: &mut Vec<String>, label: &str, before: &str, after: &str) {
    if before != after {
        warnings.push(format!("{label} differs ({before:?} vs {after:?})"));
    }
}

fn round_two(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}
