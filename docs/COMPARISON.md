<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Ecosystem Comparison

Comparison of NucleusFlow against other static site generation and content processing tools.

| Capability | NucleusFlow | Zola | Hugo | Cobalt |
| :--- | :---: | :---: | :---: | :---: |
| **Language** | Rust | Rust | Go | Rust |
| **Zero `unsafe`** | Yes (`#![forbid(unsafe_code)]`) | No | N/A | No |
| **Library First** | Yes (Modular crates & traits) | CLI focused | CLI focused | CLI focused |
| **Handlebars Templating** | Yes | Tera | Go Templates | Liquid |
| **Built-in HTML/CSS Minification**| Yes (lightningcss & minify-html) | Optional | Built-in | Limited |
| **Path Traversal Protection** | Enforced & Fuzzed | Partial | Standard | Basic |
| **Concurrency Model** | In-Memory `parking_lot` + Rayon | Multi-threaded | Goroutines | Single-threaded |
