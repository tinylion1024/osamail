# OsaMail

[English](README.md) | [简体中文](README.zh-CN.md)

A tiny, scriptable CLI for Apple Mail, powered by `osascript`.

OsaMail controls accounts already configured in Apple Mail. It does not connect
directly to Gmail, iCloud Mail, Exchange, IMAP, SMTP, or any other email
provider.

```text
$ osamail unread --count
12

$ osamail recent --limit 2
RECEIVED              STATUS   SENDER                        SUBJECT
2026-07-26T02:30:00Z  unread   GitHub <noreply@github.com>   New pull request
  ref: eyJ2ZXJzaW9uIjoxLC...
2026-07-25T18:04:00Z  read     Build service                 Release complete
  ref: eyJ2ZXJzaW9uIjoxLC...
```

The output above illustrates the terminal format. Account names, messages, and
opaque references come from the user's Apple Mail data.

## Features

- Diagnose the local Mail and macOS Automation environment.
- List Apple Mail accounts without reading credentials.
- List recent or unread messages without loading message bodies.
- Print only message subjects when a compact title list is enough.
- Search sender and subject metadata, with opt-in body search.
- Show a message as terminal text or structured JSON.
- Open a referenced message in Apple Mail.
- Send plain-text messages, with a no-send dry-run mode.
- Emit human-readable output for interactive use and JSON for scripts.

OsaMail is local-first, has no telemetry, and makes no network requests of its
own. Apple Mail remains responsible for communicating with email providers.

## Requirements

- macOS with `/System/Applications/Mail.app` and `/usr/bin/osascript`.
- At least one account configured in Apple Mail for account or message commands.
- Automation permission for the terminal or application that invokes OsaMail.
- Rust 1.85 or newer only when building from source.

OsaMail 0.1.2 has been developed and live-tested against Mail 16.0 on macOS
15.3. Automation authorization remains specific to the terminal or application
that invokes OsaMail.

## Installation

### Homebrew

```bash
brew install tinylion1024/tap/osamail
```

Version tags publish a GitHub Release and automatically update the formula in
the Homebrew tap after the release archive passes its build and installation
checks. Maintainers can follow
[the Homebrew publishing guide](docs/homebrew.md) for setup and recovery details.

### GitHub Release

After a release is published, download
`osamail-v0.1.2-universal-apple-darwin.tar.gz` from the repository's Releases
page, verify its adjacent SHA-256 file, and install the binary:

```bash
tar -xzf osamail-v0.1.2-universal-apple-darwin.tar.gz
install -m 0755 osamail-v0.1.2/osamail /usr/local/bin/osamail
osamail --version
```

On Apple Silicon, `/opt/homebrew/bin` is another common user-managed destination.
Use a directory already present in your `PATH`; administrator access may be
required for `/usr/local/bin`.

### Cargo

After version 0.1.2 is published to crates.io:

```bash
cargo install osamail
```

To install the current checkout without waiting for a registry release:

```bash
cargo install --path .
```

No release has been published by the repository automation during development.

## Quick start

Configure the desired accounts in Apple Mail first, then run:

```bash
osamail doctor
osamail accounts
osamail unread --count
osamail recent --limit 5
osamail search "GitHub"
osamail show <ref>
osamail open <ref>
```

`recent`, `unread`, and `search` return an opaque `ref` for each message. Pass
that single shell-safe value to `show` or `open`.

## Commands

Global options may appear before or after a subcommand:

```text
--json               Emit structured JSON
--timeout <SECONDS>  Override the command-specific timeout (1-3600)
--quiet              Suppress successful human-readable output
```

`--quiet` does not hide errors and does not suppress requested JSON.

### Diagnose the environment

```bash
osamail doctor
osamail doctor --json
osamail --timeout 30 doctor
```

The check covers macOS, `/usr/bin/osascript`, Mail.app, Mail automation, and the
configured account count.

### List accounts

```bash
osamail accounts
osamail accounts --json
```

Account output includes only the account name, configured email addresses, and
enabled state. It never includes passwords, tokens, or server credentials.

### List recent messages

```bash
osamail recent
osamail recent --limit 10
osamail recent --titles
osamail recent --account "Personal"
osamail recent --mailbox "INBOX"
osamail recent --account "Personal" --mailbox "Receipts" --json
```

The default limit is 10; the accepted range is 1 through 200. Message bodies are
not loaded.

