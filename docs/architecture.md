# Architecture

OsaMail is a single-process Rust CLI that delegates Apple Mail operations to the
macOS scripting interface.

```text
Shell or local tool
        |
        v
Rust CLI and command coordination
        |
        | constant script + path to 0600 JSON request
        v
/usr/bin/osascript -l JavaScript
        |
        v
Apple Events -> Apple Mail
```

Apple Mail owns account configuration, credentials, message storage, and
provider communication. OsaMail is not a mail client, daemon, server, database,
or plugin host.

## Component responsibilities

| Component | Responsibility |
| --- | --- |
| `src/cli.rs` | clap command definitions, global options, argument relationships, defaults, and limits |
| `src/commands.rs` | command coordination, semantic validation, typed requests, references, and response conversion |
| `src/automation/runner.rs` | private request files, `/usr/bin/osascript`, timeout and process handling, and JSON parsing |
| `src/automation/scripts/*.js` | Mail-specific JXA operations and structured protocol responses |
| `src/model.rs` | serializable request, response, account, message, doctor, and send models |
| `src/reference.rs` | versioned opaque message-reference encoding, decoding, and validation |
| `src/output.rs` | human-readable and JSON output, body truncation, and error envelopes |
| `src/error.rs` | stable error categories, exit codes, and actionable hints |
| `src/main.rs` | stdin/stdout/stderr wiring and process exit status |

Scripts are embedded with `include_str!`. An installed binary does not need the
repository's `src/automation/scripts` directory.

## Command flow

1. clap parses the command and validates structural constraints such as limit
   ranges and mutually exclusive body inputs.
2. `commands.rs` performs semantic checks, reads an explicitly selected body
   source, and constructs a typed request.
3. `commands.rs` serializes the request to `serde_json::Value` and invokes the
   `AutomationRunner` trait with a command-specific timeout.
4. `OsascriptRunner` writes the request to an unpredictable temporary JSON file
   and sets mode `0600`.
5. The runner launches the absolute `/usr/bin/osascript` path directly. The
   arguments select JavaScript, provide the constant embedded source, and pass
   the temporary path.
6. JXA reads and parses the file, calls Apple Mail, and emits one JSON response.
7. Rust checks process status, parses the response, converts it to typed models,
   and writes either human-readable output or one JSON envelope.
8. Dropping the temporary-file guard removes the request file on normal and
   handled-error paths.

The subprocess uses piped stdout and stderr, null stdin, and a polling timeout.
On timeout, Rust kills and waits for the child before returning exit code 6.

## Automation protocol

Requests are internally tagged by operation. For example:

```json
{
  "operation": "list_messages",
  "mode": "search",
  "limit": 10,
  "account": "Personal",
  "mailbox": null,
  "count_only": false,
  "query": "GitHub",
  "unread": false,
  "from": null,
  "subject": null,
  "search_body": false
}
```

Successful scripts return:

```json
{
  "ok": true,
  "data": {}
}
```

Expected failures return a machine-readable code:

```json
{
  "ok": false,
  "error": {
    "code": "ACCOUNT_NOT_FOUND",
    "message": "Account not found."
  }
}
```

Rust does not use localized natural-language text as the primary protocol.
Malformed output and unknown script failures map to explicit Rust error
categories.

## Security invariants

- Business values never appear in script source or process arguments.
- The request path is the only business argument passed to JXA.
- Request filenames are unpredictable and files are restricted to `0600`.
- The runner invokes `/usr/bin/osascript` with `std::process::Command`.
- No shell parses the command; OsaMail never uses `sh -c`.
- Both output streams are captured, subprocess status is checked, and execution
  is time-bounded.
- OsaMail does not read passwords, query Mail's private database, add telemetry,
  or make network requests.
- Default and CI tests never send real email.

These are product contracts, not implementation preferences. Changes that weaken
them require rejection or a new reviewed security design.

## JXA and AppleScript

All 0.1.0 commands use JXA. JXA provides direct `JSON.parse` and
`JSON.stringify`, supports a `run(argv)` entry point, and bridges Mail's
scripting dictionary without text interpolation.

AppleScript remains an allowed command-specific fallback only if a reproducible
JXA incompatibility is demonstrated. A fallback must retain the same temporary
JSON protocol, structured response, subprocess checks, and test boundary. See
[apple-mail-automation.md](apple-mail-automation.md) for the dictionary evidence.

## Message references

A message list returns `ref`, a URL-safe unpadded Base64 encoding of a versioned
JSON locator:

```json
{
  "version": 1,
  "account": "Personal",
  "mailbox_path": ["Inbox"],
  "message_id": 42,
  "internet_message_id": "<example@example.com>"
}
```

Rust validates the version, account, mailbox path, positive Mail integer ID, and
shell-safe size before automation. The reference carries location data, not
executable code, and is opaque rather than encrypted.

References are deliberately not permanent. Mailbox moves, renames, account
changes, or Mail database changes can invalidate them. `show` and `open`
cross-check the optional internet message ID when present.

## Query behavior

List and metadata-search scripts request vectorized Mail property arrays, then
filter, sort by received time, and limit to 1 through 200 entries inside JXA.
Explicit body search uses a Mail `whose` predicate so matching content is not
copied into Rust. Bodies are omitted from list rows and are fetched only by
`show` or inspected by explicit `search --body`.

When no mailbox is specified, listing uses Mail's aggregate inbox. Named
mailboxes are resolved recursively and may match localized or nested names.
Supplying an account constrains lookup to that exact account name.

## Output and errors

Normal results use stdout. Errors and remediation hints use stderr.
Human-readable `--quiet` suppresses successful output; requested JSON remains
available.

JSON success and error values use stable envelopes:

```text
{"ok":true,"data":...}
{"ok":false,"error":{"code":...,"message":...,"hint":...}}
```

Exit codes are grouped by caller action:

| Code | Meaning |
| ---: | --- |
| 0 | Success |
| 1 | I/O or serialization failure |
| 2 | Invalid arguments or message reference |
| 3 | Unsupported platform or missing macOS component |
| 4 | Automation permission denied |
| 5 | Account, mailbox, or message not found |
| 6 | `osascript` timeout |
| 7 | Script failure or invalid script output |

## Platform boundary

Mail operations are macOS-only. The crate avoids a top-level compile error so
`osamail --help`, `osamail --version`, compilation, and static analysis remain
available on other platforms. A Mail command outside macOS returns
`OsaMail only supports macOS.` with exit code 3.

## Test boundary

Command tests inject an `AutomationRunner` replacement and assert typed request
and output behavior without Apple Mail. Runner tests use a local executable
fixture to verify arguments, JSON parsing, errors, permissions, and timeout
handling. JXA syntax checks compile embedded script files without executing Mail
operations.

Live integration is ignored by default and gated with
`OSAMAIL_INTEGRATION=1`. Any real-send test requires the additional
`OSAMAIL_ALLOW_SEND_TEST=1` gate and must never run in CI.
