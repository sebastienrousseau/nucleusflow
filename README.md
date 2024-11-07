<!-- markdownlint-disable MD033 MD041 -->
<img src="https://kura.pro/nucleusflow/images/logos/nucleusflow.svg"
alt="NucleusFlow logo" height="66" align="right" />
<!-- markdownlint-enable MD033 MD041 -->

# `NucleusFlow`

A fast, flexible and secure static site generator written in Rust.

<!-- markdownlint-disable MD033 MD041 -->
<center>
<!-- markdownlint-enable MD033 MD041 -->

[![Made With Love][made-with-rust]][08] [![Crates.io][crates-badge]][03] [![lib.rs][libs-badge]][01] [![Docs.rs][docs-badge]][04] [![Codecov][codecov-badge]][06] [![Build Status][build-badge]][07] [![GitHub][github-badge]][09]

• [Website][00] • [Documentation][04] • [Report Bug][02] • [Request Feature][02] • [Contributing Guidelines][05]

<!-- markdownlint-disable MD033 MD041 -->
</center>
<!-- markdownlint-enable MD033 MD041 -->

## Overview

NucleusFlow is a powerful content processing library and static site generator that prioritises security, performance and flexibility. Built in Rust, it offers a comprehensive toolkit for managing content lifecycles, from processing raw content to generating optimised static websites.

## Features

- **Secure Content Processing**
  - Robust input validation and sanitisation
  - Path traversal protection
  - Memory-safe operations
  - Secure defaults for all operations

- **Flexible Content Pipeline**
  - Markdown processing with frontmatter support
  - Template rendering with Handlebars
  - HTML generation with minification
  - Asset management and optimisation

- **Performance Optimised**
  - Parallel processing capabilities
  - Efficient memory usage
  - Content caching
  - Minimal dependencies

- **Developer Experience**
  - Intuitive CLI interface
  - Rich error messages
  - Extensive documentation
  - Type-safe configurations

## Installation

Add `nucleusflow` to your `Cargo.toml`:

```toml
[dependencies]
nucleusflow = "0.0.1"
```

## Usage

Here's a basic example of how to use `nucleusflow`:

```rust,no_run
use nucleusflow::{NucleusFlow, NucleusFlowConfig, FileContentProcessor, HtmlOutputGenerator, HtmlTemplateRenderer};
use std::path::PathBuf;

// Create configuration
let config = NucleusFlowConfig::new(
    "content",
    "public",
    "templates"
).expect("Failed to create config");

// Initialize processors with the concrete implementations
let content_processor = FileContentProcessor::new(PathBuf::from("content"));
let template_renderer = HtmlTemplateRenderer::new(PathBuf::from("templates"));
let output_generator = HtmlOutputGenerator::new(PathBuf::from("public"));

// Create NucleusFlow instance
let nucleus = NucleusFlow::new(
    config,
    Box::new(content_processor),
    Box::new(template_renderer),
    Box::new(output_generator)
);

// Process content
nucleus.process().expect("Failed to process content");
```

### CLI Usage

```bash
# Create a new site
nucleusflow new my-site --template blog

# Build the site
nucleusflow build --content content/ --output public/

```

This example demonstrates setting up NucleusFlow with a Markdown processor, Handlebars templating, and HTML output generation.

## Documentation

For full API documentation, please visit [docs.rs/nucleusflow][04].

## Examples

To explore more examples, clone the repository and run the following command:

```shell
cargo run --example example_name
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of

- [Apache License, Version 2.0][10]
- [MIT license][11]

at your option.

## Acknowledgements

Special thanks to all contributors who have helped build the `nucleusflow` library.

[00]: https://nucleusflow.com
[01]: https://lib.rs/crates/nucleusflow
[02]: https://github.com/sebastienrousseau/nucleusflow/issues
[03]: https://crates.io/crates/nucleusflow
[04]: https://docs.rs/nucleusflow
[05]: https://github.com/sebastienrousseau/nucleusflow/blob/main/CONTRIBUTING.md
[06]: https://codecov.io/gh/sebastienrousseau/nucleusflow
[07]: https://github.com/sebastienrousseau/nucleusflow/actions?query=branch%3Amain
[08]: https://www.rust-lang.org/
[09]: https://github.com/sebastienrousseau/nucleusflow
[10]: https://www.apache.org/licenses/LICENSE-2.0
[11]: https://opensource.org/licenses/MIT

[build-badge]: https://img.shields.io/github/actions/workflow/status/sebastienrousseau/nucleusflow/release.yml?branch=main&style=for-the-badge&logo=github
[codecov-badge]: https://img.shields.io/codecov/c/github/sebastienrousseau/nucleusflow?style=for-the-badge&token=psbZ8MASWj&logo=codecov
[crates-badge]: https://img.shields.io/crates/v/nucleusflow.svg?style=for-the-badge&color=fc8d62&logo=rust
[docs-badge]: https://img.shields.io/badge/docs.rs-nucleusflow-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
[github-badge]: https://img.shields.io/badge/github-sebastienrousseau/nucleusflow-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.1-orange.svg?style=for-the-badge
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust
