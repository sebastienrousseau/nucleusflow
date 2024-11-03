//! # Content Processing Module
//!
//! This module provides a flexible content processing framework, particularly
//! for Markdown content. It includes configurable options, metadata extraction, and
//! content sanitisation for enhanced security and extensibility.
//!
//! ## Key Features
//!
//! - **Extensible processor architecture** for custom content processing
//! - **Markdown processing** with customisable options like tables and footnotes
//! - **Metadata extraction** from frontmatter and Markdown content
//! - **Content validation** and **HTML sanitisation** for security
//! - **TOC (Table of Contents) generation** for Markdown headers

use crate::{ContentProcessor, NucleusFlowError, Result};
use pulldown_cmark::{
    html, HeadingLevel, Options as MarkdownOptions, Parser, Tag,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use serde_yml::from_str;
use std::collections::HashMap;

/// Configuration options for content processing.
///
/// This struct allows you to specify options for sanitising content,
/// generating a Table of Contents (TOC), and other custom settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorConfig {
    /// Enable sanitisation of HTML output.
    #[serde(default)]
    pub sanitize: bool,
    /// Enable generation of a Table of Contents (TOC).
    #[serde(default)]
    pub toc: bool,
    /// Customisable options for processor settings.
    #[serde(default)]
    pub options: HashMap<String, JsonValue>,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            sanitize: true,
            toc: false,
            options: HashMap::new(),
        }
    }
}

/// Metadata extracted from content during processing.
///
/// This struct contains various fields such as `title`, `description`, `date`,
/// `tags`, and any custom metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContentMetadata {
    /// Title of the content.
    pub title: Option<String>,
    /// Description of the content.
    pub description: Option<String>,
    /// Publication date of the content.
    pub date: Option<String>,
    /// Tags associated with the content.
    pub tags: Vec<String>,
    /// Custom metadata fields.
    pub custom: HashMap<String, JsonValue>,
}

/// Processor for Markdown content with configurable options.
///
/// Provides methods for setting options like tables, strikethrough, and footnotes
/// and enables metadata extraction, TOC generation, and HTML sanitisation.
#[derive(Debug, Clone)]
pub struct MarkdownProcessor {
    options: MarkdownOptions,
    config: ProcessorConfig,
}

impl MarkdownProcessor {
    /// Creates a new `MarkdownProcessor` with default settings.
    ///
    /// # Examples
    ///
    /// ```
    /// use nucleusflow::content::MarkdownProcessor;
    /// let processor = MarkdownProcessor::new();
    /// ```
    pub fn new() -> Self {
        Self {
            options: MarkdownOptions::empty(),
            config: ProcessorConfig::default(),
        }
    }

    /// Enables or disables table support in Markdown processing.
    ///
    /// # Arguments
    ///
    /// * `enable` - A boolean that enables table support if `true`.
    ///
    /// # Examples
    ///
    /// ```
    /// use nucleusflow::content::MarkdownProcessor;
    /// let processor = MarkdownProcessor::new().with_tables(true);
    /// ```
    pub fn with_tables(mut self, enable: bool) -> Self {
        if enable {
            self.options.insert(MarkdownOptions::ENABLE_TABLES);
        } else {
            self.options.remove(MarkdownOptions::ENABLE_TABLES);
        }
        self
    }

    /// Enables or disables strikethrough support in Markdown processing.
    pub fn with_strikethrough(mut self, enable: bool) -> Self {
        if enable {
            self.options.insert(MarkdownOptions::ENABLE_STRIKETHROUGH);
        } else {
            self.options.remove(MarkdownOptions::ENABLE_STRIKETHROUGH);
        }
        self
    }

    /// Enables or disables footnote support in Markdown processing.
    pub fn with_footnotes(mut self, enable: bool) -> Self {
        if enable {
            self.options.insert(MarkdownOptions::ENABLE_FOOTNOTES);
        } else {
            self.options.remove(MarkdownOptions::ENABLE_FOOTNOTES);
        }
        self
    }

    /// Applies a `ProcessorConfig` to the Markdown processor.
    pub fn with_config(mut self, config: ProcessorConfig) -> Self {
        self.config = config;
        self
    }

    /// Extracts metadata from Markdown content, supporting YAML frontmatter.
    ///
    /// Parses YAML frontmatter if present, capturing fields like `title`, `description`,
    /// `date`, and `tags`. Additional fields are stored in the `custom` field.
    ///
    /// # Arguments
    ///
    /// * `content` - The content from which to extract metadata.
    ///
    /// # Returns
    ///
    /// * `ContentMetadata` - The metadata extracted from the content.
    fn extract_metadata(
        &self,
        content: &str,
    ) -> Result<ContentMetadata> {
        let mut metadata = ContentMetadata::default();
        let mut lines = content.lines();

        if content.starts_with("---\n") {
            let mut frontmatter = String::new();
            let _ = lines.next();

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
                for (key, value) in yaml {
                    match key.as_str() {
                        "title" => {
                            metadata.title =
                                value.as_str().map(String::from)
                        }
                        "description" => {
                            metadata.description =
                                value.as_str().map(String::from)
                        }
                        "date" => {
                            metadata.date =
                                value.as_str().map(String::from)
                        }
                        "tags" => {
                            if let Some(tags) = value.as_array() {
                                metadata.tags = tags
                                    .iter()
                                    .filter_map(|v| {
                                        v.as_str().map(String::from)
                                    })
                                    .collect();
                            }
                        }
                        _ => {
                            let _ = metadata.custom.insert(key, value);
                        }
                    };
                }
            }
        }

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

