//! # Error Handling for NucleusFlow
//!
//! This module defines custom error types for various components of the
//! NucleusFlow static site generator. The `thiserror` crate is used to
//! simplify error creation and ensure consistent handling across the library.

use std::path::PathBuf;
use thiserror::Error;

/// A unified result type for the NucleusFlow library.
///
/// This type alias simplifies function signatures by defining a result type that always uses `NucleusFlowError` as the error variant.
pub type Result<T> = std::result::Result<T, NucleusFlowError>;

/// The main error type for NucleusFlow, encompassing all potential error cases.
///
/// `NucleusFlowError` is an enumerated type that represents different errors that can occur throughout the library. Each variant describes a specific error type with associated details.
#[derive(Error, Debug)]
pub enum NucleusFlowError {
    /// Error related to configuration initialisation or validation.
    ///
    /// This error occurs when there is a problem with configuration files or values.
    #[error("Configuration error: {message}.")]
    ConfigError {
        /// Detailed description of the configuration error.
        message: String,
        /// Optional path of the configuration file that caused the error.
        path: Option<PathBuf>,
    },

    /// Error encountered during content processing.
    ///
    /// This variant covers errors in operations such as parsing, validating,
    /// or transforming content.
    #[error("Content processing error: {message}.")]
    ContentProcessingError {
        /// Detailed description of the content processing error.
        message: String,
        /// Optional source error providing additional context, if available.
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Error in HTML output generation.
    ///
    /// This variant represents issues that arise during the creation of output files, particularly if there is a problem writing to the output path.
    #[error("Output generation error: {message} at {path:?}.")]
    OutputGenerationError {
        /// Description of the output generation error.
        message: String,
        /// Path associated with the error.
        path: PathBuf,
        /// Optional source error providing additional context, if available.
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Error related to template rendering.
    ///
    /// This variant is used when rendering templates fails, either due to syntax issues or missing templates.
    #[error(
        "Template rendering error: {message} in template `{template}`."
    )]
    TemplateRenderingError {
        /// Description of the template rendering error.
        message: String,
        /// The specific template file or identifier associated with the error.
        template: String,
        /// Optional source error providing additional context, if available.
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

impl From<std::io::Error> for NucleusFlowError {
    /// Converts a standard IO error into a `NucleusFlowError::IOError`.
    ///
    /// # Parameters
    /// - `source`: The IO error encountered.
    ///
    /// # Returns
    /// - A `NucleusFlowError::IOError` with an empty path if no path is provided.
    fn from(source: std::io::Error) -> Self {
        NucleusFlowError::IOError {
            path: PathBuf::new(),
            source,
        }
    }
}

impl NucleusFlowError {
    /// Creates a `ConfigError` with a specific message.
    ///
    /// # Parameters
    /// - `message`: A description of the configuration error.
    /// - `path`: Optional path of the configuration file causing the error.
    ///
    /// # Returns
    /// - A `NucleusFlowError::ConfigError` containing the message and optional path.
    pub fn config_error<S: Into<String>>(
        message: S,
        path: Option<PathBuf>,
    ) -> Self {
        NucleusFlowError::ConfigError {
            message: message.into(),
            path,
        }
    }

    /// Creates a `ContentProcessingError` with a specific message and optional source.
    ///
    /// # Parameters
    /// - `message`: A description of the content processing error.
    /// - `source`: An optional source error providing additional context.
    ///
    /// # Returns
    /// - A `NucleusFlowError::ContentProcessingError` with the message and optional source.
    pub fn content_processing_error<S: Into<String>>(
        message: S,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        NucleusFlowError::ContentProcessingError {
            message: message.into(),
            source,
        }
    }

    /// Creates an `OutputGenerationError` with a specific message, path, and optional source.
    ///
    /// # Parameters
    /// - `message`: A description of the output generation error.
    /// - `path`: The path associated with the error.
    /// - `source`: An optional source error providing additional context.
    ///
    /// # Returns
    /// - A `NucleusFlowError::OutputGenerationError` with the message, path, and optional source.
    pub fn output_generation_error<S: Into<String>>(
        message: S,
        path: PathBuf,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        NucleusFlowError::OutputGenerationError {
            message: message.into(),
            path,
            source,
        }
    }

    /// Creates a `TemplateRenderingError` with a message, template name, and optional source.
    ///
    /// # Parameters
    /// - `message`: A description of the template rendering error.
    /// - `template`: The template associated with the error.
    /// - `source`: An optional source error providing additional context.
    ///
    /// # Returns
    /// - A `NucleusFlowError::TemplateRenderingError` with the message, template name, and optional source.
    pub fn template_rendering_error<S: Into<String>>(
        message: S,
        template: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        NucleusFlowError::TemplateRenderingError {
            message: message.into(),
            template,
            source,
        }
    }

    /// Wraps an IO error as an `IOError` variant with the specified path.
    ///
    /// # Parameters
    /// - `path`: The file path associated with the IO error.
    /// - `source`: The original IO error.
    ///
    /// # Returns
    /// - A `NucleusFlowError::IOError` with the specified path and source.
    pub fn io_error(path: PathBuf, source: std::io::Error) -> Self {
        NucleusFlowError::IOError { path, source }
    }

    /// Creates a general internal error with a custom message.
    ///
    /// # Parameters
    /// - `message`: A description of the internal error.
    ///
    /// # Returns
    /// - A `NucleusFlowError::InternalError` with the provided message.
    pub fn internal_error<S: Into<String>>(message: S) -> Self {
        NucleusFlowError::InternalError(message.into())
    }
}
