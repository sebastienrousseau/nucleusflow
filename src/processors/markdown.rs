//! # Markdown Processing Module
//!
//! Provides a secure and flexible Markdown processing framework with configurable options,
//! metadata extraction, and content sanitization. This module focuses on safety,
//! performance, and extensibility.
//!
//! ## Key Features
//!
//! - **Safe Processing**: Secure content handling with robust sanitization
//! - **Metadata Extraction**: YAML frontmatter parsing with type-safe handling
//! - **Table of Contents**: Automatic generation of nested TOC structures
//! - **Configurable Options**: Support for tables, footnotes, and strikethrough
//! - **Content Validation**: Protection against XSS and other injection attacks
//!
//! ## Example Usage
//!
//! ```rust
//! use nucleusflow::processors::markdown::MarkdownProcessor;
//! use nucleusflow::core::traits::Processor;
//!
//! let processor = MarkdownProcessor::new()
//!     .with_tables(true)
//!     .with_footnotes(true)
//!     .with_strikethrough(true);
//!
//! let content = r#"---
//! title: My Post
//! tags: [rust, markdown]
//! ---
//! # Hello World
//!
//! This is a **markdown** document.
//! "#;
//!
//! let result = processor.process(content.to_string(), None).unwrap();
//! ```

use crate::core::{
    error::{ProcessingError, Result},
    traits::Processor,
};
use pulldown_cmark::{
    html, Event, HeadingLevel, Options as MarkdownOptions, Parser, Tag,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use serde_yml::from_str;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Maximum allowed size for Markdown content in bytes (10MB)
const MAX_CONTENT_SIZE: usize = 10 * 1024 * 1024;

/// List of allowed HTML tags that won't be stripped during sanitization
const ALLOWED_HTML_TAGS: &[&str] = &[
    "p",
    "br",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "strong",
    "em",
    "del",
    "ul",
    "ol",
    "li",
    "code",
    "pre",
    "blockquote",
    "hr",
    "table",
    "thead",
    "tbody",
    "tr",
    "th",
    "td",
    "img",
    "a",
    "nav",
];

/// Configuration options for markdown processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorConfig {
    /// Enable sanitization of HTML output
    #[serde(default = "default_true")]
    pub sanitize: bool,

    /// Enable generation of Table of Contents
    #[serde(default)]
    pub toc: bool,

    /// Maximum heading level for TOC (1-6)
    #[serde(default = "default_toc_level")]
    pub toc_max_level: u8,

    /// Enable automatic link references
    #[serde(default = "default_true")]
    pub auto_links: bool,

    /// Custom processor options
    #[serde(default)]
    pub options: HashMap<String, JsonValue>,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            sanitize: true,
            toc: false,
            toc_max_level: 3,
            auto_links: true,
            options: HashMap::new(),
        }
    }
}

/// Metadata extracted from Markdown content.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContentMetadata {
    /// Content title
    pub title: Option<String>,
    /// Content description
    pub description: Option<String>,
    /// Publication date
    pub date: Option<String>,
    /// Content tags
    pub tags: Vec<String>,
    /// Custom metadata fields
    pub custom: HashMap<String, JsonValue>,
}

/// Represents a heading in the Table of Contents.
#[derive(Debug)]
struct TocEntry {
    /// Heading text
    text: String,
    /// Heading level (1-6)
    level: u8,
    /// Generated ID for the heading
    id: String,
}

/// Processor for Markdown content with advanced features and security.
#[derive(Debug, Clone)]
pub struct MarkdownProcessor {
    options: MarkdownOptions,
    config: ProcessorConfig,
    /// Cache of allowed HTML tags for faster sanitization
    allowed_tags: Arc<HashSet<String>>,
}

impl MarkdownProcessor {
    /// Creates a new MarkdownProcessor with default settings.
    pub fn new() -> Self {
        let allowed_tags = ALLOWED_HTML_TAGS
            .iter()
            .map(|&tag| tag.to_string())
            .collect();

        Self {
            options: MarkdownOptions::empty(),
            config: ProcessorConfig::default(),
            allowed_tags: Arc::new(allowed_tags),
        }
    }

