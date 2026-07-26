# Security policy

## Supported versions

Security fixes target the latest released version. Before the first public
release, fixes target the default branch.

## Report a vulnerability

Use the repository's **Security** tab to submit a private vulnerability report
through GitHub Security Advisories:

<https://github.com/tinylion1024/osamail/security/advisories/new>

If private reporting is unavailable, open a public issue that contains only a
request for a private contact channel. Do not include exploit details, message
bodies, email addresses, account names, opaque message references, or other
personal data in a public issue.

Include the OsaMail version, macOS version, Mail version, impact, and minimal
reproduction steps after a private channel is established. Maintainers will
acknowledge the report, investigate it, and coordinate disclosure; this project
does not promise a fixed response deadline.

## Security model

OsaMail controls accounts already configured in Apple Mail through the stable
boundary:

```text
OsaMail -> /usr/bin/osascript -> embedded JXA -> Apple Mail
```

It does not:

- connect directly to email servers;
- read account passwords, access tokens, or server credentials;
- access Apple Mail's private database;
- make its own network requests;
- send telemetry; or
- execute user-provided programs or scripts.

Apple Mail performs provider communication using the accounts and credentials it
already manages.

## Automation input handling

Every business request is serialized as JSON to an unpredictable temporary
file. On Unix platforms, OsaMail explicitly sets that file's permissions to
`0600`. The embedded JXA source is constant, and only the request file path is
passed as its business argument to the absolute `/usr/bin/osascript` path.

OsaMail never interpolates recipient addresses, subjects, bodies, search text,
account names, mailbox names, or message references into:

- JXA or AppleScript source;
- a shell command;
- process arguments; or
- automation error details.

The runner does not use `sh -c`. It captures stdout and stderr, checks process
status, enforces a timeout, and accepts one JSON response value.

The temporary file is deleted when the runner unwinds normally or with a handled
error. An abrupt process termination or system failure can prevent normal
cleanup, so users with a high-sensitivity threat model should also protect their
macOS temporary directory and user account.

## Mail data exposure

Read commands return message metadata, body text, or headers only when the
selected command requests them. Message lists do not load bodies by default.
`show` returns body text, and `show --headers` additionally returns raw textual
headers. `search --body` asks Mail to inspect content and may expose matching
content to the local automation process.

Opaque references are URL-safe Base64 encodings of a versioned locator, not
encryption. Treat them as message metadata and do not publish them.

JSON and human-readable results go to stdout. Errors and hints go to stderr.
Callers are responsible for securing redirected output, shell history, CI logs,
and downstream pipeline tools.

## Sending safeguards

`osamail send` sends a real plain-text message unless `--dry-run` is present.
Dry-run validates local inputs and does not create or send a message. If an
account is specified, dry-run queries Mail to confirm that the exact enabled
account exists.

Default tests never send email. CI must never set
`OSAMAIL_ALLOW_SEND_TEST=1`. Any future send integration test must require both
`OSAMAIL_INTEGRATION=1` and `OSAMAIL_ALLOW_SEND_TEST=1`.

## macOS Automation permission

macOS controls Apple Events access. OsaMail does not bypass, modify, or grant
Automation permission. Grant access only to a trusted terminal or launcher, and
review it under **System Settings -> Privacy & Security -> Automation**.
