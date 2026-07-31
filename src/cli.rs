use std::path::PathBuf;

use clap::{ArgAction, ArgGroup, Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "osamail",
    version = env!("CARGO_PKG_VERSION"),
    about = "A tiny, scriptable CLI for Apple Mail, powered by osascript."
)]
pub struct Cli {
    /// Emit stable JSON instead of human-readable text.
    #[arg(long, global = true)]
    pub json: bool,

    /// Override the command timeout in seconds.
    #[arg(
        long,
        global = true,
        value_name = "SECONDS",
        value_parser = clap::value_parser!(u64).range(1..=3600)
    )]
    pub timeout: Option<u64>,

    /// Suppress successful human-readable output.
    #[arg(long, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Check the local macOS, Mail, and Automation environment.
    Doctor,
    /// List accounts already configured in Apple Mail.
    Accounts,
    /// List mailbox paths and destination references.
    Mailboxes(MailboxesArgs),
    /// List recently received messages without loading bodies.
    Recent(ListArgs),
    /// List unread messages or print their count.
    Unread(UnreadArgs),
    /// Search message metadata using Mail's scripting filters.
    Search(SearchArgs),
    /// Show one message in the terminal.
    Show(ShowArgs),
    /// Open one message in Apple Mail.
    Open(ReferenceArgs),
    /// Mark up to 50 messages as read, unread, flagged, or unflagged.
    Mark(MarkArgs),
    /// Move up to 50 messages to an explicitly selected mailbox.
    Move(OrganizeArgs),
    /// Archive up to 50 messages in an explicitly selected mailbox.
    Archive(OrganizeArgs),
    /// Send a plain-text message through an Apple Mail account.
    Send(SendArgs),
}

impl Command {
    pub const fn default_timeout_seconds(&self) -> u64 {
        match self {
            Self::Doctor | Self::Accounts | Self::Open(_) => 10,
            Self::Mailboxes(_)
            | Self::Recent(_)
            | Self::Unread(_)
            | Self::Show(_)
            | Self::Mark(_)
            | Self::Move(_)
            | Self::Archive(_)
            | Self::Send(_) => 20,
            Self::Search(_) => 30,
        }
    }

    pub const fn progress_message(&self) -> Option<&'static str> {
        match self {
            Self::Mailboxes(_) => Some("Reading mailboxes from Apple Mail..."),
            Self::Recent(_) | Self::Unread(_) | Self::Search(_) => Some("Searching Apple Mail..."),
            Self::Show(_) => Some("Reading message from Apple Mail..."),
            Self::Mark(_) => Some("Checking message in Apple Mail..."),
            Self::Move(_) | Self::Archive(_) => Some("Organizing messages in Apple Mail..."),
            Self::Doctor | Self::Accounts | Self::Open(_) | Self::Send(_) => None,
        }
    }
}

#[derive(Debug, Args)]
pub struct MailboxesArgs {
    /// Restrict results to an exact Apple Mail account name.
    #[arg(long)]
    pub account: Option<String>,
}

#[derive(Debug, Args)]
pub struct ListArgs {
    /// Maximum number of messages to return.
    #[arg(long, default_value_t = 10, value_parser = parse_limit)]
    pub limit: u16,
    /// Print only message subjects.
    #[arg(long)]
    pub titles: bool,
    /// Restrict results to an exact Apple Mail account name.
    #[arg(long)]
    pub account: Option<String>,
    /// Restrict results to a mailbox name.
    #[arg(long)]
    pub mailbox: Option<String>,
    /// Include messages received on or after this local date.
    #[arg(long, value_name = "YYYY-MM-DD", value_parser = parse_calendar_date)]
    pub since: Option<String>,
    /// Include messages received before this local date.
    #[arg(long, value_name = "YYYY-MM-DD", value_parser = parse_calendar_date)]
    pub before: Option<String>,
}

#[derive(Debug, Args)]
pub struct UnreadArgs {
    /// Print only the unread count.
    #[arg(long)]
    pub count: bool,
    /// Maximum number of messages to return.
    #[arg(long, default_value_t = 10, value_parser = parse_limit)]
    pub limit: u16,
    /// Print only message subjects.
    #[arg(long, conflicts_with = "count")]
    pub titles: bool,
    /// Restrict results to an exact Apple Mail account name.
    #[arg(long)]
    pub account: Option<String>,
    /// Restrict results to a mailbox name.
    #[arg(long)]
    pub mailbox: Option<String>,
    /// Include messages received on or after this local date.
    #[arg(long, value_name = "YYYY-MM-DD", value_parser = parse_calendar_date)]
    pub since: Option<String>,
    /// Include messages received before this local date.
    #[arg(long, value_name = "YYYY-MM-DD", value_parser = parse_calendar_date)]
    pub before: Option<String>,
}

