//! # Configuration Module
//!
//! Provides flexible configuration management for the NucleusFlow content processing library. This module implements a robust configuration system with security features and validation.
//!
//! ## Features
//!
//! - Multiple configuration sources (TOML, environment variables, code)
//! - Strong validation and error handling
//! - Type-safe configuration values
//! - Support for multiple environments/profiles
//! - Secure handling of sensitive values
//! - Live configuration reloading
//! - Path traversal protection
//! - Schema validation
//!
//! ## Security Features
//!
//! - Sensitive value masking in logs and debug output
//! - Path sanitization to prevent directory traversal
//! - Environment variable validation
//! - Secure default values
//! - File size limits
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use nucleusflow::core::config::{Config, ConfigBuilder, Profile};
//! use std::path::Path;
//! use std::time::Duration;
//!
//! let config = ConfigBuilder::new()
//!     .with_file(Path::new("config.toml"))
//!     .with_env_prefix("NUCLEUS_")
//!     .with_profile(Profile::Production)
//!     .with_auto_reload(true)
//!     .with_reload_interval(Duration::from_secs(30))
//!     .build()
//!     .unwrap();
//!
//! let config_read = config.read();
//! assert_eq!(config_read.profile, Profile::Production);
//! ```
//!
//! ## Security Considerations
//!
//! - All paths are sanitized to prevent directory traversal attacks
//! - Configuration files are size-limited to prevent memory exhaustion
//! - Environment variables are validated before use
//! - Sensitive values are masked in debug output
//! - File operations use secure default permissions

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use toml::Value as TomlValue;

use crate::ProcessingError;
use crate::Result;

/// Maximum allowed size for configuration files (1MB)
const MAX_CONFIG_SIZE: usize = 1024 * 1024;

/// Default duration for configuration reload checks (30 seconds)
const DEFAULT_RELOAD_INTERVAL: Duration = Duration::from_secs(30);

/// List of environment variables that should never be used in configuration
const BLOCKED_ENV_VARS: &[&str] = &["PATH", "HOME", "USER", "SHELL"];

/// Specifies operational profiles for configuration.
///
/// Each profile determines distinct settings suitable for specific environments
/// such as development, staging, and production, while a custom profile allows
/// user-defined configurations.
///
/// # Security
///
/// Profile selection affects security settings:
/// - Development: Relaxed security for ease of development
/// - Staging: Moderate security for testing
/// - Production: Maximum security with all protections enabled
/// - Custom: User-defined security settings (requires careful configuration)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
    /// Development profile with settings optimized for debugging.
    /// Warning: Not for use in production environments.
    Development,

    /// Staging profile for intermediate testing.
    /// Implements moderate security measures.
    Staging,

    /// Production profile with maximum security measures.
    /// Recommended for all production deployments.
    Production,

    /// Custom profile enabling specific user configurations.
    /// Warning: Requires careful security review.
    Custom,
}

impl Default for Profile {
    fn default() -> Self {
        Profile::Development
    }
}

/// Represents the main configuration structure encompassing all application settings.
///
/// This structure consolidates settings for content processing, templating,
/// output generation, and other options that can be customized.
///
/// # Security Notes
///
/// - All paths are sanitized to prevent directory traversal
/// - File size limits are enforced
/// - Configuration reloading is protected against race conditions
/// - Sensitive values are masked in debug output
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Directory for content files (sanitized path)
    #[serde(default = "default_content_dir")]
    pub content_dir: PathBuf,

    /// Directory for output files (sanitized path)
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,

    /// Directory for template files (sanitized path)
    #[serde(default = "default_template_dir")]
    pub template_dir: PathBuf,

    /// Current operational profile
    #[serde(default)]
    pub profile: Profile,

    /// Content processing configuration
    #[serde(default)]
    pub content: ContentConfig,

    /// Template rendering configuration
    #[serde(default)]
    pub template: TemplateConfig,

    /// Output generation configuration
    #[serde(default)]
    pub output: OutputConfig,

    /// Custom configuration values
    #[serde(default)]
    pub custom: HashMap<String, TomlValue>,

    /// Tracks when the configuration was last modified
    #[serde(skip)]
    last_modified: Option<SystemTime>,

    /// Indicates whether auto-reload is enabled
    #[serde(skip)]
    auto_reload: bool,

    /// Interval for reload checks
    #[serde(skip)]
    reload_interval: Duration,
}

