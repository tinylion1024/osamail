# External blockers

## Open: crates.io publishing authorization

Status: open since 2026-07-29

The local release environment can access GitHub but has no crates.io API token.
`cargo owner --list osamail` currently returns `no token found`, so the
sequential v0.2.0, v0.3.0, and v0.4.0 publication cannot begin.

The repository and every release candidate pass `cargo publish --dry-run`; no
secret is required for those checks. To unlock the real publication, the
repository owner must run `cargo login` locally with a crates.io API token.
Tokens must never be committed, pasted into project files, or shared in issue
or pull-request text.

## Resolved: live Apple Mail automation authorization

Status: resolved on 2026-07-26

The first read-only JXA probe remained pending until the invoking application
received macOS Automation authorization. After authorization became available,
the live checks completed successfully against Mail 16.0 on macOS 15.3.

Validated operations included `doctor`, `accounts`, recent and unread listing,
metadata search, `show`, `open` on an already-read message, `unread --count`,
Unicode `send --dry-run`, and the opt-in read-only macOS integration suite. No
real email was sent.
