use std::{
    fs,
    io::{Read, Write},
    time::Duration,
};

use serde::{Serialize, de::DeserializeOwned};

use crate::{
    automation::{AutomationRunner, Script},
    cli::{Cli, Command, ListArgs, SearchArgs, SendArgs, ShowArgs, UnreadArgs},
    error::OsaMailError,
    model::{
        AccountsData, AccountsOutput, AutomationRequest, AutomationResponse, CountOutput,
        DoctorAutomationData, DoctorCheck, DoctorReport, ListMessagesRequest, ListMode,
        MessageDetail, MessageSummary, MessagesData, MessagesOutput, OpenMessageRequest,
        OpenResult, RawMessageDetail, SendRequest, SendResult, ShowMessageRequest,
    },
    output, reference,
};

const OSASCRIPT_PATH: &str = "/usr/bin/osascript";
const MAIL_PATH: &str = "/System/Applications/Mail.app";

pub fn execute(
    cli: &Cli,
    runner: &dyn AutomationRunner,
    input: &mut dyn Read,
    output_writer: &mut dyn Write,
) -> Result<(), OsaMailError> {
    if !cfg!(target_os = "macos") && !cfg!(test) {
        return Err(OsaMailError::UnsupportedPlatform);
    }

    let timeout = Duration::from_secs(
        cli.timeout
            .unwrap_or_else(|| cli.command.default_timeout_seconds()),
    );

    match &cli.command {
        Command::Doctor => doctor(cli, runner, output_writer, timeout),
        Command::Accounts => accounts(cli, runner, output_writer, timeout),
        Command::Recent(args) => list_recent(cli, runner, output_writer, timeout, args),
        Command::Unread(args) => unread(cli, runner, output_writer, timeout, args),
        Command::Search(args) => search(cli, runner, output_writer, timeout, args),
        Command::Show(args) => show(cli, runner, output_writer, timeout, args),
        Command::Open(args) => {
            let locator = reference::decode(&args.reference)?;
            let request = AutomationRequest::OpenMessage(OpenMessageRequest { locator });
            let result: OpenResult =
                run_automation(runner, Script::OpenMessage, &request, timeout)?;
            if !result.opened {
                return Err(OsaMailError::ScriptFailed {
                    message: "Mail did not accept the open request".to_owned(),
                });
            }
            if cli.json {
                output::write_json_success(output_writer, result)
            } else if !cli.quiet {
                output::write_open_result(output_writer)
            } else {
                Ok(())
            }
        }
        Command::Send(args) => send(cli, runner, input, output_writer, timeout, args),
    }
}

fn doctor(
    cli: &Cli,
    runner: &dyn AutomationRunner,
    writer: &mut dyn Write,
    timeout: Duration,
) -> Result<(), OsaMailError> {
    if !cfg!(target_os = "macos") {
        return Err(OsaMailError::UnsupportedPlatform);
    }
    if !std::path::Path::new(OSASCRIPT_PATH).is_file() {
        return Err(OsaMailError::OsaScriptNotFound);
    }
    if !std::path::Path::new(MAIL_PATH).is_dir() {
        return Err(OsaMailError::MailNotFound);
    }

    let data: DoctorAutomationData =
        run_automation(runner, Script::Doctor, &AutomationRequest::Doctor, timeout)?;
    let checks = vec![
        DoctorCheck {
            name: "platform".to_owned(),
            ok: true,
            message: "macOS detected".to_owned(),
        },
        DoctorCheck {
            name: "osascript".to_owned(),
            ok: true,
            message: "osascript found".to_owned(),
        },
        DoctorCheck {
            name: "mail".to_owned(),
            ok: true,
            message: "Apple Mail found".to_owned(),
        },
        DoctorCheck {
            name: "automation".to_owned(),
            ok: true,
            message: "Mail automation available".to_owned(),
        },
        DoctorCheck {
            name: "accounts".to_owned(),
            ok: data.account_count > 0,
            message: format!("{} account(s) configured", data.account_count),
        },
    ];
    let report = DoctorReport {
        ready: data.account_count > 0,
        architecture: std::env::consts::ARCH.to_owned(),
        mail_version: Some(data.mail_version),
        account_count: data.account_count,
        checks,
    };
    if cli.json {
        output::write_json_success(writer, report)
    } else if !cli.quiet {
        output::write_doctor(writer, &report)
    } else {
        Ok(())
    }
}