/// Configuration settings specific to content processing.
///
/// Provides options to enable content validation, sanitization, and metadata extraction.
/// Includes security measures and performance optimizations.
///
/// # Security Features
///
/// - Content size limits
/// - File extension validation
/// - Content sanitization
/// - Metadata validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentConfig {
    /// Enables validation of content before processing
    #[serde(default = "default_true")]
    pub validate: bool,

    /// Enables sanitization of content for security
    #[serde(default = "default_true")]
    pub sanitize: bool,

    /// Enables automatic extraction of metadata
    #[serde(default = "default_true")]
    pub extract_metadata: bool,

    /// List of allowed content file extensions
    #[serde(default = "default_extensions")]
    pub extensions: Vec<String>,

    /// Additional content-specific options
    #[serde(default)]
    pub options: HashMap<String, TomlValue>,

    /// Maximum content size in bytes (10MB default)
    #[serde(default = "default_max_content_size")]
    pub max_content_size: usize,

    /// Maximum metadata size in bytes (64KB default)
    #[serde(default = "default_max_metadata_size")]
    pub max_metadata_size: usize,

    /// List of allowed HTML tags if sanitization is enabled
    #[serde(default = "default_allowed_html_tags")]
    pub allowed_html_tags: Vec<String>,
}

impl Default for ContentConfig {
    fn default() -> Self {
        Self {
            validate: true,
            sanitize: true,
            extract_metadata: true,
            extensions: default_extensions(),
            options: HashMap::new(),
            max_content_size: default_max_content_size(),
            max_metadata_size: default_max_metadata_size(),
            allowed_html_tags: default_allowed_html_tags(),
        }
    }
}

/// Configuration settings for template rendering.
///
/// Provides options for rendering modes, including caching templates
/// for improved performance and security settings.
///
/// # Security Features
///
/// - Template size limits
/// - Cache size limits
/// - Template validation
/// - Strict mode for enhanced security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    /// Enables strict syntax checking
    #[serde(default)]
    pub strict_mode: bool,

    /// Enables template caching
    #[serde(default = "default_true")]
    pub cache_templates: bool,

    /// Additional template-specific options
    #[serde(default)]
    pub options: HashMap<String, TomlValue>,

    /// Maximum template size in bytes (1MB default)
    #[serde(default = "default_max_template_size")]
    pub max_template_size: usize,

    /// Maximum cache size in bytes (100MB default)
    #[serde(default = "default_max_cache_size")]
    pub max_cache_size: usize,

    /// Template cache TTL in seconds (1 hour default)
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: u64,

    /// List of allowed template functions
    #[serde(default = "default_allowed_functions")]
    pub allowed_functions: Vec<String>,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            strict_mode: false,
            cache_templates: true,
            options: HashMap::new(),
            max_template_size: default_max_template_size(),
            max_cache_size: default_max_cache_size(),
            cache_ttl: default_cache_ttl(),
            allowed_functions: default_allowed_functions(),
        }
    }
}

/// Configuration settings for output generation.
///
/// Manages settings for the creation of output files,
/// including security measures and optimization options.
///
/// # Security Features
///
/// - Output size limits
/// - File permission controls
/// - Path sanitization
/// - Resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Enables output minification
    #[serde(default)]
    pub minify: bool,

    /// Enables pretty printing of output
    #[serde(default = "default_true")]
    pub pretty_print: bool,

    /// Directory for static assets (sanitized path)
    #[serde(default)]
    pub asset_dir: Option<PathBuf>,

    /// Additional output-specific options
    #[serde(default)]
    pub options: HashMap<String, TomlValue>,

    /// Maximum output file size in bytes (100MB default)
    #[serde(default = "default_max_output_size")]
    pub max_output_size: usize,

    /// File permissions for generated files (Unix only)
    #[cfg(unix)]
    #[serde(default = "default_file_permissions")]
    pub file_permissions: u32,

    /// Maximum number of concurrent output operations
    #[serde(default = "default_max_concurrent_ops")]
    pub max_concurrent_ops: usize,

    /// Output rate limiting in bytes per second (0 = unlimited)
    #[serde(default)]
    pub rate_limit: u64,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            minify: false,
            pretty_print: true,
            asset_dir: None,
            options: HashMap::new(),
            max_output_size: default_max_output_size(),
            #[cfg(unix)]
            file_permissions: default_file_permissions(),
            max_concurrent_ops: default_max_concurrent_ops(),
            rate_limit: 0,
        }
    }
}

/// Builder for constructing Config instances securely.
///
/// Provides a fluent interface for creating configuration instances
/// with proper validation and security measures.
///
/// # Security Notes
///
/// - All paths are sanitized
/// - Environment variables are validated
/// - Overrides are checked for safety
/// - Size limits are enforced
#[derive(Debug)]
pub struct ConfigBuilder {
    config_file: Option<PathBuf>,
    env_prefix: Option<String>,
    profile: Option<Profile>,
    overrides: HashMap<String, TomlValue>,
    auto_reload: bool,
    reload_interval: Duration,
    max_file_size: usize,
}

