#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_dir"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "universal macOS builds require macOS" >&2
  exit 1
fi

for command in cargo lipo file shasum tar; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "required command not found: $command" >&2
    exit 1
  fi
done

for required_file in README.md LICENSE CHANGELOG.md; do
  if [[ ! -f "$required_file" ]]; then
    echo "required archive file not found: $required_file" >&2
    exit 1
  fi
done

version="$(sed -nE 's/^version = "([^"]+)"/\1/p' Cargo.toml | head -n 1)"
if [[ -z "$version" ]]; then
  echo "could not read package version from Cargo.toml" >&2
  exit 1
fi

tag="v${version}"
archive_name="osamail-${tag}-universal-apple-darwin.tar.gz"
dist_dir="$repo_dir/dist"
temp_dir="$(mktemp -d "${TMPDIR:-/tmp}/osamail-universal.XXXXXXXX")"
cleanup() {
  rm -rf "$temp_dir"
}
trap cleanup EXIT

echo "==> Building aarch64-apple-darwin"
cargo build --release --locked --target aarch64-apple-darwin

echo "==> Building x86_64-apple-darwin"
cargo build --release --locked --target x86_64-apple-darwin

mkdir -p "$dist_dir"
universal_binary="$dist_dir/osamail"

echo "==> Creating universal binary"
/usr/bin/lipo -create \
  "$repo_dir/target/aarch64-apple-darwin/release/osamail" \
  "$repo_dir/target/x86_64-apple-darwin/release/osamail" \
  -output "$universal_binary"
chmod 0755 "$universal_binary"

echo "==> Verifying universal binary"
/usr/bin/lipo "$universal_binary" -verify_arch arm64 x86_64
file "$universal_binary"
"$universal_binary" --version
"$universal_binary" --help >/dev/null

stage_dir="$temp_dir/osamail-${tag}"
mkdir -p "$stage_dir"
cp "$universal_binary" README.md LICENSE CHANGELOG.md "$stage_dir/"

echo "==> Creating archive"
tar -C "$temp_dir" -czf "$dist_dir/$archive_name" "osamail-${tag}"
(
  cd "$dist_dir"
  shasum -a 256 "$archive_name" >"${archive_name}.sha256"
  shasum -a 256 -c "${archive_name}.sha256"
)

echo "==> Created dist/$archive_name"
echo "==> Created dist/${archive_name}.sha256"