### List or count unread messages

```bash
osamail unread
osamail unread --limit 20
osamail unread --titles
osamail unread --account "Personal"
osamail unread --mailbox "INBOX"
osamail unread --count
osamail unread --count --json
```

Human-readable `--count` output is one integer.

### Search messages

```bash
osamail search "GitHub"
osamail search "invoice" --account "Personal"
osamail search "release" --limit 20
osamail search "release" --titles
osamail search "security" --unread
osamail search "notice" --from "alerts@example.com"
osamail search "quarterly" --subject "report"
osamail search "exact body text" --body
```

The positional query searches subject and sender by default. `--from` and
`--subject` add filters. `--body` also searches Mail's message content and can
be substantially slower; OsaMail does not transfer every body to Rust for
searching. `--titles` asks Mail for the minimum properties needed to filter and
sort the results, then prints one subject per line. In JSON mode it returns
`data.titles` and `data.count`.

### Show a message

```bash
osamail show <ref>
osamail show <ref> --body
osamail show <ref> --headers
osamail show <ref> --max-body-bytes 131072
osamail show <ref> --json
```

The human view includes the body by default and truncates it at 65,536 bytes.
`--body` is accepted when a caller wants to make that default explicit. Use
`--max-body-bytes` to change the human display limit. JSON keeps the full body.
`--headers` requests Mail's raw textual headers. Showing a message does not
intentionally change its read status or load attachments.

### Open a message in Mail

```bash
osamail open <ref>
```

OsaMail validates the reference, asks Mail to open the matching message, and
activates Mail. Window focus and selection remain subject to Mail's current UI
state.

### Send plain text

The following command sends a real message:

```bash
osamail send \
  --to user@example.com \
  --subject "Hello" \
  --body "Test message"
```

Repeat recipient flags and select an exact Apple Mail account when needed:

```bash
osamail send \
  --to first@example.com \
  --to second@example.com \
  --cc copy@example.com \
  --bcc audit@example.com \
  --account "Personal" \
  --subject "Status" \
  --body "Complete"
```

Read the body from a file or standard input:

```bash
osamail send \
  --to user@example.com \
  --subject "Hello" \
  --body-file message.txt

printf '%s\n' 'Test message' \
  | osamail send --to user@example.com --subject "Hello" --stdin
```

`--body`, `--body-file`, and `--stdin` are mutually exclusive. Validate
recipients and body input without creating or sending a message:

```bash
osamail send \
  --to user@example.com \
  --subject "Hello" \
  --body "Test message" \
  --dry-run
```

When `--account` is present, even dry-run checks that an enabled account with
that exact name exists and therefore requires Mail automation permission. A
successful real send means Apple Mail accepted the request; it is not proof of
remote delivery.

Run `osamail <command> --help` for the authoritative option list.

## JSON output

`--json` writes exactly one JSON value to stdout. Successful responses use
`{"ok":true,"data":...}`; failures use
`{"ok":false,"error":{"code":...,"message":...}}` and go to stderr.

For example, a successful no-send dry-run produces:

```json
{
  "ok": true,
  "data": {
    "sent": false,
    "dry_run": true,
    "account": null,
    "recipient_count": 1
  }
}
```

Message-list data has this shape:

```json
{
  "ok": true,
  "data": {
    "messages": [
      {
        "ref": "opaque-reference",
        "account": "Personal",
        "mailbox": "INBOX",
        "sender": "GitHub <noreply@github.com>",
        "subject": "New pull request",
        "received_at": "2026-07-26T02:30:00.000Z",
        "unread": true
      }
    ],
    "count": 1
  }
}
```

Field names form the 0.1.2 machine-readable interface. Optional Mail values may
be `null` or omitted depending on the response model.

## Shell pipelines

Extract subjects with `jq`:

```bash
osamail search "invoice" --json \
  | jq -r '.data.messages[].subject'
```

Count unread mail as a number:

```bash
osamail unread --count --json \
  | jq -r '.data.count'
```

Keep JSON stdout separate from diagnostics:

```bash
if ! result="$(osamail recent --json)"; then
  printf '%s\n' "OsaMail failed" >&2
  exit 1
fi
printf '%s\n' "$result" | jq '.data.messages'
```

## macOS Automation permission

The first live command may cause macOS to request permission for the invoking
terminal or application to control Mail. OsaMail cannot grant this permission.

If access is denied or the command cannot proceed:

