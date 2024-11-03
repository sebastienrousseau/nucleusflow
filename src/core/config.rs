//! # Configuration Module
//!
//! Provides flexible configuration management for the NucleusFlow content processing library. This module implements a robust configuration system supporting multiple sources including TOML files, environment variables, and programmatic configuration.
//!
//! ## Features
//!
//! - Multiple configuration sources (TOML, environment variables, code)
//! - Strong validation and error handling
//! - Type-safe configuration values
//! - Support for multiple environments/profiles
//! - Secure handling of sensitive values
//!
//! ## Example
//!
//! ```rust,no_run
//! use nucleusflow::core::config::{Config, ConfigBuilder, Profile};
//! use std::path::Path;
//!
//! let config = ConfigBuilder::new()
//!     .with_file(Path::new("config.toml"))
//!     .with_env_prefix("NUCLEUS_")
//!     .with_profile(Profile::Production)
//!     .build()
//!     .unwrap();
//!
//! // Acquire a read lock to access the `profile` field
//! let config_read = config.read();
//! assert_eq!(config_read.profile, Profile::Production);
//! ```

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use toml::Value as TomlValue;

use crate::ProcessingError;
use crate::Result;

/// Specifies operational profiles for configuration.
///
/// Each profile determines distinct settings suitable for specific environments such as development, staging, and production, while a custom profile allows user-defined configurations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
    /// Development profile with settings optimised for debugging.
    Development,
    /// Staging profile for intermediate testing between development and production.
    Staging,
    /// Production profile with performance-focused settings.
    Production,
    /// Custom profile enabling specific user configurations.
    Custom,
}

impl Default for Profile {
    fn default() -> Self {
        Profile::Development
    }
}

/// Represents the main configuration structure encompassing all application settings.
///
/// This structure consolidates settings for content processing, templating, output generation, and other options that can be customised. It also enables environment-based profile selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_content_dir")]
    /// Specifies the path to the directory containing content files.
    pub content_dir: PathBuf,

    #[serde(default = "default_output_dir")]
    /// Specifies the path to the output directory for processed files.
    pub output_dir: PathBuf,

    #[serde(default = "default_template_dir")]
    /// Specifies the path to the directory containing template files.
    pub template_dir: PathBuf,

    #[serde(default)]
    /// Indicates the current operational profile.
    pub profile: Profile,

    #[serde(default)]
    /// Configuration for content processing settings.
    pub content: ContentConfig,

    #[serde(default)]
    /// Configuration for template rendering settings.
    pub template: TemplateConfig,

    #[serde(default)]
    /// Configuration for output generation settings.
    pub output: OutputConfig,

    #[serde(default)]
    /// Holds custom configuration values specified by the user.
    pub custom: HashMap<String, TomlValue>,
}

/// Configuration settings specific to content processing.
///
/// Provides options to enable content validation, sanitisation, and metadata extraction.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContentConfig {
    #[serde(default = "default_true")]
    /// Enables validation of content before processing.
    pub validate: bool,

    #[serde(default = "default_true")]
    /// Enables sanitisation of content to ensure it adheres to safety standards.
    pub sanitize: bool,

    #[serde(default = "default_true")]
    /// Enables automatic extraction of metadata from content files.
    pub extract_metadata: bool,

    #[serde(default = "default_extensions")]
    /// List of supported content file extensions.
    pub extensions: Vec<String>,

    #[serde(default)]
    /// Holds additional content-specific configuration options.
    pub options: HashMap<String, TomlValue>,
}

/// Configuration settings for template rendering.
///
/// Provides options for rendering modes, including caching templates for improved performance.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateConfig {
    #[serde(default)]
    /// Enables strict mode during template rendering, enforcing stricter syntax rules.
    pub strict_mode: bool,

    #[serde(default = "default_true")]
    /// Enables caching of rendered templates to optimise performance.
    pub cache_templates: bool,

    #[serde(default)]
    /// Holds additional template-specific configuration options.
    pub options: HashMap<String, TomlValue>,
}

/// Configuration settings for output generation.
///
/// Manages settings for the creation of output files, such as minification and asset management.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OutputConfig {
    #[serde(default)]
    /// Enables minification of output files to reduce file size.
    pub minify: bool,

    #[serde(default = "default_true")]
    /// Enables pretty-printing of output files for readability.
    pub pretty_print: bool,

    #[serde(default)]
    /// Specifies an optional directory for storing static assets.
    pub asset_dir: Option<PathBuf>,

    #[serde(default)]
    /// Holds additional output-specific configuration options.
    pub options: HashMap<String, TomlValue>,
}

