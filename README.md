<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/clients/nucleusflow/v1/logos/nucleusflow.svg" alt="NucleusFlow logo" width="128" />
</p>

<h1 align="center">NucleusFlow</h1>

<p align="center">
  A fast, flexible, and secure static site generator and content processing pipeline written in Rust.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/nucleusflow/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/nucleusflow/ci.yml?style=for-the-badge&logo=github" alt="Build" height="28" /></a>
  <a href="https://crates.io/crates/nucleusflow"><img src="https://img.shields.io/crates/v/nucleusflow.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Registry" height="28" /></a>
  <a href="https://docs.rs/nucleusflow"><img src="https://img.shields.io/badge/docs.rs-nucleusflow-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs" height="28" /></a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/sebastienrousseau/nucleusflow"><img src="https://img.shields.io/ossf-scorecard/github.com/sebastienrousseau/nucleusflow?style=for-the-badge&label=OpenSSF%20Scorecard&logo=openssf" alt="OpenSSF Scorecard" height="28" /></a>
  <a href="LICENSE-APACHE"><img src="https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue.svg?style=for-the-badge" alt="License: Apache-2.0 OR MIT" height="28" /></a>
  <a href="https://github.com/sebastienrousseau/nucleusflow/blob/main/docs/POLICIES.md"><img src="https://img.shields.io/badge/MSRV-1.89.0-93450a.svg?style=for-the-badge&logo=rust" alt="MSRV 1.89.0" height="28" /></a>
</p>

---

## Contents

**Getting started**

