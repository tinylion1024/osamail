# Architecture decisions

## ADR-001: JXA is the primary automation language

JXA is used for every 0.1.0 operation unless a verified Mail compatibility issue
requires a command-specific AppleScript fallback. It accepts an argument vector,
can parse and emit JSON without text scraping, and can read the request through
Foundation without enabling shell interpolation.

## ADR-002: One secure request protocol

Every automation call serializes its request into an unpredictable `0600`
temporary file. The embedded script source is constant, and the request path is
the only business argument passed to `/usr/bin/osascript`. User content never
appears in script source or process arguments.

## ADR-003: Typed coordination over a small JSON runner

Command code uses typed Rust request and response models. The runner itself has a
small object-safe interface over `serde_json::Value`, allowing tests to replace
Mail automation without a plugin framework or runtime dependency.

## ADR-004: Opaque references encode locators, not code

Message references are URL-safe, unpadded Base64 encodings of a versioned JSON
locator containing account, mailbox path, Mail integer id, and optional RFC
message id. Rust validates the version and fields before automation runs.

## ADR-005: Portable CLI, macOS backend

The crate builds on other platforms so help, version, packaging, and static
analysis remain available. Mail commands return a structured unsupported-platform
error outside macOS rather than using a top-level compilation error.

## ADR-006: Vectorized metadata reads for bounded mailbox queries

Mail collection properties are fetched as parallel vectors, then filtered,
sorted, and limited inside JXA before returning JSON to Rust. This keeps the
default recent, unread, and metadata-search paths from reading message bodies or
making one Apple event per field. Explicit body search is the only list path
that uses a Mail `whose` clause over `content`.

## ADR-007: `open` uses Mail's message command with best-effort UI semantics

Mail 16.0 successfully accepted `message.open()` during live validation on an
already-read message, so no command-specific AppleScript fallback is needed for
0.1.0. A successful response means Mail accepted the open request; exact window
focus and selection remain controlled by Mail. Opaque references are locators,
not durable identifiers, and can become stale when Mail data changes.
