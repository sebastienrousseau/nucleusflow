//! # HTML Output Generation
//!
//! This module provides a secure and flexible HTML output generator implementing the `Generator` trait.
//! It focuses on secure content handling and efficient processing, maintaining high performance.
//!
//! # Features
//!
//! - Secure HTML content processing with proper escaping
//! - Configurable minification and formatting
//! - Thread-safe metadata management
//! - Secure asset handling with path validation
//! - Memory-efficient string processing
//!
//! # Examples
//!
//! Basic usage:
//! ```rust,no_run
//! use nucleusflow::generators::html::{HtmlGenerator, OutputConfig};
//! use nucleusflow::core::traits::Generator;
//! use std::path::PathBuf;
//!
//! let generator = HtmlGenerator::new()
//!     .with_minification(true)
//!     .with_pretty_print(false)
//!     .with_metadata(serde_json::json!({
//!         "description": "My page description"
//!     }))
//!     .with_asset_dir("assets").unwrap();
//!
//! generator.generate(
//!     "<html><body>Hello World</body></html>",
//!     &PathBuf::from("output/index.html"),
//!     None
//! ).unwrap();
//! ```

use crate::core::traits::Generator;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use log;
use minify_html::{minify, Cfg};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::{ProcessingError, Result};

/// List of HTML5 void elements that don't need closing tags
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link",
    "meta", "param", "source", "track", "wbr",
];

/// List of optional tags in HTML5
const OPTIONAL_TAGS: &[&str] = &[
    "html", "head", "body", "tbody", "thead", "tfoot", "tr", "th",
    "td", "li", "dt", "dd",
];

/// Configuration options for HTML output generation.
/// Provides thread-safe, comprehensive control over HTML processing and generation.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Controls HTML minification
    pub minify: bool,

    /// Enables formatted output with proper indentation
    pub pretty_print: bool,

    /// Optional metadata for HTML head injection
    pub metadata: Option<JsonValue>,

    /// Optional directory for static assets
    pub asset_dir: Option<PathBuf>,

    /// Additional configuration options
    pub options: HashMap<String, JsonValue>,
}

/// HTML output generator with secure processing and asset management.
/// Provides thread-safe HTML generation with features like:
/// - Content sanitization
/// - Asset management
/// - Metadata injection
/// - Output formatting
#[derive(Clone)]
pub struct HtmlGenerator {
    /// Thread-safe configuration storage
    config: Arc<RwLock<OutputConfig>>,

    /// Thread-safe asset cache
    asset_cache: Arc<RwLock<HashMap<PathBuf, Vec<u8>>>>,
}

