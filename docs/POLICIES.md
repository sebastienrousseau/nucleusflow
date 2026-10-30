<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# NucleusFlow Engineering Policies

This document serves as the policy baseline for NucleusFlow covering MSRV, SemVer, panics, security audits, and concurrency guarantees.

## 1. MSRV (Minimum Supported Rust Version)

- The project MSRV is **1.89.0**, pinned in `Cargo.toml` under `rust-version`.
- Any bump to the MSRV requires verification across the entire feature matrix in CI.

## 2. SemVer & API Stability

- Releases follow strict incremental semantic versioning (`+0.0.1`).
- Breaking changes in `0.0.x` are documented in `CHANGELOG.md`.
- Changes to output file paths or HTML structure are considered breaking.

## 3. Panic Policy

- Library methods MUST return `Result<T, NucleusFlowError>` rather than panicking on invalid input.
- Input validation bounds check all file operations prior to execution.

## 4. Concurrency & Soundness

- NucleusFlow forbids `unsafe` code entirely (`#![forbid(unsafe_code)]`).
- Concurrent operations are verified via Miri and multi-threaded integration suites.
