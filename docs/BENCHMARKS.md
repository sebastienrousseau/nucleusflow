<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Benchmarks

Benchmarks measured on Apple Silicon (M4 / macOS) using criterion:

| Scenario | Result | Notes |
| :--- | ---: | :--- |
| Markdown parsing + frontmatter | ~4.2 µs | Standard article with metadata |
| HTML templating (Handlebars) | ~12.5 µs | Layout with nested partials |
| HTML minification | ~8.1 µs | Minification and whitespace collapse |
| End-to-end page pipeline | ~28.0 µs | Read, parse, render, minify, write |

Run benchmarks locally via `cargo bench`.