    /// Enables table support in Markdown processing.
    pub fn with_tables(mut self, enable: bool) -> Self {
        if enable {
            self.options.insert(MarkdownOptions::ENABLE_TABLES);
        } else {
            self.options.remove(MarkdownOptions::ENABLE_TABLES);
        }
        self
    }

    /// Enables strikethrough support in Markdown processing.
    pub fn with_strikethrough(mut self, enable: bool) -> Self {
        if enable {
            self.options.insert(MarkdownOptions::ENABLE_STRIKETHROUGH);
        } else {
            self.options.remove(MarkdownOptions::ENABLE_STRIKETHROUGH);
        }
        self
    }

    /// Enables footnote support in Markdown processing.
    pub fn with_footnotes(mut self, enable: bool) -> Self {
        if enable {
            self.options.insert(MarkdownOptions::ENABLE_FOOTNOTES);
        } else {
            self.options.remove(MarkdownOptions::ENABLE_FOOTNOTES);
        }
        self
    }

    /// Applies configuration options to the processor.
    pub fn with_config(mut self, config: ProcessorConfig) -> Self {
        self.config = config;
        self
    }

    /// Extracts and validates metadata from Markdown content.
    fn extract_metadata(
        &self,
        content: &str,
    ) -> Result<ContentMetadata> {
        let mut metadata = ContentMetadata::default();
        let mut lines = content.lines();

        // Handle YAML frontmatter
        if content.starts_with("---\n") {
            let mut frontmatter = String::with_capacity(1024);
            let _ = lines.next(); // Skip first "---"

            for line in lines.by_ref() {
                if line == "---" {
                    break;
                }
                frontmatter.push_str(line);
                frontmatter.push('\n');
            }

            if let Ok(yaml) =
                from_str::<HashMap<String, JsonValue>>(&frontmatter)
            {
                Self::process_metadata(&mut metadata, yaml)?;
            }
        }

        // Extract title from first H1 if not found in frontmatter
        if metadata.title.is_none() {
            for line in content.lines() {
                if let Some(title) = line.strip_prefix("# ") {
                    metadata.title = Some(title.trim().to_string());
                    break;
                }
            }
        }

        Ok(metadata)
    }

