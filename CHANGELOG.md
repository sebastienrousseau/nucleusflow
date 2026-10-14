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

## [0.0.2] - 2026-10-11

### Changed

- Bumped package version to 0.0.2.
- Updated dependencies: noyalib (0.0.57), staticdatagen (0.0.22), html5ever (0.40).
- Cleared upstream supply-chain advisories.

## [0.0.1] - 2024-05-01

### Added

- Initial release of NucleusFlow content processing library.
