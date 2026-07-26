# OsaMail contributor guidance

## Product boundary

- OsaMail is a small macOS-native CLI that controls accounts already configured in Apple Mail.
- The stable boundary is Rust -> `/usr/bin/osascript` -> JXA/AppleScript -> Apple Mail.
- It is not an IMAP, SMTP, Exchange, JMAP, GUI, TUI, daemon, web service, AI assistant, or plugin platform.
- Real Mail operations are macOS-only; help and version output must remain portable.

## Rust structure

- Use stable Rust, Edition 2024, Cargo, and the smallest practical dependency set.
- `cli.rs` owns clap definitions; `commands.rs` owns command coordination.
- `automation/runner.rs` owns secure subprocess execution and protocol parsing.
- `model.rs`, `reference.rs`, `output.rs`, and `error.rs` own their named concerns.
- Keep scripts embedded with `include_str!`; installed binaries must not need repository files.

## Security invariants

- Serialize every business input to an unpredictable `0600` temporary JSON file.
- Pass only that file path as the script's business argument.
- Never interpolate user input into JXA, AppleScript, shell text, logs, or error detail.
- Invoke the absolute `/usr/bin/osascript` path with `std::process::Command`.
- Never use `sh -c`, shell command construction, Mail's private database, telemetry, or network requests.
- Always check subprocess status, capture both output streams, enforce a timeout, and clean up.
- Never read account passwords or credentials.
- Default and CI tests must never send real email. Sending tests require explicit opt-in gates.

## Change discipline

- Keep README examples synchronized with clap behavior.
- Do not add dependencies, unsafe code, or broad abstractions without a demonstrated need.
- Preserve stdout for results and stderr for errors; JSON mode must emit JSON only.
- Record architecture choices in `DECISIONS.md`, external blockers in `BLOCKERS.md`, and fresh checks in `PROGRESS.md`.

## Required checks

Run the smallest relevant test while editing. Before completion run:

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

Do not claim completion while a required check is failing or unverified.
