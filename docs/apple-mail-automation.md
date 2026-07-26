# Apple Mail automation investigation

Investigation date: 2026-07-26
Host: macOS 15.3 (24D2059), Apple Silicon
Installed Mail version: 16.0

## Evidence sources

The active developer directory contains Command Line Tools rather than full
Xcode, so invoking `sdef /System/Applications/Mail.app` reports that `sdef`
requires Xcode. Mail ships its authoritative scripting definition directly at:

```text
/System/Applications/Mail.app/Contents/Resources/Mail.sdef
```

The findings below come from that installed dictionary, the implemented scripts,
offline `osacompile` checks, and local JXA protocol probes. They are not inferred
from remembered Mail behavior.

Apple's archived JXA release notes describe `Application(...)`, lazy object
specifiers, `whose` filters, object construction, `run(argv)`, and the Objective-C
bridge. The installed Mail dictionary is the stronger source for current
Mail-specific class, property, and command names.

## Confirmed dictionary surface

| Mail term | Confirmed surface | OsaMail use |
| --- | --- | --- |
| application | accounts, aggregate inbox, outgoing messages, read-only selection | account discovery, default inbox queries, draft creation, activation |
| account | `name`, string `id`, `email addresses`, `enabled`, contained mailboxes | account output and exact-name selection |
| mailbox | nested mailboxes, messages, `name`, `unread count`, `account`, `container` | recursive lookup, count, and locator paths |
| message | integer `id`, mailbox, content, date received, read status, message ID, sender, subject, recipients, all headers | list metadata, references, `show`, and `open` |
| outgoing message | writable sender, subject, content, visibility, and to/cc/bcc recipient elements | plain-text sending |
| recipient | display name and address; specialized to/cc/bcc classes | message detail and outgoing recipients |
| commands | message `open`, outgoing-message `send`, application activation | `open` and `send` |

The account dictionary contains a password property but explicitly states that
it cannot be read through scripting. OsaMail never requests or sets it.

The dictionary describes message content as rich text. JXA exposes it to OsaMail
as text; this is not raw MIME, attachment data, or an HTML-rendering contract.

## JXA object behavior used

JXA application properties are lazy object specifiers. A property call such as
`message.subject()` fetches a value, while collections can be filtered before
they are resolved:

```javascript
var unread = Mail.inbox.messages.whose({ readStatus: false });
var messages = unread();
```

For ordinary metadata queries, OsaMail asks Mail for vectorized property arrays
and filters those values inside JXA. This avoids one Apple Event per property per
message while keeping mailbox-scale filtering and limiting out of Rust. Explicit
body search uses a Mail `whose` predicate combining subject, sender, and content;
message bodies are not transferred to Rust for searching.

The dictionary does not specify the order of a mailbox's `messages` collection.
OsaMail therefore reads each matched row's received date, sorts rows in JXA, and
then applies the requested limit. It does not rely on an undocumented range
method for “latest N” semantics.

## Secure JSON request experiment

The implemented invocation is:

```text
/usr/bin/osascript -l JavaScript -e <constant-embedded-script> <request-path>
```

The constant script implements `run(argv)`, requires exactly one argument, and
reads `argv[0]` through Foundation:

```javascript
ObjC.import("Foundation");

function run(argv) {
    var text = $.NSString.stringWithContentsOfFileEncodingError(
        argv[0],
        $.NSUTF8StringEncoding,
        null
    );
    var request = JSON.parse(ObjC.unwrap(text));
    return JSON.stringify({ ok: true, data: request });
}
```

Rust creates the unpredictable request file, sets mode `0600`, writes JSON, and
passes only the path. The protocol probe round-tripped Chinese, emoji, quotes,
backslashes, tabs, and newlines. User values never appear in the embedded source
or process argument list.

Each production script returns one JSON envelope:

```json
{"ok":true,"data":{}}
```

or:

```json
{
  "ok": false,
  "error": {
    "code": "AUTOMATION_PERMISSION_DENIED",
    "message": "Apple Events automation permission was denied."
  }
}
```

Known failures such as account, mailbox, message, request, and permission errors
use machine-readable codes. Rust treats malformed output and unknown failures as
protocol or script errors rather than parsing localized Mail text as success.

