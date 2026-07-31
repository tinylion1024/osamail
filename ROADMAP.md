# OsaMail roadmap

OsaMail is moving from safe Mail reading and sending toward a small set of
reversible mailbox actions. The command surface should remain easy to discover,
safe by default, and suitable for shell automation.

## v0.2.0 — faster reading

Status: released on 2026-07-29.

- Print message subjects only with `--titles`.
- Show delayed progress feedback for slow interactive reads.
- Add recovery hints for account, mailbox, and stale-reference errors.
- Keep the English and Simplified Chinese READMEs optimized for a short
  install-to-first-result path.

Release gate:

- All required repository checks pass.
- The package is available from crates.io.
- The universal GitHub Release and checksum are published.
- The Homebrew tap is updated and its formula passes installation and test
  checks.

## v0.3.0 — explicit message state

Status: released on 2026-07-29.

- Add one discoverable `mark` command with `read`, `unread`, `flag`, and
  `unflag` actions.
- Accept the same opaque message references produced by list and search
  commands.
- Provide `--dry-run` so users can validate a mutation without changing Mail.
- Return stable human-readable and JSON results without adding new global
  concepts.

Default and CI tests will use mock automation. They will never change real
message state.

## v0.4.0 — organize messages

Status: released on 2026-07-29.

- List mailbox names and paths for exact destination discovery.
- Move messages to an explicitly selected mailbox.
- Provide an archive workflow without assuming that every account exposes the
  same localized archive mailbox name.
- Extend state and organization actions to bounded batches with clear
  per-message results and partial-failure reporting.
- Keep `--dry-run` available for every mailbox mutation.

Moving a message may invalidate an older opaque reference. Commands and
documentation will make that behavior explicit.

## v0.5.0 — focused mailbox workflows

Status: release candidate prepared on 2026-08-01.

- Filter `recent`, `unread`, and `search` by an inclusive local `--since`
  date and an exclusive local `--before` date, both written as `YYYY-MM-DD`.
- Let `mark`, `move`, and `archive` read opaque message references from
  standard input without adding another batch-command hierarchy.
- Preserve the existing 50-message batch limit, dry-run behavior, ordered
  per-message outcomes, and single-message output shape.
- Keep metadata-only queries from reading message bodies and add regression
  coverage for the properties required by title, count, and date-filtered
  paths.

Date filters use the Mac's local calendar and apply to each message's received
date. An empty search query is valid only when at least one metadata or date
filter is present.

## Not planned through v0.5.0

Attachments, HTML composition or rendering, reply, forward, delete, rules,
background notifications, a GUI or TUI, direct provider connections, and a
plugin system remain outside this roadmap.

## Release policy

Versions are released sequentially. Published tags are immutable. Each version
must pass the repository's required checks and be verified independently on
crates.io, GitHub Releases, and the Homebrew tap before the next version is
published.
