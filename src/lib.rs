// Copyright © 2024 NucleusFlow. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # NucleusFlow Library
//!
//! NucleusFlow provides a suite of tools for processing, rendering, and generating
//! content for static sites or other document-based applications.
//! This library includes support for content transformation, template rendering, and output
//! generation with a configurable pipeline for flexible usage.
//!
//! For more information, visit the [NucleusFlow documentation](https://docs.rs/nucleusflow).

#![doc = include_str!("../README.md")]
#![doc(
    html_favicon_url = "https://kura.pro/nucleusflow/images/favicon.ico",
    html_logo_url = "https://kura.pro/nucleusflow/images/logos/nucleusflow.svg",
    html_root_url = "https://docs.rs/nucleusflow"
)]
#![crate_name = "nucleusflow"]
#![crate_type = "lib"]

use crate::core::error::{NucleusFlowError, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Module containing core utilities, such as configuration and error handling.
pub mod core {
    /// Handles configuration of the NucleusFlow application.
    pub mod config;
    /// Contains error types and handling for NucleusFlow.
    pub mod error;
}

/// Provides command-line interface utilities.
pub mod cli;

/// Provides content processing utilities.
pub mod content;

/// Provides output generation utilities.
pub mod generators;

/// Provides processing pipeline utilities.
pub mod process;

/// Provides template rendering utilities.
pub mod template;

/// Trait for content processing implementations.
///
/// Implementations of this trait process content, transforming it based on
/// a given context.
pub trait ContentProcessor: Send + Sync + std::fmt::Debug {
    /// Processes the provided content with an optional context.
    ///
    /// # Arguments
    /// * `content` - The content to be processed.
    /// * `context` - An optional context for additional processing.
    ///
    /// # Returns
    /// * `Result<String>` - The processed content, or an error if processing fails.
    fn process(
        &self,
        content: &str,
        context: Option<&serde_json::Value>,
    ) -> Result<String>;

    /// Validates the content without processing.
    ///
    /// # Arguments
    /// * `content` - The content to be validated.
    ///
    /// # Returns
    /// * `Result<()>` - Indicates success if the content is valid, or an error if invalid.
    fn validate(&self, content: &str) -> Result<()>;
}

/// Trait for template rendering implementations.
///
/// This trait defines methods for rendering and validating templates.
pub trait TemplateRenderer: Send + Sync + std::fmt::Debug {
    /// Renders a template with the specified context.
    ///
    /// # Arguments
    /// * `template` - The template name or identifier.
    /// * `context` - The context data for rendering the template.
    ///
    /// # Returns
    /// * `Result<String>` - The rendered output, or an error if rendering fails.
    fn render(
        &self,
        template: &str,
        context: &serde_json::Value,
    ) -> Result<String>;

    /// Validates the template against the context.
    ///
    /// # Arguments
    /// * `template` - The template name or identifier.
    /// * `context` - The context data.
    ///
    /// # Returns
    /// * `Result<()>` - Indicates success if valid, or an error otherwise.
    fn validate(
        &self,
        template: &str,
        context: &serde_json::Value,
    ) -> Result<()>;
}

/// Trait for output generation implementations.
///
/// Defines methods for generating output files.
pub trait OutputGenerator: Send + Sync + std::fmt::Debug {
    /// Generates output from the given content to the specified path.
    ///
    /// # Arguments
    /// * `content` - The content to be output.
    /// * `path` - The output file path.
    /// * `options` - Optional settings for generation.
    ///
    /// # Returns
    /// * `Result<()>` - Indicates success, or an error if generation fails.
    fn generate(
        &self,
        content: &str,
        path: &Path,
        options: Option<&serde_json::Value>,
    ) -> Result<()>;

    /// Validates the path and options for output generation.
    ///
    /// # Arguments
    /// * `path` - The output file path.
    /// * `options` - Optional settings for generation.
    ///
    /// # Returns
    /// * `Result<()>` - Indicates success if valid, or an error otherwise.
    fn validate(
        &self,
        path: &Path,
        options: Option<&serde_json::Value>,
    ) -> Result<()>;
}

/// Concrete implementation of `ContentProcessor` that processes file content.
///
/// This processor transforms content to uppercase as a simple example.
#[derive(Debug)]
pub struct FileContentProcessor {
    /// The base path for content files.
    pub base_path: PathBuf,
}

impl FileContentProcessor {
    /// Creates a new `FileContentProcessor`.
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
}

impl ContentProcessor for FileContentProcessor {
    fn process(
        &self,
        content: &str,
        _context: Option<&serde_json::Value>,
    ) -> Result<String> {
        Ok(content.to_uppercase())
    }

    fn validate(&self, _content: &str) -> Result<()> {
        Ok(())
    }
}

/// Concrete implementation of `TemplateRenderer` for rendering HTML templates.
#[derive(Debug)]
pub struct HtmlTemplateRenderer {
    /// The base path for template files.
    pub base_path: PathBuf,
}

impl HtmlTemplateRenderer {
    /// Creates a new `HtmlTemplateRenderer`.
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
}

impl TemplateRenderer for HtmlTemplateRenderer {
    fn render(
        &self,
        _template: &str,
        context: &serde_json::Value,
    ) -> Result<String> {
        Ok(format!(
            "<html>{}</html>",
            context["content"].as_str().unwrap_or("")
        ))
    }

    fn validate(
        &self,
        _template: &str,
        _context: &serde_json::Value,
    ) -> Result<()> {
        Ok(())
    }
}

/// Concrete implementation of `OutputGenerator` for generating HTML files.
#[derive(Debug)]
pub struct HtmlOutputGenerator {
    /// The base path for output files.
    pub base_path: PathBuf,
}

impl HtmlOutputGenerator {
    /// Creates a new `HtmlOutputGenerator`.
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
}

impl OutputGenerator for HtmlOutputGenerator {
    fn generate(
        &self,
        content: &str,
        path: &Path,
        _options: Option<&serde_json::Value>,
    ) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                NucleusFlowError::io_error(parent.to_path_buf(), e)
            })?;
        }
        let mut file = fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    fn validate(
        &self,
        path: &Path,
        _options: Option<&serde_json::Value>,
    ) -> Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    NucleusFlowError::io_error(parent.to_path_buf(), e)
                })?;
            }
        }
        Ok(())
    }
}

