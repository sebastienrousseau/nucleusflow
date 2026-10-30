# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.3] - Unreleased

### Changed

- **Breaking: the MSRV is now Rust 1.89.0** (was 1.56.0), one floor
  shared with the maintainer's other Rust crates. `rust-version` in
  `Cargo.toml` is the only place nucleusflow states it.
- `Profile` derives `Default` (`#[default] Development`) instead of a
  hand-written impl, as clippy asks once the floor allows it. The
  default is unchanged.
- The unused `euxis-commons` git dependency is gone. Nothing in the
  crate used it, and as a git dependency without a version it made
  `cargo package` and the crates.io publish fail.
- The trait and processor module docs link their items by full path,
  so `cargo doc` with `-D warnings` is clean.
- The unused `staticdatagen` dependency is gone. Nothing in the crate
  used it, and it brought 71 crates into the build, among them
  `bincode` 1.3.3 (RUSTSEC-2025-0141, unmaintained) through `syntect`.
- `Cargo.lock` is tracked. nucleusflow ships a binary, and the
  release builds and `cargo audit` now resolve the same, reviewed
  dependency versions instead of whatever is newest on the day. It
  builds with `--locked` on Rust 1.89.0.
- CI runs the shared pipelines audit job against the committed
  lockfile. It was switched off because the lockfile was ignored, which
  is why the Audit check showed as skipped on the release PR; the
  in-repo `cargo-audit` job that stood in for it is gone.
- The release workflow's packaging and changelog steps pass
  actionlint and shellcheck: `tar` takes its file list after `--`, so
  a file whose name starts with a dash is archived instead of read as an
  option, and the changelog is written by one grouped redirect.
- `deny.toml` allows Unicode-3.0 (ICU4X, unicode-ident,
  unicode-id-start) and Apache-2.0 WITH LLVM-exception
  (dragonbox_ecma), two permissive licences the allow list never named.
- `pulldown-cmark` requires `>= 0.13.5` (resolved to `0.13.6` in
  `Cargo.lock`), clearing RUSTSEC-2026-0414 (exponential event growth
  with `ENABLE_WIKILINKS` in `0.13.4`).
- `noyalib` is updated from `0.0.57` to `0.0.58`.
- The CLI build pipeline wires `MarkdownProcessor`, `HandlebarsRenderer`,
  and `HtmlGenerator`, executing full markdown-to-HTML generation with
  built-in default templating.
- `NucleusFlow::process` processes content directories recursively.
- `nucleusflow serve` implements an embedded HTTP static file server.
- `nucleusflow new` scaffolds starter markdown content and default
  Handlebars templates.
- Resolved an `RwLock` reentrancy deadlock during Handlebars template
  loading and fixed brace validation for raw `{{{...}}}` expressions.
- `build.rs` asserts Rust 1.89.0 MSRV and names `nucleusflow`.
- `.github/workflows/release.yml` conforms to the canonical GitHub
  release title specification `NucleusFlow ${VERSION}`.

## [0.0.2] - 2026-10-11

### Changed

- Bumped package version to 0.0.2.
- Updated dependencies: noyalib (0.0.57), staticdatagen (0.0.22), html5ever (0.40).
- Cleared upstream supply-chain advisories.

## [0.0.1] - 2024-05-01

### Added

- Initial release of NucleusFlow content processing library.