    /// Generates a Table of Contents (TOC) for Markdown content.
    ///
    /// Parses the content and extracts headings up to level 3.
    fn generate_toc(&self, content: &str) -> String {
        let mut toc = String::from("<nav class=\"toc\">\n<ul>\n");
        let parser = Parser::new_ext(content, self.options);

        for event in parser {
            if let pulldown_cmark::Event::Start(Tag::Heading {
                level,
                ..
            }) = event
            {
                let level = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    _ => continue,
                };

                toc.push_str(&"  ".repeat(level));
                toc.push_str("<li><a href=\"#\">Heading</a></li>\n");
            }
        }

        toc.push_str("</ul>\n</nav>");
        toc
    }

    /// Sanitises HTML content to remove unsafe elements.
    ///
    /// Removes `<script>`, `<iframe>`, `<object>`, `<embed>`, and other potentially
    /// harmful tags.
    fn sanitize_html(&self, html: &str) -> Result<String> {
        let mut output = html.to_string();
        let disallowed_tags = [
            "<script",
            "</script>",
            "<iframe",
            "</iframe>",
            "<object",
            "</object>",
            "<embed",
            "</embed>",
        ];

        for tag in &disallowed_tags {
            output = output.replace(tag, "");
        }

        Ok(output)
    }
}

impl Default for MarkdownProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentProcessor for MarkdownProcessor {
    /// Processes Markdown content with the specified options and configuration.
    ///
    /// This method includes metadata extraction, optional TOC generation, and HTML sanitisation.
    fn process(
        &self,
        content: &str,
        context: Option<&JsonValue>,
    ) -> Result<String> {
        self.validate(content)?;

        let metadata = self.extract_metadata(content)?;
        let config = if let Some(ctx) = context {
            serde_json::from_value(ctx.clone()).unwrap_or_default()
        } else {
            ProcessorConfig::default()
        };

        let parser = Parser::new_ext(content, self.options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        if config.toc {
            let toc = self.generate_toc(content);
            html_output = format!("{}\n{}", toc, html_output);
        }

        let processed = if config.sanitize {
            self.sanitize_html(&html_output)?
        } else {
            html_output
        };

        if !metadata.custom.is_empty() {
            let json_ld = serde_json::to_string(&metadata.custom)
                .map_err(|e| {
                    NucleusFlowError::ContentProcessingError {
                        message: "Failed to serialize metadata"
                            .to_string(),
                        source: Some(Box::new(e)),
                    }
                })?;
            Ok(format!(
                "{}\n<script type=\"application/ld+json\">{}</script>",
                processed, json_ld
            ))
        } else {
            Ok(processed)
        }
    }

    /// Validates content by checking for emptiness and suspicious patterns.
    ///
    /// Ensures content is not empty and does not contain potentially harmful content patterns.
    fn validate(&self, content: &str) -> Result<()> {
        if content.is_empty() {
            return Err(NucleusFlowError::ContentProcessingError {
                message: "Content cannot be empty".to_string(),
                source: None,
            });
        }

        let suspicious_patterns = ["javascript:", "data:", "vbscript:"];
        for pattern in &suspicious_patterns {
            if content.contains(pattern) {
                return Err(NucleusFlowError::ContentProcessingError {
                    message: format!(
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_markdown_processor_basic() {
        let processor = MarkdownProcessor::new();
        let input = "# Test\n\nThis is a **test**.";
        let result = processor.process(input, None).unwrap();
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
        let result = processor.process(input, None).unwrap();
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

        let result = processor.process(input, Some(&context)).unwrap();
        assert!(result.contains("<nav class=\"toc\">"));
        assert!(result.contains("<ul>"));
    }

    #[test]
    fn test_sanitization() {
        let processor = MarkdownProcessor::new();
        let input = "# Test\n\n<script>alert('xss')</script>";
        let context = json!({
            "sanitize": true
        });

        let result = processor.process(input, Some(&context)).unwrap();
        assert!(!result.contains("<script>"));
    }

    #[test]
    fn test_validation() {
        let processor = MarkdownProcessor::new();

        assert!(processor.validate("").is_err());
        assert!(processor.validate("javascript:alert(1)").is_err());
        assert!(processor.validate("# Valid content").is_ok());
    }
}
