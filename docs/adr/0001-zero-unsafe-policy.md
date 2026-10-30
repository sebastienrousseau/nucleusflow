# ADR 0001: Zero-Unsafe Code Policy

- **Status**: Accepted
- **Date**: 2026-10-30
- **Authors**: Sebastien Rousseau

## Context

Static site generators and content processing pipelines frequently parse untrusted user inputs, including third-party Markdown files, arbitrary YAML/JSON frontmatter, and asset paths. In memory-unsafe environments or Rust codebases that rely on unverified `unsafe` blocks, memory corruption bugs (buffer overflows, use-after-free, strict provenance violations) present critical security vulnerabilities.

## Decision

Enforce a strict `#![forbid(unsafe_code)]` compiler directive at the crate root of both the core library (`src/lib.rs`) and binary CLI (`src/main.rs`).

1. No `unsafe` block may be introduced into the repository for any reason.
2. If an optimization or trait implementation traditionally requires `unsafe`, a safe abstraction using Rust standard library data structures or vetted, memory-safe dependencies must be employed instead.
3. Automated CI gates check and reject any PR that weakens or bypasses this policy.

## Consequences

- **Positive**: Total compiler-enforced elimination of undefined behaviour (UB), spatial memory safety bugs, and temporal memory corruption within the crate boundaries.
- **Positive**: Simplified formal verification and automated review by static analysis engines.
- **Negative**: Marginal theoretical performance differences in micro-optimizations, offset by modern LLVM optimization passes on safe Rust code.
