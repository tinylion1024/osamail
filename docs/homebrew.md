# Homebrew distribution

OsaMail is distributed through the public third-party tap
`tinylion1024/homebrew-tap`. Users install it with:

```bash
brew install tinylion1024/tap/osamail
```

The formula installs the universal binary from the matching GitHub Release.
Formula publication is automated and remains gated on a successful release
build and a clean Homebrew installation test.

## Automated release flow

Pushing a version tag such as `v0.1.2` starts
`.github/workflows/release.yml`:

1. Confirm the tag matches the version in `Cargo.toml`.
2. Run the complete project quality gate.
3. Build and verify the universal `arm64` and `x86_64` archive.
4. Create the GitHub Release and upload the archive and checksum.
5. Download the public release archive and update the formula URL, version, and
   SHA-256.
6. Install and test that formula on a fresh GitHub-hosted macOS runner.
7. Commit the validated formula to
   `tinylion1024/homebrew-tap/Formula/osamail.rb`.

The release step is idempotent: rerunning it replaces the release assets when
the release already exists. The tap step makes no commit when the formula is
already current.

## Repository setup

The `tinylion1024/homebrew-tap` repository must exist with `main` as its default
branch. The OsaMail repository must have an Actions secret named
`HOMEBREW_TAP_DEPLOY_KEY`.

That secret is the private half of a write-enabled deploy key registered only
on `tinylion1024/homebrew-tap`. This keeps cross-repository write access scoped
to the tap and avoids storing a broad personal access token.

To rotate the credential:

1. Generate a new Ed25519 key pair in a secure temporary location.
2. Add the public key to the tap under **Settings -> Deploy keys** with write
   access.
3. Replace the OsaMail Actions secret `HOMEBREW_TAP_DEPLOY_KEY` with the private
   key.
4. Remove the old deploy key after a release or controlled workflow run proves
   the replacement works.

Never commit either key to a repository or print the private key in workflow
logs.

## Formula generation

The release workflow runs:

```bash
./scripts/update-homebrew-formula.sh v0.1.2
```

The script downloads the public release archive and updates
`packaging/homebrew/osamail.rb`. Before a release exists, maintainers can test
the same calculation against a local archive:

```bash
./scripts/update-homebrew-formula.sh \
  v0.1.2 \
  dist/osamail-v0.1.2-universal-apple-darwin.tar.gz
```

The generated formula must contain:

- homepage `https://github.com/tinylion1024/osamail`;
- the immutable public release archive URL;
- the exact archive SHA-256;
- a version matching `Cargo.toml`; and
- license `MIT`.

Do not publish placeholder URLs or checksums.

## Manual recovery

If the release succeeds but tap publication fails:

1. Fix or rotate the deploy key when authentication failed.
2. Rerun the failed workflow jobs from GitHub Actions.
3. Confirm `Formula/osamail.rb` changed in the tap.
4. Verify from a clean Homebrew environment:

```bash
brew update
brew install tinylion1024/tap/osamail
brew test tinylion1024/tap/osamail
```

As a last-resort manual recovery, run the formula update script, copy
`packaging/homebrew/osamail.rb` to the tap's `Formula/osamail.rb`, validate it,
and push that single formula change. Do not move or replace an existing release
tag to repair a failure; publish a new patch version when release contents must
change.

## Troubleshooting

- **Checksum mismatch:** download the public asset again and compare its
  SHA-256. Never change the formula to match an untrusted archive.
- **Wrong version:** compare the Git tag, `Cargo.toml`, archive name, binary
  `--version`, and formula version.
- **Deploy-key failure:** confirm the public key remains a write-enabled deploy
  key on the tap and the matching private key is stored in the OsaMail secret.
- **Automation denial after installation:** the formula is installed correctly;
  grant the invoking terminal access under **System Settings -> Privacy &
  Security -> Automation**.
- **Gatekeeper warning:** 0.1.x artifacts are not signed or notarized. Do not
  instruct users to disable Gatekeeper globally.
