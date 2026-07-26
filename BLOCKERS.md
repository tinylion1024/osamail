# External blockers

No external blockers are currently open.

## Resolved: live Apple Mail automation authorization

Status: resolved on 2026-07-26

The first read-only JXA probe remained pending until the invoking application
received macOS Automation authorization. After authorization became available,
the live checks completed successfully against Mail 16.0 on macOS 15.3.

Validated operations included `doctor`, `accounts`, recent and unread listing,
metadata search, `show`, `open` on an already-read message, `unread --count`,
Unicode `send --dry-run`, and the opt-in read-only macOS integration suite. No
real email was sent.
