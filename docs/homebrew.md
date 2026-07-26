# Homebrew distribution

OsaMail uses a third-party tap rather than Homebrew/core. The intended user
command is:

```bash
brew install tinylion1024/tap/osamail
```

The formula installs the universal binary produced by the matching GitHub
Release. Creating the tap and pushing formula updates are manual maintainer
actions.

## Create the tap

Create the public GitHub repository:

```text
tinylion1024/homebrew-tap
```

In a local checkout, create the standard formula directory:

```bash
git clone git@github.com:tinylion1024/homebrew-tap.git
cd homebrew-tap
mkdir -p Formula
```

Do not run these remote commands as part of automated repository tests.

## Prepare the first formula

First publish the matching GitHub Release so its archive and checksum are
available. In the OsaMail source checkout, run:

```bash
./scripts/update-homebrew-formula.sh v0.1.0
```

The script updates `packaging/homebrew/osamail.rb` from the versioned release
URL and its SHA-256. It does not commit or push.

Before publication, maintainers can exercise the same update logic with a local
archive:

```bash
./scripts/update-homebrew-formula.sh \
  v0.1.0 \
  dist/osamail-v0.1.0-universal-apple-darwin.tar.gz
```

The resulting formula still points to the future GitHub Release URL, so do not
publish it until that exact asset is public.

Review the formula and copy it into the tap checkout:

```bash
cp packaging/homebrew/osamail.rb ../homebrew-tap/Formula/osamail.rb
```

Adjust the relative path if the two repositories are elsewhere.

The formula must contain:

- homepage `https://github.com/tinylion1024/osamail`;
- the public universal archive URL;
- the exact archive SHA-256;
- a version matching `Cargo.toml`; and
- license `MIT`.

Never publish placeholder URLs or checksums.

## Test the formula

From the tap checkout:

```bash
brew audit --strict --online Formula/osamail.rb
brew install --build-from-source Formula/osamail.rb
brew test osamail
osamail --version
osamail --help
```

The formula test uses only help and version output, so it does not require
Automation permission or access Apple Mail.

If `osamail` is already installed, test an upgrade or use Homebrew's documented
reinstall workflow in a disposable environment. Do not overwrite a user's
working installation merely to validate a formula.

## Publish the tap

After the formula tests pass:

```bash
git add Formula/osamail.rb
git commit -m "osamail 0.1.0"
git push origin main
```

Users can then install:

```bash
brew install tinylion1024/tap/osamail
```

Verify the public path from a clean Homebrew environment:

```bash
brew update
brew install tinylion1024/tap/osamail
brew test tinylion1024/tap/osamail
```

## Update a later release

For version `0.1.1`:

1. Complete the crate and GitHub release procedure in
   [releasing.md](releasing.md).
2. Confirm the public universal asset exists.
3. Run `./scripts/update-homebrew-formula.sh v0.1.1`.
4. Review the formula diff, especially URL, SHA-256, and version.
5. Copy it to the tap's `Formula/osamail.rb`.
6. Run the audit, install, test, and version checks.
7. Commit and push the tap change.

Do not update Homebrew before the release asset is public: Homebrew must be able
to fetch the exact immutable archive referenced by the formula.

## Troubleshooting

- **Checksum mismatch:** download the public asset again and compare its SHA-256.
  Do not edit the formula to match an untrusted or local-only archive.
- **Wrong version:** compare the Git tag, `Cargo.toml`, archive name, binary
  `--version`, and formula `version`.
- **Automation denial after installation:** the formula is installed correctly;
  grant the invoking terminal access under **System Settings -> Privacy &
  Security -> Automation**.
- **Gatekeeper warning:** 0.1.x artifacts are not signed or notarized. Do not
  instruct users to disable Gatekeeper globally.
