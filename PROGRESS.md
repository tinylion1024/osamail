# Progress

Last updated: 2026-07-29

## Phase 0 — environment and Apple Mail automation

- [x] Inspected the initially empty workspace and initialized a local Git repository.
- [x] Confirmed macOS 15.3 on Apple Silicon (`arm64`).
- [x] Confirmed Command Line Tools at `/Library/Developer/CommandLineTools`.
- [x] Found Rust stable 1.89.0 at the existing local rustup toolchain path.
- [x] Confirmed `/usr/bin/osascript`, `/usr/bin/osacompile`, and Mail.app.
- [x] Located the authoritative bundled dictionary at
  `/System/Applications/Mail.app/Contents/Resources/Mail.sdef`.
- [x] Verified JXA `run(argv)` can read a `0600` JSON request through Foundation and
  round-trip Chinese, emoji, quotes, slashes, tabs, and newlines as JSON.
- [x] Completed live Mail account/read-only probes against Mail 16.0 after
  Automation authorization became available.

## Implementation

- [x] Phase 1 — project foundation
- [x] Phase 2 — secure automation runner
- [x] Phase 3 — read-only commands
- [x] Phase 4 — send and dry-run
- [x] Phase 5 — complete tests
- [x] Phase 6 — documentation and distribution
- [x] Phase 7 — final acceptance
- [x] Added synchronized English and Simplified Chinese README documentation.
- [x] Added tag-gated GitHub releases and automated Homebrew tap publication.
- [x] Prepared OsaMail 0.2.0 with subject-only listings, delayed interactive
  progress feedback, and actionable lookup-error recovery hints.
- [x] Reworked the English and Simplified Chinese READMEs around a
  value-to-install-to-first-result path, goal-based command discovery, and
  concise workflow and FAQ sections.
- [x] Published a bounded roadmap for the v0.2.0, v0.3.0, and v0.4.0 release
  sequence.
- [x] Prepared OsaMail 0.3.0 with one `mark` command for read, unread, flag,
  and unflag actions, including validation-only dry runs and idempotent
  outcomes.
- [x] Documented the message-state workflow in both READMEs without adding
  another top-level concept.

## Fresh validation evidence

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all-features`: 47 passed, 0 failed; 6 opt-in macOS tests ignored.
- `cargo test --doc`: passed.
- `cargo build --release --locked`: passed for OsaMail 0.3.0.
- `cargo package --allow-dirty`: passed for 46 packaged files.
- Exact `cargo publish --dry-run`: passed; no upload occurred.
- `./scripts/check.sh`: passed, including all seven JXA syntax checks, release
  build, package, and publish dry-run checks.
- `./scripts/smoke-test.sh`: passed.
- `./scripts/build-universal.sh`: passed; the produced Mach-O contains both
  `arm64` and `x86_64`, and reports `osamail 0.3.0`.
- Source install and universal-archive install into isolated temporary prefixes:
  passed.
- `OSAMAIL_INTEGRATION=1 cargo test --test macos_integration -- --ignored
  --nocapture`: 5 passed, 0 failed, including the subject-only unread listing.
- Privacy-preserving live acceptance checks passed for `doctor`, account schema,
  100-message recent listing, unread listing/count, metadata search, full
  `show`, and Unicode/quotes/newline send dry-run.
- A live `open` check passed on an already-read message. No real email was sent.
- Homebrew formula generation, checksum matching, Ruby syntax, and local archive
  installation passed.
- English and Simplified Chinese README section parity passed; the universal
  release archive contains both `README.md` and `README.zh-CN.md`.
- Both READMEs passed heading, code-fence, and local-link validation with one
  H1, 12 matching H2 sections, and 16 closed code blocks each.
- Every command and option used in the READMEs was checked against the current
  top-level and subcommand help output.
- GitHub Actions workflow syntax passed `actionlint` 1.7.7 and YAML parsing.
- The release workflow now builds and uploads universal assets, validates the
  generated Homebrew formula by installing it, and publishes it with a
  tap-scoped deploy key.
- The `v0.1.0` and `v0.1.1` GitHub Release assets published successfully.
  Their Homebrew jobs exposed increasingly strict current-Homebrew requirements
  for formula paths; `v0.1.2` validates the formula inside an isolated local tap
  before publishing the real tap.
- The complete required local check sequence passed after giving immediate
  subprocess fixtures a shared test-only timeout to avoid load-related CI
  flakes while retaining the dedicated 50 ms timeout behavior test.
- The complete required local check sequence passed again on 2026-07-29 from
  release commit `3f93c39`, including package verification, publish dry-run,
  smoke tests, and the universal archive checksum check.
- The complete required local check sequence passed for OsaMail 0.3.0 on
  release-candidate commit `296db4c`, including 47 tests, seven embedded JXA
  syntax checks, 46 packaged files, publish dry-run, smoke tests, and a
  universal `arm64`/`x86_64` archive reporting `osamail 0.3.0`.
- A live, privacy-preserving `mark read --dry-run --json` acceptance check and
  its opt-in macOS integration test passed without changing message state.

Validation results are recorded only after the command has completed.
