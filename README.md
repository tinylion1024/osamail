# OsaMail — Apple Mail CLI for macOS

[English](README.md) | [简体中文](README.zh-CN.md)

Read, search, automate, and send Apple Mail from the terminal.

OsaMail is an open-source, local-first Apple Mail CLI for macOS. It controls the
accounts already configured in Mail through Apple's built-in `osascript`, so
you can automate email without setting up IMAP, SMTP, OAuth, or provider API
credentials.

```bash
brew install tinylion1024/tap/osamail
osamail doctor
osamail unread --titles
```

OsaMail can list, search, show, open, and send plain-text messages. It provides
clean terminal output for people and structured JSON for scripts. OsaMail is
not a standalone email client and does not connect directly to Gmail, iCloud
Mail, Exchange, or any other provider.

## Why OsaMail?

| Need | What OsaMail provides |
| --- | --- |
| Check email without leaving the terminal | Unread counts, compact subject lists, and recent messages |
| Automate an existing Apple Mail setup | Stable CLI commands and JSON output |
| Avoid another credentials flow | Uses accounts already authenticated in Apple Mail |
| Keep email access local | No telemetry, private Mail database access, or OsaMail network requests |
| Stay in control before sending | Plain-text send with a no-send `--dry-run` mode |

OsaMail is designed for developers, automation workflows, and macOS users who
want a small command-line interface instead of another email application.

## 30-second quick start

Configure at least one account in Apple Mail, then run:

```bash
# Confirm that macOS, Mail, and Automation permission are ready
osamail doctor

# See unread subjects without loading message bodies
osamail unread --titles

# Search subject and sender metadata
osamail search "invoice" --titles

# Get structured output for a script
osamail recent --limit 5 --json
```

Message lists return an opaque `ref`. Use that single shell-safe value to show
the message in the terminal or open it in Apple Mail:

```bash
osamail show <ref>
osamail open <ref>
```

## Choose a command

| Goal | Command |
| --- | --- |
| Check the environment | `osamail doctor` |
| List configured Mail accounts | `osamail accounts` |
| List recent messages | `osamail recent` |
| List unread messages | `osamail unread` |
| Count unread messages | `osamail unread --count` |
| Print only subjects | `osamail unread --titles` |
| Search messages | `osamail search "query"` |
| Read a message in the terminal | `osamail show <ref>` |
| Open a message in Apple Mail | `osamail open <ref>` |
| Validate a message without sending | `osamail send ... --dry-run` |
| Send a plain-text message | `osamail send ...` |

Run `osamail <command> --help` for the authoritative option list.

## Install

### Homebrew (recommended)

```bash
brew install tinylion1024/tap/osamail
```

The public tap is updated automatically after a versioned GitHub Release passes
its build and installation checks.

### GitHub Release

