use std::io::Write;

use serde::Serialize;

use crate::{
    error::OsaMailError,
    model::{
        Account, DoctorReport, MarkOutcome, MarkResult, MessageDetail, MessageSummary, SendResult,
        Success,
    },
};

#[derive(Serialize)]
struct ErrorEnvelope<'a> {
    ok: bool,
    error: ErrorBody<'a>,
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    hint: Option<&'a str>,
}

pub fn write_json<T: Serialize>(writer: &mut dyn Write, value: &T) -> Result<(), OsaMailError> {
    serde_json::to_writer_pretty(&mut *writer, value)?;
    writeln!(writer)?;
    Ok(())
}

pub fn write_json_success<T: Serialize>(
    writer: &mut dyn Write,
    data: T,
) -> Result<(), OsaMailError> {
    write_json(writer, &Success::new(data))
}

pub fn write_error(
    writer: &mut dyn Write,
    json: bool,
    error: &OsaMailError,
) -> std::io::Result<()> {
    if json {
        let envelope = ErrorEnvelope {
            ok: false,
            error: ErrorBody {
                code: error.code(),
                message: error.to_string(),
                hint: error.hint(),
            },
        };
        serde_json::to_writer_pretty(&mut *writer, &envelope).map_err(std::io::Error::other)?;
        writeln!(writer)?;
    } else {
        writeln!(writer, "error: {error}")?;
        if let Some(hint) = error.hint() {
            writeln!(writer, "hint: {hint}")?;
        }
    }
    Ok(())
}

pub fn write_doctor(writer: &mut dyn Write, report: &DoctorReport) -> Result<(), OsaMailError> {
    writeln!(writer, "OsaMail doctor")?;
    writeln!(writer)?;
    for check in &report.checks {
        let status = if check.ok { "OK" } else { "FAIL" };
        writeln!(writer, "[{status}] {}", check.message)?;
    }
    writeln!(writer)?;
    if report.ready {
        writeln!(writer, "OsaMail is ready.")?;
    } else {
        writeln!(writer, "OsaMail is not ready.")?;
    }
    Ok(())
}

pub fn write_accounts(writer: &mut dyn Write, accounts: &[Account]) -> Result<(), OsaMailError> {
    if accounts.is_empty() {
        writeln!(writer, "No Apple Mail accounts are configured.")?;
        return Ok(());
    }
    for account in accounts {
        let status = if account.enabled {
            "enabled"
        } else {
            "disabled"
        };
        let addresses = if account.email_addresses.is_empty() {
            "-".to_owned()
        } else {
            account.email_addresses.join(", ")
        };
        writeln!(writer, "{}\t{}\t{}", account.name, addresses, status)?;
    }
    Ok(())
}

pub fn write_messages(
    writer: &mut dyn Write,
    messages: &[MessageSummary],
) -> Result<(), OsaMailError> {
    if messages.is_empty() {
        writeln!(writer, "No messages found.")?;
        return Ok(());
    }

    writeln!(
        writer,
        "{:<20}  {:<7}  {:<28}  SUBJECT",
        "RECEIVED", "STATUS", "SENDER"
    )?;
    for message in messages {
        let received = message.received_at.as_deref().unwrap_or("-");
        let status = if message.unread { "unread" } else { "read" };
        writeln!(
            writer,
            "{:<20}  {:<7}  {:<28}  {}",
            truncate(received, 20),
            status,
            truncate(&message.sender, 28),
            message.subject
        )?;
        writeln!(writer, "  ref: {}", message.reference)?;
    }
    Ok(())
}

pub fn write_titles(writer: &mut dyn Write, titles: &[String]) -> Result<(), OsaMailError> {
    if titles.is_empty() {
        writeln!(writer, "No messages found.")?;
        return Ok(());
    }
    for title in titles {
        writeln!(writer, "{title}")?;
    }
    Ok(())
}