fn accounts(
    cli: &Cli,
    runner: &dyn AutomationRunner,
    writer: &mut dyn Write,
    timeout: Duration,
) -> Result<(), OsaMailError> {
    let data: AccountsData = run_automation(
        runner,
        Script::Accounts,
        &AutomationRequest::Accounts,
        timeout,
    )?;
    if cli.json {
        output::write_json_success(
            writer,
            AccountsOutput {
                accounts: data.accounts,
            },
        )
    } else if !cli.quiet {
        output::write_accounts(writer, &data.accounts)
    } else {
        Ok(())
    }
}

fn list_recent(
    cli: &Cli,
    runner: &dyn AutomationRunner,
    writer: &mut dyn Write,
    timeout: Duration,
    args: &ListArgs,
) -> Result<(), OsaMailError> {
    list_messages(
        cli,
        runner,
        writer,
        timeout,
        ListMessagesRequest {
            mode: ListMode::Recent,
            limit: args.limit,
            account: args.account.clone(),
            mailbox: args.mailbox.clone(),
            count_only: false,
            query: None,
            unread: false,
            from: None,
            subject: None,
            search_body: false,
        },
    )
}

fn unread(
    cli: &Cli,
    runner: &dyn AutomationRunner,
    writer: &mut dyn Write,
    timeout: Duration,
    args: &UnreadArgs,
) -> Result<(), OsaMailError> {
    let request = ListMessagesRequest {
        mode: ListMode::Unread,
        limit: args.limit,
        account: args.account.clone(),
        mailbox: args.mailbox.clone(),
        count_only: args.count,
        query: None,
        unread: true,
        from: None,
        subject: None,
        search_body: false,
    };
    if args.count {
        let data: MessagesData = run_automation(
            runner,
            Script::ListMessages,
            &AutomationRequest::ListMessages(request),
            timeout,
        )?;
        if cli.json {
            output::write_json_success(writer, CountOutput { count: data.count })
        } else if !cli.quiet {
            writeln!(writer, "{}", data.count)?;
            Ok(())
        } else {
            Ok(())
        }
    } else {
        list_messages(cli, runner, writer, timeout, request)
    }
}

fn search(
    cli: &Cli,
    runner: &dyn AutomationRunner,
    writer: &mut dyn Write,
    timeout: Duration,
    args: &SearchArgs,
) -> Result<(), OsaMailError> {
    if args.query.is_empty() && args.from.is_none() && args.subject.is_none() {
        return Err(OsaMailError::InvalidArguments(
            "provide a query, --from, or --subject".to_owned(),
        ));
    }
    list_messages(
        cli,
        runner,
        writer,
        timeout,
        ListMessagesRequest {
            mode: ListMode::Search,
            limit: args.limit,
            account: args.account.clone(),
            mailbox: args.mailbox.clone(),
            count_only: false,
            query: (!args.query.is_empty()).then(|| args.query.clone()),
            unread: args.unread,
            from: args.from.clone(),
            subject: args.subject.clone(),
            search_body: args.body,
        },
    )
}

fn list_messages(
    cli: &Cli,
    runner: &dyn AutomationRunner,
    writer: &mut dyn Write,
    timeout: Duration,
    request: ListMessagesRequest,
) -> Result<(), OsaMailError> {
    let data: MessagesData = run_automation(
        runner,
        Script::ListMessages,
        &AutomationRequest::ListMessages(request),
        timeout,
    )?;
    let mut messages = Vec::with_capacity(data.messages.len());
    for raw in data.messages {
        messages.push(summary_from_raw(raw)?);
    }
    if cli.json {
        output::write_json_success(
            writer,
            MessagesOutput {
                count: data.count,
                messages,
            },
        )
    } else if !cli.quiet {
        output::write_messages(writer, &messages)
    } else {
        Ok(())
    }
}

fn show(
    cli: &Cli,
    runner: &dyn AutomationRunner,
    writer: &mut dyn Write,
    timeout: Duration,
    args: &ShowArgs,
) -> Result<(), OsaMailError> {
    let locator = reference::decode(&args.reference)?;
    let raw: RawMessageDetail = run_automation(
        runner,
        Script::ShowMessage,
        &AutomationRequest::ShowMessage(ShowMessageRequest {
            locator,
            include_headers: args.headers,
        }),
        timeout,
    )?;
    let detail = detail_from_raw(raw)?;
    if cli.json {
        output::write_json_success(writer, detail)
    } else if !cli.quiet {
        output::write_message_detail(writer, &detail, args.max_body_bytes)
    } else {
        Ok(())
    }
}

