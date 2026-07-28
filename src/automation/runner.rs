use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use serde_json::Value;
use tempfile::Builder;

use crate::error::OsaMailError;

const OSASCRIPT_PATH: &str = "/usr/bin/osascript";

pub trait AutomationRunner {
    fn execute(
        &self,
        script: Script,
        request: &Value,
        timeout: Duration,
    ) -> Result<Value, OsaMailError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Script {
    Doctor,
    Accounts,
    ListMailboxes,
    ListMessages,
    ShowMessage,
    OpenMessage,
    MarkMessage,
    SendMessage,
}

impl Script {
    pub const fn source(self) -> &'static str {
        match self {
            Self::Doctor => include_str!("scripts/doctor.js"),
            Self::Accounts => include_str!("scripts/accounts.js"),
            Self::ListMailboxes => include_str!("scripts/list_mailboxes.js"),
            Self::ListMessages => include_str!("scripts/list_messages.js"),
            Self::ShowMessage => include_str!("scripts/show_message.js"),
            Self::OpenMessage => include_str!("scripts/open_message.js"),
            Self::MarkMessage => include_str!("scripts/mark_message.js"),
            Self::SendMessage => include_str!("scripts/send_message.js"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OsascriptRunner {
    program: PathBuf,
    temp_directory: Option<PathBuf>,
}

impl Default for OsascriptRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl OsascriptRunner {
    pub fn new() -> Self {
        Self {
            program: PathBuf::from(OSASCRIPT_PATH),
            temp_directory: None,
        }
    }

    #[cfg(test)]
    fn with_program(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            temp_directory: None,
        }
    }

    #[cfg(test)]
    fn with_program_and_temp_directory(
        program: impl Into<PathBuf>,
        temp_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            program: program.into(),
            temp_directory: Some(temp_directory.into()),
        }
    }

    fn execute_platform(
        &self,
        script: Script,
        request: &Value,
        timeout: Duration,
    ) -> Result<Value, OsaMailError> {
        if !self.program.is_file() {
            return Err(OsaMailError::OsaScriptNotFound);
        }

        let mut builder = Builder::new();
        builder.prefix("osamail-request-").suffix(".json");
        let mut request_file = match &self.temp_directory {
            Some(directory) => builder.tempfile_in(directory)?,
            None => builder.tempfile()?,
        };
        restrict_permissions(request_file.path())?;
        serde_json::to_writer(request_file.as_file_mut(), request)?;
        request_file.as_file_mut().flush()?;

        let mut command = self.command(script, request_file.path());
        let mut child = command.spawn()?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| OsaMailError::ScriptFailed {
                message: "failed to capture osascript stdout".to_owned(),
            })?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| OsaMailError::ScriptFailed {
                message: "failed to capture osascript stderr".to_owned(),
            })?;

        let stdout_reader = thread::spawn(move || read_all(stdout));
        let stderr_reader = thread::spawn(move || read_all(stderr));

        let status = match wait_for_child(&mut child, timeout)? {
            Some(status) => status,
            None => {
                child.kill()?;
                let _status = child.wait()?;
                let _stdout = join_reader(stdout_reader)?;
                let _stderr = join_reader(stderr_reader)?;
                return Err(OsaMailError::Timeout {
                    seconds: timeout.as_secs(),
                });
            }
        };

        let stdout = join_reader(stdout_reader)?;
        let stderr = join_reader(stderr_reader)?;

        if !status.success() {
            if looks_like_permission_error(&stderr) {
                return Err(OsaMailError::AutomationPermissionDenied);
            }
            return Err(OsaMailError::ScriptFailed {
                message: format!(
                    "osascript exited with status {}",
                    status
                        .code()
                        .map_or_else(|| "unknown".to_owned(), |code| code.to_string())
                ),
            });
        }

        let stdout = String::from_utf8(stdout).map_err(|_| {
            OsaMailError::InvalidScriptOutput("stdout was not valid UTF-8".to_owned())
        })?;
        serde_json::from_str(stdout.trim()).map_err(|_| {
            OsaMailError::InvalidScriptOutput(
                "osascript did not return exactly one JSON value".to_owned(),
            )
        })
    }

