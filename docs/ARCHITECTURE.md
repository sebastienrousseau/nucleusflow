<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# NucleusFlow Architecture

NucleusFlow is designed as a modular, pipeline-driven static site generator and document transformation engine written in pure Rust.

## Architectural Overview

The core processing pipeline is split into clean stages with explicit trait boundaries:

```text
Raw Content (.md)
       │
       ▼
┌───────────────────────┐
│ Content Processor     │ ── extracts frontmatter (YAML), converts Markdown to HTML
└───────────────────────┘
       │
       ▼
┌───────────────────────┐
│ Template Renderer     │ ── applies Handlebars layout templates and contextual data
└───────────────────────┘
       │
       ▼
┌───────────────────────┐
│ Output Generator      │ ── minifies HTML/CSS assets, enforces secure output paths
└───────────────────────┘
       │
       ▼
Static Site Artifacts (public/)
```

## Core Abstractions

1. **`Processor` trait**: Defines content reading, parsing, and metadata transformation.
2. **`Renderer` trait**: Defines layout templating and variable injection.
3. **`Generator` trait**: Defines the writing phase, including asset placement and minification.
4. **`NucleusFlowConfig`**: Strong type-safe configuration validating content directory, output directory, and template directories.

## Security Boundaries & Guarantees

- **`#![forbid(unsafe_code)]`**: Compile-time prohibition of unsafe Rust across library and binary crates.
- **Path Sanitisation**: Validates all relative file targets against path traversal exploits (`..`, absolute paths, symlink escapes).
- **In-Memory Concurrency**: Thread-safe operations using `parking_lot::RwLock` for high-throughput batch generation without disk contention.