fn send(
    cli: &Cli,
    runner: &dyn AutomationRunner,
    input: &mut dyn Read,
    writer: &mut dyn Write,
    timeout: Duration,
    args: &SendArgs,
) -> Result<(), OsaMailError> {
    validate_recipients(&args.to, &args.cc, &args.bcc)?;
    let body = read_body(args, input)?;
    let request = SendRequest {
        to: args.to.clone(),
        cc: args.cc.clone(),
        bcc: args.bcc.clone(),
        subject: args.subject.clone(),
        body,
        account: args.account.clone(),
    };
    let recipient_count = request.to.len() + request.cc.len() + request.bcc.len();

    let result = if args.dry_run {
        if let Some(account_name) = &request.account {
            let accounts: AccountsData = run_automation(
                runner,
                Script::Accounts,
                &AutomationRequest::Accounts,
                timeout,
            )?;
            if !accounts
                .accounts
                .iter()
                .any(|account| account.name == *account_name && account.enabled)
            {
                return Err(OsaMailError::AccountNotFound(String::new()));
            }
        }
        SendResult {
            sent: false,
            dry_run: true,
            account: request.account,
            recipient_count,
        }
    } else {
        let mut result: SendResult = run_automation(
            runner,
            Script::SendMessage,
            &AutomationRequest::SendMessage(request),
            timeout,
        )?;
        result.recipient_count = recipient_count;
        result
    };

    if cli.json {
        output::write_json_success(writer, result)
    } else if !cli.quiet {
        output::write_send_result(writer, &result)
    } else {
        Ok(())
    }
}

fn read_body(args: &SendArgs, input: &mut dyn Read) -> Result<String, OsaMailError> {
    if let Some(body) = &args.body {
        return Ok(body.clone());
    }
    if let Some(path) = &args.body_file {
        return fs::read_to_string(path).map_err(OsaMailError::Io);
    }
    if args.stdin {
        let mut body = String::new();
        input.read_to_string(&mut body)?;
        return Ok(body);
    }
    Ok(String::new())
}

fn validate_recipients(to: &[String], cc: &[String], bcc: &[String]) -> Result<(), OsaMailError> {
    if to.is_empty() {
        return Err(OsaMailError::InvalidArguments(
            "at least one --to recipient is required".to_owned(),
        ));
    }
    if to
        .iter()
        .chain(cc)
        .chain(bcc)
        .any(|address| address.trim().is_empty() || address.chars().any(char::is_whitespace))
    {
        return Err(OsaMailError::InvalidArguments(
            "recipient addresses must be non-empty and contain no whitespace".to_owned(),
        ));
    }
    Ok(())
}

fn run_automation<TRequest: Serialize, TResponse: DeserializeOwned>(
    runner: &dyn AutomationRunner,
    script: Script,
    request: &TRequest,
    timeout: Duration,
) -> Result<TResponse, OsaMailError> {
    let request = serde_json::to_value(request)?;
    let value = runner.execute(script, &request, timeout)?;
    let response: AutomationResponse<TResponse> = serde_json::from_value(value).map_err(|_| {
        OsaMailError::InvalidScriptOutput("response did not match the JSON protocol".to_owned())
    })?;
    if response.ok {
        response.data.ok_or_else(|| {
            OsaMailError::InvalidScriptOutput("successful response omitted data".to_owned())
        })
    } else if let Some(error) = response.error {
        Err(OsaMailError::from_automation(&error.code, error.message))
    } else {
        Err(OsaMailError::InvalidScriptOutput(
            "failed response omitted error details".to_owned(),
        ))
    }
}

fn summary_from_raw(raw: crate::model::RawMessageSummary) -> Result<MessageSummary, OsaMailError> {
    let mailbox = raw.locator.mailbox_path.join(" / ");
    let account = raw.locator.account.clone();
    Ok(MessageSummary {
        reference: reference::encode(&raw.locator)?,
        account,
        mailbox,
        sender: raw.sender,
        subject: raw.subject,
        received_at: raw.received_at,
        unread: raw.unread,
    })
}