    fn command(&self, script: Script, request_path: &Path) -> Command {
        let mut command = Command::new(&self.program);
        command
            .arg("-l")
            .arg("JavaScript")
            .arg("-e")
            .arg(script.source())
            .arg(request_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }
}

impl AutomationRunner for OsascriptRunner {
    fn execute(
        &self,
        script: Script,
        request: &Value,
        timeout: Duration,
    ) -> Result<Value, OsaMailError> {
        if cfg!(target_os = "macos") {
            self.execute_platform(script, request, timeout)
        } else {
            Err(OsaMailError::UnsupportedPlatform)
        }
    }
}

fn wait_for_child(
    child: &mut std::process::Child,
    timeout: Duration,
) -> std::io::Result<Option<std::process::ExitStatus>> {
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status));
        }
        if started.elapsed() >= timeout {
            return Ok(None);
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn read_all(mut reader: impl Read) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn join_reader(
    handle: thread::JoinHandle<std::io::Result<Vec<u8>>>,
) -> Result<Vec<u8>, OsaMailError> {
    handle
        .join()
        .map_err(|_| OsaMailError::ScriptFailed {
            message: "failed to collect osascript output".to_owned(),
        })?
        .map_err(OsaMailError::Io)
}

fn looks_like_permission_error(stderr: &[u8]) -> bool {
    let text = String::from_utf8_lossy(stderr).to_ascii_lowercase();
    text.contains("-1743")
        || text.contains("not authorized")
        || text.contains("not permitted")
        || text.contains("automation")
}

#[cfg(unix)]
fn restrict_permissions(path: &Path) -> Result<(), OsaMailError> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) -> Result<(), OsaMailError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::OsStr,
        fs::File,
        os::unix::fs::PermissionsExt,
        time::{Duration, Instant},
    };

    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    const FIXTURE_TIMEOUT: Duration = Duration::from_secs(10);

    fn executable_fixture(contents: &str) -> (tempfile::TempDir, PathBuf) {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("fake-osascript");
        let mut file = File::create(&path).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
        let mut permissions = file.metadata().unwrap().permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&path, permissions).unwrap();
        (directory, path)
    }

    #[test]
    fn production_command_uses_absolute_osascript_without_shell() {
        let runner = OsascriptRunner::new();
        let command = runner.command(Script::Doctor, Path::new("/tmp/request.json"));
        assert_eq!(command.get_program(), OsStr::new(OSASCRIPT_PATH));
        let args: Vec<_> = command.get_args().collect();
        assert_eq!(args.last(), Some(&OsStr::new("/tmp/request.json")));
        assert!(!args.iter().any(|arg| *arg == OsStr::new("-c")));
    }

    #[test]
    fn user_content_is_not_present_in_process_arguments() {
        let secret = "private body 中文 🚀";
        let runner = OsascriptRunner::new();
        let command = runner.command(Script::SendMessage, Path::new("/tmp/request.json"));
        assert!(
            !command
                .get_args()
                .any(|arg| arg.to_string_lossy().contains(secret))
        );
    }

    #[test]
    fn parses_successful_json() {
        let script = "#!/bin/sh\nprintf '%s' '{\"ok\":true,\"data\":{\"count\":1}}'\n";
        let (_directory, path) = executable_fixture(script);
        let runner = OsascriptRunner::with_program(path);
        let response = runner
            .execute_platform(
                Script::Doctor,
                &json!({"secret":"not-in-args"}),
                FIXTURE_TIMEOUT,
            )
            .unwrap();
        assert_eq!(response["data"]["count"], 1);
    }

    #[test]
    fn request_files_have_private_permissions() {
        let request_file = tempfile::NamedTempFile::new().unwrap();
        restrict_permissions(request_file.path()).unwrap();
        let mode = request_file.path().metadata().unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn request_files_are_removed_after_execution() {
        let script = "#!/bin/sh\nprintf '%s' '{\"ok\":true,\"data\":{}}'\n";
        let (_fixture_directory, path) = executable_fixture(script);
        let request_directory = tempfile::tempdir().unwrap();
        let runner =
            OsascriptRunner::with_program_and_temp_directory(path, request_directory.path());

        runner
            .execute_platform(Script::Doctor, &json!({}), FIXTURE_TIMEOUT)
            .unwrap();

        assert!(
            request_directory
                .path()
                .read_dir()
                .unwrap()
                .next()
                .is_none()
        );
    }

    #[test]
    fn nonzero_status_is_a_script_failure() {
        let (_directory, path) = executable_fixture("#!/bin/sh\nexit 9\n");
        let error = OsascriptRunner::with_program(path)
            .execute_platform(Script::Doctor, &json!({}), FIXTURE_TIMEOUT)
            .unwrap_err();
        assert_eq!(error.code(), "SCRIPT_FAILED");
    }

    #[test]
    fn invalid_json_is_rejected() {
        let (_directory, path) = executable_fixture("#!/bin/sh\nprintf '%s' 'not-json'\n");
        let error = OsascriptRunner::with_program(path)
            .execute_platform(Script::Doctor, &json!({}), FIXTURE_TIMEOUT)
            .unwrap_err();
        assert_eq!(error.code(), "INVALID_SCRIPT_OUTPUT");
    }

    #[test]
    fn timeout_kills_the_child() {
        let (_directory, path) = executable_fixture("#!/bin/sh\nexec sleep 5\n");
        let started = Instant::now();
        let error = OsascriptRunner::with_program(path)
            .execute_platform(Script::Doctor, &json!({}), Duration::from_millis(50))
            .unwrap_err();
        assert_eq!(error.code(), "TIMEOUT");
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn maps_native_permission_error() {
        let script =
            "#!/bin/sh\nprintf '%s' 'Not authorized to send Apple events. (-1743)' >&2\nexit 1\n";
        let (_directory, path) = executable_fixture(script);
        let error = OsascriptRunner::with_program(path)
            .execute_platform(Script::Doctor, &json!({}), FIXTURE_TIMEOUT)
            .unwrap_err();
        assert_eq!(error.code(), "AUTOMATION_PERMISSION_DENIED");
    }

    #[test]
    fn show_script_does_not_swallow_property_read_errors() {
        let source = Script::ShowMessage.source();
        let string_helper = source
            .split_once("function optionalString")
            .unwrap()
            .1
            .split_once("function optionalDate")
            .unwrap()
            .0;
        let date_helper = source
            .split_once("function optionalDate")
            .unwrap()
            .1
            .split_once("function recipients")
            .unwrap()
            .0;

        assert!(!string_helper.contains("catch"));
        assert!(!date_helper.contains("catch"));
    }

    #[test]
    fn mark_script_gates_every_write_behind_dry_run() {
        let source = Script::MarkMessage.source();
        assert!(source.contains("if (!request.dry_run && !alreadySet)"));
        assert_eq!(source.matches("message.readStatus = desired").count(), 1);
        assert_eq!(source.matches("message.flaggedStatus = desired").count(), 1);
    }
}
