# ADR 0003: Strict Path Traversal Rejection

- **Status**: Accepted
- **Date**: 2026-10-30
- **Authors**: Sebastien Rousseau

## Context

Static site generators write generated HTML and static assets to disk based on user-supplied file paths, slugs, and directory structures. Attackers who control Markdown frontmatter metadata or file names can attempt directory traversal attacks (`../`, absolute paths, null bytes, or URL-encoded dots) to overwrite arbitrary files outside the targeted output directory.

## Decision

1. Require all input, output, and template paths to undergo mandatory sanitisation through `ConfigBuilder::sanitize_path`.
2. Reject any path containing directory traversal sequences (`..`), null bytes (`\0`), control characters, or absolute root prefixes when relative paths are expected.
3. Assert path confinement to the specified root directory before any filesystem read or write operation is executed.
4. Enforce this property continuously using property-based fuzz tests in `tests/property_tests.rs`.

## Consequences

- **Positive**: Complete defense against arbitrary file read/write vulnerabilities (CWE-22) originating from untrusted site content.
- **Positive**: Hardened sandboxing guarantees for multi-tenant static site generation workloads.
