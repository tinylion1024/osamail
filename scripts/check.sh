#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_dir"

temp_dir="$(mktemp -d "${TMPDIR:-/tmp}/osamail-check.XXXXXXXX")"
cleanup() {
  rm -rf "$temp_dir"
}
trap cleanup EXIT

echo "==> Checking formatting"
cargo fmt --all -- --check

echo "==> Running clippy"
cargo clippy --all-targets --all-features -- -D warnings

echo "==> Running all tests"
cargo test --all-features

echo "==> Running documentation tests"
cargo test --doc

if [[ "$(uname -s)" == "Darwin" ]]; then
  echo "==> Compiling embedded JXA scripts"
  while IFS= read -r script; do
    output="$temp_dir/$(basename "${script%.js}").scpt"
    /usr/bin/osacompile -l JavaScript -o "$output" "$script"
  done < <(find src/automation/scripts -maxdepth 1 -type f -name '*.js' -print | sort)

  echo "==> Compiling embedded AppleScript files"
  while IFS= read -r script; do
    output="$temp_dir/$(basename "${script%.applescript}").scpt"
    /usr/bin/osacompile -l AppleScript -o "$output" "$script"
  done < <(find src/automation/scripts -maxdepth 1 -type f -name '*.applescript' -print | sort)
fi

echo "==> Building locked release binary"
cargo build --release --locked

echo "==> Validating crate package"
cargo package --allow-dirty

echo "==> Running crates.io publish dry-run"
cargo publish --dry-run --allow-dirty

echo "==> All checks passed"
