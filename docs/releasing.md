# Releasing OsaMail

This guide is for maintainers. It describes publication steps but does not grant
credentials or publish anything by itself.

## Release outputs

A release consists of:

- a versioned crate on crates.io;
- a Git tag and GitHub Release;
- `osamail-vVERSION-universal-apple-darwin.tar.gz`;
- an adjacent `.sha256` checksum file; and
- an updated formula in `tinylion1024/homebrew-tap`.

The archive contains `osamail`, `README.md`, `README.zh-CN.md`, `LICENSE`, and
`CHANGELOG.md`. OsaMail 0.1.x releases are not code-signed or notarized.

## Prerequisites

- A clean checkout on macOS.
- Stable Rust with `aarch64-apple-darwin` and `x86_64-apple-darwin` targets.
- Write access to `tinylion1024/osamail`.
- A crates.io account and API token authorized for the `osamail` crate.
- Write access to `tinylion1024/homebrew-tap`.
- GitHub Actions permissions to create releases and upload assets.
- A write-enabled Homebrew tap deploy key stored in the OsaMail repository as
  the `HOMEBREW_TAP_DEPLOY_KEY` Actions secret.

Do not store tokens in the repository or pass them in command-line arguments.
Use the authenticated Cargo and GitHub mechanisms appropriate to the maintainer
environment.

## 1. Prepare the version

`Cargo.toml` is the version source of truth. For a version such as `0.1.0`:

1. Set `[package].version` in `Cargo.toml`.
2. Run Cargo so `Cargo.lock` reflects the package version.
3. Move release notes from `[Unreleased]` into a dated changelog section.
4. Verify README commands against current clap help.
5. Confirm that `Cargo.toml`, the changelog heading, intended tag `v0.1.0`, and
   Homebrew formula version agree.

The binary version comes from `env!("CARGO_PKG_VERSION")`; do not hard-code it
in Rust.

## 2. Run the release gate

Run every command and stop on the first failure:

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

Then inspect the universal artifact:

```bash
file dist/osamail
./dist/osamail --version
./dist/osamail --help
```

The `file` output must list both `arm64` and `x86_64`. The reported version must
match `Cargo.toml`.

Live read-only Mail checks require an authorized macOS terminal:

```bash
OSAMAIL_INTEGRATION=1 cargo test --test macos_integration -- --ignored
```

Do not publish while an applicable required check is failing. If Automation
consent blocks only the live integration check, record that fact explicitly in
the release notes rather than describing the check as passed.

## 3. Publish the crate

Inspect the package before publication:

```bash
cargo package --list
cargo publish --dry-run
```

Publication is irreversible for that version. After the commit and version are
final:

```bash
cargo publish
```

Wait for the version to appear on crates.io, then verify installation from a
clean temporary Cargo home if desired. Do not reuse a version number after
publication; prepare a new patch release for corrections.

## 4. Create the GitHub release

Commit the version and documentation, then create and push an annotated tag:

```bash
git tag -a v0.1.0 -m "OsaMail 0.1.0"
git push origin v0.1.0
```

The release workflow for `v*` tags verifies that the tag matches the Cargo
version, runs the quality gate, builds both Apple targets, combines them with
`lipo`, archives the required files, writes SHA-256, and creates the GitHub
Release.

After the workflow completes, verify:

```text
osamail-v0.1.0-universal-apple-darwin.tar.gz
osamail-v0.1.0-universal-apple-darwin.tar.gz.sha256
```

Download both assets, verify the checksum using the format produced by the
workflow, unpack the archive, and run:

```bash
tar -xzf osamail-v0.1.0-universal-apple-darwin.tar.gz
./osamail-v0.1.0/osamail --version
./osamail-v0.1.0/osamail --help
```

Do not move the tag to repair an immutable release. Fix the issue and publish a
new version.

## 5. Verify the automated Homebrew update

After the release job succeeds, the same workflow downloads the public asset,
updates and installs the formula on a clean macOS runner, and commits it to
`tinylion1024/homebrew-tap`.

Confirm the workflow completed and inspect:

```bash
gh run list --workflow Release
gh api repos/tinylion1024/homebrew-tap/contents/Formula/osamail.rb
```

The formula URL, version, and SHA-256 must match the GitHub Release. If tap
publication failed after the release succeeded, fix the scoped deploy key or
the formula issue and rerun the failed workflow jobs.

See [homebrew.md](homebrew.md) for setup, rotation, and manual recovery.

## 6. Post-release verification

From a clean macOS environment:

```bash
brew update
brew install tinylion1024/tap/osamail
osamail --version
osamail --help
```

Also confirm that the crates.io and GitHub pages show the intended README,
license, changelog, source commit, and release assets.

Create the next `[Unreleased]` changelog entries during normal development. Do
not make a remote publication merely to test the release process; use
`cargo publish --dry-run`, local universal builds, and workflow review.
