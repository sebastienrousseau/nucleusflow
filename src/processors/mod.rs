//! # Content Processors Module
//!
//! This module provides various content processors for different formats and types.
//! Each processor implements the core processing traits and handles specific content
//! types (e.g., Markdown, HTML, etc.).
//!
//! ## Available Processors
//!
//! - [`markdown`]: Processes Markdown content with support for frontmatter and extensions
//!
//! ## Usage
//!
//! ```rust,no_run
//! use nucleusflow::processors::markdown::MarkdownProcessor;
//! use nucleusflow::core::traits::Processor;
//! use nucleusflow::ContentProcessor;
//!
//! let processor = MarkdownProcessor::new()
//!     .with_tables(true)
//!     .with_footnotes(true);
//!
//! let content = "# Hello World\n\nThis is a test.";
//! let result = processor.process(content.to_string(), None).unwrap();
//! ```
//!
//! ## Implementing Custom Processors
//!
//! Custom processors can be implemented by creating a new type that implements
//! the [`Processor`] trait:
//!
//! ```rust
//! # use nucleusflow::core::traits::Processor;
//! # use nucleusflow::core::error::Result;
//! # use serde_json::Value;
//! #[derive(Debug)]
//! struct CustomProcessor;
//!
//! impl Processor for CustomProcessor {
//!     type Input = String;
//!     type Output = String;
//!     type Context = Value;
//!
//!     fn process(
//!         &self,
//!         input: Self::Input,
//!         _context: Option<&Self::Context>
//!     ) -> Result<Self::Output> {
//!         Ok(input.to_uppercase())
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Markdown processing functionality.
pub mod markdown;

// Re-export commonly used types
pub use markdown::MarkdownProcessor;

/// Configuration options for content processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorConfig {
    /// Enable content validation
    #[serde(default = "default_true")]
    pub validate: bool,

    /// Enable content sanitization
    #[serde(default = "default_true")]
    pub sanitize: bool,

    /// Custom processor options
    #[serde(default)]
    pub options: HashMap<String, JsonValue>,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            validate: true,
            sanitize: true,
            options: HashMap::new(),
        }
    }
}

/// Common metadata structure for processed content.
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

// Helper function for serde defaults
fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_config_defaults() {
        let config = ProcessorConfig::default();
        assert!(config.validate);
        assert!(config.sanitize);
        assert!(config.options.is_empty());
    }

    #[test]
    fn test_content_metadata_defaults() {
        let metadata = ContentMetadata::default();
        assert!(metadata.title.is_none());
        assert!(metadata.description.is_none());
        assert!(metadata.date.is_none());
        assert!(metadata.tags.is_empty());
        assert!(metadata.custom.is_empty());
    }
}