/// Configuration settings for NucleusFlow.
#[derive(Debug, Clone)]
pub struct NucleusFlowConfig {
    /// The directory containing content files.
    pub content_dir: PathBuf,
    /// The directory for generated output files.
    pub output_dir: PathBuf,
    /// The directory containing template files.
    pub template_dir: PathBuf,
}

impl NucleusFlowConfig {
    /// Creates a new `NucleusFlowConfig` and validates directory paths.
    pub fn new<P: AsRef<Path>>(
        content_dir: P,
        output_dir: P,
        template_dir: P,
    ) -> Result<Self> {
        let content_dir = content_dir.as_ref().to_path_buf();
        let output_dir = output_dir.as_ref().to_path_buf();
        let template_dir = template_dir.as_ref().to_path_buf();

        for dir in [&content_dir, &template_dir] {
            if !dir.exists() || !dir.is_dir() {
                return Err(NucleusFlowError::config_error(
                    "Invalid directory",
                    Some(dir.clone()),
                ));
            }
        }

        if !output_dir.exists() {
            fs::create_dir_all(&output_dir).map_err(|e| {
                NucleusFlowError::config_error(
                    format!("Failed to create output directory: {}", e),
                    Some(output_dir.clone()),
                )
            })?;
        }

        Ok(Self {
            content_dir,
            output_dir,
            template_dir,
        })
    }
}

/// Main content processing pipeline for NucleusFlow.
#[derive(Debug)]
pub struct NucleusFlow {
    config: NucleusFlowConfig,
    content_processor: Box<dyn ContentProcessor>,
    template_renderer: Box<dyn TemplateRenderer>,
    output_generator: Box<dyn OutputGenerator>,
}

impl NucleusFlow {
    /// Creates a new instance of `NucleusFlow`.
    pub fn new(
        config: NucleusFlowConfig,
        content_processor: Box<dyn ContentProcessor>,
        template_renderer: Box<dyn TemplateRenderer>,
        output_generator: Box<dyn OutputGenerator>,
    ) -> Self {
        Self {
            config,
            content_processor,
            template_renderer,
            output_generator,
        }
    }

    /// Processes content files, transforms, renders, and generates HTML output.
    pub fn process(&self) -> Result<()> {
        for entry in fs::read_dir(&self.config.content_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                self.process_file(&path)?;
            }
        }
        Ok(())
    }

    /// Processes a single file within the pipeline.
    ///
    /// # Arguments
    /// * `path` - The path to the file to be processed.
    ///
    /// # Returns
    /// * `Result<()>` - Indicates success, or an error if processing fails.
    fn process_file(&self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)?;
        let processed =
            self.content_processor.process(&content, None)?;
        let context =
            serde_json::json!({ "content": processed, "path": path });

        let template_name = "default";
        let rendered =
            self.template_renderer.render(template_name, &context)?;

        let relative_path = path
            .strip_prefix(&self.config.content_dir)
            .map_err(|e| NucleusFlowError::ContentProcessingError {
                message: format!(
                    "Failed to determine relative path: {}",
                    e
                ),
                source: None,
            })?;
        let output_path = self
            .config
            .output_dir
            .join(relative_path)
            .with_extension("html");

        self.output_generator.generate(
            &rendered,
            &output_path,
            None,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_nucleus_flow_config_new() {
        let temp_dir = TempDir::new().unwrap();
        let content_path = temp_dir.path().join("content");
        let output_path = temp_dir.path().join("output");
        let template_path = temp_dir.path().join("templates");

        fs::create_dir(&content_path).unwrap();
        fs::create_dir(&template_path).unwrap();

        let config = NucleusFlowConfig::new(
            &content_path,
            &output_path,
            &template_path,
        )
        .unwrap();

        assert_eq!(config.content_dir, content_path);
        assert_eq!(config.output_dir, output_path);
        assert_eq!(config.template_dir, template_path);
    }

    #[test]
    fn test_nucleus_flow_process() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let content_path = temp_dir.path().join("content");
        let output_path = temp_dir.path().join("output");
        let template_path = temp_dir.path().join("templates");

        fs::create_dir(&content_path)?;
        fs::create_dir(&template_path)?;

        let test_content = "test content";
        let content_file = content_path.join("test.txt");
        fs::write(&content_file, test_content)?;

        let config = NucleusFlowConfig::new(
            &content_path,
            &output_path,
            &template_path,
        )?;

        let nucleus = NucleusFlow::new(
            config,
            Box::new(FileContentProcessor::new(content_path.clone())),
            Box::new(HtmlTemplateRenderer::new(template_path.clone())),
            Box::new(HtmlOutputGenerator::new(output_path.clone())),
        );

        nucleus.process()?;

        let output_file = output_path.join("test.html");
        assert!(output_file.exists());

        let output_content = fs::read_to_string(output_file)?;
        assert_eq!(output_content, "<html>TEST CONTENT</html>");

        Ok(())
    }
}
