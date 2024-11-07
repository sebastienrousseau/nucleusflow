//! # Core Traits Module
//!
//! This module defines foundational traits for the NucleusFlow processing library. These traits provide a flexible, composable foundation for implementing various content processing, transformation, and generation capabilities.
//!
//! ## Key Traits
//!
//! - [`Processor`]: Core trait for content processing implementations
//! - [`Transform`]: Trait for content transformation operations
//! - [`Generator`]: Trait for output generation
//! - [`Validator`]: Trait for content validation
//! - [`ProcessingContext`]: Trait for converting types into a processing context
//! - [`Shareable`]: Trait for wrapping types in a shareable, thread-safe container
//!
//! ## Design Principles
//!
//! - **Separation of Concerns**: Each trait has a single, well-defined responsibility
//! - **Composability**: Traits can be combined to create complex processing pipelines
//! - **Type Safety**: Generic type parameters ensure type-safe processing chains
//! - **Error Handling**: Consistent error handling via the `Result` type

use std::fmt::Debug;
use std::path::Path;
use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::core::error::Result;

/// Core trait for implementing content processors.
///
/// This trait defines the interface for any type that can process content
/// from one form to another. It's designed to be flexible and composable through
/// generic type parameters.
///
/// # Type Parameters
///
/// * `Input`: The type of content being processed
/// * `Output`: The type of content produced
/// * `Context`: Additional context or configuration for processing
pub trait Processor: Send + Sync + Debug {
    /// `Input` the type of content being processed
    type Input;
    /// `Output` the type of content produced
    type Output;
    /// `Context` additional context or configuration for processing
    type Context;

    /// Processes the input content using optional context information.
    ///
    /// # Arguments
    ///
    /// * `input` - The content to process
    /// * `context` - Optional processing context or configuration
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing either the processed content or an error.
    fn process(
        &self,
        input: Self::Input,
        context: Option<&Self::Context>,
    ) -> Result<Self::Output>;
}

/// Trait for implementing pure content transformations.
///
/// This trait is for processors that transform content without side effects, ideal for pure transformations that don't require additional context.
pub trait Transform: Send + Sync + Debug {
    /// `Input` the type of content to transform
    type Input;
    /// `Output` the type of content produced
    type Output;

    /// Transforms the input content into the output format.
    ///
    /// # Arguments
    ///
    /// * `input` - The content to transform
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing either the transformed content or an error.
    fn transform(&self, input: Self::Input) -> Result<Self::Output>;
}

/// Trait for implementing output generation.
///
/// This trait defines the interface for types that can generate output in various
/// formats, with support for configuration options and validation.
pub trait Generator: Send + Sync + Debug {
    /// Generates output from the given content.
    ///
    /// # Arguments
    ///
    /// * `content` - The content to generate output from
    /// * `path` - The path where the output should be written
    /// * `options` - Optional configuration for the generation process
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if generation succeeds, or an error if it fails.
    fn generate(
        &self,
        content: &str,
        path: &Path,
        options: Option<&JsonValue>,
    ) -> Result<()>;

    /// Validates the generation parameters without performing the generation.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to validate
    /// * `options` - Optional configuration to validate
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if validation succeeds, or an error if it fails.
    fn validate(
        &self,
        path: &Path,
        options: Option<&JsonValue>,
    ) -> Result<()>;
}

/// Trait for implementing content validation.
///
/// This trait provides a standard interface for validating content before
/// processing or transformation occurs.
pub trait Validator: Send + Sync + Debug {
    /// `Input` the type of content to validate
    type Input;

    /// Validates the input content.
    ///
    /// # Arguments
    ///
    /// * `input` - The content to validate
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if validation succeeds, or an error if it fails.
    fn validate(&self, input: &Self::Input) -> Result<()>;
}

/// Trait for types that can provide processing context.
///
/// This trait provides a standard way to convert various types into processing context objects.
pub trait ProcessingContext {
    /// Converts the type into a processing context.
    ///
    /// # Returns
    ///
    /// Returns a `JsonValue` representing the processing context.
    fn into_context(self) -> JsonValue;
}

/// Trait for types that can be shared between multiple processors.
///
/// This trait provides a standard way to wrap types in thread-safe
/// reference-counted containers for sharing between processors.
pub trait Shareable: Sized + Send + Sync + Debug {
    /// Converts the type into a shareable form.
    ///
    /// # Returns
    ///
    /// Returns an `Arc<RwLock<Self>>` containing the shareable value.
    fn into_shared(self) -> Arc<RwLock<Self>>;
}

// Blanket implementation of Shareable for all eligible types
impl<T: Send + Sync + Debug + 'static> Shareable for T {
    fn into_shared(self) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(self))
    }
}