/// Builds a `Config` instance by allowing multiple configuration options to be set.
///
/// The `ConfigBuilder` provides methods for customising settings by specifying configuration files, environment variables, profiles, and overrides.
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    config_file: Option<PathBuf>,
    env_prefix: Option<String>,
    profile: Option<Profile>,
    overrides: HashMap<String, TomlValue>,
}

impl ConfigBuilder {
    /// Initialises a new `ConfigBuilder` instance with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a configuration file to the builder.
    ///
    /// # Parameters
    /// - `path`: The path to the TOML configuration file.
    pub fn with_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config_file = Some(path.as_ref().to_path_buf());
        self
    }

    /// Adds a prefix for environment variables to override configuration values.
    ///
    /// # Parameters
    /// - `prefix`: The prefix for environment variables (e.g., "NUCLEUS_").
    pub fn with_env_prefix<S: Into<String>>(
        mut self,
        prefix: S,
    ) -> Self {
        self.env_prefix = Some(prefix.into());
        self
    }

    /// Sets the profile for the configuration, such as Development or Production.
    ///
    /// # Parameters
    /// - `profile`: The operational profile to be used (e.g., `Profile::Production`).
    pub fn with_profile<P: Into<Profile>>(
        mut self,
        profile: P,
    ) -> Self {
        self.profile = Some(profile.into());
        self
    }

    /// Adds a key-value pair to override configuration values.
    ///
    /// # Parameters
    /// - `key`: The configuration key to override.
    /// - `value`: The new value for the key.
    pub fn with_override<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<TomlValue>,
    {
        _ = self.overrides.insert(key.into(), value.into());
        self
    }

    /// Builds the final configuration by applying all specified settings and overrides.
    ///
    /// Loads configuration from file, applies environment and manual overrides,
    /// and validates the final configuration.
    pub fn build(self) -> Result<Arc<RwLock<Config>>> {
        let mut config = if let Some(path) = self.config_file {
            load_from_file(&path)?
        } else {
            Config::default()
        };

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

impl Config {
    /// Validates the configuration, ensuring necessary directories and settings are correct.
    pub fn validate(&self) -> Result<()> {
        validate_config(self)
    }

    /// Retrieves a custom configuration value by key, if it exists.
    ///
    /// # Parameters
    /// - `key`: The key to look up.
    ///
    /// # Returns
    /// - `Ok(Some(T))` if the key exists and can be converted to the specified type.
    /// - `Ok(None)` if the key does not exist.
    /// - `Err` if the key exists but cannot be converted to the specified type.
    pub fn get_custom<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>> {
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

    /// Sets a custom configuration value for the given key.
    ///
    /// # Parameters
    /// - `key`: The key to set.
    /// - `value`: The value to store, which must implement `Serialize`.
    pub fn set_custom<T: Serialize>(
        &mut self,
        key: &str,
        value: T,
    ) -> Result<()> {
        let value = toml::Value::try_from(value).map_err(|e| {
            ProcessingError::Configuration {
                details: format!("Invalid custom config value: {}", e),
                path: None,
                source: None,
            }
        })?;
        _ = self.custom.insert(key.to_string(), value);
        Ok(())
    }
}

impl Default for Config {
    /// Creates a default `Config` instance with preset values.
    ///
    /// This method initialises the configuration with default paths for content, output, and templates, while setting the profile to `Development`.
    /// It also includes default settings for content, template, and output configurations as well as an empty collection for custom configuration values.
    ///
    /// # Returns
    /// - A `Config` instance with all fields set to default values.
    fn default() -> Self {
        Self {
            content_dir: default_content_dir(),
            output_dir: default_output_dir(),
            template_dir: default_template_dir(),
            profile: Profile::default(),
            content: ContentConfig::default(),
            template: TemplateConfig::default(),
            output: OutputConfig::default(),
            custom: HashMap::new(),
        }
    }
}

// Internal helper functions

fn load_from_file(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path).map_err(|e| {
        ProcessingError::Configuration {
            details: format!("Failed to read config file: {}", e),
            path: Some(path.to_path_buf()),
            source: None,
        }
    })?;

    toml::from_str(&content).map_err(|e| {
        ProcessingError::Configuration {
            details: format!("Failed to parse config file: {}", e),
            path: Some(path.to_path_buf()),
            source: None,
        }
    })
}

fn apply_env_overrides(
    config: &mut Config,
    prefix: &str,
) -> Result<()> {
    for (key, value) in env::vars() {
        // Strip the prefix and ensure no leading underscores remain
        if let Some(stripped) = key.strip_prefix(prefix) {
            let config_key =
                stripped.trim_start_matches('_').to_lowercase();
            apply_config_value(config, &config_key, &value)?;
        }
    }
    Ok(())
}

fn apply_overrides(
    config: &mut Config,
    overrides: &HashMap<String, TomlValue>,
) -> Result<()> {
    for (key, value) in overrides {
        apply_config_value(config, key, value)?;
    }
    Ok(())
}

fn validate_config(config: &Config) -> Result<()> {
    validate_path(&config.content_dir, "content", true)?;
    validate_path(&config.template_dir, "template", true)?;

    if let Some(asset_dir) = &config.output.asset_dir {
        validate_path(asset_dir, "asset", true)?;
    }

    if config.content.extensions.is_empty() {
        return Err(ProcessingError::Configuration {
            details: "No content extensions specified".to_string(),
            path: None,
            source: None,
        });
    }

    Ok(())
}

fn apply_config_value<T: ToString>(
    config: &mut Config,
    key: &str,
    value: &T,
) -> Result<()> {
    let value_str = value.to_string().trim_matches('"').to_string(); // Remove extra quotes
    match key {
        "content_dir" => config.content_dir = PathBuf::from(value_str),
        "output_dir" => config.output_dir = PathBuf::from(value_str),
        "template_dir" => {
            config.template_dir = PathBuf::from(value_str)
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
                        _ = config.custom.insert(
                            key.to_string(),
                            TomlValue::String(value_str),
                        );
                    }
                    _ => {
                        return Err(ProcessingError::configuration(
                            format!(
                                "Unknown configuration section: {}",
                                section
                            ),
                            None,
                            None,
                        ));
                    }
                }
            } else {
                return Err(ProcessingError::configuration(
                    format!("Unknown configuration key: {}", key),
                    None,
                    None,
                ));
            }
        }
    }
    Ok(())
}

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
        _ => {
            _ = config.options.insert(
                key.to_string(),
                TomlValue::String(value.to_string()),
            );
        }
    }
    Ok(())
}

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
        _ => {
            _ = config.options.insert(
                key.to_string(),
                TomlValue::String(value.to_string()),
            );
        }
    }
    Ok(())
}

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
            config.asset_dir = Some(PathBuf::from(value));
        }
        _ => {
            _ = config.options.insert(
                key.to_string(),
                TomlValue::String(value.to_string()),
            );
        }
    }
    Ok(())
}

