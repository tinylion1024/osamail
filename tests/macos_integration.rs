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
