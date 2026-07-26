#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_dir"

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 <vVERSION> [release-archive]" >&2
  exit 2
fi

tag="$1"
version="${tag#v}"
if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$ ]]; then
  echo "invalid release version: $tag" >&2
  exit 2
fi
tag="v${version}"

cargo_version="$(sed -nE 's/^version = "([^"]+)"/\1/p' Cargo.toml | head -n 1)"
if [[ "$version" != "$cargo_version" ]]; then
  echo "release version $version does not match Cargo.toml version $cargo_version" >&2
  exit 1
fi

formula="$repo_dir/packaging/homebrew/osamail.rb"
if [[ ! -f "$formula" ]]; then
  echo "Homebrew formula not found: $formula" >&2
  exit 1
fi

archive_name="osamail-${tag}-universal-apple-darwin.tar.gz"
url="https://github.com/tinylion1024/osamail/releases/download/${tag}/${archive_name}"
temp_dir="$(mktemp -d "${TMPDIR:-/tmp}/osamail-homebrew.XXXXXXXX")"
cleanup() {
  rm -rf "$temp_dir"
}
trap cleanup EXIT

if [[ $# -eq 2 ]]; then
  archive="$2"
  if [[ ! -f "$archive" ]]; then
    echo "release archive not found: $archive" >&2
    exit 1
  fi
else
  archive="$temp_dir/$archive_name"
  echo "==> Downloading $url"
  curl --fail --location --silent --show-error \
    --retry 5 --retry-all-errors --retry-delay 2 \
    "$url" --output "$archive"
fi

sha256="$(shasum -a 256 "$archive" | awk '{print $1}')"
updated_formula="$temp_dir/osamail.rb"
awk -v url="$url" -v sha="$sha256" -v version="$version" '
  /^[[:space:]]*url / { print "  url \"" url "\""; next }
  /^[[:space:]]*sha256 / { print "  sha256 \"" sha "\""; next }
  /^[[:space:]]*version / { print "  version \"" version "\""; next }
  { print }
' "$formula" >"$updated_formula"
mv "$updated_formula" "$formula"

echo "==> Updated packaging/homebrew/osamail.rb"
echo "    version: $version"
echo "    url: $url"
echo "    sha256: $sha256"
echo "Formula is ready for validation and publication."
