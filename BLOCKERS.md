# External blockers

## Open: crates.io email verification

Status: open since 2026-07-29

The crates.io API token is installed and accepted, but the first
`cargo publish --locked` attempt for OsaMail 0.2.0 returned HTTP 400 because
the crates.io account does not have a verified email address.

The account owner must add and verify an email address at
<https://crates.io/settings/profile>. No package was uploaded, and no release
tag was created.

## Resolved: live Apple Mail automation authorization

Status: resolved on 2026-07-26

The first read-only JXA probe remained pending until the invoking application
received macOS Automation authorization. After authorization became available,
the live checks completed successfully against Mail 16.0 on macOS 15.3.

Validated operations included `doctor`, `accounts`, recent and unread listing,
metadata search, `show`, `open` on an already-read message, `unread --count`,
Unicode `send --dry-run`, and the opt-in read-only macOS integration suite. No
real email was sent.