1. Open **System Settings**.
2. Open **Privacy & Security**.
3. Open **Automation**.
4. Allow the invoking terminal application to control **Mail**.
5. Run `osamail doctor` again.

Authorization is associated with the invoking application, so a different
terminal, IDE, or packaged launcher may need separate consent.

## Apple Mail accounts

OsaMail does not maintain account configuration. Add, remove, enable, and
authenticate accounts in Apple Mail. Values passed through `--account` must
match the Mail account name exactly; OsaMail does not silently fall back to a
different account.

Mailbox names come from Mail and may be localized or nested. A name such as
`INBOX` is an example, not a universal mailbox name.

## Security and privacy

Rust serializes each automation request into an unpredictable temporary JSON
file with mode `0600`. Only the file path is passed as business input to the
constant, embedded JXA source. User input is never interpolated into JXA,
AppleScript, or a shell command.

OsaMail:

- invokes the absolute `/usr/bin/osascript` path without `sh -c`;
- captures subprocess output, checks its status, and enforces a timeout;
- removes the request file when the runner completes normally or with an error;
- does not read account passwords or access Mail's private database;
- does not add telemetry or make network requests; and
- does not send mail during default tests.

See [SECURITY.md](SECURITY.md) for the data-handling model and vulnerability
reporting process.

## Known limitations

- Real commands require macOS, Apple Mail, configured accounts, and Automation
  permission. Help and version output remain portable.
- Live read-only validation covered `doctor`, `accounts`, recent and unread
  listing, metadata search, `show`, and `unread --count` on Mail 16.0. `open`
  was validated with an already-read message. Real sending was deliberately not
  performed; `send --dry-run` covered input and JSON behavior without creating
  an outgoing message.
- Opaque references encode a versioned Mail locator. They are safe as one shell
  argument but are not durable identifiers; mailbox moves, account changes, or
  Mail database changes can invalidate them.
- `open` relies on Mail's scripting behavior. Mail may open the message without
  focusing the expected viewer or preserving a particular selection.
- Mail provides message content as text/rich text. OsaMail does not render HTML,
  load attachments, or expose raw MIME.
- Body search and large mailboxes can be slow. Increase `--timeout` when needed.
- Version 0.1.2 does not support attachments, reply, forward, delete, move,
  archive, read-state changes, flags, rules, notifications, templates, HTML
  composition, signing, or encryption.
- The release is not code-signed or notarized.

Resolved external validation issues are recorded in [BLOCKERS.md](BLOCKERS.md).

## Development

Clone the repository and build with stable Rust:

```bash
git clone https://github.com/tinylion1024/osamail.git
cd osamail
cargo build
cargo run -- --help
```

The automation boundary is Rust -> `/usr/bin/osascript` -> embedded JXA ->
Apple Mail. See [the architecture guide](docs/architecture.md) and
[the Mail automation investigation](docs/apple-mail-automation.md).

## Testing

Default checks use mocks and never send real email:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo test --doc
./scripts/check.sh
./scripts/smoke-test.sh
```

On a macOS host with Automation permission, run the ignored, read-only
integration checks explicitly:

```bash
OSAMAIL_INTEGRATION=1 cargo test --test macos_integration -- --ignored
```

No send integration test runs without both `OSAMAIL_INTEGRATION=1` and
`OSAMAIL_ALLOW_SEND_TEST=1`. Do not enable send testing in CI. See
[CONTRIBUTING.md](CONTRIBUTING.md) for the full validation policy.

## Releasing

Maintainers must run the complete local gate, update the changelog, publish the
crate, and create a `v*` tag. The GitHub release workflow builds the release and
updates the Homebrew tap automatically. No development command performs an
actual publication.

See [docs/releasing.md](docs/releasing.md) for the ordered checklist and
[docs/homebrew.md](docs/homebrew.md) for tap maintenance.

## Roadmap

Possible post-0.1.2 work:

- code signing and notarization;
- stronger integration coverage across supported macOS and Mail versions;
- measured improvements for large-mailbox queries; and
- selected mail operations only after their safety and scripting behavior are
  verified.

Attachments, reply/forward, mutations, and background behavior are not promised.

## License

OsaMail is available under the [MIT License](LICENSE).

## Independent project

OsaMail is an independent open-source project and is not affiliated with or
endorsed by Apple Inc. Apple, Apple Mail, and macOS are trademarks of Apple Inc.