Download the universal macOS archive and adjacent SHA-256 file from
[GitHub Releases](https://github.com/tinylion1024/osamail/releases):

```bash
tar -xzf osamail-v0.2.0-universal-apple-darwin.tar.gz
install -m 0755 osamail-v0.2.0/osamail /usr/local/bin/osamail
osamail --version
```

The release binary supports Apple Silicon (`arm64`) and Intel (`x86_64`) Macs.
Use a destination already present in your `PATH`; `/opt/homebrew/bin` is another
common choice on Apple Silicon.

### Cargo

After version 0.2.0 is available on crates.io:

```bash
cargo install osamail
```

To install the current checkout:

```bash
cargo install --path .
```

### Requirements

- macOS with Apple Mail and `/usr/bin/osascript`.
- At least one account configured in Apple Mail for account or message commands.
- Automation permission for the terminal, IDE, or application running OsaMail.
- Rust 1.85 or newer only when building from source.

OsaMail 0.2.0 has been developed and live-tested against Mail 16.0 on macOS
15.3.

## Common workflows

### Read less, faster

Use `--titles` when subjects are enough. OsaMail asks Mail only for the minimum
properties needed to filter and sort results, then prints one subject per line:

```bash
osamail recent --titles
osamail unread --titles
osamail search "release" --titles
```

Listing commands do not load message bodies. `--titles` cannot be combined with
`unread --count`.

### Filter recent or unread mail

```bash
osamail recent --limit 20
osamail recent --account "Personal"
osamail unread --mailbox "INBOX"
osamail unread --count --json
```

The default list limit is 10; the accepted range is 1 through 200. Account names
must match Apple Mail exactly. Mailbox names may be localized or nested.

### Search Apple Mail

```bash
osamail search "GitHub"
osamail search "notice" --from "alerts@example.com"
osamail search "quarterly" --subject "report"
osamail search "security" --unread
osamail search "exact body text" --body
```

The positional query searches subject and sender metadata. `--from` and
`--subject` add filters. Body search is opt-in because large mailboxes can be
substantially slower.

### Show or open a message

```bash
osamail show <ref>
osamail show <ref> --headers
osamail show <ref> --max-body-bytes 131072
osamail show <ref> --json
osamail open <ref>
```

The human view includes the body by default and truncates it at 65,536 bytes.
JSON keeps the full body. Showing a message does not intentionally change its
read status or load attachments. References are locators, not durable message
IDs; get a fresh `ref` after messages move or accounts change.

### Send safely

Validate recipients and body input without creating or sending a message:

```bash
osamail send \
  --to user@example.com \
  --subject "Hello" \
  --body "Test message" \
  --dry-run
```

Remove `--dry-run` only when you intend to send. Repeat `--to`, `--cc`, or
`--bcc` for multiple recipients, select an exact account with `--account`, and
optionally provide a body with one of `--body`, `--body-file`, or `--stdin`.

A successful real send means Apple Mail accepted the request; it does not prove
remote delivery. Default and CI tests never send real email.

## JSON and shell automation

Add `--json` to any command for exactly one JSON value. Successful output uses
`{"ok":true,"data":...}` on stdout. Errors use
`{"ok":false,"error":{"code":...,"message":...}}` on stderr and include a
`hint` when OsaMail can suggest a concrete recovery step.

```bash
# Extract subjects
osamail search "invoice" --json | jq -r '.data.messages[].subject'

# Use a count in another command
unread_count="$(osamail unread --count --json | jq -r '.data.count')"
```

Global options can appear before or after a subcommand:

```text
--json               Emit structured JSON
--timeout <SECONDS>  Override the command timeout (1-3600)
--quiet              Suppress successful human-readable output
```

`--quiet` never hides errors or explicitly requested JSON. In an interactive
terminal, slow read commands print one delayed status message to stderr. Piped,
JSON, and quiet output remain unchanged.

Field names form the 0.2.0 machine-readable interface. Optional Mail values may
be `null` or omitted depending on the response model.

## macOS Automation permission

The first live command may ask for permission to control Mail. If access is
denied:

1. Open **System Settings → Privacy & Security → Automation**.
2. Allow the terminal, IDE, or application running OsaMail to control **Mail**.
3. Run `osamail doctor` again.

Permission belongs to the invoking application. Switching terminals or running
OsaMail from an IDE may trigger a separate prompt.

## Security and privacy

OsaMail uses the boundary:

```text
Rust → /usr/bin/osascript → embedded JXA → Apple Mail
```

Every request is serialized into an unpredictable temporary JSON file with mode
`0600`. User input is never interpolated into JXA, AppleScript, or a shell
command.

- No passwords, tokens, or server credentials are read.
- No private Apple Mail database is accessed.
- No telemetry or direct network requests are added by OsaMail.
- `/usr/bin/osascript` is invoked directly without `sh -c`.
- Temporary request files are removed after completion or failure.

Apple Mail remains responsible for communication with email providers. See
[SECURITY.md](SECURITY.md) for the complete data-handling model.

## Frequently asked questions

### Does OsaMail work with Gmail, iCloud Mail, or Exchange?

Yes, when the account is already configured and working in Apple Mail. OsaMail
controls Mail; it does not connect directly to the provider.

### Does OsaMail need my email password?

No. OsaMail never reads account passwords, OAuth tokens, or server credentials.

### Is OsaMail an IMAP or SMTP client?

No. It is a macOS-native command-line interface over Apple Mail automation.

### Does listing or showing email mark it as read?

Listing and default metadata search are read-only and do not load bodies.
`show` does not intentionally change read state. `open` hands the message to
Apple Mail, so the resulting UI behavior is controlled by Mail.

### Can OsaMail mark messages as read?

Not in version 0.2.0. Read-state changes and other mailbox mutations are outside
the current command set.

### Does OsaMail work on Linux or Windows?

Mail operations require macOS and Apple Mail. `--help` and `--version` remain
portable.

## Current limits

OsaMail does not currently support attachments, HTML rendering or composition,
reply, forward, delete, move, archive, read-state changes, flags, rules,
background notifications, signing, or encryption. Release artifacts are not
code-signed or notarized.

`open` and window focus depend on Apple Mail's scripting and current UI state.
Large-mailbox and body searches may require a larger `--timeout`.

## Project

- [Contributing](CONTRIBUTING.md)
- [Architecture](docs/architecture.md)
- [Release guide](docs/releasing.md)
- [Homebrew publishing](docs/homebrew.md)
- [Changelog](CHANGELOG.md)
- [Security policy](SECURITY.md)

Build and test locally:

```bash
cargo build
./scripts/check.sh
./scripts/smoke-test.sh
```

Live integration tests are read-only unless both explicit send-test gates are
enabled. See [CONTRIBUTING.md](CONTRIBUTING.md) for the full policy.

## License

OsaMail is available under the [MIT License](LICENSE).

OsaMail is an independent open-source project and is not affiliated with or
endorsed by Apple Inc. Apple, Apple Mail, and macOS are trademarks of Apple Inc.
