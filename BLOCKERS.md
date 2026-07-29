# External blockers

## Resolved: crates.io publishing authorization

Status: resolved on 2026-07-29

The crates.io API token is installed and the account email address is verified.
OsaMail 0.2.0 and 0.3.0 were published successfully and verified by installing
each version from the public registry.

Tokens remain outside the repository and must never be committed, pasted into
project files, or shared in issue or pull-request text.

## Resolved: live Apple Mail automation authorization

Status: resolved on 2026-07-26

The first read-only JXA probe remained pending until the invoking application
received macOS Automation authorization. After authorization became available,
the live checks completed successfully against Mail 16.0 on macOS 15.3.

Validated operations included `doctor`, `accounts`, recent and unread listing,
metadata search, `show`, `open` on an already-read message, `unread --count`,
Unicode `send --dry-run`, and the opt-in read-only macOS integration suite. No
real email was sent.