impl HtmlGenerator {
    /// Creates a new HtmlGenerator with default settings.
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(OutputConfig::default())),
            asset_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Enables or disables HTML minification.
    pub fn with_minification(self, enable: bool) -> Self {
        self.config.write().minify = enable;
        self
    }

    /// Enables or disables pretty printing of output HTML.
    pub fn with_pretty_print(self, enable: bool) -> Self {
        self.config.write().pretty_print = enable;
        self
    }

    /// Sets metadata to be injected into the HTML head.
    pub fn with_metadata(self, metadata: JsonValue) -> Self {
        self.config.write().metadata = Some(metadata);
        self
    }

    /// Configures the directory for static assets.
    pub fn with_asset_dir<P: AsRef<Path>>(
        self,
        path: P,
    ) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() || !path.is_dir() {
            return Err(ProcessingError::FileOperation {
                details: "Invalid or non-existent asset directory"
                    .to_string(),
                path: path.clone(),
                source: None,
            });
        }
        _ = fs::read_dir(&path).map_err(|e| {
            ProcessingError::FileOperation {
                details: "Cannot read asset directory".to_string(),
                path: path.clone(),
                source: Some(Box::new(e)),
            }
        })?;
        self.config.write().asset_dir = Some(path);
        Ok(self)
    }

    /// Processes and optimizes HTML content based on configuration.
    ///
    /// This function handles:
    /// - Content validation and sanitization
    /// - Metadata injection
    /// - HTML optimization (minification/pretty printing)
    /// - Error handling with detailed context
    fn process_html(&self, content: &str) -> Result<String> {
        let config = self.config.read();

        // Step 1: Validate HTML structure before any processing
        if !self.is_valid_html(content) {
            return Err(ProcessingError::FileOperation {
                details: "Initial HTML structure validation failed"
                    .to_string(),
                path: PathBuf::new(),
                source: None,
            });
        }

        // Step 2: Copy the content to allow modifications, allocate buffer size
        let estimated_size = content.len()
            + config
                .metadata
                .as_ref()
                .map_or(0, |m| m.to_string().len());
        let mut processed = String::with_capacity(estimated_size);
        processed.push_str(content);

        // Step 3: Inject metadata if provided in the configuration
        if let Some(metadata) = &config.metadata {
            if let Err(e) =
                self.inject_metadata(&mut processed, metadata)
            {
                return Err(ProcessingError::FileOperation {
                    details: "Failed to inject metadata".to_string(),
                    path: PathBuf::new(),
                    source: Some(Box::new(e)),
                });
            }
        }

        // Step 4: Apply minification or pretty printing based on configuration
        let optimized_content =
            match (config.minify, config.pretty_print) {
                (true, _) => self.minify_html(&processed)?,
                (false, true) => self.pretty_print_html(&processed),
                (false, false) => processed.clone(),
            };

        // Step 5: Final validation of processed HTML content
        if !self.is_valid_html(&optimized_content) {
            return Err(ProcessingError::FileOperation {
                details:
                    "Processed HTML is invalid after transformation"
                        .to_string(),
                path: PathBuf::new(),
                source: None,
            });
        }

        Ok(optimized_content)
    }

    /// Validates basic HTML structure and syntax with HTML5 support
    fn is_valid_html(&self, content: &str) -> bool {
        let mut tag_stack: Vec<String> = Vec::new();
        let mut in_tag = false;
        let mut in_comment = false;
        let mut tag_start = 0;

        let mut chars = content.chars().enumerate().peekable();
        while let Some((i, c)) = chars.next() {
            match c {
                '<' => {
                    if !in_tag && !in_comment {
                        in_tag = true;
                        tag_start = i;

                        // Check for comment start
                        if content[i..].starts_with("<!--") {
                            in_comment = true;
                            in_tag = false;
                            // Skip the rest of comment opening
                            for _ in 0..3 {
                                let _ = chars.next();
                            }
                            continue;
                        }
                    }
                }
                '>' => {
                    if in_comment {
                        // Check for comment end
                        if i >= 2 && &content[i - 2..=i] == "-->" {
                            in_comment = false;
                        }
                    } else if in_tag {
                        in_tag = false;
                        let tag = &content[tag_start..=i];

                        // Skip doctypes, XML declarations, etc.
                        if tag.starts_with("<!")
                            || tag.starts_with("<?")
                        {
                            continue;
                        }

                        // Extract tag name, handling attributes
                        let tag_name = if let Some(stripped) =
                            tag.strip_prefix("</")
                        {
                            // Closing tag
                            stripped
                                .split_whitespace()
                                .next()
                                .unwrap_or("")
                                .trim_end_matches('>')
                                .to_lowercase()
                        } else {
                            // Opening tag
                            tag[1..]
                                .split_whitespace()
                                .next()
                                .unwrap_or("")
                                .trim_end_matches('>')
                                .trim_end_matches('/')
                                .to_lowercase()
                        };

                        // Skip empty or invalid tags
                        if tag_name.is_empty() {
                            continue;
                        }

                        if tag.starts_with("</") {
                            // Handle closing tag
                            if !VOID_ELEMENTS
                                .contains(&tag_name.as_str())
                            {
                                match tag_stack.last() {
                                    Some(last_tag)
                                        if last_tag == &tag_name =>
                                    {
                                        _ = tag_stack.pop();
                                    }
                                    Some(last_tag)
                                        if OPTIONAL_TAGS.contains(
                                            &last_tag.as_str(),
                                        ) =>
                                    {
                                        // Pop optional tags until we find a match
                                        while let Some(top) =
                                            tag_stack.last()
                                        {
                                            if top == &tag_name {
                                                _ = tag_stack.pop();
                                                break;
                                            } else if OPTIONAL_TAGS
                                                .contains(&top.as_str())
                                            {
                                                _ = tag_stack.pop();
                                            } else {
                                                return false; // Mismatched non-optional tag
                                            }
                                        }
                                    }
                                    Some(_) => return false, // Mismatched non-optional tag
                                    None => {} // Ignore extra closing tags for optional elements
                                }
                            }
                        } else if !tag.ends_with("/>")
                            && !VOID_ELEMENTS
                                .contains(&tag_name.as_str())
                        {
                            // Push opening tag
                            tag_stack.push(tag_name);
                        }
                    }
                }
                '-' if in_comment => {
                    // Check for premature comment end
                    if i >= 1 && content[i - 1..=i] == *"--" {
                        if let Some((_, '>')) = chars.peek() {
                            in_comment = false;
                            let _ = chars.next();
                        }
                    }
                }
                _ => continue,
            }
        }

        // Handle any remaining tags - only optional tags can be unclosed
        !in_tag
            && !in_comment
            && tag_stack
                .iter()
                .all(|tag| OPTIONAL_TAGS.contains(&tag.as_str()))
    }

    /// Injects metadata into HTML head section with proper escaping and structure handling
    fn inject_metadata(
        &self,
        content: &mut String,
        metadata: &JsonValue,
    ) -> Result<()> {
        // First ensure we have DOCTYPE and html structure
        if !content.trim_start().starts_with("<!DOCTYPE")
            && !content.trim_start().starts_with("<!doctype")
        {
            content.insert_str(0, "<!DOCTYPE html>");
        }

        // Ensure we have a head section
        if !content.contains("<head") {
            let (prefix, insert_pos) =
                if let Some(pos) = content.find("<html") {
                    // Insert after html tag
                    let end_pos = content[pos..]
                        .find('>')
                        .map(|p| p + pos + 1)
                        .unwrap_or(pos + 5);
                    ("<head>", end_pos)
                } else {
                    // Add html tag if missing
                    let prefix = if !content.contains("<html") {
                        "<html><head>"
                    } else {
                        "<head>"
                    };
                    (
                        prefix,
                        content
                            .find("<!DOCTYPE html>")
                            .map_or(0, |p| p + "<!DOCTYPE html>".len()),
                    )
                };

            content.insert_str(insert_pos, prefix);
            // Don't insert closing head here - we'll handle it after metadata
        }

        // Generate and insert meta tags
        let meta_tags = self.generate_meta_tags(metadata)?;

        if let Some(head_pos) = content.find("</head>") {
            content.insert_str(head_pos, &meta_tags);
        } else {
            // If no closing head tag, add meta tags and close head
            if let Some(head_start) = content.find("<head>") {
                content.insert_str(
                    head_start + 6,
                    &format!("{}</head>", meta_tags),
                );
            } else {
                return Err(ProcessingError::FileOperation {
                    details: "Failed to locate or create head section"
                        .to_string(),
                    path: PathBuf::new(),
                    source: None,
                });
            }
        }

        Ok(())
    }

    /// Generates HTML meta tags from metadata JSON.
    fn generate_meta_tags(
        &self,
        metadata: &JsonValue,
    ) -> Result<String> {
        let mut meta_tags = String::new();
        if let Some(obj) = metadata.as_object() {
            for (key, value) in obj {
                if let Some(content) = value.as_str() {
                    // Escape key and content here for security
                    meta_tags.push_str(&format!(
                        r#"<meta name="{}" content="{}">"#,
                        handlebars::html_escape(key),
                        handlebars::html_escape(content)
                    ));
                }
            }
        }
        Ok(meta_tags)
    }

    /// Minifies HTML content using the `minify-html` crate.
    fn minify_html(&self, content: &str) -> Result<String> {
        let cfg = Cfg {
            minify_css: true,
            minify_js: true,
            ..Cfg::default()
        };
        String::from_utf8(minify(content.as_bytes(), &cfg)).map_err(
            |e| ProcessingError::FileOperation {
                details: "HTML minification failed".to_string(),
                path: PathBuf::new(),
                source: Some(Box::new(e)),
            },
        )
    }

    /// Formats HTML with indentation and line breaks.
    fn pretty_print_html(&self, content: &str) -> String {
        let mut pretty = String::new();
        let mut depth: i32 = 0;
        let mut in_tag = false;
        let mut is_closing_tag = false;

        for c in content.chars() {
            match c {
                '<' => {
                    if !in_tag {
                        if is_closing_tag {
                            depth = depth.saturating_sub(1);
                        }
                        pretty.push('\n');
                        pretty.push_str(
                            &"    ".repeat(depth.try_into().unwrap()),
                        );
                        if !is_closing_tag {
                            depth += 1;
                        }
                    }
                    in_tag = true;
                    is_closing_tag = false;
                    pretty.push('<');
                }
                '/' if in_tag => is_closing_tag = true,
                '>' => {
                    pretty.push('>');
                    in_tag = false;
                }
                _ => pretty.push(c),
            }
        }
        pretty
    }

    /// Copies static assets to the output directory with caching.
    fn copy_assets(&self, output_dir: &Path) -> Result<()> {
        if let Some(asset_dir) = &self.config.read().asset_dir {
            let mut cache = self.asset_cache.write();
            for entry in fs::read_dir(asset_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    self.process_asset(
                        &path, asset_dir, output_dir, &mut cache,
                    )?;
                }
            }
        }
        Ok(())
    }

    /// Processes a single asset file, caching and copying it as needed.
    fn process_asset(
        &self,
        path: &Path,
        asset_dir: &Path,
        output_dir: &Path,
        cache: &mut HashMap<PathBuf, Vec<u8>>,
    ) -> Result<()> {
        let cached_content = cache
            .entry(path.to_path_buf())
            .or_insert_with(|| fs::read(path).unwrap_or_default());
        let relative_path =
            path.strip_prefix(asset_dir).map_err(|_| {
                ProcessingError::FileOperation {
                    details: "Invalid asset path".to_string(),
                    path: path.to_path_buf(),
                    source: None,
                }
            })?;
        let output_path = output_dir.join(relative_path);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&output_path, cached_content)?;
        Ok(())
    }

    /// Adds a custom configuration option
    pub fn with_option(self, key: &str, value: JsonValue) -> Self {
        let _ =
            self.config.write().options.insert(key.to_string(), value);
        self
    }

    /// Gets the current configuration
    pub fn get_config(&self) -> OutputConfig {
        self.config.read().clone()
    }

    /// Validates HTML content without processing it
    pub fn validate_content(&self, content: &str) -> Result<()> {
        if !self.is_valid_html(content) {
            return Err(ProcessingError::FileOperation {
                details: "Invalid HTML structure".to_string(),
                path: PathBuf::new(),
                source: None,
            });
        }
        Ok(())
    }

    /// Clears the asset cache to free memory
    pub fn clear_cache(&self) -> Result<()> {
        self.asset_cache.write().clear();
        Ok(())
    }

    /// Updates metadata without regenerating the entire document
    pub fn update_metadata(
        &self,
        path: &Path,
        metadata: JsonValue,
    ) -> Result<()> {
        let content = fs::read_to_string(path)?;
        let mut processed = content.clone();

        // Remove existing meta tags
        if let (Some(start), Some(end)) =
            (processed.find("<head>"), processed.find("</head>"))
        {
            let head_content = &processed[start + 6..end];
            let new_head = head_content
                .lines()
                .filter(|line| !line.trim().starts_with("<meta"))
                .collect::<Vec<_>>()
                .join("\n");
            processed.replace_range(start + 6..end, &new_head);
        }

        // Add new metadata
        self.inject_metadata(&mut processed, &metadata)?;

        // Write back to file
        fs::write(path, processed)?;
        Ok(())
    }

    /// Gets statistics about the processed HTML
    pub fn get_stats(&self, content: &str) -> HashMap<String, usize> {
        let mut stats = HashMap::new();

        // Count tags
        let mut tag_count = 0;
        let mut inside_tag = false;

        for c in content.chars() {
            match c {
                '<' if !inside_tag => {
                    inside_tag = true;
                    tag_count += 1;
                }
                '>' if inside_tag => {
                    inside_tag = false;
                }
                _ => {}
            }
        }

        let _ = stats.insert("tag_count".to_string(), tag_count);
        let _ = stats.insert("size_bytes".to_string(), content.len());
        let _ = stats
            .insert("line_count".to_string(), content.lines().count());

        stats
    }

    /// Gets the list of cached assets
    pub fn get_cached_assets(&self) -> Vec<PathBuf> {
        self.asset_cache.read().keys().cloned().collect()
    }

    /// Checks if an asset is cached
    pub fn is_asset_cached(&self, path: &Path) -> bool {
        self.asset_cache.read().contains_key(path)
    }
}

