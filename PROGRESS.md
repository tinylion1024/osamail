# Progress

Last updated: 2026-07-26

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

## Fresh validation evidence

- `cargo fmt --all -- --check`: passed using an isolated Rust 1.89.0 toolchain.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all-features`: 35 passed, 0 failed; 4 opt-in macOS tests ignored.
- `cargo test --doc`: passed.
- `./scripts/check.sh`: passed, including all six JXA syntax checks, release
  build, package, and publish dry-run checks.
- Exact `cargo publish --dry-run`: passed from a clean temporary Git snapshot;
  no upload occurred.
- `./scripts/smoke-test.sh`: passed.
- `./scripts/build-universal.sh`: passed; the produced Mach-O contains both
  `arm64` and `x86_64`.
- Source install and universal-archive install into isolated temporary prefixes:
  passed.
- `OSAMAIL_INTEGRATION=1 cargo test --test macos_integration -- --ignored
  --nocapture`: 4 passed, 0 failed.
- Privacy-preserving live acceptance checks passed for `doctor`, account schema,
  100-message recent listing, unread listing/count, metadata search, full
  `show`, and Unicode/quotes/newline send dry-run.
- A live `open` check passed on an already-read message. No real email was sent.
- Homebrew formula generation, checksum matching, Ruby syntax, and local archive
  installation passed.
- English and Simplified Chinese README section parity passed; the universal
  release archive contains both `README.md` and `README.zh-CN.md`.
- GitHub Actions workflow syntax passed `actionlint` 1.7.7 and YAML parsing.
- The release workflow now builds and uploads universal assets, validates the
  generated Homebrew formula by installing it, and publishes it with a
  tap-scoped deploy key.
- The `v0.1.0` GitHub Release assets published successfully. Its Homebrew job
  exposed a local-formula path parsing error; the explicit-path correction and
  complete `v0.1.1` local release gate passed before the recovery release.
- The complete required local check sequence passed after giving immediate
  subprocess fixtures a shared test-only timeout to avoid load-related CI
  flakes while retaining the dedicated 50 ms timeout behavior test.

Validation results are recorded only after the command has completed.