fn validate_path(
    path: &Path,
    name: &str,
    must_exist: bool,
) -> Result<()> {
    if must_exist && !path.exists() {
        return Err(ProcessingError::Configuration {
            details: format!(
                "{} directory does not exist: {}",
                name,
                path.display()
            ),
            path: Some(path.to_path_buf()),
            source: None,
        });
    }

    if path.exists() && !path.is_dir() {
        return Err(ProcessingError::Configuration {
            details: format!(
                "{} path is not a directory: {}",
                name,
                path.display()
            ),
            path: Some(path.to_path_buf()),
            source: None,
        });
    }

    Ok(())
}

// Default value functions
fn default_true() -> bool {
    true
}

fn default_content_dir() -> PathBuf {
    PathBuf::from("content")
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("public")
}

fn default_template_dir() -> PathBuf {
    PathBuf::from("templates")
}

fn default_extensions() -> Vec<String> {
    vec!["md".to_string(), "markdown".to_string()]
}

/// Tests for the configuration module.
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_profile() {
        assert_eq!(Profile::default(), Profile::Development);
    }

    #[test]
    fn test_load_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.toml");
        fs::write(&config_file, "content_dir = 'content'").unwrap();

        let config = load_from_file(&config_file).unwrap();
        assert_eq!(config.content_dir, PathBuf::from("content"));
    }

    #[test]
    fn test_missing_content_dir_validation() {
        let config = Config {
            content_dir: PathBuf::from("missing_content"),
            ..Default::default()
        };
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_apply_overrides() {
        let mut config = Config::default();
        let mut overrides = HashMap::new();
        _ = overrides.insert(
            "content_dir".to_string(),
            TomlValue::String("new_content".to_string()),
        );

        apply_overrides(&mut config, &overrides).unwrap();
        assert_eq!(config.content_dir, PathBuf::from("new_content"));
    }

    #[test]
    fn test_env_overrides() {
        env::set_var("NUCLEUS_CONTENT_DIR", "env_content");
        let mut config = Config::default();

        apply_env_overrides(&mut config, "NUCLEUS").unwrap();
        assert_eq!(config.content_dir, PathBuf::from("env_content"));
    }
}
