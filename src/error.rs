use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum OsaMailError {
    #[error("OsaMail only supports macOS.")]
    UnsupportedPlatform,
    #[error("/usr/bin/osascript was not found.")]
    OsaScriptNotFound,
    #[error("Apple Mail was not found.")]
    MailNotFound,
    #[error("Automation permission to control Mail was denied.")]
    AutomationPermissionDenied,
    #[error("Account not found.")]
    AccountNotFound(String),
    #[error("Mailbox not found.")]
    MailboxNotFound(String),
    #[error("Message not found.")]
    MessageNotFound,
    #[error("Invalid message reference: {0}")]
    InvalidReference(String),
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),
    #[error("osascript timed out after {seconds} seconds.")]
    Timeout { seconds: u64 },
    #[error("Apple Mail automation failed: {message}")]
    ScriptFailed { message: String },
    #[error("Apple Mail returned an invalid response: {0}")]
    InvalidScriptOutput(String),
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("JSON error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl OsaMailError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedPlatform => "UNSUPPORTED_PLATFORM",
            Self::OsaScriptNotFound => "OSASCRIPT_NOT_FOUND",
            Self::MailNotFound => "MAIL_NOT_FOUND",
            Self::AutomationPermissionDenied => "AUTOMATION_PERMISSION_DENIED",
            Self::AccountNotFound(_) => "ACCOUNT_NOT_FOUND",
            Self::MailboxNotFound(_) => "MAILBOX_NOT_FOUND",
            Self::MessageNotFound => "MESSAGE_NOT_FOUND",
            Self::InvalidReference(_) => "INVALID_REFERENCE",
            Self::InvalidArguments(_) => "INVALID_ARGUMENTS",
            Self::Timeout { .. } => "TIMEOUT",
            Self::ScriptFailed { .. } => "SCRIPT_FAILED",
            Self::InvalidScriptOutput(_) => "INVALID_SCRIPT_OUTPUT",
            Self::Io(_) => "IO",
            Self::Serialization(_) => "SERIALIZATION",
        }
    }

    pub const fn exit_code(&self) -> u8 {
        match self {
            Self::InvalidArguments(_) | Self::InvalidReference(_) => 2,
            Self::UnsupportedPlatform | Self::OsaScriptNotFound | Self::MailNotFound => 3,
            Self::AutomationPermissionDenied => 4,
            Self::AccountNotFound(_) | Self::MailboxNotFound(_) | Self::MessageNotFound => 5,
            Self::Timeout { .. } => 6,
            Self::ScriptFailed { .. } | Self::InvalidScriptOutput(_) => 7,
            Self::Io(_) | Self::Serialization(_) => 1,
        }
    }

    pub const fn hint(&self) -> Option<&'static str> {
        match self {
            Self::AutomationPermissionDenied => Some(
                "Open System Settings -> Privacy & Security -> Automation, then allow your terminal application to control Mail.",
            ),
            Self::Timeout { .. } => {
                Some("Retry with a larger global timeout, for example: osamail --timeout 60 ...")
            }
            Self::UnsupportedPlatform => Some("Run OsaMail on macOS with Apple Mail installed."),
            Self::OsaScriptNotFound => Some("Restore the macOS system tool at /usr/bin/osascript."),
            Self::MailNotFound => Some("Install or restore /System/Applications/Mail.app."),
            _ => None,
        }
    }

    pub fn from_automation(code: &str, message: String) -> Self {
        match code {
            "AUTOMATION_PERMISSION_DENIED" => Self::AutomationPermissionDenied,
            "ACCOUNT_NOT_FOUND" => Self::AccountNotFound(String::new()),
            "MAILBOX_NOT_FOUND" => Self::MailboxNotFound(String::new()),
            "MESSAGE_NOT_FOUND" => Self::MessageNotFound,
            "MAIL_NOT_FOUND" => Self::MailNotFound,
            "INVALID_ARGUMENTS" => Self::InvalidArguments(message),
            _ => Self::ScriptFailed {
                message: if message.is_empty() {
                    "Mail rejected the automation request".to_owned()
                } else {
                    message
                },
            },
        }
    }
}
