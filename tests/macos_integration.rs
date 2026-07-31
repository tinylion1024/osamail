#![cfg(target_os = "macos")]

use std::io::Write;
use std::process::{Command, Output, Stdio};

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

fn run_read_only_with_stdin(args: &[&str], input: &str) -> Option<Output> {
    if std::env::var("OSAMAIL_INTEGRATION").as_deref() != Ok("1") {
        eprintln!("skipping: set OSAMAIL_INTEGRATION=1 to run local Apple Mail checks");
        return None;
    }

    let mut child = Command::new(assert_cmd::cargo::cargo_bin!("osamail"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("OsaMail integration command should start");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("message reference should be written to stdin");
    Some(
        child
            .wait_with_output()
            .expect("OsaMail integration command should finish"),
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
fn recent_local_date_range() {
    if let Some(output) = run_read_only(&[
        "recent",
        "--since",
        "1970-01-01",
        "--before",
        "2100-01-01",
        "--limit",
        "1",
        "--titles",
    ]) {
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

    let stdin = format!("{reference}\n");
    let output =
        run_read_only_with_stdin(&["mark", "read", "--stdin", "--dry-run", "--json"], &stdin)
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

#[test]
#[ignore = "requires an authorized local Apple Mail account with at least one message"]
fn move_dry_run() {
    let Some(recent) = run_read_only(&["recent", "--limit", "1", "--json"]) else {
        return;
    };
    assert!(recent.status.success(), "recent command should succeed");
    let recent: serde_json::Value =
        serde_json::from_slice(&recent.stdout).expect("recent output should be JSON");
    let message = &recent["data"]["messages"][0];
    let message_reference = message["ref"]
        .as_str()
        .expect("recent output should contain one message reference");
    let account = message["account"]
        .as_str()
        .expect("recent output should contain an account");

    let mailboxes = run_read_only(&["mailboxes", "--account", account, "--json"])
        .expect("integration gate should remain enabled");
    assert!(
        mailboxes.status.success(),
        "mailboxes command should succeed"
    );
    let mailboxes: serde_json::Value =
        serde_json::from_slice(&mailboxes.stdout).expect("mailboxes output should be JSON");
    let destination_reference = mailboxes["data"]["mailboxes"][0]["ref"]
        .as_str()
        .expect("account should expose at least one mailbox");

    let output = run_read_only(&[
        "move",
        "--to",
        destination_reference,
        message_reference,
        "--dry-run",
        "--json",
    ])
    .expect("integration gate should remain enabled");
    assert!(output.status.success(), "move dry-run should succeed");
    let result: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("move output should be JSON");
    assert_eq!(result["data"]["dry_run"], true);
    assert_eq!(result["data"]["failed"], 0);
}