impl Generator for HtmlGenerator {
    fn generate(
        &self,
        content: &str,
        path: &Path,
        options: Option<&JsonValue>,
    ) -> Result<()> {
        self.validate(path, options)?;
        let processed = self.process_html(content)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(processed.as_bytes())?;
        writer.flush()?;
        if let Some(parent) = path.parent() {
            self.copy_assets(parent)?;
        }
        Ok(())
    }

    fn validate(
        &self,
        path: &Path,
        options: Option<&JsonValue>,
    ) -> Result<()> {
        if path.extension().and_then(|s| s.to_str()) != Some("html") {
            return Err(ProcessingError::FileOperation {
                details: "Invalid file extension - expected .html"
                    .to_string(),
                path: path.to_path_buf(),
                source: None,
            });
        }
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        if let Some(opts) = options {
            if !opts.is_object() {
                return Err(ProcessingError::FileOperation {
                    details:
                        "Invalid options format - expected JSON object"
                            .to_string(),
                    path: path.to_path_buf(),
                    source: None,
                });
            }
            if let Some(obj) = opts.as_object() {
                for (key, value) in obj {
                    match key.as_str() {
                        "minify" if !value.is_boolean() => {
                            return Err(ProcessingError::FileOperation {
                                details: "minify option must be a boolean".to_string(),
                                path: path.to_path_buf(),
                                source: None,
                            });
                        }
                        "indent_size" if !value.is_number() => {
                            return Err(ProcessingError::FileOperation {
                                details: "indent_size option must be a number".to_string(),
                                path: path.to_path_buf(),
                                source: None,
                            });
                        }
                        _ => log::warn!("Unknown option key: {}", key),
                    }
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for HtmlGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HtmlGenerator")
            .field("config", &*self.config.read())
            .field("asset_cache_size", &self.asset_cache.read().len())
            .finish()
    }
}

impl Default for HtmlGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::traits::Generator;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn test_basic_output() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("output.html");
        let content = "<h1>Test</h1>";

        let generator = HtmlGenerator::new(); // Pretty print is now off by default
        generator.generate(content, &output_path, None)?;

        let result = fs::read_to_string(&output_path)?;
        assert_eq!(result.trim(), content);

        Ok(())
    }

    #[test]
    fn test_minification() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("output.html");
        let content = "<h1>\n    Test\n</h1>";

        let generator = HtmlGenerator::new()
            .with_minification(true)
            .with_pretty_print(false);

        generator.generate(content, &output_path, None)?;

        let result = fs::read_to_string(&output_path)?;
        assert_eq!(result, "<h1>Test</h1>");

        Ok(())
    }