- [Install](#install) - Cargo, CLI binary, and library setup
- [Requirements](#requirements) - toolchain floor, platforms
- [Quick Start](#quick-start) - compile and run a static site pipeline in Rust

**The NucleusFlow ecosystem**

- [The NucleusFlow ecosystem](#the-nucleusflow-ecosystem) - core processing components and traits

**Library reference**

- [Capabilities at a glance](#capabilities-at-a-glance) - the current surface by theme
- [Ecosystem comparison](#ecosystem-comparison) - short matrix; full table at [`docs/COMPARISON.md`](docs/COMPARISON.md)
- [Benchmarks](#benchmarks) - headline numbers; full table at [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md)
- [Features](#features) - module-level capability list
- [Configuration](#configuration) - core options
- [Examples](#examples) - runnable example index

**Operational**

- [When not to use NucleusFlow](#when-not-to-use-nucleusflow) - limitations
- [Development](#development) - make targets, test suites, CI
- [Security](#security) - guarantees and compliance
- [Documentation](#documentation) - all reference docs
- [Stability guarantees](#stability-guarantees) - SemVer axis, output stability, minimum toolchain discipline
- [License](#license)

---

## Install

### Pre-built binaries

Download the latest pre-compiled archive for your platform from [GitHub Releases](https://github.com/sebastienrousseau/nucleusflow/releases/latest):

```bash
# Extract and install into your local PATH
tar -xzf nucleusflow-*.tar.gz
sudo make -f GNUmakefile install
```

### Via Cargo

```bash
cargo install nucleusflow --locked
```

### As a Rust library dependency

Add NucleusFlow to your `Cargo.toml`:

```toml
[dependencies]
nucleusflow = "0.0.7"
```

Or run:

```bash
cargo add nucleusflow
```

### Package managers and distribution

Formulas and packaging recipes are provided under [`pkg/`](pkg/):

- **Homebrew**: [`pkg/brew/nucleusflow.rb`](pkg/brew/nucleusflow.rb)
- **Arch Linux (AUR)**: [`pkg/aur/PKGBUILD`](pkg/aur/PKGBUILD)
- **Debian / Ubuntu**: [`pkg/debian/`](pkg/debian/)
- **RPM (Fedora / RHEL)**: [`pkg/rpm/nucleusflow.spec`](pkg/rpm/nucleusflow.spec)
- **Nix**: [`pkg/nix/flake.nix`](pkg/nix/flake.nix)
- **Container**: `docker build -t nucleusflow .`

---

## Requirements

- Rust **1.89.0 or newer** for building and running all library and test targets.
- Linux, macOS, and Windows are tested in continuous integration.

| Component | Minimum toolchain | Enforcement |
| :--- | :---: | :--- |
| `nucleusflow` | **1.89.0** | Pinned via `rust-version` in `Cargo.toml`; enforced by CI. |

---

## Quick Start

```rust,no_run
use nucleusflow::{
    FileContentProcessor, HtmlOutputGenerator, HtmlTemplateRenderer,
    NucleusFlow, NucleusFlowConfig,
};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = NucleusFlowConfig::new("content", "public", "templates")?;
    let content_processor = FileContentProcessor::new(PathBuf::from("content"));
    let template_renderer = HtmlTemplateRenderer::new(PathBuf::from("templates"));
    let output_generator = HtmlOutputGenerator::new(PathBuf::from("public"));

    let flow = NucleusFlow::new(
        config,
        Box::new(content_processor),
        Box::new(template_renderer),
        Box::new(output_generator),
    );

    flow.process()?;
    Ok(())
}
```

This snippet initializes a full static site generation pipeline with Markdown parsing, Handlebars layout templating, and minified HTML output generation.

---

## The NucleusFlow ecosystem

NucleusFlow provides modular building blocks for document transformation pipelines:

| Component | Purpose | Use case |
| :--- | :--- | :--- |
| `NucleusFlow` | Top-level orchestrator | Coordinates processor, renderer, and output stages |
| `FileContentProcessor` | Content reader & parser | Reads Markdown sources and parses YAML frontmatter |
| `HtmlTemplateRenderer` | Layout engine | Renders dynamic Handlebars templates with site context |
| `HtmlOutputGenerator` | Output writer | Minifies HTML/CSS and writes static assets securely |
| `ConfigBuilder` | Fluent configuration | Customises file size limits, environment prefixes, and paths |

---

## Capabilities at a glance

| Area | Capability | Status |
| :--- | :--- | :--- |
| Safety | Zero `unsafe` guarantee (`#![forbid(unsafe_code)]`) | Active |
| Security | Path traversal rejection and input sanitisation | Active |
| Testing | Property-based state space exploration via `proptest` | Active |
| Concurrency | In-memory thread safety without filesystem locks | Active |
| Tooling | CLI for scaffold (`new`), build, and development server | Active |

---

## Ecosystem comparison

NucleusFlow focuses on library-first architecture, memory safety guarantees, and embeddable content pipelines.

| Project | Zero `unsafe` | Library-first API | Handlebars templates |
| :--- | :---: | :---: | :---: |
| **NucleusFlow** | Yes (`#![forbid(unsafe_code)]`) | Yes | Yes |
| **Zola** | No | No (CLI only) | No (Tera) |
| **Cobalt** | No | No (CLI only) | No (Liquid) |

See [`docs/COMPARISON.md`](docs/COMPARISON.md) for the evidence and complete matrix.

---

## Benchmarks

Measurements on Apple Silicon (M4 / macOS) executing the end-to-end static generation pipeline:

| Scenario | Result | Environment |
| :--- | ---: | :--- |
| Single Page Processing | ~28.0 µs | Apple M4, Rust 1.89.0 |
| Markdown with Frontmatter | ~4.2 µs | Apple M4, Rust 1.89.0 |
| HTML Minification | ~8.1 µs | Apple M4, Rust 1.89.0 |

See [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md) for methodology and full results.

---

## Features

- **Strict Path Security**: Rejects all path traversal attempts (`..`, root directory escapes, invalid Unicode).
- **Zero `unsafe` Code**: Guaranteed memory safety at compile time via `#![forbid(unsafe_code)]`.
- **Property-Tested Correctness**: Verified against arbitrary Unicode strings and edge-case inputs.
- **Embedded Static Server**: Built-in development server with custom mime detection and streaming responses.
- **Minification Engine**: Integrated HTML and CSS minification via `minify-html` and `lightningcss`.

---

## Configuration

NucleusFlow is configured via `NucleusFlowConfig` or the fluent `ConfigBuilder`:

```rust,no_run
use nucleusflow::ConfigBuilder;

let config = ConfigBuilder::new()
    .with_max_file_size(10 * 1024 * 1024) // 10 MB limit
    .with_auto_reload(true)
    .build()
    .expect("Valid configuration");
```

---

## Examples

To run the basic library example:

```bash
cargo run --example basic
```

CLI scaffolding:

```bash
nucleusflow new my-site --template blog
nucleusflow build --content content/ --output public/
```

---

## When not to use NucleusFlow

- **Dynamic Server-Side Rendering**: NucleusFlow generates static files ahead of time; it is not a dynamic web application framework.
- **Distributed Database Storage**: Content is ingested from directory trees and filesystems, not remote relational databases.

---

## Development

```bash
make check   # check code for syntax and typing errors
make clippy  # run clippy lints with warnings denied
make test    # run unit, integration, and property tests
make fmt     # verify code formatting
```

To reproduce the primary verification gate:

```bash
make
```

---

## Security

NucleusFlow enforces strict memory safety guarantees and rejects malicious path navigation.

Report vulnerabilities according to [`SECURITY.md`](SECURITY.md).

---

## Documentation

- [API Reference (docs.rs)](https://docs.rs/nucleusflow)
- [Architecture Guide](docs/ARCHITECTURE.md)
- [Engineering Policies](docs/POLICIES.md)
- [Ecosystem Comparison](docs/COMPARISON.md)
- [Benchmarks](docs/BENCHMARKS.md)
- [Developer Guide](DEVELOPMENT.md)

---

## Stability guarantees

- Semantic Versioning is strictly adhered to on a `+0.0.1` iteration cycle.
- Public trait signatures and configuration models follow deprecation windows prior to breaking changes.
- Output file path generation and templating semantics remain backwards compatible across non-breaking releases.

---

## License

Dual-licensed under either:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
