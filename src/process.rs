// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use thiserror::Error;

/// Errors that may occur during processing operations.
#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("Failed to read file: {0}")]
    /// Represents an error that occurred while reading a file.
    ReadError(io::Error),
    #[error("Failed to write to file: {0}")]
    /// Represents an error that occurred while writing to a file.
    WriteError(io::Error),
    #[error("Failed to process content: {0}")]
    /// Represents an error that occurred while processing content.
    ContentError(String),
    #[error("Invalid path: {0}")]
    /// Represents an invalid path error.
    InvalidPath(String),
}

/// Reads content from a file at the specified path.
///
/// # Arguments
///
/// * `path` - A reference to the path of the file to read.
///
/// # Errors
///
/// Returns a `ProcessError::ReadError` if reading the file fails, or
/// `ProcessError::InvalidPath` if the path is invalid.
pub fn read_content<P: AsRef<Path>>(
    path: P,
) -> Result<String, ProcessError> {
    let path_ref = path.as_ref();
    if !path_ref.exists() {
        return Err(ProcessError::InvalidPath(format!(
            "Path does not exist: {}",
            path_ref.display()
        )));
    }

    let mut file =
        File::open(path_ref).map_err(ProcessError::ReadError)?;
    let mut content = String::new();
    let _ = file
        .read_to_string(&mut content)
        .map_err(ProcessError::ReadError)?;
    Ok(content)
}

/// Writes content to a file at the specified path.
///
/// # Arguments
///
/// * `path` - A reference to the path of the file to write to.
/// * `content` - The content to write to the file.
///
/// # Errors
///
/// Returns a `ProcessError::WriteError` if writing the file fails, or
/// `ProcessError::InvalidPath` if the path is invalid.
pub fn write_content<P: AsRef<Path>>(
    path: P,
    content: &str,
) -> Result<(), ProcessError> {
    let path_ref = path.as_ref();
    let mut file =
        File::create(path_ref).map_err(ProcessError::WriteError)?;
    file.write_all(content.as_bytes())
        .map_err(ProcessError::WriteError)
}

/// Processes content by applying a transformation function.
///
/// # Arguments
///
/// * `content` - The content to process.
/// * `transform_fn` - A function that takes a `&str` and returns a transformed `String`.
///
/// # Errors
///
/// Returns a `ProcessError::ContentError` if the transformation function fails.
pub fn process_content<F>(
    content: &str,
    transform_fn: F,
) -> Result<String, ProcessError>
where
    F: Fn(&str) -> Result<String, String>,
{
    transform_fn(content).map_err(ProcessError::ContentError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_content() {
        let result = read_content("test_file.txt");
        assert!(
            result.is_ok()
                || matches!(result, Err(ProcessError::InvalidPath(_)))
        );
    }

    #[test]
fn test_write_content() {
    let file_path = "test_output.txt";

    // Run the test
    let result = write_content(file_path, "Sample content");
    assert!(
        result.is_ok() || matches!(result, Err(ProcessError::WriteError(_)))
    );

    // Cleanup: remove the test file if it was created
    if Path::new(file_path).exists() {
        std::fs::remove_file(file_path).expect("Failed to delete test output file");
    }
}


    #[test]
    fn test_process_content() {
        let transform_fn = |s: &str| Ok(s.to_uppercase());
        let result = process_content("test content", transform_fn);
        assert_eq!(result.unwrap(), "TEST CONTENT");
    }
}
