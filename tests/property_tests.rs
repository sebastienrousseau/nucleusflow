// Copyright © 2024 NucleusFlow. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Property-based tests and formal invariant checks for NucleusFlow.

#![forbid(unsafe_code)]

use nucleusflow::core::config::sanitize_path;
use nucleusflow::generators::html::HtmlGenerator;
use nucleusflow::processors::markdown::MarkdownProcessor;
use nucleusflow::template::HandlebarsRenderer;
use nucleusflow::{ContentProcessor, Generator, TemplateRenderer};
use proptest::prelude::*;
use std::path::PathBuf;
use tempfile::TempDir;

proptest! {
    /// Invariant: MarkdownProcessor::process must never panic on arbitrary string input.
    #[test]
    fn prop_markdown_processor_no_panic(input in "\\PC*") {
        let processor = MarkdownProcessor::new()
            .with_tables(true)
            .with_footnotes(true)
            .with_strikethrough(true);

        // Validation must be deterministic and panic-free
        let val_1 = processor.validate(&input).is_ok();
        let val_2 = processor.validate(&input).is_ok();
        prop_assert_eq!(val_1, val_2);

        // Process must terminate cleanly (Ok or Err, never panic)
        let _ = processor.process(&input, None);
    }

    /// Invariant: MarkdownProcessor with structured frontmatter never panics.
    #[test]
    fn prop_markdown_with_frontmatter(
        title in "[a-zA-Z0-9 _-]{1,64}",
        author in "[a-zA-Z0-9 _-]{1,32}",
        body in "\\PC*"
    ) {
        let processor = MarkdownProcessor::new();
        let document = format!("---\ntitle: {}\nauthor: {}\n---\n\n{}", title, author, body);
        let result = processor.process(&document, None);
        prop_assert!(result.is_ok());
    }

    /// Invariant: Path sanitization must reject path traversal components.
    #[test]
    fn prop_path_sanitization(
        prefix in "[a-zA-Z0-9_-]{1,16}",
        traversal in "(\\.\\.[\\\\/])+",
        suffix in "[a-zA-Z0-9_.-]{1,16}"
    ) {
        let malicious_path = PathBuf::from(format!("{}{}{}", prefix, traversal, suffix));
        let sanitized = sanitize_path(&malicious_path);
        // Sanitization must strip traversal components
        for comp in sanitized.components() {
            prop_assert_ne!(comp.as_os_str(), "..");
        }
    }

    /// Invariant: HtmlGenerator minification never panics on arbitrary string input.
    #[test]
    fn prop_html_minification_no_panic(html in "\\PC*") {
        let temp_dir = TempDir::new().unwrap();
        let out_file = temp_dir.path().join("prop_out.html");
        let generator = HtmlGenerator::new().with_minification(true);

        // Minification must terminate safely without panicking
        let _ = generator.generate(&html, &out_file, None);
    }
}

/// Miri-compatible pure in-memory concurrency and template validation test.
#[test]
fn test_in_memory_concurrency_and_provenance() {
    use parking_lot::RwLock;
    use std::sync::Arc;

    let temp_dir = TempDir::new().unwrap();
    let renderer = Arc::new(RwLock::new(
        HandlebarsRenderer::new(temp_dir.path()).unwrap(),
    ));

    let mut handles = Vec::new();
    for i in 0..4 {
        let renderer_clone = Arc::clone(&renderer);
        handles.push(std::thread::spawn(move || {
            let context = serde_json::json!({
                "title": format!("Concurrent Page {}", i),
                "content": "<p>Thread-safe rendering</p>"
            });

            // Concurrent read-access to template renderer
            let guard = renderer_clone.read();
            let rendered = guard.render("default", &context);
            assert!(rendered.is_ok());
            let html = rendered.unwrap();
            assert!(html.contains("Concurrent Page"));
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