#[derive(Debug, Args)]
pub struct SearchArgs {
    /// Text to search in subject and sender.
    pub query: Option<String>,
    /// Restrict results to an exact Apple Mail account name.
    #[arg(long)]
    pub account: Option<String>,
    /// Restrict results to a mailbox name.
    #[arg(long)]
    pub mailbox: Option<String>,
    /// Maximum number of messages to return.
    #[arg(long, default_value_t = 10, value_parser = parse_limit)]
    pub limit: u16,
    /// Print only matching message subjects.
    #[arg(long)]
    pub titles: bool,
    /// Return only unread matches.
    #[arg(long)]
    pub unread: bool,
    /// Require the sender to contain this text.
    #[arg(long, value_name = "TEXT")]
    pub from: Option<String>,
    /// Require the subject to contain this text.
    #[arg(long, value_name = "TEXT")]
    pub subject: Option<String>,
    /// Include body content in the Mail-side search.
    #[arg(long)]
    pub body: bool,
    /// Include messages received on or after this local date.
    #[arg(long, value_name = "YYYY-MM-DD", value_parser = parse_calendar_date)]
    pub since: Option<String>,
    /// Include messages received before this local date.
    #[arg(long, value_name = "YYYY-MM-DD", value_parser = parse_calendar_date)]
    pub before: Option<String>,
}

#[derive(Debug, Args)]
pub struct ShowArgs {
    /// Opaque reference returned by recent, unread, or search.
    #[arg(value_name = "REF")]
    pub reference: String,
    /// Include raw textual headers in addition to normal fields.
    #[arg(long)]
    pub headers: bool,
    /// Explicitly request the body (the default human view already includes it).
    #[arg(long)]
    pub body: bool,
    /// Maximum body bytes printed in human-readable mode.
    #[arg(long, default_value_t = 65_536, value_parser = parse_positive_usize)]
    pub max_body_bytes: usize,
}

#[derive(Debug, Args)]
pub struct ReferenceArgs {
    /// Opaque reference returned by recent, unread, or search.
    #[arg(value_name = "REF")]
    pub reference: String,
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("reference_source")
        .args(["references", "stdin"])
        .required(true)
        .multiple(true)
))]
pub struct MarkArgs {
    /// State to apply to the message.
    #[arg(value_enum)]
    pub action: MarkActionArg,
    /// Opaque references returned by recent, unread, or search.
    #[arg(value_name = "REF", num_args = 0..=50)]
    pub references: Vec<String>,
    /// Read one opaque message reference per line from standard input.
    #[arg(long)]
    pub stdin: bool,
    /// Validate the message and report the outcome without changing Mail.
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("reference_source")
        .args(["references", "stdin"])
        .required(true)
        .multiple(true)
))]
pub struct OrganizeArgs {
    /// Opaque mailbox reference returned by mailboxes.
    #[arg(long, value_name = "MAILBOX_REF")]
    pub to: String,
    /// Opaque message references returned by recent, unread, or search.
    #[arg(value_name = "REF", num_args = 0..=50)]
    pub references: Vec<String>,
    /// Read one opaque message reference per line from standard input.
    #[arg(long)]
    pub stdin: bool,
    /// Validate messages and the destination without moving anything.
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum MarkActionArg {
    Read,
    Unread,
    Flag,
    Unflag,
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("body_source")
        .args(["body", "body_file", "stdin"])
        .multiple(false)
))]
pub struct SendArgs {
    /// Recipient address. Repeat for multiple recipients.
    #[arg(long, required = true, action = ArgAction::Append)]
    pub to: Vec<String>,
    /// Carbon-copy address. Repeat for multiple recipients.
    #[arg(long, action = ArgAction::Append)]
    pub cc: Vec<String>,
    /// Blind-carbon-copy address. Repeat for multiple recipients.
    #[arg(long, action = ArgAction::Append)]
    pub bcc: Vec<String>,
    /// Message subject.
    #[arg(long, default_value = "")]
    pub subject: String,
    /// Literal plain-text body.
    #[arg(long)]
    pub body: Option<String>,
    /// Read the plain-text body from this file.
    #[arg(long, value_name = "PATH")]
    pub body_file: Option<PathBuf>,
    /// Read the plain-text body from standard input.
    #[arg(long)]
    pub stdin: bool,
    /// Send from this exact Apple Mail account name.
    #[arg(long)]
    pub account: Option<String>,
    /// Validate and display metadata without creating or sending a message.
    #[arg(long)]
    pub dry_run: bool,
}

