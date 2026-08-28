use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use sqlite_workload_lab::{
    Report, compare_reports, ensure_parent, load_manifest, refuse_overwrite, sha256_bytes,
};

#[derive(Debug, Parser)]
#[command(
    name = "sqlite-workload-lab",
    version,
    about = "Reproducible SQLite workload and CPU-compatibility evidence",
    long_about = "Run pinned, declarative SQLite workloads and capture the context needed to review a performance or binary-compatibility claim. Reports clearly distinguish hardware, virtualized, container, and emulator evidence. No network or telemetry."
)]
struct Cli {
    /// Print command results as JSON for scripts.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create a valid starter manifest and pinned FTS5 fixture.
    Init {
        #[arg(default_value = "lab.toml")]
        manifest: PathBuf,
    },
    /// Validate the manifest, fixture hash, SQLite pin, profiles, and SQL shape.
    Check {
        #[arg(default_value = "lab.toml")]
        manifest: PathBuf,
    },
    /// Execute one declared CPU profile and write its evidence report.
    Run {
        #[arg(default_value = "lab.toml")]
        manifest: PathBuf,
        /// Profile ID declared in the manifest.
        #[arg(long)]
        profile: String,
        /// Directory for <profile>.json and <profile>.md.
        #[arg(long, default_value = "reports")]
        out: PathBuf,
        /// Report artifact(s) to write.
        #[arg(long, value_enum, default_value_t = Format::Both)]
        format: Format,
        /// Record investigative evidence even when CPU features do not match.
        #[arg(long)]
        allow_profile_mismatch: bool,
    },
    /// Run every declared native profile in this environment.
    Matrix {
        #[arg(default_value = "lab.toml")]
        manifest: PathBuf,
        /// Directory for per-profile JSON and Markdown reports.
        #[arg(long, default_value = "reports")]
        out: PathBuf,
        /// Record profiles even when CPU features do not match.
        #[arg(long)]
        allow_profile_mismatch: bool,
    },
    /// Compare a candidate report with a baseline and enforce a regression gate.
    Compare {
        baseline: PathBuf,
        candidate: PathBuf,
        /// Regression threshold as a percentage.
        #[arg(long, default_value_t = 15.0)]
        threshold: f64,
        /// Write a Markdown comparison report.
        #[arg(long)]
        markdown: Option<PathBuf>,
        /// Write the structured comparison as JSON.
        #[arg(long)]
        json_out: Option<PathBuf>,
        /// Permit exploratory comparison when profile or pins differ.
        #[arg(long)]
        allow_context_mismatch: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Format {
    Json,
    Markdown,
    Both,
}

#[derive(Serialize)]
struct Status<'a> {
    ok: bool,
    action: &'a str,
    message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    files: Vec<String>,
}

enum Outcome {
    Success,
    Regression,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match execute(&cli) {
        Ok(Outcome::Success) => ExitCode::SUCCESS,
        Ok(Outcome::Regression) => ExitCode::from(2),
        Err(error) => {
            if cli.json {
                let payload = serde_json::json!({ "ok": false, "error": format!("{error:#}") });
                eprintln!("{}", serde_json::to_string(&payload).unwrap());
            } else {
                eprintln!("error: {error:#}");
            }
            ExitCode::FAILURE
        }
    }
}

fn execute(cli: &Cli) -> Result<Outcome> {
    match &cli.command {
        Command::Init { manifest } => {
            let files = init(manifest)?;
            emit_status(
                cli.json,
                "init",
                "Created a pinned starter workload.",
                files,
            )?;
            Ok(Outcome::Success)
        }
        Command::Check { manifest } => {
            let parsed = load_manifest(manifest)?;
            emit_status(
                cli.json,
                "check",
                &format!(
                    "Valid: {} queries across {} CPU profiles; SQLite {}.",
                    parsed.queries.len(),
                    parsed.profiles.len(),
                    parsed.lab.sqlite_version
                ),
                Vec::new(),
            )?;
            Ok(Outcome::Success)
        }
        Command::Run {
            manifest,
            profile,
            out,
            format,
            allow_profile_mismatch,
        } => {
            let parsed = load_manifest(manifest)?;
            let profile = parsed.profile(profile)?;
            let report = sqlite_workload_lab::runner::run(
                manifest,
                &parsed,
                profile,
                *allow_profile_mismatch,
            )?;
            let files = write_report(out, &report, *format)?;
            emit_status(
                cli.json,
                "run",
                &format!(
                    "Captured {} query result(s) for {} {} evidence.",
                    report.queries.len(),
                    report.environment.profile,
                    report.environment.evidence_kind
                ),
                files,
            )?;
            Ok(Outcome::Success)
        }
        Command::Matrix {
            manifest,
            out,
            allow_profile_mismatch,
        } => {
            let parsed = load_manifest(manifest)?;
            let mut files = Vec::new();
            for profile in &parsed.profiles {
                let report = sqlite_workload_lab::runner::run(
                    manifest,
                    &parsed,
                    profile,
                    *allow_profile_mismatch,
                )
                .with_context(|| format!("profile {} failed", profile.id))?;
                files.extend(write_report(out, &report, Format::Both)?);
            }
            emit_status(
                cli.json,
                "matrix",
                &format!("Captured {} declared CPU profiles.", parsed.profiles.len()),
                files,
            )?;
            Ok(Outcome::Success)
        }
        Command::Compare {
            baseline,
            candidate,
            threshold,
            markdown,
            json_out,
            allow_context_mismatch,
        } => {
            let baseline_report = read_report(baseline)?;
            let candidate_report = read_report(candidate)?;
            let comparison = compare_reports(
                &baseline_report,
                &candidate_report,
                *threshold,
                *allow_context_mismatch,
            )?;
            let mut files = Vec::new();
            if let Some(path) = markdown {
                ensure_parent(path)?;
                fs::write(path, comparison.markdown())
                    .with_context(|| format!("could not write {}", path.display()))?;
                files.push(path.display().to_string());
            }
            if let Some(path) = json_out {
                ensure_parent(path)?;
                fs::write(path, serde_json::to_string_pretty(&comparison)? + "\n")
                    .with_context(|| format!("could not write {}", path.display()))?;
                files.push(path.display().to_string());
            }
            if cli.json {
                println!("{}", serde_json::to_string(&comparison)?);
            } else {
                println!("{}", comparison.markdown());
                for file in &files {
                    println!("Wrote {file}");
                }
            }
            if comparison.regressions > 0 {
                Ok(Outcome::Regression)
            } else {
                Ok(Outcome::Success)
            }
        }
    }
}

fn init(manifest_path: &Path) -> Result<Vec<String>> {
    refuse_overwrite(manifest_path)?;
    let base = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let fixture_path = base.join("fixtures/sample.sql");
    refuse_overwrite(&fixture_path)?;
    let fixture = "CREATE VIRTUAL TABLE docs USING fts5(title, body);\nINSERT INTO docs(title, body) VALUES\n  ('Measure first', 'A pinned SQLite workload makes a release claim reviewable.'),\n  ('Portable builds', 'Test SQLite extensions without AVX and AVX2 before publishing.'),\n  ('Query plans', 'Capture EXPLAIN QUERY PLAN beside repeated timing samples.'),\n  ('Local evidence', 'The report stays on your machine and belongs in version control.');\n";
    let hash = sha256_bytes(fixture.as_bytes());
    let sqlite_version = rusqlite::version();
    let manifest = format!(
        r#"schema_version = 1

[lab]
name = "sqlite-release-check"
database = "workload.db"
fixture = "fixtures/sample.sql"
fixture_sha256 = "{hash}"
sqlite_version = "{sqlite_version}"
warmups = 2
repetitions = 10

[[profiles]]
id = "host"
environment = "hardware"
runner = "native"
notes = "Run on the release host."

[[profiles]]
id = "container"
environment = "container"
runner = "native"
notes = "Run this selection from the pinned container image."

[[profiles]]
id = "emulated-x86-v2"
environment = "emulator"
runner = "native"
required_cpu_features = ["sse4_2", "popcnt"]
forbidden_cpu_features = ["avx", "avx2"]
notes = "Run under QEMU with an x86-64-v2 CPU model."

[[pragmas]]
name = "journal_mode"
value = "WAL"

[[pragmas]]
name = "cache_size"
value = "-8000"

[[queries]]
name = "release-evidence"
sql = "SELECT rowid, title FROM docs WHERE docs MATCH 'sqlite OR workload' ORDER BY rank LIMIT 20"
capture_plan = true
"#
    );
    ensure_parent(&fixture_path)?;
    fs::write(&fixture_path, fixture)
        .with_context(|| format!("could not write {}", fixture_path.display()))?;
    if let Err(error) = fs::write(manifest_path, manifest) {
        let _ = fs::remove_file(&fixture_path);
        return Err(error).with_context(|| format!("could not write {}", manifest_path.display()));
    }
    Ok(vec![
        manifest_path.display().to_string(),
        fixture_path.display().to_string(),
    ])
}

fn write_report(directory: &Path, report: &Report, format: Format) -> Result<Vec<String>> {
    fs::create_dir_all(directory)
        .with_context(|| format!("could not create {}", directory.display()))?;
    let safe_profile = report
        .environment
        .profile
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    if safe_profile.is_empty() {
        bail!("profile name cannot be converted to a safe output filename");
    }
    let mut files = Vec::new();
    if matches!(format, Format::Json | Format::Both) {
        let path = directory.join(format!("{safe_profile}.json"));
        fs::write(&path, serde_json::to_string_pretty(report)? + "\n")
            .with_context(|| format!("could not write {}", path.display()))?;
        files.push(path.display().to_string());
    }
    if matches!(format, Format::Markdown | Format::Both) {
        let path = directory.join(format!("{safe_profile}.md"));
        fs::write(&path, report.markdown())
            .with_context(|| format!("could not write {}", path.display()))?;
        files.push(path.display().to_string());
    }
    Ok(files)
}

fn read_report(path: &Path) -> Result<Report> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("could not read report {}", path.display()))?;
    serde_json::from_str(&source)
        .with_context(|| format!("could not parse report {}", path.display()))
}

fn emit_status(json: bool, action: &'static str, message: &str, files: Vec<String>) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string(&Status {
                ok: true,
                action,
                message: message.into(),
                files
            })?
        );
    } else {
        println!("{message}");
        for file in files {
            println!("Wrote {file}");
        }
    }
    Ok(())
}
