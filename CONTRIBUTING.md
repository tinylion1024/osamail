# Contributing to OsaMail

OsaMail is a small macOS-native CLI. Keep changes focused on controlling
accounts already configured in Apple Mail through `/usr/bin/osascript`.

## Prerequisites

- Rust stable 1.85 or newer
- macOS for JXA syntax checks and live Mail integration
- `/usr/bin/osascript` and `/usr/bin/osacompile`
- Apple Mail for opt-in integration tests

Help, version, unit tests, and portability checks should remain usable outside
macOS.

## Set up a development checkout

```bash
git clone https://github.com/tinylion1024/osamail.git
cd osamail
cargo build
cargo test --all-features
cargo run -- --help
```

Do not add dependencies unless the change demonstrates a concrete need. Use
stable Rust Edition 2024 and preserve the existing module boundaries described
in [docs/architecture.md](docs/architecture.md).

## Development rules

- Keep the product boundary Rust -> `/usr/bin/osascript` -> JXA/AppleScript ->
  Apple Mail.
- Embed automation scripts with `include_str!`; installed binaries must not
  require repository files.
- Serialize every business request to an unpredictable `0600` temporary JSON
  file.
- Pass only the request path as business input to the automation script.
- Never interpolate user input into scripts, shell text, logs, or error detail.
- Never use `sh -c`, Mail's private database, network clients, or telemetry.
- Preserve stdout for results and stderr for errors. JSON mode must emit only
  JSON on its selected stream.
- Keep README commands synchronized with clap definitions.
- Do not use default tests to send mail or mutate message state.

## Test while editing

Run the smallest relevant test first. Before submitting a change, run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo test --doc
cargo build --release --locked
cargo package --allow-dirty
cargo publish --dry-run
./scripts/check.sh
./scripts/smoke-test.sh
./scripts/build-universal.sh
```

`scripts/check.sh`, the smoke test, and the universal build are macOS release
gates. Do not claim they passed unless they completed in the current checkout.

### Opt-in Mail integration

Default tests replace the automation runner and must not access real Mail. On a
macOS host with Automation permission, explicitly run read-only integration
checks:

```bash
OSAMAIL_INTEGRATION=1 cargo test --test macos_integration -- --ignored
```

Allowed read-only coverage includes `doctor`, `accounts`, `recent --limit 1`,
and `unread --count`.

Never enable real sending in CI. A send integration test must remain ignored and
must require both:

```text
OSAMAIL_INTEGRATION=1
OSAMAIL_ALLOW_SEND_TEST=1
```

Prefer `osamail send --dry-run` for ordinary validation.

## Documentation changes

Verify every documented command against current `--help` output. If a command
requires Automation permission, test its parsing and clearly record the live
validation limit instead of presenting sample data as observed output.

Update:

- `README.md` when CLI behavior or user-visible limitations change;
- `CHANGELOG.md` for release-visible changes;
- `docs/architecture.md` for component or protocol changes;
- `docs/apple-mail-automation.md` for scripting-dictionary evidence;
- `DECISIONS.md` for architecture choices; and
- `BLOCKERS.md` only for external issues that code cannot resolve.

## Pull requests

Keep each change small and reviewable. In the description, include:

- the user-visible result;
- security or privacy impact;
- the exact validation commands and their results; and
- any macOS, Mail, TCC, or packaging check that could not run.

Do not include real account names, addresses, subjects, bodies, headers, or
opaque references in tests, fixtures, issues, or review output.