fn detail_from_raw(raw: RawMessageDetail) -> Result<MessageDetail, OsaMailError> {
    let mailbox = raw.locator.mailbox_path.join(" / ");
    let account = raw.locator.account.clone();
    Ok(MessageDetail {
        reference: reference::encode(&raw.locator)?,
        account,
        mailbox,
        sender: raw.sender,
        to: raw.to,
        cc: raw.cc,
        bcc: raw.bcc,
        subject: raw.subject,
        received_at: raw.received_at,
        unread: raw.unread,
        body: raw.body,
        headers: raw.headers,
        message_id: raw.message_id,
    })
}

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, sync::Mutex};

    use clap::Parser;
    use serde_json::{Value, json};

    use super::*;

    struct FakeRunner {
        responses: Mutex<VecDeque<Result<Value, OsaMailError>>>,
        requests: Mutex<Vec<Value>>,
    }

    impl FakeRunner {
        fn new(responses: Vec<Result<Value, OsaMailError>>) -> Self {
            Self {
                responses: Mutex::new(responses.into()),
                requests: Mutex::new(Vec::new()),
            }
        }
    }

    impl AutomationRunner for FakeRunner {
        fn execute(
            &self,
            _script: Script,
            request: &Value,
            _timeout: Duration,
        ) -> Result<Value, OsaMailError> {
            self.requests.lock().unwrap().push(request.clone());
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| {
                    Err(OsaMailError::ScriptFailed {
                        message: "missing fake response".to_owned(),
                    })
                })
        }
    }

    #[test]
    fn unread_count_json_uses_structured_output() {
        let cli = Cli::try_parse_from(["osamail", "unread", "--count", "--json"]).unwrap();
        let runner = FakeRunner::new(vec![Ok(json!({
            "ok": true,
            "data": {"messages": [], "count": 9}
        }))]);
        let mut output = Vec::new();
        execute(&cli, &runner, &mut "".as_bytes(), &mut output).unwrap();
        let value: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(value["data"]["count"], 9);
    }

    #[test]
    fn unread_count_human_output_is_only_the_number() {
        let cli = Cli::try_parse_from(["osamail", "unread", "--count"]).unwrap();
        let runner = FakeRunner::new(vec![Ok(json!({
            "ok": true,
            "data": {"messages": [], "count": 9}
        }))]);
        let mut output = Vec::new();

        execute(&cli, &runner, &mut "".as_bytes(), &mut output).unwrap();

        assert_eq!(String::from_utf8(output).unwrap(), "9\n");
    }

    #[test]
    fn unicode_send_request_serializes_without_source_interpolation() {
        let cli = Cli::try_parse_from([
            "osamail",
            "send",
            "--to",
            "user@example.com",
            "--subject",
            "你好 🚀",
            "--body",
            "line 1\n\"quoted\"",
        ])
        .unwrap();
        let runner = FakeRunner::new(vec![Ok(json!({
            "ok": true,
            "data": {
                "sent": true,
                "dry_run": false,
                "account": null,
                "recipient_count": 1
            }
        }))]);
        execute(&cli, &runner, &mut "".as_bytes(), &mut Vec::new()).unwrap();
        let requests = runner.requests.lock().unwrap();
        assert_eq!(requests[0]["subject"], "你好 🚀");
        assert_eq!(requests[0]["body"], "line 1\n\"quoted\"");
    }

    #[test]
    fn dry_run_never_calls_send_without_account_validation() {
        let cli = Cli::try_parse_from([
            "osamail",
            "send",
            "--to",
            "user@example.com",
            "--body",
            "secret",
            "--dry-run",
            "--json",
        ])
        .unwrap();
        let runner = FakeRunner::new(Vec::new());
        let mut output = Vec::new();
        execute(&cli, &runner, &mut "".as_bytes(), &mut output).unwrap();
        assert!(runner.requests.lock().unwrap().is_empty());
        let value: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(value["data"]["dry_run"], true);
        assert_eq!(value["data"]["sent"], false);
    }

    #[test]
    fn automation_error_codes_map_to_exit_codes() {
        let cli = Cli::try_parse_from(["osamail", "accounts"]).unwrap();
        let runner = FakeRunner::new(vec![Ok(json!({
            "ok": false,
            "error": {
                "code": "AUTOMATION_PERMISSION_DENIED",
                "message": "denied"
            }
        }))]);
        let error = execute(&cli, &runner, &mut "".as_bytes(), &mut Vec::new()).unwrap_err();
        assert_eq!(error.exit_code(), 4);
    }
}
