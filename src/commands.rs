use std::{
    fs,
    io::{Read, Write},
    time::Duration,
};

use serde::{Serialize, de::DeserializeOwned};

use crate::{
    automation::{AutomationRunner, Script},
    cli::{
        Cli, Command, ListArgs, MailboxesArgs, MarkActionArg, MarkArgs, OrganizeArgs, SearchArgs,
        SendArgs, ShowArgs, UnreadArgs,
    },
    error::OsaMailError,
    model::{
        AccountsData, AccountsOutput, AutomationRequest, AutomationResponse, BatchItemError,
        CountOutput, DoctorAutomationData, DoctorCheck, DoctorReport, ListMailboxesRequest,
        ListMessagesRequest, ListMode, MailboxLocator, MailboxSummary, MailboxesData,
        MailboxesOutput, MarkAction, MarkAutomationData, MarkBatchItem, MarkBatchResult,
        MarkMessageRequest, MarkOutcome, MarkResult, MessageDetail, MessageSummary, MessagesData,
        MessagesOutput, MoveAutomationData, MoveMessageRequest, OpenMessageRequest, OpenResult,
        OrganizationAction, OrganizationItem, OrganizationOutcome, OrganizationResult,
        REFERENCE_VERSION, RawMessageDetail, SendRequest, SendResult, ShowMessageRequest,
        TitlesOutput,
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
        Command::Mailboxes(args) => mailboxes(cli, runner, output_writer, timeout, args),
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
        Command::Mark(args) => mark(cli, runner, output_writer, timeout, args),
        Command::Move(args) => organize(
            cli,
            runner,
            output_writer,
            timeout,
            args,
            OrganizationAction::Move,
        ),
        Command::Archive(args) => organize(
            cli,
            runner,
            output_writer,
            timeout,
            args,
            OrganizationAction::Archive,
        ),
        Command::Send(args) => send(cli, runner, input, output_writer, timeout, args),
    }
}

