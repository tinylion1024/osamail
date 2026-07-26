# Changelog

All notable changes to OsaMail are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0]

### Added

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

[Unreleased]: https://github.com/tinylion1024/osamail/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/tinylion1024/osamail/releases/tag/v0.1.0
