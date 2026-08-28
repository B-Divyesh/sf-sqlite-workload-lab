use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

fn lab() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("sqlite-workload-lab"))
}

fn assert_json_parse_error(arguments: &[&str], expected: &str) {
    let assertion = lab()
        .args(arguments)
        .assert()
        .code(1)
        .stdout(predicate::str::is_empty());
    let output = assertion.get_output();
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stderr).unwrap_or_else(|error| {
            panic!(
                "stderr was not one JSON document: {error}; stderr={}",
                String::from_utf8_lossy(&output.stderr)
            )
        });
    assert_eq!(payload["ok"], false);
    assert!(
        payload["error"].as_str().unwrap().contains(expected),
        "unexpected parser error: {}",
        payload["error"]
    );
}

// @claim:evidence-report
#[test]
fn documented_init_check_and_run_flow() {
    let temp = tempfile::tempdir().unwrap();
    let manifest = temp.path().join("lab.toml");
    let reports = temp.path().join("reports");

    lab()
        .arg("init")
        .arg(&manifest)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Created a pinned starter workload",
        ));
    lab()
        .arg("check")
        .arg(&manifest)
        .assert()
        .success()
        .stdout(predicate::str::contains("3 CPU profiles"));
    lab()
        .args(["--json", "run"])
        .arg(&manifest)
        .args(["--profile", "host", "--out"])
        .arg(&reports)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ok\":true"));

    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(reports.join("host.json")).unwrap()).unwrap();
    assert_eq!(report["environment"]["profile"], "host");
    assert_eq!(report["environment"]["evidence_kind"], "hardware");
    assert!(
        report["environment"]["cpu_features"]
            .as_array()
            .unwrap()
            .len()
            > 10
    );
    assert!(
        report["sqlite"]["compile_options"]
            .as_array()
            .unwrap()
            .len()
            > 10
    );
    assert!(!report["sqlite"]["source_id"].as_str().unwrap().is_empty());
    assert_eq!(report["pragmas"].as_array().unwrap().len(), 2);
    assert_eq!(report["queries"][0]["name"], "release-evidence");
    assert!(!report["queries"][0]["plan"].as_array().unwrap().is_empty());
    assert_eq!(
        report["workload"]["fixture_sha256"].as_str().unwrap().len(),
        64
    );
    assert_eq!(report["methodology"]["repetitions"], 10);
    assert!(reports.join("host.md").exists());
}

// @claim:demo-sample
#[test]
fn demo_runs_bundled_sample_in_a_new_directory() {
    let temp = tempfile::tempdir().unwrap();
    let demo = temp.path().join("sample-run");

    lab()
        .args(["--json", "demo", "--out"])
        .arg(&demo)
        .assert()
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(
            predicate::str::contains("\"action\":\"demo\"")
                .and(predicate::str::contains("\"ok\":true")),
        );

    assert!(demo.join("lab.toml").is_file());
    assert!(demo.join("fixtures/sample.sql").is_file());
    assert!(demo.join("reports/host.json").is_file());
    assert!(demo.join("reports/host.md").is_file());
}

#[test]
fn init_refuses_to_overwrite() {
    let temp = tempfile::tempdir().unwrap();
    let manifest = temp.path().join("lab.toml");
    lab().arg("init").arg(&manifest).assert().success();
    lab()
        .arg("init")
        .arg(&manifest)
        .assert()
        .failure()
        .stderr(predicate::str::contains("refusing to overwrite"));
}

// @claim:regression-exit
#[test]
fn comparison_uses_exit_code_two_for_regression() {
    let temp = tempfile::tempdir().unwrap();
    let manifest = temp.path().join("lab.toml");
    let reports = temp.path().join("reports");
    lab().arg("init").arg(&manifest).assert().success();
    lab()
        .arg("run")
        .arg(&manifest)
        .args(["--profile", "host", "--out"])
        .arg(&reports)
        .assert()
        .success();

    let baseline = reports.join("host.json");
    let candidate = temp.path().join("candidate.json");
    let mut value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&baseline).unwrap()).unwrap();
    let median = value["queries"][0]["timing_us"]["median"].as_u64().unwrap();
    value["queries"][0]["timing_us"]["median"] = serde_json::json!(median * 2 + 1);
    fs::write(&candidate, serde_json::to_string_pretty(&value).unwrap()).unwrap();

    lab()
        .arg("compare")
        .arg(&baseline)
        .arg(&candidate)
        .args(["--threshold", "15"])
        .assert()
        .code(2)
        .stdout(predicate::str::contains("Gate: FAIL"));
}

// @claim:behavior-change-gate
#[test]
fn comparison_rejects_a_changed_query_result() {
    let temp = tempfile::tempdir().unwrap();
    let manifest = temp.path().join("lab.toml");
    let reports = temp.path().join("reports");
    lab().arg("init").arg(&manifest).assert().success();
    lab()
        .arg("run")
        .arg(&manifest)
        .args(["--profile", "host", "--out"])
        .arg(&reports)
        .assert()
        .success();

    let baseline = reports.join("host.json");
    let candidate = temp.path().join("changed-result.json");
    let mut value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&baseline).unwrap()).unwrap();
    value["queries"][0]["result_digest"] = serde_json::json!("changed");
    fs::write(&candidate, serde_json::to_string_pretty(&value).unwrap()).unwrap();

    lab()
        .arg("compare")
        .arg(&baseline)
        .arg(&candidate)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("result digest changed"));
}

#[test]
fn json_error_is_machine_readable() {
    lab()
        .args(["--json", "check", "missing.toml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("\"ok\":false"));
}

#[test]
fn json_unknown_subcommand_is_invalid_input_not_a_regression() {
    assert_json_parse_error(
        &["--json", "nonsense"],
        "unrecognized subcommand 'nonsense'",
    );
}

#[test]
fn json_missing_required_run_profile_is_invalid_input_not_a_regression() {
    assert_json_parse_error(&["--json", "run", "lab.toml"], "--profile <PROFILE>");
}

#[test]
fn json_invalid_threshold_is_invalid_input_not_a_regression() {
    assert_json_parse_error(
        &[
            "--json",
            "compare",
            "baseline.json",
            "candidate.json",
            "--threshold",
            "nope",
        ],
        "invalid value 'nope'",
    );
}

// @claim:read-only-preflight
#[test]
fn check_rejects_a_mutating_measured_query() {
    let temp = tempfile::tempdir().unwrap();
    let manifest = temp.path().join("mutating.toml");
    lab().arg("init").arg(&manifest).assert().success();

    let source = fs::read_to_string(&manifest).unwrap();
    let source = source.replace(
        "sql = \"SELECT rowid, title FROM docs WHERE docs MATCH 'sqlite OR workload' ORDER BY rank LIMIT 20\"",
        "sql = \"DELETE FROM docs\"",
    );
    fs::write(&manifest, source).unwrap();

    lab()
        .args(["--json", "check"])
        .arg(&manifest)
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("\"ok\":false").and(predicate::str::contains(
                "query release-evidence mutates the database; measured queries must be read-only",
            )),
        );
}