fn mailboxes(
    cli: &Cli,
    runner: &dyn AutomationRunner,
    writer: &mut dyn Write,
    timeout: Duration,
    args: &MailboxesArgs,
) -> Result<(), OsaMailError> {
    let data: MailboxesData = run_automation(
        runner,
        Script::ListMailboxes,
        &AutomationRequest::ListMailboxes(ListMailboxesRequest {
            account: args.account.clone(),
        }),
        timeout,
    )?;
    let mut mailboxes = data
        .mailboxes
        .into_iter()
        .map(|raw| {
            let locator = MailboxLocator {
                kind: reference::MAILBOX_REFERENCE_KIND.to_owned(),
                version: REFERENCE_VERSION,
                account: raw.account.clone(),
                mailbox_path: raw.path.clone(),
            };
            Ok(MailboxSummary {
                reference: reference::encode_mailbox(&locator)?,
                account: raw.account,
                path: raw.path,
            })
        })
        .collect::<Result<Vec<_>, OsaMailError>>()?;
    mailboxes.sort_by(|left, right| {
        left.account
            .cmp(&right.account)
            .then_with(|| left.path.cmp(&right.path))
    });

    if cli.json {
        output::write_json_success(writer, MailboxesOutput { mailboxes })
    } else if !cli.quiet {
        output::write_mailboxes(writer, &mailboxes)
    } else {
        Ok(())
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
            titles_only: args.titles,
            account: args.account.clone(),
            mailbox: args.mailbox.clone(),
            count_only: false,
            query: None,
            unread: false,
            from: None,
            subject: None,
            search_body: false,
            since: args.since.clone(),
            before: args.before.clone(),
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
        titles_only: args.titles,
        account: args.account.clone(),
        mailbox: args.mailbox.clone(),
        count_only: args.count,
        query: None,
        unread: true,
        from: None,
        subject: None,
        search_body: false,
        since: args.since.clone(),
        before: args.before.clone(),
    };
    validate_date_range(request.since.as_deref(), request.before.as_deref())?;
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
    if args.query.is_none()
        && args.from.is_none()
        && args.subject.is_none()
        && args.since.is_none()
        && args.before.is_none()
    {
        return Err(OsaMailError::InvalidArguments(
            "provide a query, --from, --subject, --since, or --before".to_owned(),
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
            titles_only: args.titles,
            account: args.account.clone(),
            mailbox: args.mailbox.clone(),
            count_only: false,
            query: args.query.clone(),
            unread: args.unread,
            from: args.from.clone(),
            subject: args.subject.clone(),
            search_body: args.body,
            since: args.since.clone(),
            before: args.before.clone(),
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
    validate_date_range(request.since.as_deref(), request.before.as_deref())?;
    let titles_only = request.titles_only;
    let data: MessagesData = run_automation(
        runner,
        Script::ListMessages,
        &AutomationRequest::ListMessages(request),
        timeout,
    )?;
    if titles_only {
        return if cli.json {
            output::write_json_success(
                writer,
                TitlesOutput {
                    count: data.count,
                    titles: data.titles,
                },
            )
        } else if !cli.quiet {
            output::write_titles(writer, &data.titles)
        } else {
            Ok(())
        };
    }
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

fn validate_date_range(since: Option<&str>, before: Option<&str>) -> Result<(), OsaMailError> {
    if let (Some(since), Some(before)) = (since, before)
        && since >= before
    {
        return Err(OsaMailError::InvalidArguments(
            "--since must be earlier than --before".to_owned(),
        ));
    }
    Ok(())
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

fn mark(
    cli: &Cli,
    runner: &dyn AutomationRunner,
    writer: &mut dyn Write,
    timeout: Duration,
    args: &MarkArgs,
) -> Result<(), OsaMailError> {
    let action = match args.action {
        MarkActionArg::Read => MarkAction::Read,
        MarkActionArg::Unread => MarkAction::Unread,
        MarkActionArg::Flag => MarkAction::Flag,
        MarkActionArg::Unflag => MarkAction::Unflag,
    };
    if args.references.len() == 1 {
        let result = mark_one(runner, timeout, &args.references[0], action, args.dry_run)?;
        return if cli.json {
            output::write_json_success(writer, result)
        } else if !cli.quiet {
            output::write_mark_result(writer, &result)
        } else {
            Ok(())
        };
    }

    let mut items = Vec::with_capacity(args.references.len());
    let mut succeeded = 0;
    for reference in &args.references {
        match mark_one(runner, timeout, reference, action, args.dry_run) {
            Ok(result) => {
                succeeded += 1;
                items.push(MarkBatchItem {
                    reference: result.reference,
                    outcome: Some(result.outcome),
                    error: None,
                });
            }
            Err(error) => items.push(MarkBatchItem {
                reference: reference.clone(),
                outcome: None,
                error: Some(BatchItemError {
                    code: error.code().to_owned(),
                    message: error.to_string(),
                    hint: error.hint().map(str::to_owned),
                }),
            }),
        }
    }
    let result = MarkBatchResult {
        action,
        dry_run: args.dry_run,
        total: items.len(),
        succeeded,
        failed: items.len() - succeeded,
        items,
    };

    if cli.json {
        output::write_json_success(writer, result)
    } else if !cli.quiet {
        output::write_mark_batch_result(writer, &result)
    } else {
        Ok(())
    }
}

fn mark_one(
    runner: &dyn AutomationRunner,
    timeout: Duration,
    reference: &str,
    action: MarkAction,
    dry_run: bool,
) -> Result<MarkResult, OsaMailError> {
    let locator = reference::decode(reference)?;
    let data: MarkAutomationData = run_automation(
        runner,
        Script::MarkMessage,
        &AutomationRequest::MarkMessage(MarkMessageRequest {
            locator,
            action,
            dry_run,
        }),
        timeout,
    )?;
    let outcome = if data.already_set {
        MarkOutcome::AlreadySet
    } else if dry_run {
        MarkOutcome::WouldChange
    } else {
        MarkOutcome::Changed
    };
    Ok(MarkResult {
        reference: reference.to_owned(),
        action,
        outcome,
    })
}

fn organize(
    cli: &Cli,
    runner: &dyn AutomationRunner,
    writer: &mut dyn Write,
    timeout: Duration,
    args: &OrganizeArgs,
    action: OrganizationAction,
) -> Result<(), OsaMailError> {
    let destination = reference::decode_mailbox(&args.to)?;
    if args.references.len() == 1 {
        let item = organize_one(
            runner,
            timeout,
            &args.references[0],
            &destination,
            args.dry_run,
        )?;
        let result = OrganizationResult {
            action,
            destination_reference: args.to.clone(),
            dry_run: args.dry_run,
            total: 1,
            succeeded: 1,
            failed: 0,
            items: vec![item],
        };
        return write_organization_result(cli, writer, &result);
    }

    let mut items = Vec::with_capacity(args.references.len());
    let mut succeeded = 0;
    for message_reference in &args.references {
        match organize_one(
            runner,
            timeout,
            message_reference,
            &destination,
            args.dry_run,
        ) {
            Ok(item) => {
                succeeded += 1;
                items.push(item);
            }
            Err(error) => items.push(OrganizationItem {
                reference: message_reference.clone(),
                outcome: None,
                error: Some(BatchItemError {
                    code: error.code().to_owned(),
                    message: error.to_string(),
                    hint: error.hint().map(str::to_owned),
                }),
            }),
        }
    }
    let result = OrganizationResult {
        action,
        destination_reference: args.to.clone(),
        dry_run: args.dry_run,
        total: items.len(),
        succeeded,
        failed: items.len() - succeeded,
        items,
    };
    write_organization_result(cli, writer, &result)
}

fn organize_one(
    runner: &dyn AutomationRunner,
    timeout: Duration,
    message_reference: &str,
    destination: &MailboxLocator,
    dry_run: bool,
) -> Result<OrganizationItem, OsaMailError> {
    let locator = reference::decode(message_reference)?;
    if locator.account != destination.account {
        return Err(OsaMailError::InvalidArguments(
            "source and destination accounts must match".to_owned(),
        ));
    }
    let data: MoveAutomationData = run_automation(
        runner,
        Script::MoveMessage,
        &AutomationRequest::MoveMessage(MoveMessageRequest {
            locator,
            destination: destination.clone(),
            dry_run,
        }),
        timeout,
    )?;
    let outcome = if data.already_there {
        OrganizationOutcome::AlreadyThere
    } else if dry_run {
        OrganizationOutcome::WouldMove
    } else {
        OrganizationOutcome::Moved
    };
    Ok(OrganizationItem {
        reference: message_reference.to_owned(),
        outcome: Some(outcome),
        error: None,
    })
}

fn write_organization_result(
    cli: &Cli,
    writer: &mut dyn Write,
    result: &OrganizationResult,
) -> Result<(), OsaMailError> {
    if cli.json {
        output::write_json_success(writer, result)
    } else if !cli.quiet {
        output::write_organization_result(writer, result)
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
    fn mailboxes_emit_sorted_opaque_destination_references() {
        let cli =
            Cli::try_parse_from(["osamail", "mailboxes", "--account", "iCloud 中文", "--json"])
                .unwrap();
        let runner = FakeRunner::new(vec![Ok(json!({
            "ok": true,
            "data": {
                "mailboxes": [
                    {"account": "iCloud 中文", "path": ["项目", "归档"]},
                    {"account": "iCloud 中文", "path": ["Inbox"]}
                ]
            }
        }))]);
        let mut output = Vec::new();

        execute(&cli, &runner, &mut "".as_bytes(), &mut output).unwrap();

        let requests = runner.requests.lock().unwrap();
        assert_eq!(requests[0]["operation"], "list_mailboxes");
        assert_eq!(requests[0]["account"], "iCloud 中文");
        let value: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(value["data"]["mailboxes"][0]["path"][0], "Inbox");
        let mailbox_reference = value["data"]["mailboxes"][1]["ref"].as_str().unwrap();
        let locator = reference::decode_mailbox(mailbox_reference).unwrap();
        assert_eq!(locator.account, "iCloud 中文");
        assert_eq!(locator.mailbox_path, ["项目", "归档"]);
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
    fn unread_titles_requests_minimal_output() {
        let cli = Cli::try_parse_from(["osamail", "unread", "--titles"]).unwrap();
        let runner = FakeRunner::new(vec![Ok(json!({
            "ok": true,
            "data": {
                "messages": [],
                "titles": ["First subject", "第二封"],
                "count": 2
            }
        }))]);
        let mut output = Vec::new();

        execute(&cli, &runner, &mut "".as_bytes(), &mut output).unwrap();

        assert_eq!(
            String::from_utf8(output).unwrap(),
            "First subject\n第二封\n"
        );
        assert_eq!(runner.requests.lock().unwrap()[0]["titles_only"], true);
    }

    #[test]
    fn search_titles_json_is_structured() {
        let cli =
            Cli::try_parse_from(["osamail", "search", "release", "--titles", "--json"]).unwrap();
        let runner = FakeRunner::new(vec![Ok(json!({
            "ok": true,
            "data": {
                "titles": ["Release complete"],
                "count": 1
            }
        }))]);
        let mut output = Vec::new();

        execute(&cli, &runner, &mut "".as_bytes(), &mut output).unwrap();

        let value: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(value["data"]["titles"][0], "Release complete");
        assert_eq!(value["data"]["count"], 1);
    }

    #[test]
    fn date_filters_are_serialized_without_requiring_a_search_query() {
        let cli = Cli::try_parse_from([
            "osamail",
            "search",
            "--since",
            "2024-01-01",
            "--before",
            "2024-02-01",
            "--titles",
        ])
        .unwrap();
        let runner = FakeRunner::new(vec![Ok(json!({
            "ok": true,
            "data": {"messages": [], "titles": [], "count": 0}
        }))]);

        execute(&cli, &runner, &mut "".as_bytes(), &mut Vec::new()).unwrap();

        let requests = runner.requests.lock().unwrap();
        assert_eq!(requests[0]["operation"], "list_messages");
        assert_eq!(requests[0]["mode"], "search");
        assert_eq!(requests[0]["query"], Value::Null);
        assert_eq!(requests[0]["since"], "2024-01-01");
        assert_eq!(requests[0]["before"], "2024-02-01");
    }

    #[test]
    fn invalid_date_range_is_rejected_before_mail_access() {
        for arguments in [
            vec![
                "osamail",
                "recent",
                "--since",
                "2024-02-01",
                "--before",
                "2024-02-01",
            ],
            vec![
                "osamail",
                "unread",
                "--count",
                "--since",
                "2024-03-01",
                "--before",
                "2024-02-01",
            ],
        ] {
            let cli = Cli::try_parse_from(arguments).unwrap();
            let runner = FakeRunner::new(Vec::new());

            let error = execute(&cli, &runner, &mut "".as_bytes(), &mut Vec::new()).unwrap_err();

            assert_eq!(error.code(), "INVALID_ARGUMENTS");
            assert!(runner.requests.lock().unwrap().is_empty());
        }
    }

    #[test]
    fn mark_dry_run_serializes_the_locator_without_applying_a_change() {
        let reference = reference::encode(&crate::model::MessageLocator {
            version: crate::model::REFERENCE_VERSION,
            account: "iCloud 中文".to_owned(),
            mailbox_path: vec!["Inbox".to_owned()],
            message_id: 42,
            internet_message_id: Some("message@example.test".to_owned()),
        })
        .unwrap();
        let cli =
            Cli::try_parse_from(["osamail", "mark", "read", &reference, "--dry-run", "--json"])
                .unwrap();
        let runner = FakeRunner::new(vec![Ok(json!({
            "ok": true,
            "data": {"already_set": false}
        }))]);
        let mut output = Vec::new();

        execute(&cli, &runner, &mut "".as_bytes(), &mut output).unwrap();

        let request = &runner.requests.lock().unwrap()[0];
        assert_eq!(request["operation"], "mark_message");
        assert_eq!(request["action"], "read");
        assert_eq!(request["dry_run"], true);
        assert_eq!(request["locator"]["account"], "iCloud 中文");
        let value: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(value["data"]["action"], "read");
        assert_eq!(value["data"]["outcome"], "would_change");
        assert_eq!(value["data"]["ref"], reference);
    }

    #[test]
    fn mark_batch_reports_each_result_and_continues_after_failure() {
        let first = reference::encode(&crate::model::MessageLocator {
            version: crate::model::REFERENCE_VERSION,
            account: "iCloud".to_owned(),
            mailbox_path: vec!["Inbox".to_owned()],
            message_id: 41,
            internet_message_id: None,
        })
        .unwrap();
        let second = reference::encode(&crate::model::MessageLocator {
            version: crate::model::REFERENCE_VERSION,
            account: "iCloud".to_owned(),
            mailbox_path: vec!["Inbox".to_owned()],
            message_id: 42,
            internet_message_id: None,
        })
        .unwrap();
        let cli = Cli::try_parse_from([
            "osamail",
            "mark",
            "flag",
            &first,
            &second,
            "--dry-run",
            "--json",
        ])
        .unwrap();
        let runner = FakeRunner::new(vec![
            Ok(json!({
                "ok": true,
                "data": {"already_set": false}
            })),
            Ok(json!({
                "ok": false,
                "error": {
                    "code": "MESSAGE_NOT_FOUND",
                    "message": "Message not found."
                }
            })),
        ]);
        let mut output = Vec::new();

        execute(&cli, &runner, &mut "".as_bytes(), &mut output).unwrap();

        let value: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(value["data"]["total"], 2);
        assert_eq!(value["data"]["succeeded"], 1);
        assert_eq!(value["data"]["failed"], 1);
        assert_eq!(value["data"]["items"][0]["outcome"], "would_change");
        assert_eq!(
            value["data"]["items"][1]["error"]["code"],
            "MESSAGE_NOT_FOUND"
        );
        assert_eq!(runner.requests.lock().unwrap().len(), 2);
    }

    #[test]
    fn archive_dry_run_uses_an_explicit_typed_destination() {
        let message_reference = reference::encode(&crate::model::MessageLocator {
            version: crate::model::REFERENCE_VERSION,
            account: "iCloud 中文".to_owned(),
            mailbox_path: vec!["Inbox".to_owned()],
            message_id: 42,
            internet_message_id: None,
        })
        .unwrap();
        let destination_reference = reference::encode_mailbox(&MailboxLocator {
            kind: reference::MAILBOX_REFERENCE_KIND.to_owned(),
            version: REFERENCE_VERSION,
            account: "iCloud 中文".to_owned(),
            mailbox_path: vec!["Archive".to_owned()],
        })
        .unwrap();
        let cli = Cli::try_parse_from([
            "osamail",
            "archive",
            "--to",
            &destination_reference,
            &message_reference,
            "--dry-run",
            "--json",
        ])
        .unwrap();
        let runner = FakeRunner::new(vec![Ok(json!({
            "ok": true,
            "data": {"already_there": false}
        }))]);
        let mut output = Vec::new();

        execute(&cli, &runner, &mut "".as_bytes(), &mut output).unwrap();

        let requests = runner.requests.lock().unwrap();
        assert_eq!(requests[0]["operation"], "move_message");
        assert_eq!(requests[0]["destination"]["kind"], "mailbox");
        assert_eq!(requests[0]["destination"]["mailbox_path"][0], "Archive");
        assert_eq!(requests[0]["dry_run"], true);
        let value: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(value["data"]["action"], "archive");
        assert_eq!(value["data"]["items"][0]["outcome"], "would_move");
    }

    #[test]
    fn move_batch_reports_invalid_items_without_skipping_valid_ones() {
        let message_reference = reference::encode(&crate::model::MessageLocator {
            version: crate::model::REFERENCE_VERSION,
            account: "iCloud".to_owned(),
            mailbox_path: vec!["Inbox".to_owned()],
            message_id: 42,
            internet_message_id: None,
        })
        .unwrap();
        let destination_reference = reference::encode_mailbox(&MailboxLocator {
            kind: reference::MAILBOX_REFERENCE_KIND.to_owned(),
            version: REFERENCE_VERSION,
            account: "iCloud".to_owned(),
            mailbox_path: vec!["Projects".to_owned()],
        })
        .unwrap();
        let cli = Cli::try_parse_from([
            "osamail",
            "move",
            "--to",
            &destination_reference,
            &message_reference,
            "invalid-reference",
            "--dry-run",
            "--json",
        ])
        .unwrap();
        let runner = FakeRunner::new(vec![Ok(json!({
            "ok": true,
            "data": {"already_there": false}
        }))]);
        let mut output = Vec::new();

        execute(&cli, &runner, &mut "".as_bytes(), &mut output).unwrap();

        let value: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(value["data"]["succeeded"], 1);
        assert_eq!(value["data"]["failed"], 1);
        assert_eq!(value["data"]["items"][0]["outcome"], "would_move");
        assert_eq!(
            value["data"]["items"][1]["error"]["code"],
            "INVALID_REFERENCE"
        );
        assert_eq!(runner.requests.lock().unwrap().len(), 1);
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