/// Struct representing configuration options for processors.
///
/// Provides configurable options for processing behavior across various processor implementations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingOptions {
    /// Enable strict mode for processing
    #[serde(default)]
    pub strict_mode: bool,
    /// Enable validation before processing
    #[serde(default = "default_true")]
    pub validate: bool,
    /// Enable caching of processed content
    #[serde(default = "default_true")]
    pub cache_enabled: bool,
    /// Custom options for specific processors
    #[serde(default)]
    pub custom: JsonValue,
}

impl Default for ProcessingOptions {
    fn default() -> Self {
        Self {
            strict_mode: false,
            validate: true,
            cache_enabled: true,
            custom: JsonValue::Null,
        }
    }
}

// Helper function for serde defaults
fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_processor() {
        #[derive(Debug)]
        struct TestProcessor;

        impl Processor for TestProcessor {
            type Input = String;
            type Output = String;
            type Context = ProcessingOptions;

            fn process(
                &self,
                input: Self::Input,
                context: Option<&Self::Context>,
            ) -> Result<Self::Output> {
                if let Some(ctx) = context {
                    if ctx.strict_mode && input.is_empty() {
                        return Err(crate::core::error::ProcessingError::Validation {
                            details: "Input cannot be empty in strict mode".to_string(),
                            context: None,
                        });
                    }
                }
                Ok(input.to_uppercase())
            }
        }

        let processor = TestProcessor;
        let options = ProcessingOptions {
            strict_mode: true,
            ..Default::default()
        };

        // Test with valid input
        let result =
            processor.process("test".to_string(), Some(&options));
        assert_eq!(result.unwrap(), "TEST");

        // Test with empty input in strict mode
        let result = processor.process("".to_string(), Some(&options));
        assert!(result.is_err());
    }

    #[test]
    fn test_transform() {
        #[derive(Debug)]
        struct TestTransform;

        impl Transform for TestTransform {
            type Input = String;
            type Output = String;

            fn transform(
                &self,
                input: Self::Input,
            ) -> Result<Self::Output> {
                Ok(input.chars().rev().collect())
            }
        }

        let transform = TestTransform;
        let result = transform.transform("hello".to_string()).unwrap();
        assert_eq!(result, "olleh");
    }

    #[test]
    fn test_validator() {
        #[derive(Debug)]
        struct TestValidator;

        impl Validator for TestValidator {
            type Input = String;

            fn validate(&self, input: &Self::Input) -> Result<()> {
                if input.is_empty() {
                    return Err(crate::core::error::ProcessingError::Validation {
                        details: "Input cannot be empty".to_string(),
                        context: None,
                    });
                }
                Ok(())
            }
        }

        let validator = TestValidator;
        assert!(validator.validate(&"test".to_string()).is_ok());
        assert!(validator.validate(&"".to_string()).is_err());
    }

    #[test]
    fn test_processing_context() {
        #[derive(Debug, Clone)]
        struct TestContext {
            settings: HashMap<String, String>,
        }

        impl ProcessingContext for TestContext {
            fn into_context(self) -> JsonValue {
                serde_json::to_value(self.settings)
                    .unwrap_or(JsonValue::Null)
            }
        }

        let mut settings = HashMap::new();
        _ = settings.insert("key".to_string(), "value".to_string());

        let context = TestContext { settings };
        let json_context = context.into_context();

        assert_eq!(json_context["key"], "value");
    }

    #[test]
    fn test_generator() {
        #[derive(Debug)]
        struct TestGenerator;

        impl Generator for TestGenerator {
            fn generate(
                &self,
                content: &str,
                path: &Path,
                _options: Option<&JsonValue>,
            ) -> Result<()> {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(path, content)?;
                Ok(())
            }

            fn validate(
                &self,
                path: &Path,
                _options: Option<&JsonValue>,
            ) -> Result<()> {
                if let Some(parent) = path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)?;
                    }
                }
                Ok(())
            }
        }

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        let generator = TestGenerator;

        generator
            .generate("test content", &test_file, None)
            .unwrap();
        assert!(test_file.exists());
        assert_eq!(
            fs::read_to_string(test_file).unwrap(),
            "test content"
        );
    }

    #[test]
    fn test_shareable() {
        #[derive(Debug)]
        struct TestState {
            counter: usize,
        }

        let state = TestState { counter: 0 };
        let shared = state.into_shared();

        {
            let mut write_guard = shared.write();
            write_guard.counter += 1;
        }

        let read_guard = shared.read();
        assert_eq!(read_guard.counter, 1);
    }

    #[test]
    fn test_processing_options() {
        let default_options = ProcessingOptions::default();
        assert!(!default_options.strict_mode);
        assert!(default_options.validate);
        assert!(default_options.cache_enabled);

        let custom_options = ProcessingOptions {
            strict_mode: true,
            validate: false,
            cache_enabled: false,
            custom: serde_json::json!({"key": "value"}),
        };
        assert!(custom_options.strict_mode);
        assert!(!custom_options.validate);
        assert!(!custom_options.cache_enabled);
        assert_eq!(custom_options.custom["key"], "value");
    }
}
