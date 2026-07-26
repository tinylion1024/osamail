# Changelog

All notable changes to OsaMail are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-07-27

### Added

- Add `--titles` to recent, unread, and search commands for minimal
  subject-only output and reduced Apple Mail property reads.
- Add delayed, TTY-only progress feedback for Apple Mail read operations
  without changing piped, JSON, or quiet output.
- Add actionable recovery hints for invalid accounts, mailbox names, and stale
  message references in both human-readable and JSON errors.

## [0.1.2] - 2026-07-26

### Fixed

- Validate the generated Homebrew formula inside an isolated local tap, as
  required by current Homebrew versions, before publishing the real tap.

## [0.1.1] - 2026-07-26

### Fixed

- Treat the repository Homebrew formula as an explicit local path during
  release validation instead of parsing it as a remote tap name.

## [0.1.0] - 2026-07-26

### Added

- Bilingual English and Simplified Chinese README documentation.
- GitHub CI/CD for versioned universal macOS releases and automatic, validated
  Homebrew tap publication.
- Environment diagnostics for macOS, `osascript`, Apple Mail, Automation
  permission, and configured accounts.
- Apple Mail account listing without credentials.
- Recent and unread message listing, including unread counts.
- Sender, subject, unread-state, account, and mailbox search filters.
- Terminal message viewing with optional headers and configurable body
  truncation.
- Opaque message references for `show` and `open`.
- Best-effort opening of referenced messages in Apple Mail.
- Plain-text email sending with multiple recipients, file and stdin body input,
  exact account selection, and no-send dry-run validation.
- Human-readable and structured JSON output.
- Secure `0600` temporary-file requests to embedded JXA automation scripts.
- Portable help and version behavior outside macOS.
- Universal macOS release build and Homebrew packaging support.

[Unreleased]: https://github.com/tinylion1024/osamail/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/tinylion1024/osamail/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/tinylion1024/osamail/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/tinylion1024/osamail/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/tinylion1024/osamail/releases/tag/v0.1.0
