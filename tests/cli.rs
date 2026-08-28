use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

fn lab() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("sqlite-workload-lab"))
}

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
    assert_eq!(report["queries"][0]["name"], "release-evidence");
    assert_eq!(report["methodology"]["repetitions"], 10);
    assert!(reports.join("host.md").exists());
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

#[test]
fn json_error_is_machine_readable() {
    lab()
        .args(["--json", "check", "missing.toml"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("\"ok\":false"));
}
