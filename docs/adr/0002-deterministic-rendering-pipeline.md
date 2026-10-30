# ADR 0002: Deterministic Thread-Safe Rendering Pipeline

- **Status**: Accepted
- **Date**: 2026-10-30
- **Authors**: Sebastien Rousseau

## Context

Static site pipelines must be fast, highly concurrent, and completely deterministic. When rendering thousands of Markdown content pages in parallel across CPU cores, shared mutable state can cause race conditions, non-deterministic output order, or deadlock conditions during template compilation.

## Decision

1. Separate template registration and compilation from page execution. Templates are compiled once into an immutable or read-biased `parking_lot::RwLock` registry before worker threads process content documents.
2. Design document transformation stages around pure, side-effect-free processing traits (`Processor`, `Generator`, `Transform`).
3. Ensure identical input markdown and configuration always produces identical bit-for-bit HTML output across runs and operating systems.
4. Verify parallel concurrency without filesystem state pollution using isolated in-memory unit and property tests.

## Consequences

- **Positive**: Guaranteed reproducibility of built websites across CI/CD and developer machines.
- **Positive**: High throughput scaling linearly with CPU core counts.
- **Positive**: Elimination of thread reentrancy deadlocks during layout rendering.