    #[test]
    fn test_asset_handling() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let asset_dir = temp_dir.path().join("assets");
        let output_dir = temp_dir.path().join("output");
        fs::create_dir(&asset_dir)?;

        // Create test asset
        let asset_content = "test asset";
        fs::write(asset_dir.join("test.txt"), asset_content)?;

        let generator =
            HtmlGenerator::new().with_asset_dir(&asset_dir)?;

        let output_path = output_dir.join("index.html");
        generator.generate("<h1>Test</h1>", &output_path, None)?;

        let copied_asset =
            fs::read_to_string(output_dir.join("test.txt"))?;
        assert_eq!(copied_asset, asset_content);

        Ok(())
    }

    #[test]
    fn test_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let generator = HtmlGenerator::new();

        // Test invalid file extension
        let result = generator.generate(
            "test",
            &temp_dir.path().join("test.txt"),
            None,
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid file extension"));

        // Test invalid options
        let result = generator.generate(
            "test",
            &temp_dir.path().join("test.html"),
            Some(&json!("invalid")),
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid options format"));
    }

    #[test]
    fn test_options_validation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("output.html");
        let generator = HtmlGenerator::new();

        // Test valid options
        generator.validate(
            &output_path,
            Some(&json!({
                "minify": true,
                "indent_size": 4
            })),
        )?;

        // Test invalid minify option
        let result = generator.validate(
            &output_path,
            Some(&json!({
                "minify": "true" // Should be boolean
            })),
        );
        assert!(result.is_err());

        // Test invalid indent_size option
        let result = generator.validate(
            &output_path,
            Some(&json!({
                "indent_size": "4" // Should be number
            })),
        );
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_concurrent_output() -> Result<()> {
        use std::sync::Arc;
        use std::thread;

        let temp_dir = TempDir::new()?;
        let generator = Arc::new(HtmlGenerator::new());
        let mut handles = vec![];

        for i in 0..10 {
            let generator = Arc::clone(&generator);
            let output_path =
                temp_dir.path().join(format!("output{}.html", i));

            let handle = thread::spawn(move || {
                generator.generate(
                    &format!("<h1>Test {}</h1>", i),
                    &output_path,
                    None,
                )
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap()?;
        }

        Ok(())
    }

    #[test]
    fn test_large_file_handling() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("large.html");

        // Generate a large HTML file
        let mut content = String::with_capacity(1_000_000);
        for i in 0..10_000 {
            content.push_str(&format!("<div>Test {}</div>\n", i));
        }

        let generator = HtmlGenerator::new();
        generator.generate(&content, &output_path, None)?;

        let result = fs::read_to_string(&output_path)?;
        assert_eq!(
            result
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count(),
            10_000
        );

        Ok(())
    }

    #[test]
    fn test_memory_efficiency() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("output.html");
        let generator = HtmlGenerator::new();

        // Test processing with pre-allocated buffer
        let content = "<div>".repeat(1000) + &"</div>".repeat(1000);
        generator.generate(&content, &output_path, None)?;

        Ok(())
    }

    #[test]
    fn test_html5_void_elements() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("output.html");
        let content = r#"
            <html>
                <head>
                    <meta charset="utf-8">
                    <link rel="stylesheet" href="style.css">
                </head>
                <body>
                    <img src="test.jpg">
                    <br>
                    <input type="text">
                </body>
            </html>"#;

        let generator = HtmlGenerator::new();
        generator.generate(content, &output_path, None)?;

        let result = fs::read_to_string(&output_path)?;
        assert!(result.contains("<meta"));
        assert!(result.contains("<img"));
        assert!(result.contains("<br"));

        Ok(())
    }

    #[test]
    fn test_comment_handling() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("output.html");
        let content = r#"<!-- Header -->
        <header>Test</header>
        <!-- Multi-line
             comment -->
        <main>
            <!-- Nested <div>Test</div> -->
            <p>Content</p>
        </main>"#;

        let generator = HtmlGenerator::new();
        generator.generate(content, &output_path, None)?;

        let result = fs::read_to_string(&output_path)?;
        assert!(result.contains("<!-- Header -->"));
        assert!(result.contains("<!-- Multi-line"));
        assert!(result.contains("<!-- Nested"));

        Ok(())
    }

    #[test]
    fn test_head_section_creation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("output.html");

        // Test with no head section
        let content = "<body>Test</body>";
        let generator = HtmlGenerator::new().with_metadata(json!({
            "description": "Test page"
        }));

        generator.generate(content, &output_path, None)?;

        let result = fs::read_to_string(&output_path)?;
        assert!(result.contains("<head>"));
        assert!(result.contains("</head>"));
        assert!(result.contains(
            r#"<meta name="description" content="Test page">"#
        ));

        Ok(())
    }

    #[test]
    fn test_doctype_handling() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("output.html");

        // Test without DOCTYPE
        let content = "<html><body>Test</body></html>";
        let generator = HtmlGenerator::new().with_metadata(json!({
            "description": "Test page"
        }));

        generator.generate(content, &output_path, None)?;

        let result = fs::read_to_string(&output_path)?;
        assert!(result.contains("<!DOCTYPE html>"));

        // Test with existing DOCTYPE
        let content = "<!DOCTYPE html><html><body>Test</body></html>";
        generator.generate(content, &output_path, None)?;

        let result = fs::read_to_string(&output_path)?;
        assert_eq!(result.matches("<!DOCTYPE html>").count(), 1);

        Ok(())
    }

    #[test]
    fn test_optional_tags() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_path = temp_dir.path().join("output.html");
        let content = r#"
            <html>
                <body>
                    <table>
                        <tr><td>Cell 1</td><td>Cell 2</td>
                        <tr><td>Cell 3</td><td>Cell 4</td>
                    </table>
                </body>
            </html>"#;

        let generator = HtmlGenerator::new();
        generator.generate(content, &output_path, None)?;

        // Should not fail validation despite missing </tr> tags
        let result = fs::read_to_string(&output_path)?;
        assert!(result.contains("Cell 1"));
        assert!(result.contains("Cell 4"));

        Ok(())
    }

    #[test]
    fn test_with_option() -> Result<()> {
        let generator = HtmlGenerator::new()
            .with_option("custom_key", json!("custom_value"));
        assert_eq!(
            generator.config.read().options.get("custom_key"),
            Some(&json!("custom_value"))
        );
        Ok(())
    }

    #[test]
    fn test_validate_content() -> Result<()> {
        let generator = HtmlGenerator::new();

        // Valid HTML
        assert!(generator.validate_content("<div>Test</div>").is_ok());

        // Invalid HTML
        assert!(generator.validate_content("<div>Test</p>").is_err());

        Ok(())
    }

    #[test]
    fn test_update_metadata() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let path = temp_dir.path().join("test.html");

        let initial_content = r#"<!DOCTYPE html><html><head><meta name="old" content="old"></head><body>Test</body></html>"#;
        fs::write(&path, initial_content)?;

        let generator = HtmlGenerator::new();
        generator.update_metadata(&path, json!({"new": "new"}))?;

        let result = fs::read_to_string(&path)?;
        assert!(!result.contains(r#"name="old""#));
        assert!(result.contains(r#"name="new""#));

        Ok(())
    }

    #[test]
    fn test_get_stats() -> Result<()> {
        let generator = HtmlGenerator::new();
        let content = "<div>\n<p>Test</p>\n</div>";

        let stats = generator.get_stats(content);
        assert_eq!(stats.get("tag_count"), Some(&4));
        assert_eq!(stats.get("line_count"), Some(&3));

        Ok(())
    }

    #[test]
    fn test_cache_operations() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let asset_path = temp_dir.path().join("test.txt");
        fs::write(&asset_path, "test")?;

        let generator =
            HtmlGenerator::new().with_asset_dir(temp_dir.path())?;

        // Test cache operations
        assert!(!generator.is_asset_cached(&asset_path));
        generator.generate(
            "<div>Test</div>",
            &temp_dir.path().join("test.html"),
            None,
        )?;
        assert!(generator.is_asset_cached(&asset_path));

        generator.clear_cache()?;
        assert!(!generator.is_asset_cached(&asset_path));

        Ok(())
    }
}
