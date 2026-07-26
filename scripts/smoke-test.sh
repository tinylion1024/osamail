#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_dir"

echo "==> Building release binary"
cargo build --release --locked
binary="$repo_dir/target/release/osamail"

echo "==> Checking help and version"
"$binary" --help >/dev/null
"$binary" --version | grep -Eq '^osamail [0-9]+\.[0-9]+\.[0-9]+'

echo "==> Checking argument validation"
if "$binary" recent --limit 0 >/dev/null 2>&1; then
  echo "recent --limit 0 unexpectedly succeeded" >&2
  exit 1
fi
if "$binary" show invalid-reference >/dev/null 2>&1; then
  echo "show invalid-reference unexpectedly succeeded" >&2
  exit 1
fi
if "$binary" send --subject "missing recipient" --dry-run >/dev/null 2>&1; then
  echo "send without --to unexpectedly succeeded" >&2
  exit 1
fi

echo "==> Checking send dry-run (no message is created or sent)"
dry_run_output="$("$binary" send --to smoke-test@example.invalid --subject "OsaMail smoke test" --body "not sent" --dry-run --json)"
grep -Eq '"dry_run"[[:space:]]*:[[:space:]]*true' <<<"$dry_run_output"
grep -Eq '"sent"[[:space:]]*:[[:space:]]*false' <<<"$dry_run_output"

echo "==> Smoke tests passed"
