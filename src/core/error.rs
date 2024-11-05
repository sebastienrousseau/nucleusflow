//! # Error Handling for NucleusFlow
//!
//! This module defines custom error types for various components of the
//! NucleusFlow static site generator. The `thiserror` crate is used to
//! simplify error creation and ensure consistent handling across the library.

use std::path::PathBuf;
use thiserror::Error;

/// A unified result type for the NucleusFlow library.
///
/// This type alias simplifies function signatures by defining a result type
/// that always uses `ProcessingError` as the error variant.
pub type Result<T> = std::result::Result<T, ProcessingError>;

/// The main error type for NucleusFlow, encompassing all potential error cases.
///
/// `ProcessingError` is an enumerated type that represents different errors
/// that can occur throughout the library. Each variant describes a specific
/// error type with associated details.
#[derive(Error, Debug)]
pub enum ProcessingError {
    /// Represents errors that occur during content parsing or processing.
    #[error("Failed to process content: {details}")]
    ContentProcessing {
        /// Detailed description of what went wrong
        details: String,
        /// The source error if one exists
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Represents errors related to file operations.
    #[error("File operation failed for '{path}': {details}")]
    FileOperation {
        /// The path where the operation failed
        path: PathBuf,
        /// Description of what went wrong
        details: String,
        /// The underlying IO error if one exists
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Represents missing file errors.
    #[error("File not found: {path}")]
    FileNotFound {
        /// The path that wasn't found
        path: PathBuf,
        /// Additional context about why the file was being accessed
        details: String,
    },

    /// Represents template processing errors.
    #[error("Template error in '{template_name}': {details}")]
    TemplateProcessing {
        /// Name of the template that failed
        template_name: String,
        /// Description of what went wrong
        details: String,
        /// The source error if one exists
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Represents configuration validation errors.
    #[error("Configuration error: {details}")]
    Configuration {
        /// Description of the configuration error
        details: String,
        /// The path to the configuration file if relevant
        path: Option<PathBuf>,
        /// The source error if one exists
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Represents validation errors during processing.
    #[error("Validation failed: {details}")]
    Validation {
        /// Description of what failed validation
        details: String,
        /// Additional context or data related to the validation
        context: Option<String>,
    },

    /// Represents errors during output generation.
    #[error("Output generation failed for '{path}': {details}")]
    OutputGeneration {
        /// The path where output was being generated
        path: PathBuf,
        /// Description of what went wrong
        details: String,
        /// The source error if one exists
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Represents serialization/deserialization errors.
    #[error("Serialization error: {details}")]
    Serialization {
        /// Description of what went wrong
        details: String,
        /// The source error if one exists
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Represents plugin-related errors.
    #[error("Plugin error for '{plugin_name}': {details}")]
    Plugin {
        /// Name of the plugin that encountered an error
        plugin_name: String,
        /// Description of what went wrong
        details: String,
        /// The source error if one exists
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Represents unexpected or internal errors.
    #[error("Internal error: {details}")]
    Internal {
        /// Description of the internal error
        details: String,
        /// The source error if one exists
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// IO error encountered during file operations.
    ///
    /// This variant is used for errors encountered while reading or writing files.
    #[error("File IO error at `{path:?}`: {source}")]
    IOError {
        /// Path associated with the IO error.
        path: PathBuf,
        /// The underlying IO error.
        #[source]
        source: std::io::Error,
    },

    /// General internal error.
    ///
    /// This variant represents miscellaneous errors within the library that
    /// do not fall under any specific category.
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl ProcessingError {
    /// Creates a new `ContentProcessing` error with the given details and source.
    pub fn content_processing<S: Into<String>>(
        details: S,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::ContentProcessing {
            details: details.into(),
            source,
        }
    }

    /// Creates a new `FileOperation` error with the specified path, details, and source error.
    pub fn file_operation<P: Into<PathBuf>, S: Into<String>>(
        path: P,
        details: S,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::FileOperation {
            path: path.into(),
            details: details.into(),
            source,
        }
    }

    /// Creates a new `TemplateProcessing` error for a specified template with a message and source error.
    pub fn template_processing<S1: Into<String>, S2: Into<String>>(
        template_name: S1,
        details: S2,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::TemplateProcessing {
            template_name: template_name.into(),
            details: details.into(),
            source,
        }
    }

    /// Creates a new `Configuration` error with specified details, optional path, and source error.
    pub fn configuration<S: Into<String>>(
        details: S,
        path: Option<PathBuf>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::Configuration {
            details: details.into(),
            path,
            source,
        }
    }

    /// Creates a new `Validation` error with a message and optional context.
    pub fn validation<S1: Into<String>, S2: Into<String>>(
        details: S1,
        context: Option<S2>,
    ) -> Self {
        Self::Validation {
            details: details.into(),
            context: context.map(|s| s.into()),
        }
    }

    /// Creates a new `OutputGeneration` error for a specified path with details and source error.
    pub fn output_generation<P: Into<PathBuf>, S: Into<String>>(
        path: P,
        details: S,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::OutputGeneration {
            path: path.into(),
            details: details.into(),
            source,
        }
    }

    /// Creates a new `Serialization` error with specified details and optional source error.
    pub fn serialization<S: Into<String>>(
        details: S,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::Serialization {
            details: details.into(),
            source,
        }
    }

    /// Creates a new `Plugin` error for a specified plugin with details and source error.
    pub fn plugin<S1: Into<String>, S2: Into<String>>(
        plugin_name: S1,
        details: S2,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::Plugin {
            plugin_name: plugin_name.into(),
            details: details.into(),
            source,
        }
    }

    /// Creates a new `Internal` error with specified details and source error.
    pub fn internal<S: Into<String>>(
        details: S,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::Internal {
            details: details.into(),
            source,
        }
    }

    /// Wraps an IO error as an `IOError` variant with the specified path.
    pub fn io_error(path: PathBuf, source: std::io::Error) -> Self {
        ProcessingError::IOError { path, source }
    }

    /// Creates a general `InternalError` with a custom message.
    pub fn internal_error<S: Into<String>>(message: S) -> Self {
        ProcessingError::InternalError(message.into())
    }
}

impl From<std::io::Error> for ProcessingError {
    fn from(error: std::io::Error) -> Self {
        ProcessingError::FileOperation {
            path: PathBuf::new(),
            details: error.to_string(),
            source: Some(Box::new(error)),
        }
    }
}

impl From<serde_json::Error> for ProcessingError {
    fn from(error: serde_json::Error) -> Self {
        ProcessingError::Serialization {
            details: error.to_string(),
            source: Some(Box::new(error)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error, ErrorKind};

    #[test]
    fn test_content_processing() {
        let error = ProcessingError::content_processing(
            "Failed to process",
            None,
        );
        assert!(matches!(
            error,
            ProcessingError::ContentProcessing { .. }
        ));
        assert!(error.to_string().contains("Failed to process"));
    }

    #[test]
    fn test_file_not_found_error() {
        let path = PathBuf::from("/test/path");
        let io_error =
            Error::new(ErrorKind::NotFound, "file not found");
        let error = ProcessingError::file_operation(
            path,
            "Operation failed",
            Some(Box::new(io_error)),
        );

        assert!(matches!(error, ProcessingError::FileOperation { .. }));
        assert!(error.to_string().contains("/test/path"));
    }

    #[test]
    fn test_template_processing_error() {
        let error = ProcessingError::template_processing(
            "main.hbs",
            "Template syntax error",
            None,
        );
        assert!(matches!(
            error,
            ProcessingError::TemplateProcessing { .. }
        ));
        assert!(error.to_string().contains("main.hbs"));
    }

    #[test]
    fn test_configuration_error() {
        let path = PathBuf::from("/config/file.toml");
        let error = ProcessingError::configuration(
            "Invalid configuration",
            Some(path),
            None,
        );
        assert!(matches!(error, ProcessingError::Configuration { .. }));
        assert!(error.to_string().contains("Invalid configuration"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error =
            Error::new(ErrorKind::NotFound, "file not found");
        let error: ProcessingError = io_error.into();
        assert!(matches!(error, ProcessingError::FileOperation { .. }));
    }

    #[test]
    fn test_validation_error() {
        let error = ProcessingError::validation(
            "Invalid input",
            Some("Expected positive number"),
        );
        assert!(matches!(error, ProcessingError::Validation { .. }));
        assert!(error.to_string().contains("Invalid input"));
    }
}
