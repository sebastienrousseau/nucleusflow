//! # Core Traits Module
//!
//! This module defines the fundamental traits that form the backbone of the NucleusFlow processing library. These traits provide a flexible and extensible foundation for implementing various content processing, transformation, and generation capabilities.
//!
//! ## Key Traits
//!
//! - [`Processor`]: Core trait for content processing implementations
//! - [`Transform`]: Trait for content transformation operations
//! - [`Generator`]: Trait for output generation
//! - [`Validator`]: Trait for content validation
//! - [`IntoContext`]: Trait for converting types into a processing context
//! - [`Shareable`]: Trait for wrapping types in a shareable, thread-safe container
//!
//! ## Design Philosophy
//!
//! The traits in this module follow these key principles:
//!
//! - **Separation of Concerns**: Each trait has a single, well-defined responsibility
//! - **Composability**: Traits can be combined to create more complex processing pipelines
//! - **Type Safety**: Generic type parameters ensure type-safe processing chains
//! - **Error Handling**: Consistent error handling through the `Result` type

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::core::error::Result;
use parking_lot::RwLock;
use serde_json::Value as JsonValue;

/// Core trait for implementing content processors.
///
/// This trait defines the basic interface for any type that can process content
/// from one form to another. It's designed to be flexible and composable through
/// its generic type parameters.
///
/// # Type Parameters
///
/// * `Input`: The type of content being processed
/// * `Output`: The type of content produced
/// * `Context`: Additional context or configuration for processing
pub trait Processor: Send + Sync + std::fmt::Debug {
    /// The type of input content for the processor.
    type Input;
    /// The type of output content produced by the processor.
    type Output;
    /// The type of context or configuration used by the processor.
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
    /// A `Result` containing either the processed content or an error.
    fn process(&self, input: Self::Input, context: Option<&Self::Context>) -> Result<Self::Output>;
}

/// Trait for implementing content transformation operations.
///
/// This trait is designed for processors that transform content from one form to
/// another while preserving the ability to chain transformations together.
///
/// # Type Parameters
///
/// * `Input`: The input type for the transformation
/// * `Output`: The output type produced by the transformation
pub trait Transform: Send + Sync + std::fmt::Debug {
    /// The type of input content for the transformation
    type Input;
    /// The type of output content produced by the transformation
    type Output;

    /// Transforms the input content into the output format.
    ///
    /// # Arguments
    ///
    /// * `input` - The content to transform
    ///
    /// # Returns
    ///
    /// A `Result` containing either the transformed content or an error.
    fn transform(&self, input: Self::Input) -> Result<Self::Output>;
}

/// Trait for implementing output generation.
///
/// This trait defines the interface for types that can generate output in various
/// formats, with support for configuration options and validation.
pub trait Generator: Send + Sync + std::fmt::Debug {
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
    /// A `Result` indicating success or failure of the generation process.
    fn generate(&self, content: &str, path: &Path, options: Option<&JsonValue>) -> Result<()>;

    /// Validates the generation parameters without performing the generation.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to validate
    /// * `options` - Optional configuration to validate
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the parameters are valid.
    fn validate(&self, path: &Path, options: Option<&JsonValue>) -> Result<()>;
}

/// Trait for implementing content validation.
///
/// This trait provides a standard interface for validating content before
/// processing or transformation occurs.
pub trait Validator: Send + Sync + std::fmt::Debug {
    /// The type of input content to validate.
    type Input;

    /// Validates the input content.
    ///
    /// # Arguments
    ///
    /// * `input` - The content to validate
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the content is valid.
    fn validate(&self, input: &Self::Input) -> Result<()>;
}

/// Trait for types that can be converted into a processing context.
///
/// This trait provides a standard way to convert various types into
/// processing context objects that can be used with processors.
pub trait IntoContext {
    /// Converts the type into a processing context.
    ///
    /// # Returns
    ///
    /// A `JsonValue` representing the processing context.
    fn into_context(self) -> JsonValue;
}

