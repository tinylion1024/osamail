#![cfg(target_os = "macos")]

use std::process::{Command, Output};

fn run_read_only(args: &[&str]) -> Option<Output> {
    if std::env::var("OSAMAIL_INTEGRATION").as_deref() != Ok("1") {
        eprintln!("skipping: set OSAMAIL_INTEGRATION=1 to run local Apple Mail checks");
        return None;
    }

    Some(
        Command::new(assert_cmd::cargo::cargo_bin!("osamail"))
            .args(args)
            .output()
            .expect("OsaMail integration command should start"),
    )
}

fn assert_success(output: Output) {
    assert!(
        output.status.success(),
        "command failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "requires macOS Automation permission and configured Apple Mail accounts"]
fn doctor() {
    if let Some(output) = run_read_only(&["doctor", "--json"]) {
        assert_success(output);
    }
}

#[test]
#[ignore = "requires macOS Automation permission and configured Apple Mail accounts"]
fn accounts() {
    if let Some(output) = run_read_only(&["accounts", "--json"]) {
        assert_success(output);
    }
}

#[test]
#[ignore = "requires macOS Automation permission and configured Apple Mail accounts"]
fn mailboxes() {
    if let Some(output) = run_read_only(&["mailboxes", "--json"]) {
        assert_success(output);
    }
}

#[test]
#[ignore = "requires macOS Automation permission and configured Apple Mail accounts"]
fn recent_limit_one() {
    if let Some(output) = run_read_only(&["recent", "--limit", "1", "--json"]) {
        assert_success(output);
    }
}

#[test]
#[ignore = "requires macOS Automation permission and configured Apple Mail accounts"]
fn unread_count() {
    if let Some(output) = run_read_only(&["unread", "--count", "--json"]) {
        assert_success(output);
    }
}

#[test]
#[ignore = "requires an authorized local Apple Mail account"]
fn unread_titles() {
    if let Some(output) = run_read_only(&["unread", "--limit", "1", "--titles"]) {
        assert_success(output);
    }
}

#[test]
#[ignore = "requires an authorized local Apple Mail account with at least one message"]
fn mark_read_dry_run() {
    let Some(recent) = run_read_only(&["recent", "--limit", "1", "--json"]) else {
        return;
    };
    assert!(recent.status.success(), "recent command should succeed");
    let recent: serde_json::Value =
        serde_json::from_slice(&recent.stdout).expect("recent output should be JSON");
    let reference = recent["data"]["messages"][0]["ref"]
        .as_str()
        .expect("recent output should contain one message reference");

    let output = run_read_only(&["mark", "read", reference, "--dry-run", "--json"])
        .expect("integration gate should remain enabled");
    assert!(output.status.success(), "mark dry-run should succeed");
    let result: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("mark output should be JSON");
    assert_eq!(result["data"]["action"], "read");
    assert!(
        matches!(
            result["data"]["outcome"].as_str(),
            Some("would_change" | "already_set")
        ),
        "dry-run outcome should confirm no mutation"
    );
}