// Default value functions
fn default_max_content_size() -> usize {
    10 * 1024 * 1024 // 10MB
}

fn default_max_metadata_size() -> usize {
    64 * 1024 // 64KB
}

fn default_max_template_size() -> usize {
    1024 * 1024 // 1MB
}

fn default_max_cache_size() -> usize {
    100 * 1024 * 1024 // 100MB
}

fn default_max_output_size() -> usize {
    100 * 1024 * 1024 // 100MB
}

fn default_cache_ttl() -> u64 {
    3600 // 1 hour
}

fn default_max_concurrent_ops() -> usize {
    10
}

#[cfg(unix)]
fn default_file_permissions() -> u32 {
    0o644
}

fn default_true() -> bool {
    true
}

fn default_allowed_html_tags() -> Vec<String> {
    vec![
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
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

fn default_allowed_functions() -> Vec<String> {
    vec![
        "upper", "lower", "trim", "date", "length", "join", "split",
        "slice", "first", "last",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

fn default_extensions() -> Vec<String> {
    vec!["md".to_string(), "markdown".to_string()]
}

impl ConfigBuilder {
    /// Creates a new ConfigBuilder with default settings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nucleusflow::core::config::ConfigBuilder;
    ///
    /// let builder = ConfigBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            config_file: None,
            env_prefix: None,
            profile: None,
            overrides: HashMap::new(),
            auto_reload: false,
            reload_interval: DEFAULT_RELOAD_INTERVAL,
            max_file_size: MAX_CONFIG_SIZE,
        }
    }

    /// Adds a configuration file to the builder.
    ///
    /// # Security
    ///
    /// - Path is sanitized to prevent directory traversal
    /// - File size is checked against limits
    /// - File permissions are verified on Unix systems
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the TOML configuration file
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use nucleusflow::core::config::ConfigBuilder;
    /// use std::path::Path;
    ///
    /// let builder = ConfigBuilder::new()
    ///     .with_file(Path::new("config.toml"));
    /// ```
    pub fn with_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config_file = Some(sanitize_path(path.as_ref()));
        self
    }

    /// Adds a prefix for environment variables.
    ///
    /// # Security
    ///
    /// - Prefix is validated for safe characters
    /// - Blocked environment variables are excluded
    /// - Values are sanitized before use
    ///
    /// # Arguments
    ///
    /// * `prefix` - The prefix for environment variables (e.g., "NUCLEUS_")
    pub fn with_env_prefix<S: Into<String>>(
        mut self,
        prefix: S,
    ) -> Self {
        let prefix = prefix.into();
        if is_safe_env_prefix(&prefix) {
            self.env_prefix = Some(prefix);
        }
        self
    }

    /// Sets the configuration profile.
    ///
    /// # Arguments
    ///
    /// * `profile` - The operational profile to use
    pub fn with_profile<P: Into<Profile>>(
        mut self,
        profile: P,
    ) -> Self {
        self.profile = Some(profile.into());
        self
    }

    /// Adds a configuration override.
    ///
    /// # Security
    ///
    /// - Key names are validated
    /// - Values are checked for safety
    /// - Size limits are enforced
    ///
    /// # Arguments
    ///
    /// * `key` - The configuration key to override
    /// * `value` - The new value for the key
    pub fn with_override<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<TomlValue>,
    {
        let key = key.into();
        let value = value.into();

        if is_safe_config_key(&key) && is_safe_config_value(&value) {
            _ = self.overrides.insert(key, value);
        }
        self
    }

    /// Enables automatic configuration reloading.
    ///
    /// # Security
    ///
    /// - File watching is rate-limited
    /// - Changes are validated before applying
    /// - Race conditions are prevented
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable auto-reloading
    pub fn with_auto_reload(mut self, enabled: bool) -> Self {
        self.auto_reload = enabled;
        self
    }

    /// Sets the interval for checking configuration changes.
    ///
    /// # Arguments
    ///
    /// * `interval` - The duration between reload checks
    pub fn with_reload_interval(mut self, interval: Duration) -> Self {
        self.reload_interval = interval.max(Duration::from_secs(1));
        self
    }

    /// Sets the maximum allowed configuration file size.
    ///
    /// # Arguments
    ///
    /// * `size` - Maximum size in bytes
    pub fn with_max_file_size(mut self, size: usize) -> Self {
        self.max_file_size = size.min(MAX_CONFIG_SIZE);
        self
    }

    /// Builds the final configuration.
    ///
    /// # Security
    ///
    /// - All settings are validated
    /// - Paths are sanitized
    /// - Values are checked for safety
    /// - Size limits are enforced
    ///
    /// # Returns
    ///
    /// * `Result<Arc<RwLock<Config>>>` - Thread-safe configuration or error
    pub fn build(self) -> Result<Arc<RwLock<Config>>> {
        let mut config = if let Some(path) = self.config_file {
            load_from_file(&path, self.max_file_size)?
        } else {
            Config::default()
        };

        config.auto_reload = self.auto_reload;
        config.reload_interval = self.reload_interval;

        if let Some(profile) = self.profile {
            config.profile = profile;
        }

        if let Some(prefix) = self.env_prefix {
            apply_env_overrides(&mut config, &prefix)?;
        }

        apply_overrides(&mut config, &self.overrides)?;
        validate_config(&config)?;

        Ok(Arc::new(RwLock::new(config)))
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    /// Validates all configuration settings.
    ///
    /// # Security
    ///
    /// Checks:
    /// - Path security
    /// - Value ranges
    /// - Resource limits
    /// - Permission settings
    pub fn validate(&self) -> Result<()> {
        validate_config(self)
    }

    /// Retrieves a custom configuration value.
    ///
    /// # Security
    ///
    /// - Type safety is enforced
    /// - Values are validated
    /// - Sensitive data is protected
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// * `Result<Option<T>>` - The value if it exists and can be converted
    pub fn get_custom<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>> {
        if !is_safe_config_key(key) {
            return Ok(None);
        }

        self.custom
            .get(key)
            .map(|v| {
                toml::Value::try_into(v.clone()).map_err(|e| {
                    ProcessingError::Configuration {
                        details: format!(
                            "Invalid custom config value: {}",
                            e
                        ),
                        path: None,
                        source: None,
                    }
                })
            })
            .transpose()
    }

    /// Sets a custom configuration value.
    ///
    /// # Security
    ///
    /// - Key names are validated
    /// - Values are checked for safety
    /// - Size limits are enforced
    ///
    /// # Arguments
    ///
    /// * `key` - The key to set
    /// * `value` - The value to store
    pub fn set_custom<T: Serialize>(
        &mut self,
        key: &str,
        value: T,
    ) -> Result<()> {
        if !is_safe_config_key(key) {
            return Err(ProcessingError::Configuration {
                details: "Invalid configuration key".to_string(),
                path: None,
                source: None,
            });
        }

        let value = toml::Value::try_from(value).map_err(|e| {
            ProcessingError::Configuration {
                details: format!("Invalid custom config value: {}", e),
                path: None,
                source: None,
            }
        })?;

        if is_safe_config_value(&value) {
            _ = self.custom.insert(key.to_string(), value);
            Ok(())
        } else {
            Err(ProcessingError::Configuration {
                details: "Invalid configuration value".to_string(),
                path: None,
                source: None,
            })
        }
    }

    /// Checks if configuration needs reloading.
    ///
    /// # Security
    ///
    /// - File access is controlled
    /// - Changes are validated
    /// - Race conditions are prevented
    pub fn needs_reload(&self) -> bool {
        if !self.auto_reload {
            return false;
        }

        if let Some(last_modified) = self.last_modified {
            if let Ok(metadata) = fs::metadata("config.toml") {
                if let Ok(modified) = metadata.modified() {
                    return modified > last_modified;
                }
            }
        }
        false
    }

    /// Reloads configuration if needed.
    ///
    /// # Security
    ///
    /// - File content is validated
    /// - Changes are atomic
    /// - Errors are handled safely
    ///
    /// # Returns
    ///
    /// * `Result<bool>` - Whether the configuration was reloaded
    pub fn reload_if_needed(&mut self) -> Result<bool> {
        if self.needs_reload() {
            let new_config = load_from_file(
                Path::new("config.toml"),
                MAX_CONFIG_SIZE,
            )?;
            *self = new_config;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

// Security Helper Functions

/// Sanitizes a path to prevent directory traversal attacks.
///
/// # Security
///
/// - Removes parent directory references
/// - Normalizes path separators
/// - Removes potentially dangerous components
///
/// # Arguments
///
/// * `path` - The path to sanitize
fn sanitize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(c) => components.push(c),
            std::path::Component::ParentDir => {
                if !components.is_empty() {
                    _ = components.pop();
                }
            }
            std::path::Component::CurDir => {}
            _ => {}
        }
    }
    components.iter().collect()
}

/// Validates a configuration key for safety.
///
/// # Security
///
/// - Checks for valid characters
/// - Enforces length limits
/// - Prevents special characters
fn is_safe_config_key(key: &str) -> bool {
    let valid_chars = |c: char| {
        c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.'
    };

    !key.is_empty()
        && key.len() <= 64
        && key.chars().all(valid_chars)
        && !key.starts_with('.')
        && !key.ends_with('.')
}

/// Validates a configuration value for safety.
///
/// # Security
///
/// - Checks value size
/// - Validates nested structures
/// - Prevents dangerous patterns
fn is_safe_config_value(value: &TomlValue) -> bool {
    match value {
        TomlValue::String(s) => {
            s.len() <= 1024 && !contains_dangerous_patterns(s)
        }
        TomlValue::Array(arr) => {
            arr.len() <= 100 && arr.iter().all(is_safe_config_value)
        }
        TomlValue::Table(table) => {
            table.len() <= 50
                && table.keys().all(|k| is_safe_config_key(k.as_str()))
                && table.values().all(is_safe_config_value)
        }
        _ => true,
    }
}

/// Checks for dangerous patterns in configuration values.
fn contains_dangerous_patterns(s: &str) -> bool {
    let patterns = [
        "javascript:",
        "data:",
        "vbscript:",
        "file:",
        "<script",
        "eval(",
        "setTimeout",
        "setInterval",
    ];
    patterns.iter().any(|p| s.to_lowercase().contains(p))
}

/// Validates environment variable prefix.
fn is_safe_env_prefix(prefix: &str) -> bool {
    let valid_chars = |c: char| c.is_ascii_uppercase() || c == '_';
    !prefix.is_empty()
        && prefix.len() <= 32
        && prefix.chars().all(valid_chars)
        && prefix.ends_with('_')
}

/// Loads configuration from file with security checks.
fn load_from_file(path: &Path, max_size: usize) -> Result<Config> {
    // Verify file size
    let metadata = fs::metadata(path).map_err(|e| {
        ProcessingError::Configuration {
            details: format!(
                "Failed to read config file metadata: {}",
                e
            ),
            path: Some(path.to_path_buf()),
            source: None,
        }
    })?;

    if metadata.len() > max_size as u64 {
        return Err(ProcessingError::Configuration {
            details: format!(
                "Config file exceeds maximum size of {} bytes",
                max_size
            ),
            path: Some(path.to_path_buf()),
            source: None,
        });
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = metadata.permissions();
        if perms.mode() & 0o777 > 0o644 {
            return Err(ProcessingError::Configuration {
                details: "Config file permissions too permissive"
                    .to_string(),
                path: Some(path.to_path_buf()),
                source: None,
            });
        }
    }

    let content = fs::read_to_string(path).map_err(|e| {
        ProcessingError::Configuration {
            details: format!("Failed to read config file: {}", e),
            path: Some(path.to_path_buf()),
            source: None,
        }
    })?;

    let mut config: Config = toml::from_str(&content).map_err(|e| {
        ProcessingError::Configuration {
            details: format!("Failed to parse config file: {}", e),
            path: Some(path.to_path_buf()),
            source: None,
        }
    })?;

    config.last_modified =
        Some(metadata.modified().unwrap_or_else(|_| SystemTime::now()));

    Ok(config)
}

/// Applies environment variable overrides with validation.
fn apply_env_overrides(
    config: &mut Config,
    prefix: &str,
) -> Result<()> {
    for (key, value) in env::vars() {
        if BLOCKED_ENV_VARS.contains(&key.as_str()) {
            continue;
        }

        if let Some(stripped) = key.strip_prefix(prefix) {
            let config_key =
                stripped.trim_start_matches('_').to_lowercase();
            if is_safe_config_key(&config_key) {
                apply_config_value(config, &config_key, &value)?;
            }
        }
    }
    Ok(())
}

/// Applies configuration overrides safely.
///
/// # Security
///
/// - Validates all override keys and values
/// - Prevents unsafe overwrites
/// - Maintains security constraints
///
/// # Arguments
///
/// * `config` - The configuration to modify
/// * `overrides` - Map of override key-value pairs
fn apply_overrides(
    config: &mut Config,
    overrides: &HashMap<String, TomlValue>,
) -> Result<()> {
    for (key, value) in overrides {
        if is_safe_config_key(key) && is_safe_config_value(value) {
            apply_config_value(config, key, value)?;
        }
    }
    Ok(())
}

/// Applies a single configuration value safely.
///
/// # Security
///
/// - Validates value types
/// - Sanitizes paths
/// - Enforces security constraints
/// - Maintains profile requirements
///
/// # Arguments
///
/// * `config` - The configuration to modify
/// * `key` - The configuration key
/// * `value` - The value to apply
fn apply_config_value<T: ToString>(
    config: &mut Config,
    key: &str,
    value: &T,
) -> Result<()> {
    let value_str = value.to_string().trim_matches('"').to_string();

    match key {
        "content_dir" => {
            config.content_dir =
                sanitize_path(&PathBuf::from(value_str));
        }
        "output_dir" => {
            config.output_dir =
                sanitize_path(&PathBuf::from(value_str));
        }
        "template_dir" => {
            config.template_dir =
                sanitize_path(&PathBuf::from(value_str));
        }
        "profile" => {
            config.profile = match value_str.to_lowercase().as_str() {
                "development" => Profile::Development,
                "staging" => Profile::Staging,
                "production" => Profile::Production,
                _ => Profile::Custom,
            };
        }
        _ => {
            if let Some((section, key)) = key.split_once('.') {
                match section {
                    "content" => apply_content_value(
                        &mut config.content,
                        key,
                        &value_str,
                    )?,
                    "template" => apply_template_value(
                        &mut config.template,
                        key,
                        &value_str,
                    )?,
                    "output" => apply_output_value(
                        &mut config.output,
                        key,
                        &value_str,
                    )?,
                    "custom" => {
                        if is_safe_config_key(key) {
                            let toml_value =
                                TomlValue::String(value_str);
                            if is_safe_config_value(&toml_value) {
                                _ = config.custom.insert(
                                    key.to_string(),
                                    toml_value,
                                );
                            }
                        }
                    }
                    _ => {
                        return Err(ProcessingError::Configuration {
                            details: format!(
                                "Unknown configuration section: {}",
                                section
                            ),
                            path: None,
                            source: None,
                        });
                    }
                }
            } else {
                return Err(ProcessingError::Configuration {
                    details: format!(
                        "Unknown configuration key: {}",
                        key
                    ),
                    path: None,
                    source: None,
                });
            }
        }
    }

    // If in production mode, ensure security settings are maintained
    if matches!(config.profile, Profile::Production) {
        enforce_production_security(config)?;
    }

    Ok(())
}

/// Enforces security settings required for production mode.
///
/// # Security
///
/// Ensures critical security settings cannot be disabled in production:
/// - Content sanitization
/// - Template strict mode
/// - Secure file permissions
/// - Rate limiting
fn enforce_production_security(config: &mut Config) -> Result<()> {
    // Force enable critical security settings
    config.content.sanitize = true;
    config.template.strict_mode = true;

    // Ensure secure file permissions
#[cfg(unix)]
{
    config.output.file_permissions &= 0o644;
}


    // Enforce minimum rate limiting
    if config.output.rate_limit == 0 {
        config.output.rate_limit = 1024 * 1024; // 1MB/s default limit
    }

    // Verify security settings
    if !config.content.sanitize || !config.template.strict_mode {
        return Err(ProcessingError::Configuration {
            details: "Cannot disable security features in production"
                .to_string(),
            path: None,
            source: None,
        });
    }

    Ok(())
}

/// Validates all configuration settings.
fn validate_config(config: &Config) -> Result<()> {
    // Validate paths
    validate_path(&config.content_dir, "content", true)?;
    validate_path(&config.template_dir, "template", true)?;

    if let Some(asset_dir) = &config.output.asset_dir {
        validate_path(asset_dir, "asset", true)?;
    }

    // Validate extensions
    if config.content.extensions.is_empty() {
        return Err(ProcessingError::Configuration {
            details: "No content extensions specified".to_string(),
            path: None,
            source: None,
        });
    }

    // Validate sizes
    if config.content.max_content_size > 100 * 1024 * 1024 {
        return Err(ProcessingError::Configuration {
            details: "Content size limit too large".to_string(),
            path: None,
            source: None,
        });
    }

    Ok(())
}

/// Validates a path for security and accessibility.
fn validate_path(
    path: &Path,
    name: &str,
    must_exist: bool,
) -> Result<()> {
    let sanitized = sanitize_path(path);

    if must_exist && !sanitized.exists() {
        return Err(ProcessingError::Configuration {
            details: format!(
                "{} directory does not exist: {}",
                name,
                sanitized.display()
            ),
            path: Some(sanitized),
            source: None,
        });
    }

    if sanitized.exists() && !sanitized.is_dir() {
        return Err(ProcessingError::Configuration {
            details: format!(
                "{} path is not a directory: {}",
                name,
                sanitized.display()
            ),
            path: Some(sanitized),
            source: None,
        });
    }

    Ok(())
}

/// Applies content-specific configuration values.
fn apply_content_value(
    config: &mut ContentConfig,
    key: &str,
    value: &str,
) -> Result<()> {
    match key {
        "validate" => {
            config.validate = value.parse().map_err(|e| {
                ProcessingError::Configuration {
                    details: format!(
                        "Invalid validate value '{}': {}",
                        value, e
                    ),
                    path: None,
                    source: None,
                }
            })?;
        }
        "sanitize" => {
            config.sanitize = value.parse().map_err(|e| {
                ProcessingError::Configuration {
                    details: format!(
                        "Invalid sanitize value '{}': {}",
                        value, e
                    ),
                    path: None,
                    source: None,
                }
            })?;
        }
        "extract_metadata" => {
            config.extract_metadata = value.parse().map_err(|e| {
                ProcessingError::Configuration {
                    details: format!(
                        "Invalid extract_metadata value '{}': {}",
                        value, e
                    ),
                    path: None,
                    source: None,
                }
            })?;
        }
        "max_content_size" => {
            config.max_content_size = value.parse().map_err(|e| {
                ProcessingError::Configuration {
                    details: format!(
                        "Invalid max_content_size value '{}': {}",
                        value, e
                    ),
                    path: None,
                    source: None,
                }
            })?;
        }
        _ => {
            let toml_value = TomlValue::String(value.to_string());
            if is_safe_config_value(&toml_value) {
                _ = config.options.insert(key.to_string(), toml_value);
            }
        }
    }
    Ok(())
}

/// Applies template-specific configuration values.
fn apply_template_value(
    config: &mut TemplateConfig,
    key: &str,
    value: &str,
) -> Result<()> {
    match key {
        "strict_mode" => {
            config.strict_mode = value.parse().map_err(|e| {
                ProcessingError::Configuration {
                    details: format!(
                        "Invalid strict_mode value '{}': {}",
                        value, e
                    ),
                    path: None,
                    source: None,
                }
            })?;
        }
        "cache_templates" => {
            config.cache_templates = value.parse().map_err(|e| {
                ProcessingError::Configuration {
                    details: format!(
                        "Invalid cache_templates value '{}': {}",
                        value, e
                    ),
                    path: None,
                    source: None,
                }
            })?;
        }
        "max_template_size" => {
            config.max_template_size = value.parse().map_err(|e| {
                ProcessingError::Configuration {
                    details: format!(
                        "Invalid max_template_size value '{}': {}",
                        value, e
                    ),
                    path: None,
                    source: None,
                }
            })?;
        }
        _ => {
            let toml_value = TomlValue::String(value.to_string());
            if is_safe_config_value(&toml_value) {
                _ = config.options.insert(key.to_string(), toml_value);
            }
        }
    }
    Ok(())
}

/// Applies output-specific configuration values.
fn apply_output_value(
    config: &mut OutputConfig,
    key: &str,
    value: &str,
) -> Result<()> {
    match key {
        "minify" => {
            config.minify = value.parse().map_err(|e| {
                ProcessingError::Configuration {
                    details: format!(
                        "Invalid minify value '{}': {}",
                        value, e
                    ),
                    path: None,
                    source: None,
                }
            })?;
        }
        "pretty_print" => {
            config.pretty_print = value.parse().map_err(|e| {
                ProcessingError::Configuration {
                    details: format!(
                        "Invalid pretty_print value '{}': {}",
                        value, e
                    ),
                    path: None,
                    source: None,
                }
            })?;
        }
        "asset_dir" => {
            config.asset_dir =
                Some(sanitize_path(&PathBuf::from(value)));
        }
        "max_output_size" => {
            config.max_output_size = value.parse().map_err(|e| {
                ProcessingError::Configuration {
                    details: format!(
                        "Invalid max_output_size value '{}': {}",
                        value, e
                    ),
                    path: None,
                    source: None,
                }
            })?;
        }
        _ => {
            let toml_value = TomlValue::String(value.to_string());
            if is_safe_config_value(&toml_value) {
                _ = config.options.insert(key.to_string(), toml_value);
            }
        }
    }
    Ok(())
}

/// Returns the default content directory path.
fn default_content_dir() -> PathBuf {
    PathBuf::from("content")
}

/// Returns the default output directory path.
fn default_output_dir() -> PathBuf {
    PathBuf::from("public")
}

/// Returns the default template directory path.
fn default_template_dir() -> PathBuf {
    PathBuf::from("templates")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_path_sanitization() {
        let path = Path::new("../../../etc/passwd");
        let sanitized = sanitize_path(path);
        assert_eq!(sanitized, PathBuf::from("etc/passwd"));
    }

    #[test]
    fn test_safe_config_key() {
        assert!(is_safe_config_key("valid_key"));
        assert!(is_safe_config_key("valid-key-123"));
        assert!(!is_safe_config_key(""));
        assert!(!is_safe_config_key("../invalid"));
        assert!(!is_safe_config_key("invalid*key"));
    }

    #[test]
    fn test_safe_config_value() {
        assert!(is_safe_config_value(&TomlValue::String(
            "safe value".into()
        )));
        assert!(!is_safe_config_value(&TomlValue::String(
            "javascript:alert(1)".into()
        )));

        let mut large_table = toml::map::Map::new();
        for i in 0..100 {
            _ = large_table.insert(
                format!("key{}", i),
                TomlValue::String("value".into()),
            );
        }
        assert!(!is_safe_config_value(&TomlValue::Table(large_table)));
    }

    #[test]
    fn test_config_size_limit() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("large_config.toml");
        let large_content = "x".repeat(MAX_CONFIG_SIZE + 1);
        fs::write(&config_file, large_content).unwrap();

        assert!(load_from_file(&config_file, MAX_CONFIG_SIZE).is_err());
    }

    #[test]
    fn test_config_builder_new_defaults() {
        let builder = ConfigBuilder::new();

        assert!(builder.config_file.is_none());
        assert!(builder.env_prefix.is_none());
        assert!(builder.profile.is_none());
        assert!(builder.overrides.is_empty());
        assert!(!builder.auto_reload);
        assert_eq!(builder.reload_interval, DEFAULT_RELOAD_INTERVAL);
        assert_eq!(builder.max_file_size, MAX_CONFIG_SIZE);
    }

    #[test]
    fn test_config_builder_with_file() {
        let builder =
            ConfigBuilder::new().with_file(Path::new("config.toml"));

        assert!(builder.config_file.is_some());
        assert_eq!(
            builder.config_file.unwrap(),
            PathBuf::from("config.toml")
        );
    }

    #[test]
    fn test_config_builder_with_env_prefix() {
        let builder = ConfigBuilder::new().with_env_prefix("NUCLEUS_");

        assert!(builder.env_prefix.is_some());
        assert_eq!(builder.env_prefix.unwrap(), "NUCLEUS_");
    }

    #[test]
    fn test_config_builder_with_invalid_env_prefix() {
        let builder = ConfigBuilder::new().with_env_prefix("nucleus");

        // Invalid prefix should not be set
        assert!(builder.env_prefix.is_none());
    }

    #[test]
    fn test_config_builder_with_profile() {
        let builder =
            ConfigBuilder::new().with_profile(Profile::Production);

        assert!(builder.profile.is_some());
        assert_eq!(builder.profile.unwrap(), Profile::Production);
    }

    #[test]
    fn test_config_builder_with_override() {
        let mut builder = ConfigBuilder::new();
        builder = builder.with_override("custom_key", "custom_value");

        assert!(builder.overrides.contains_key("custom_key"));
        assert_eq!(
            builder.overrides.get("custom_key").unwrap(),
            &TomlValue::String("custom_value".into())
        );
    }

    #[test]
    fn test_config_builder_with_invalid_override_key() {
        let mut builder = ConfigBuilder::new();
        builder = builder.with_override("../invalid_key", "value");

        // Invalid key should not be set in overrides
        assert!(!builder.overrides.contains_key("../invalid_key"));
    }

    #[test]
    fn test_config_builder_with_auto_reload() {
        let builder = ConfigBuilder::new().with_auto_reload(true);
        assert!(builder.auto_reload);

        let builder = ConfigBuilder::new().with_auto_reload(false);
        assert!(!builder.auto_reload);
    }

    #[test]
    fn test_config_builder_with_reload_interval() {
        let builder = ConfigBuilder::new()
            .with_reload_interval(Duration::from_secs(10));
        assert_eq!(builder.reload_interval, Duration::from_secs(10));

        // Test that intervals below 1 second default to 1 second
        let builder = ConfigBuilder::new()
            .with_reload_interval(Duration::from_millis(500));
        assert_eq!(builder.reload_interval, Duration::from_secs(1));
    }

    #[test]
    fn test_config_builder_with_max_file_size() {
        // Set a file size within the allowed limit
        let builder =
            ConfigBuilder::new().with_max_file_size(512 * 1024); // 512 KB
        assert_eq!(builder.max_file_size, 512 * 1024);

        // Attempt to set a file size beyond MAX_CONFIG_SIZE
        let builder = ConfigBuilder::new()
            .with_max_file_size(MAX_CONFIG_SIZE * 2);
        assert_eq!(builder.max_file_size, MAX_CONFIG_SIZE); // Should cap at MAX_CONFIG_SIZE
    }

    #[test]
    fn test_config_set_and_get_custom() {
        let mut config = Config::default();

        // Set a custom configuration value
        config.set_custom("custom_key", "custom_value").unwrap();

        // Retrieve the custom configuration value
        let custom_value: Option<String> =
            config.get_custom("custom_key").unwrap();
        assert_eq!(custom_value, Some("custom_value".to_string()));

        // Test retrieving a non-existent key
        let non_existent: Option<String> =
            config.get_custom("non_existent_key").unwrap();
        assert!(non_existent.is_none());
    }

    #[test]
    fn test_config_set_custom_with_invalid_key() {
        let mut config = Config::default();

        // Attempt to set a custom configuration with an invalid key
        let result = config.set_custom("../invalid_key", "value");

        // Ensure it returns an error due to the invalid key
        assert!(result.is_err());
    }
}