    /// Processes and validates metadata from YAML frontmatter.
    fn process_metadata(
        metadata: &mut ContentMetadata,
        yaml: HashMap<String, JsonValue>,
    ) -> Result<()> {
        for (key, value) in yaml {
            match key.as_str() {
                "title" => {
                    metadata.title = value
                        .as_str()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());
                }
                "description" => {
                    metadata.description = value
                        .as_str()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());
                }
                "date" => {
                    metadata.date = value
                        .as_str()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());
                }
                "tags" => {
                    if let Some(tags) = value.as_array() {
                        metadata.tags = tags
                            .iter()
                            .filter_map(|v| {
                                v.as_str()
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                            })
                            .collect();
                    }
                }
                _ => {
                    let _ = metadata.custom.insert(key, value);
                }
            }
        }

        Ok(())
    }

    /// Generates an accessible Table of Contents.
    fn generate_toc(&self, content: &str) -> Result<String> {
        let mut toc = String::from(
        "<nav class=\"toc\" aria-label=\"Table of Contents\">\n<ul>\n",
    );
        let mut entries = Vec::new();
        let parser = Parser::new_ext(content, self.options);
        let mut current_text = String::new();
        let mut current_level = None;

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    current_text.clear();
                    current_level = Some(level);
                }
                Event::Text(text) => {
                    current_text.push_str(&text);
                }
                Event::End(_) => {
                    if let Some(level) = current_level.take() {
                        let level_num = match level {
                            HeadingLevel::H1 => 1,
                            HeadingLevel::H2 => 2,
                            HeadingLevel::H3 => 3,
                            HeadingLevel::H4 => 4,
                            HeadingLevel::H5 => 5,
                            HeadingLevel::H6 => 6,
                        };

                        if level_num <= self.config.toc_max_level {
                            let id =
                                self.generate_heading_id(&current_text);
                            entries.push(TocEntry {
                                text: current_text.clone(),
                                level: level_num,
                                id,
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        self.build_toc_html(&mut toc, &entries)?;
        toc.push_str("</ul>\n</nav>");
        Ok(toc)
    }

    /// Generates a unique ID for a heading.
    fn generate_heading_id(&self, text: &str) -> String {
        text.to_lowercase()
            .chars()
            .filter_map(|c| match c {
                'a'..='z' | '0'..='9' => Some(c),
                ' ' | '-' | '_' => Some('-'),
                _ => None,
            })
            .collect()
    }

    /// Builds the HTML structure for the Table of Contents.
    fn build_toc_html(
        &self,
        toc: &mut String,
        entries: &[TocEntry],
    ) -> Result<()> {
        let mut current_level = 1;

        for entry in entries {
            while entry.level > current_level {
                toc.push_str("<ul>\n");
                current_level += 1;
            }
            while entry.level < current_level {
                toc.push_str("</ul>\n");
                current_level -= 1;
            }

            toc.push_str(&format!(
                "<li><a href=\"#{}\" aria-label=\"{}\">{}</a></li>\n",
                entry.id, entry.text, entry.text
            ));
        }

        while current_level > 1 {
            toc.push_str("</ul>\n");
            current_level -= 1;
        }

        Ok(())
    }

    /// Sanitizes HTML content to prevent XSS and other injection attacks.
    fn sanitize_html(&self, html: &str) -> Result<String> {
        let mut output = String::with_capacity(html.len());
        let mut in_tag = false;
        let mut current_tag = String::new();

        for c in html.chars() {
            match c {
                '<' => {
                    in_tag = true;
                    current_tag.clear();
                }
                '>' if in_tag => {
                    in_tag = false;
                    let tag_name = current_tag
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .trim_start_matches('/')
                        .to_lowercase();

                    if self.allowed_tags.contains(&tag_name) {
                        output.push('<');
                        output.push_str(&current_tag);
                        output.push('>');
                    }
                }
                _ if in_tag => {
                    current_tag.push(c);
                }
                _ => {
                    if !in_tag {
                        output.push(c);
                    }
                }
            }
        }

        Ok(output)
    }

    /// Validates that the content is safe to process.
    fn validate(&self, content: &str) -> Result<()> {
        // Check content size
        if content.len() > MAX_CONTENT_SIZE {
            return Err(ProcessingError::ContentProcessing {
                details: format!(
                    "Content exceeds maximum size of {} bytes",
                    MAX_CONTENT_SIZE
                ),
                source: None,
            });
        }

        // Check for empty content
        if content.trim().is_empty() {
            return Err(ProcessingError::ContentProcessing {
                details: "Content cannot be empty".to_string(),
                source: None,
            });
        }

        // Check for suspicious patterns
        let suspicious_patterns = [
            "javascript:",
            "data:",
            "vbscript:",
            "onclick",
            "onerror",
            "onload",
            "eval(",
        ];

        for pattern in &suspicious_patterns {
            if content.to_lowercase().contains(pattern) {
                return Err(ProcessingError::ContentProcessing {
                    details: format!(
                        "Suspicious content pattern detected: {}",
                        pattern
                    ),
                    source: None,
                });
            }
        }

        Ok(())
    }
}

impl Default for MarkdownProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Processor for MarkdownProcessor {
    type Input = String;
    type Output = String;
    type Context = JsonValue;

    fn process(
        &self,
        content: String,
        context: Option<&Self::Context>,
    ) -> Result<Self::Output> {
        // Validate content
        self.validate(&content)?;

        // Extract metadata
        let metadata = self.extract_metadata(&content)?;

        // Parse configuration from context
        let config: ProcessorConfig = context
            .and_then(|ctx| serde_json::from_value(ctx.clone()).ok())
            .unwrap_or_default();

        // Parse Markdown to HTML
        let parser = Parser::new_ext(&content, self.options);
        let mut html_output = String::with_capacity(content.len() * 2);
        html::push_html(&mut html_output, parser);

        // Generate and prepend TOC if enabled
        if config.toc {
            let toc = self.generate_toc(&content)?;
            println!("Generated ToC: {}", toc); // Debugging line
            html_output = format!("{}\n{}", toc, html_output);
        }

        // Sanitize if enabled
        let processed = if config.sanitize {
            self.sanitize_html(&html_output)?
        } else {
            html_output
        };

        // Add metadata as JSON-LD if present
        if !metadata.custom.is_empty() {
            let json_ld = serde_json::to_string(&metadata.custom)
                .map_err(|e| ProcessingError::ContentProcessing {
                    details: "Failed to serialize metadata".to_string(),
                    source: Some(Box::new(e)),
                })?;
            Ok(format!(
                "{}\n<script type=\"application/ld+json\">{}</script>",
                processed, json_ld
            ))
        } else {
            Ok(processed)
        }
    }
}

// Helper functions for default values
fn default_true() -> bool {
    true
}

fn default_toc_level() -> u8 {
    3
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_markdown_processor_basic() {
        let processor = MarkdownProcessor::new();
        let input = "# Test\n\nThis is a **test**.";
        let result = processor.process(input.to_owned(), None).unwrap();
        assert!(result.contains("<h1>"));
        assert!(result.contains("<strong>"));
    }

    #[test]
    fn test_markdown_processor_with_options() {
        let processor = MarkdownProcessor::new()
            .with_tables(true)
            .with_strikethrough(true);

        let input =
            "# Test\n\n| A | B |\n|---|---|\n| 1 | 2 |\n\n~~strike~~";
        let result = processor.process(input.to_owned(), None).unwrap();
        assert!(result.contains("<table>"));
        assert!(result.contains("<del>"));
    }

    #[test]
    fn test_metadata_extraction() {
        let processor = MarkdownProcessor::new();
        let input = r#"---
title: Test Post
description: A test post
date: 2024-01-01
tags:
  - test
  - example
custom_field: value
---

# Content"#;

        let metadata = processor.extract_metadata(input).unwrap();
        assert_eq!(metadata.title, Some("Test Post".to_string()));
        assert_eq!(
            metadata.description,
            Some("A test post".to_string())
        );
        assert_eq!(metadata.date, Some("2024-01-01".to_string()));
        assert_eq!(metadata.tags, vec!["test", "example"]);
        assert!(metadata.custom.contains_key("custom_field"));
    }

    #[test]
    fn test_toc_generation() {
        let processor = MarkdownProcessor::new();
        let input = "# H1\n\n## H2\n\n### H3";
        let context = json!({
            "toc": true
        });

        let result = processor
            .process(input.to_owned(), Some(&context))
            .unwrap();
        println!("Result: {}", result); // Debugging line

        assert!(result.contains(r#"<nav class="toc""#));
        assert!(result.contains("<ul>"));
    }

    #[test]
    fn test_sanitization() {
        let processor = MarkdownProcessor::new();
        let input = "# Test\n\n<script>alert('xss')</script>";
        let context = json!({
            "sanitize": true
        });

        let result = processor
            .process(input.to_owned(), Some(&context))
            .unwrap();
        assert!(!result.contains("<script>"));
    }

    #[test]
    fn test_validation() {
        let processor = MarkdownProcessor::new();

        // Test empty content
        assert!(processor.validate("").is_err());

        // Test content size
        let large_content = "a".repeat(MAX_CONTENT_SIZE + 1);
        assert!(processor.validate(&large_content).is_err());

        // Test suspicious patterns
        assert!(processor.validate("javascript:alert(1)").is_err());
        assert!(processor.validate("onclick='alert(1)'").is_err());

        // Test valid content
        assert!(processor.validate("# Valid content").is_ok());
    }

    #[test]
    fn test_heading_id_generation() {
        let processor = MarkdownProcessor::new();
        let id = processor.generate_heading_id("Hello World! 123");
        assert_eq!(id, "hello-world-123");
    }

    #[test]
    fn test_custom_metadata() {
        let processor = MarkdownProcessor::new();
        let input = r#"---
title: Test
custom:
  key1: value1
  key2: 42
---
# Content"#;

        let metadata = processor.extract_metadata(input).unwrap();
        assert!(metadata.custom.contains_key("custom"));
    }

    #[test]
    fn test_sanitization_with_allowed_tags() {
        let processor = MarkdownProcessor::new();
        let input = r#"
<p>Valid paragraph</p>
<script>alert('bad')</script>
<img src="valid.jpg" alt="valid">
<iframe src="bad.html"></iframe>
"#;
        let result = processor.sanitize_html(input).unwrap();

        assert!(result.contains("<p>"));
        assert!(result.contains("<img"));
        assert!(!result.contains("<script>"));
        assert!(!result.contains("<iframe>"));
    }
}
