//! # Core Module
//!
//! This module provides the fundamental building blocks and infrastructure for the NucleusFlow
//! static site generator. It contains essential components that are used throughout the library:
//!
//! - Configuration management
//! - Error handling
//! - Core traits and interfaces
//!
//! ## Module Structure
//!
//! - [`config`]: Handles configuration management and validation
//! - [`error`]: Provides error types and handling mechanisms
//! - [`traits`]: Defines core traits and interfaces for the library
//!
//! ## Usage
//!
//! These modules are typically used together to provide the foundation for content processing
//! and site generation:
//!
//! ```rust,no_run
//! use nucleusflow::core::{
//!     config::Config,
//!     error::Result,
//!     traits::{Generator, Processor}
//! };
//! ```

/// Configuration management module.
///
/// This module provides flexible configuration handling for the static site generator,
/// including support for different configuration sources and formats.
///
/// See [`config`](config) module documentation for more details.
pub mod config;

/// Error handling module.
///
/// This module defines custom error types and result types used throughout the library,
/// providing consistent error handling patterns.
///
/// See [`error`](error) module documentation for more details.
pub mod error;

/// Core traits module.
///
/// This module defines fundamental traits that form the backbone of the library's
/// content processing and generation capabilities.
///
/// See [`traits`](traits) module documentation for more details.
pub mod traits;

// Re-export commonly used types
pub use config::Config;
pub use error::{ProcessingError, Result};
pub use traits::{Generator, Processor, Transform};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_module_exports() {
        // This test verifies that our public exports are available
        // and correctly typed
        fn assert_exports<T: Send + Sync>() {}

        assert_exports::<Config>();
        assert_exports::<ProcessingError>();
    }
}