fn parse_limit(value: &str) -> Result<u16, String> {
    let limit = value
        .parse::<u16>()
        .map_err(|_| "limit must be an integer from 1 to 200".to_owned())?;
    if (1..=200).contains(&limit) {
        Ok(limit)
    } else {
        Err("limit must be from 1 to 200".to_owned())
    }
}

fn parse_positive_usize(value: &str) -> Result<usize, String> {
    let value = value
        .parse::<usize>()
        .map_err(|_| "value must be a positive integer".to_owned())?;
    if value == 0 {
        Err("value must be greater than zero".to_owned())
    } else {
        Ok(value)
    }
}

fn parse_calendar_date(value: &str) -> Result<String, String> {
    let invalid = || "date must be a valid calendar date in YYYY-MM-DD format".to_owned();
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes
            .iter()
            .enumerate()
            .any(|(index, byte)| index != 4 && index != 7 && !byte.is_ascii_digit())
    {
        return Err(invalid());
    }

    let year = value[0..4].parse::<u16>().map_err(|_| invalid())?;
    let month = value[5..7].parse::<u8>().map_err(|_| invalid())?;
    let day = value[8..10].parse::<u8>().map_err(|_| invalid())?;
    let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap_year => 29,
        2 => 28,
        _ => return Err(invalid()),
    };
    if year == 0 || day == 0 || day > days_in_month {
        return Err(invalid());
    }
    Ok(value.to_owned())
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser};

    use super::*;

    #[test]
    fn command_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_global_options_after_subcommand() {
        let cli = Cli::try_parse_from([
            "osamail",
            "recent",
            "--limit",
            "12",
            "--json",
            "--timeout",
            "45",
        ])
        .unwrap();
        assert!(cli.json);
        assert_eq!(cli.timeout, Some(45));
        match cli.command {
            Command::Recent(args) => assert_eq!(args.limit, 12),
            _ => panic!("expected recent"),
        }
    }

    #[test]
    fn rejects_limit_outside_range() {
        assert!(Cli::try_parse_from(["osamail", "recent", "--limit", "0"]).is_err());
        assert!(Cli::try_parse_from(["osamail", "recent", "--limit", "201"]).is_err());
    }

    #[test]
    fn unread_titles_conflicts_with_count() {
        assert!(Cli::try_parse_from(["osamail", "unread", "--titles", "--count"]).is_err());
    }

    #[test]
    fn read_commands_accept_local_date_filters() {
        let recent = Cli::try_parse_from([
            "osamail",
            "recent",
            "--since",
            "2024-02-29",
            "--before",
            "2024-03-02",
        ])
        .unwrap();
        match recent.command {
            Command::Recent(args) => {
                assert_eq!(args.since.as_deref(), Some("2024-02-29"));
                assert_eq!(args.before.as_deref(), Some("2024-03-02"));
            }
            _ => panic!("expected recent"),
        }

        let search = Cli::try_parse_from(["osamail", "search", "--since", "2024-01-01"]).unwrap();
        match search.command {
            Command::Search(args) => {
                assert!(args.query.is_none());
                assert_eq!(args.since.as_deref(), Some("2024-01-01"));
            }
            _ => panic!("expected search"),
        }
    }

    #[test]
    fn read_commands_reject_invalid_calendar_dates() {
        for date in ["2024-2-01", "2023-02-29", "2024-04-31", "0000-01-01"] {
            assert!(Cli::try_parse_from(["osamail", "unread", "--since", date]).is_err());
        }
    }

    #[test]
    fn mark_exposes_four_actions_and_dry_run() {
        for action in ["read", "unread", "flag", "unflag"] {
            let cli = Cli::try_parse_from(["osamail", "mark", action, "message-ref", "--dry-run"])
                .unwrap();
            match cli.command {
                Command::Mark(args) => {
                    assert_eq!(args.action.to_possible_value().unwrap().get_name(), action);
                    assert_eq!(args.references, ["message-ref"]);
                    assert!(args.dry_run);
                }
                _ => panic!("expected mark"),
            }
        }
    }

    #[test]
    fn mark_accepts_a_bounded_batch() {
        let references = (0..50).map(|index| format!("ref-{index}"));
        let cli = Cli::try_parse_from(
            ["osamail", "mark", "read"]
                .into_iter()
                .map(str::to_owned)
                .chain(references),
        )
        .unwrap();
        match cli.command {
            Command::Mark(args) => assert_eq!(args.references.len(), 50),
            _ => panic!("expected mark"),
        }

        let too_many = (0..51).map(|index| format!("ref-{index}"));
        assert!(
            Cli::try_parse_from(
                ["osamail", "mark", "read"]
                    .into_iter()
                    .map(str::to_owned)
                    .chain(too_many),
            )
            .is_err()
        );
    }

    #[test]
    fn mutation_commands_accept_explicit_stdin_reference_input() {
        let mark = Cli::try_parse_from(["osamail", "mark", "read", "--stdin"]).unwrap();
        match mark.command {
            Command::Mark(args) => {
                assert!(args.references.is_empty());
                assert!(args.stdin);
            }
            _ => panic!("expected mark"),
        }

        for command in ["move", "archive"] {
            let cli = Cli::try_parse_from(["osamail", command, "--to", "mailbox-ref", "--stdin"])
                .unwrap();
            let args = match cli.command {
                Command::Move(args) | Command::Archive(args) => args,
                _ => panic!("expected organization command"),
            };
            assert!(args.references.is_empty());
            assert!(args.stdin);
        }
    }

    #[test]
    fn mutation_commands_require_a_reference_source() {
        assert!(Cli::try_parse_from(["osamail", "mark", "read"]).is_err());
        assert!(Cli::try_parse_from(["osamail", "move", "--to", "mailbox-ref"]).is_err());
    }

    #[test]
    fn move_and_archive_share_one_explicit_destination_shape() {
        for command in ["move", "archive"] {
            let cli = Cli::try_parse_from([
                "osamail",
                command,
                "--to",
                "mailbox-ref",
                "message-one",
                "message-two",
                "--dry-run",
            ])
            .unwrap();
            let args = match cli.command {
                Command::Move(args) | Command::Archive(args) => args,
                _ => panic!("expected organization command"),
            };
            assert_eq!(args.to, "mailbox-ref");
            assert_eq!(args.references, ["message-one", "message-two"]);
            assert!(args.dry_run);
        }
    }

    #[test]
    fn read_commands_expose_delayed_progress_messages() {
        let recent = Cli::try_parse_from(["osamail", "recent"]).unwrap();
        let mailboxes = Cli::try_parse_from(["osamail", "mailboxes"]).unwrap();
        let show = Cli::try_parse_from(["osamail", "show", "message-ref"]).unwrap();
        let accounts = Cli::try_parse_from(["osamail", "accounts"]).unwrap();

        assert_eq!(
            recent.command.progress_message(),
            Some("Searching Apple Mail...")
        );
        assert_eq!(
            mailboxes.command.progress_message(),
            Some("Reading mailboxes from Apple Mail...")
        );
        assert_eq!(
            show.command.progress_message(),
            Some("Reading message from Apple Mail...")
        );
        assert_eq!(accounts.command.progress_message(), None);
    }

    #[test]
    fn mailboxes_accepts_an_exact_account_filter() {
        let cli =
            Cli::try_parse_from(["osamail", "mailboxes", "--account", "iCloud 中文"]).unwrap();
        match cli.command {
            Command::Mailboxes(args) => assert_eq!(args.account.as_deref(), Some("iCloud 中文")),
            _ => panic!("expected mailboxes"),
        }
    }

    #[test]
    fn send_body_sources_are_mutually_exclusive() {
        let result = Cli::try_parse_from([
            "osamail",
            "send",
            "--to",
            "a@example.com",
            "--body",
            "hello",
            "--stdin",
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn send_supports_multiple_recipients() {
        let cli = Cli::try_parse_from([
            "osamail",
            "send",
            "--to",
            "a@example.com",
            "--to",
            "b@example.com",
        ])
        .unwrap();
        match cli.command {
            Command::Send(args) => assert_eq!(args.to.len(), 2),
            _ => panic!("expected send"),
        }
    }

    #[test]
    fn send_requires_a_to_recipient() {
        assert!(Cli::try_parse_from(["osamail", "send", "--subject", "No recipient"]).is_err());
    }
}
