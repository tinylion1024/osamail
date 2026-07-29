use assert_cmd::Command;
use predicates::prelude::*;

fn osamail() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("osamail"))
}

#[test]
fn help_is_available_without_mail() {
    osamail()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn version_is_available_without_mail() {
    osamail()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"^osamail \d+\.\d+\.\d+\n$").expect("valid regex"));
}

#[test]
fn zero_limit_is_rejected_before_mail_access() {
    osamail()
        .args(["recent", "--limit", "0"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("limit must be from 1 to 200"));
}

#[test]
fn json_argument_errors_are_exactly_one_json_value() {
    let output = osamail()
        .args(["recent", "--limit", "0", "--json"])
        .output()
        .expect("run osamail");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let error: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr is exactly one JSON value");
    assert_eq!(error["ok"], false);
    assert_eq!(error["error"]["code"], "INVALID_ARGUMENTS");
}

#[test]
fn send_requires_a_to_recipient() {
    osamail()
        .args(["send", "--subject", "no recipient", "--dry-run"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("--to"));
}

#[test]
fn invalid_reference_is_rejected_before_mail_access() {
    osamail()
        .args(["show", "invalid-reference"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("Invalid message reference"));
}

#[test]
fn mark_rejects_an_invalid_reference_before_mail_access() {
    osamail()
        .args(["mark", "read", "invalid-reference", "--dry-run"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("Invalid message reference"));
}

#[test]
fn dry_run_never_sends_mail() {
    osamail()
        .args([
            "send",
            "--to",
            "test@example.invalid",
            "--subject",
            "integration dry run",
            "--body",
            "not sent",
            "--dry-run",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""dry_run": true"#))
        .stdout(predicate::str::contains(r#""sent": false"#));
}