/// Trait for types that can be shared between multiple processors.
///
/// This trait provides a standard way to wrap types in thread-safe
/// reference-counted containers for sharing between processors.
pub trait Shareable: Sized + Send + Sync + std::fmt::Debug {
    /// Converts the type into a shareable form.
    ///
    /// # Returns
    ///
    /// An `Arc<RwLock<Self>>` containing the shareable value.
    fn into_shared(self) -> Arc<RwLock<Self>>;
}

// Blanket implementation of Shareable for all eligible types
impl<T: Send + Sync + std::fmt::Debug + 'static> Shareable for T {
    fn into_shared(self) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(self))
    }
}

// Implementation of IntoContext for JsonValue
impl IntoContext for JsonValue {
    fn into_context(self) -> JsonValue {
        self
    }
}

/// Wrapper type for HashMap to allow IntoContext implementation
#[derive(Debug)]
pub struct ContextMap(HashMap<String, String>);

impl IntoContext for ContextMap {
    fn into_context(self) -> JsonValue {
        serde_json::to_value(self.0).unwrap_or(JsonValue::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_processor_implementation() {
        #[derive(Debug)]
        struct TestProcessor;

        impl Processor for TestProcessor {
            type Input = String;
            type Output = String;
            type Context = JsonValue;

            fn process(&self, input: Self::Input, _context: Option<&Self::Context>) -> Result<Self::Output> {
                Ok(input.to_uppercase())
            }
        }

        let processor = TestProcessor;
        let result = processor.process("test".to_string(), None).unwrap();
        assert_eq!(result, "TEST");
    }

    #[test]
    fn test_transform_implementation() {
        #[derive(Debug)]
        struct TestTransform;

        impl Transform for TestTransform {
            type Input = String;
            type Output = String;

            fn transform(&self, input: Self::Input) -> Result<Self::Output> {
                Ok(input.chars().rev().collect())
            }
        }

        let transform = TestTransform;
        let result = transform.transform("hello".to_string()).unwrap();
        assert_eq!(result, "olleh");
    }

    #[test]
    fn test_generator_implementation() {
        #[derive(Debug)]
        struct TestGenerator;

        impl Generator for TestGenerator {
            fn generate(&self, content: &str, path: &Path, _options: Option<&JsonValue>) -> Result<()> {
                fs::write(path, content)?;
                Ok(())
            }

            fn validate(&self, path: &Path, _options: Option<&JsonValue>) -> Result<()> {
                if let Some(parent) = path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)?;
                    }
                }
                Ok(())
            }
        }

        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("output.txt");
        let generator = TestGenerator;

        generator.generate("test content", &output_path, None).unwrap();
        assert!(output_path.exists());
        assert_eq!(fs::read_to_string(output_path).unwrap(), "test content");
    }

    #[test]
    fn test_validator_implementation() {
        #[derive(Debug)]
        struct TestValidator;

        impl Validator for TestValidator {
            type Input = String;

            fn validate(&self, input: &Self::Input) -> Result<()> {
                if input.is_empty() {
                    Err(crate::core::error::ProcessingError::Validation {
                        details: "Input cannot be empty".to_string(),
                        context: None,
                    })
                } else {
                    Ok(())
                }
            }
        }

        let validator = TestValidator;
        assert!(validator.validate(&"test".to_string()).is_ok());
        assert!(validator.validate(&"".to_string()).is_err());
    }

    #[test]
    fn test_into_context() {
        let mut map = HashMap::new();
        _ = map.insert("key".to_string(), "value".to_string());
        let context_map = ContextMap(map);
        let context = context_map.into_context();
        assert_eq!(context, json!({"key": "value"}));
    }

    #[test]
    fn test_shareable() {
        #[derive(Debug)]
        struct TestState {
            counter: usize,
        }

        let state = TestState { counter: 0 };
        let shared = state.into_shared();
        assert_eq!(shared.read().counter, 0);

        shared.write().counter += 1;
        assert_eq!(shared.read().counter, 1);
    }
}