pub fn write_message_detail(
    writer: &mut dyn Write,
    message: &MessageDetail,
    max_body_bytes: usize,
) -> Result<(), OsaMailError> {
    writeln!(writer, "From: {}", message.sender)?;
    writeln!(writer, "To: {}", message.to.join(", "))?;
    if !message.cc.is_empty() {
        writeln!(writer, "Cc: {}", message.cc.join(", "))?;
    }
    if !message.bcc.is_empty() {
        writeln!(writer, "Bcc: {}", message.bcc.join(", "))?;
    }
    writeln!(writer, "Subject: {}", message.subject)?;
    writeln!(
        writer,
        "Received: {}",
        message.received_at.as_deref().unwrap_or("-")
    )?;
    writeln!(writer, "Account: {}", message.account)?;
    writeln!(writer, "Mailbox: {}", message.mailbox)?;
    writeln!(
        writer,
        "Status: {}",
        if message.unread { "unread" } else { "read" }
    )?;
    if let Some(headers) = &message.headers {
        writeln!(writer)?;
        writeln!(writer, "Headers:")?;
        writeln!(writer, "{headers}")?;
    }
    writeln!(writer)?;
    writeln!(writer, "{}", truncate_bytes(&message.body, max_body_bytes))?;
    if message.body.len() > max_body_bytes {
        writeln!(
            writer,
            "\n[body truncated at {max_body_bytes} bytes; use --max-body-bytes to adjust]"
        )?;
    }
    Ok(())
}

pub fn write_open_result(writer: &mut dyn Write) -> Result<(), OsaMailError> {
    writeln!(writer, "Opened message in Apple Mail.")?;
    Ok(())
}

pub fn write_mark_result(writer: &mut dyn Write, result: &MarkResult) -> Result<(), OsaMailError> {
    match result.outcome {
        MarkOutcome::Changed => {
            writeln!(writer, "Marked message as {}.", result.action.as_str())?;
        }
        MarkOutcome::AlreadySet => {
            writeln!(writer, "Message is already {}.", result.action.as_str())?;
        }
        MarkOutcome::WouldChange => {
            writeln!(
                writer,
                "Dry run valid: message would be marked {}; no change was made.",
                result.action.as_str()
            )?;
        }
    }
    Ok(())
}

pub fn write_send_result(writer: &mut dyn Write, result: &SendResult) -> Result<(), OsaMailError> {
    if result.dry_run {
        writeln!(
            writer,
            "Dry run valid: {} recipient(s); no message was created or sent.",
            result.recipient_count
        )?;
    } else {
        writeln!(
            writer,
            "Message accepted by Apple Mail for {} recipient(s).",
            result.recipient_count
        )?;
    }
    Ok(())
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut characters = value.chars();
    let prefix: String = characters.by_ref().take(max_chars).collect();
    if characters.next().is_some() {
        format!("{prefix}…")
    } else {
        prefix
    }
}

fn truncate_bytes(value: &str, max_bytes: usize) -> &str {
    if value.len() <= max_bytes {
        return value;
    }
    let mut boundary = max_bytes;
    while boundary > 0 && !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    &value[..boundary]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::CountOutput;

    #[test]
    fn json_success_is_a_single_valid_value() {
        let mut bytes = Vec::new();
        write_json_success(&mut bytes, CountOutput { count: 7 }).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["ok"], true);
        assert_eq!(value["data"]["count"], 7);
    }

    #[test]
    fn error_json_is_structured() {
        let mut bytes = Vec::new();
        write_error(&mut bytes, true, &OsaMailError::AutomationPermissionDenied).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["code"], "AUTOMATION_PERMISSION_DENIED");
    }

    #[test]
    fn lookup_errors_include_recovery_hints_in_human_and_json_output() {
        let error = OsaMailError::AccountNotFound(String::new());
        let mut human = Vec::new();
        write_error(&mut human, false, &error).unwrap();
        assert!(
            String::from_utf8(human)
                .unwrap()
                .contains("hint: Run `osamail accounts`")
        );

        let mut json = Vec::new();
        write_error(&mut json, true, &error).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&json).unwrap();
        assert_eq!(
            value["error"]["hint"],
            "Run `osamail accounts` and use an exact enabled account name."
        );
    }

    #[test]
    fn body_truncation_preserves_utf8() {
        assert_eq!(truncate_bytes("a🚀b", 4), "a");
        assert_eq!(truncate_bytes("a🚀b", 5), "a🚀");
    }

    #[test]
    fn titles_are_one_per_line() {
        let mut bytes = Vec::new();
        write_titles(&mut bytes, &["First".to_owned(), "第二封".to_owned()]).unwrap();
        assert_eq!(String::from_utf8(bytes).unwrap(), "First\n第二封\n");
    }

    #[test]
    fn mark_dry_run_output_is_explicit_about_no_change() {
        let result = MarkResult {
            reference: "ref".to_owned(),
            action: crate::model::MarkAction::Unread,
            outcome: MarkOutcome::WouldChange,
        };
        let mut bytes = Vec::new();

        write_mark_result(&mut bytes, &result).unwrap();

        assert_eq!(
            String::from_utf8(bytes).unwrap(),
            "Dry run valid: message would be marked unread; no change was made.\n"
        );
    }
}