## Language decision

JXA is the primary and only automation language used by 0.1.0:

| Operation | Script |
| --- | --- |
| diagnostics | `doctor.js` |
| accounts | `accounts.js` |
| recent, unread, search | `list_messages.js` |
| show | `show_message.js` |
| open | `open_message.js` |
| send | `send_message.js` |

JXA provides native JSON parsing and serialization and a direct `run(argv)`
entry point, keeping structured data separate from script source. No current
operation needs AppleScript.

A command-specific AppleScript fallback is allowed only after a reproducible JXA
failure. It must keep the same `0600` request file, path-only argument,
structured JSON response, timeout, and tests.

## Operation notes

### Accounts and mailboxes

Account selection compares the exact Mail account name. A named mailbox is
resolved recursively because mailbox names may be localized and nested. Names
can collide; including `--account` narrows mailbox lookup and is safer when the
same name exists in multiple accounts.

The locator records the full mailbox path available through `container`, not an
assumed English `INBOX` name.

### Message identity

Mail exposes an integer message `id` and an RFC-style message ID string. OsaMail
stores the account, mailbox path, integer ID, and optional message ID in each
opaque reference. `show` and `open` resolve the path and integer ID and
cross-check the optional message ID.

The dictionary does not promise that these identifiers or paths survive moves,
renames, account changes, or Mail database rebuilds. References are best-effort
locators, not permanent IDs.

### Reading

List operations request sender, subject, received date, read status, and locator
fields only. `show` reads content and recipients; `--headers` additionally reads
`all headers`. The scripts read `read status` but never assign it, so OsaMail
does not intentionally mark messages read.

### Opening

The message class explicitly responds to `open`, and the standard application
surface supports activation. OsaMail calls the message's `open()` and then
activates Mail.

The scripting contract does not guarantee a particular viewer, window focus, or
selection state. OsaMail reports that Mail accepted the open operation, not that
a particular UI arrangement was achieved.

### Sending

The outgoing-message class exposes sender, subject, content, visibility, and
recipient elements. OsaMail constructs a non-visible outgoing message, appends
typed to/cc/bcc recipients, and calls `send()`.

When an account is specified, OsaMail resolves an enabled account by exact name
and uses its first configured email address as the sender. It does not silently
fall back if that account is absent or disabled. A Boolean success from `send`
means Mail accepted the operation; the dictionary does not establish remote
delivery.

`--dry-run` does not create an outgoing message or call `send`.

## Compatibility and validation limits

- Apple's JXA documentation is archived and no longer updated. Mail-specific
  behavior must be checked against each supported Mail version's installed
  dictionary and live integration tests.
- Automation authorization is associated with the invoking process context. A
  terminal, IDE, or packaged launcher can have different TCC state.
- Vectorized Mail property access and `whose` body queries still depend on
  mailbox size and provider state.
- Message collection ordering is unspecified; OsaMail sorts explicit received
  dates rather than assuming Mail order.
- Live checks on Mail 16.0 covered `doctor`, account discovery, recent and
  unread listing, metadata search, `show`, `open`, and Unicode no-send dry-run.
  The ignored read-only macOS integration suite also passed.
- Real sending and sender-account mapping were deliberately not exercised
  because validation must not send email. A successful `send --dry-run` does
  not establish remote delivery or the behavior of every configured provider.

## References

- Installed Mail scripting definition:
  `/System/Applications/Mail.app/Contents/Resources/Mail.sdef`
- [JavaScript for Automation release
  notes](https://developer.apple.com/library/archive/releasenotes/InterapplicationCommunication/RN-JavaScriptForAutomation/Articles/OSX10-10.html)
- [Mac Automation Scripting Guide: scripting
  terminology](https://developer.apple.com/library/archive/documentation/LanguagesUtilities/Conceptual/MacAutomationScriptingGuide/AboutScriptingTerminology.html)
- [`NSAppleEventsUsageDescription`](https://developer.apple.com/documentation/bundleresources/information-property-list/nsappleeventsusagedescription)
- [Resetting access to protected resources in
  macOS](https://developer.apple.com/documentation/xcode/resetting-access-to-protected-resources-in-macos)
