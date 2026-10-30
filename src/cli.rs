// Copyright © 2024 NucleusFlow. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Command-line interface for NucleusFlow
//!
//! This module provides the command-line interface for the NucleusFlow static site generator.
//! It handles argument parsing, command execution, and user interaction.
//!
//! # Examples
//!
//! Basic usage example to parse a `new` command with template argument:
//!
//! ```
//! use nucleusflow::cli;
//!
//! let matches = cli::build().get_matches_from(vec![
//!     "nucleusflow",
//!     "new",
//!     "my-site",
//!     "--template",
//!     "blog"
//! ]);
//!
//! assert!(matches.subcommand_matches("new").is_some());
//! let new_cmd = matches.subcommand_matches("new").unwrap();
//! assert_eq!(new_cmd.get_one::<String>("template").unwrap(), "blog");
//! ```

use crate::core::error::{ProcessingError, Result};
use clap::{value_parser, Arg, ArgAction, Command};
use log::{debug, info};
use std::fs;
use std::path::PathBuf;

/// The current version of NucleusFlow, as defined in `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default content directory used when building the static site.
pub const DEFAULT_CONTENT_DIR: &str = "content";
/// Default output directory used for generated static files.
pub const DEFAULT_OUTPUT_DIR: &str = "public";
/// Default template directory where templates are stored.
pub const DEFAULT_TEMPLATE_DIR: &str = "templates";
/// Default port for the development server.
pub const DEFAULT_PORT: u16 = 3000;

/// Builds and configures the NucleusFlow command-line interface.
pub fn build() -> Command {
    debug!("Building CLI command structure");

    Command::new("NucleusFlow")
        .author("NucleusFlow Contributors")
        .about("A fast and flexible static site generator written in Rust.")
        .version(VERSION)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("new")
                .about("Create a new project")
                .arg(
                    Arg::new("name")
                        .help("Name of the new project")
                        .required(true)
                        .value_parser(value_parser!(String))
                )
                .arg(
                    Arg::new("template")
                        .short('t')
                        .long("template")
                        .help("Template to use (blog, docs, portfolio)")
                        .value_parser(["blog", "docs", "portfolio"])
                        .default_value("blog")
                )
        )
        .subcommand(
            Command::new("build")
                .about("Build the static site")
                .arg(
                    Arg::new("content")
                        .short('c')
                        .long("content")
                        .help("Content directory")
                        .value_parser(value_parser!(PathBuf))
                        .default_value(DEFAULT_CONTENT_DIR)
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("Output directory")
                        .value_parser(value_parser!(PathBuf))
                        .default_value(DEFAULT_OUTPUT_DIR)
                )
                .arg(
                    Arg::new("template")
                        .short('t')
                        .long("template")
                        .help("Template directory")
                        .value_parser(value_parser!(PathBuf))
                        .default_value(DEFAULT_TEMPLATE_DIR)
                )
                .arg(
                    Arg::new("minify")
                        .short('m')
                        .long("minify")
                        .help("Minify output")
                        .action(ArgAction::SetTrue)
                )
        )
        .after_help(
            "\x1b[1;4mDocumentation:\x1b[0m\n\n  https://nucleusflow.com\n\n\
             \x1b[1;4mLicense:\x1b[0m\n  The project is licensed under the terms of \
             both the MIT license and the Apache License (Version 2.0)."
        )
}

/// Executes the command-line interface by matching the subcommand and arguments.
///
/// # Returns
/// * `Result<()>` - Indicates success, or an error if execution fails.
pub fn execute() -> Result<()> {
    let matches = build().get_matches();
    execute_matches(&matches)
}

/// Executes command-line logic given parsed `ArgMatches`.
pub fn execute_matches(matches: &clap::ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("new", sub_matches)) => {
            let name = sub_matches.get_one::<String>("name").unwrap();
            let default_template = "blog".to_string();
            let template = sub_matches
                .get_one::<String>("template")
                .unwrap_or(&default_template);
            create_new_project(name, template)
        }
        Some(("build", sub_matches)) => {
            let content_dir =
                sub_matches.get_one::<PathBuf>("content").unwrap();
            let output_dir =
                sub_matches.get_one::<PathBuf>("output").unwrap();
            let template_dir =
                sub_matches.get_one::<PathBuf>("template").unwrap();
            let minify = sub_matches.get_flag("minify");
            build_site(content_dir, output_dir, template_dir, minify)
        }
        _ => Err(ProcessingError::internal_error("Unknown command")),
    }
}

/// Creates a new project with the specified name and template.
fn create_new_project(name: &str, template: &str) -> Result<()> {
    info!(
        "Creating new project '{}' with template '{}'",
        name, template
    );

    if name.is_empty() {
        return Err(ProcessingError::configuration(
            "Project name cannot be empty",
            None,
            None,
        ));
    }

    Ok(())
}

/// Builds the site, generating static files in the output directory, with optional minification.
fn build_site(
    content_dir: &PathBuf,
    output_dir: &PathBuf,
    template_dir: &PathBuf,
    minify: bool,
) -> Result<()> {
    info!(
        "Building site with content at '{:?}', output to '{:?}', and templates in '{:?}'",
        content_dir, output_dir, template_dir
    );

    if !content_dir.exists() {
        return Err(ProcessingError::configuration(
            "Content directory does not exist",
            Some(content_dir.clone()),
            None,
        ));
    }

    // Example of processing files, adding minification if enabled
    let output_content =
        fs::read_to_string(content_dir)?.to_uppercase(); // Placeholder for content processing

    // Minify content if the minify flag is true
    let final_content = if minify {
        minify_content(&output_content)
    } else {
        output_content
    };

    fs::write(output_dir.join("output.html"), final_content)?;

    Ok(())
}

/// Minifies the given content for output.
///
/// # Arguments
/// * `content` - The content to be minified.
///
/// # Returns
/// * `String` - The minified content.
fn minify_content(content: &str) -> String {
    // Placeholder minification: replace multiple spaces with a single space.
    content.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Displays the NucleusFlow banner with version and description information.
pub fn print_banner() {
    info!("Displaying NucleusFlow banner");

    let title = format!("NucleusFlow 🦀 v{}", VERSION);
    let description = "A powerful Rust library for content processing, enabling static site generation, document conversion, and templating.";

    let width = title.len().max(description.len()) + 4;
    let horizontal_line = "─".repeat(width - 2);

    println!("\n┌{}┐", horizontal_line);
    println!("│{:^width$}│", title, width = width - 2);
    println!("├{}┤", horizontal_line);
    println!("│{:^width$}│", description, width = width - 2);
    println!("└{}┘\n", horizontal_line);
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::ArgMatches;

    fn get_matches(args: Vec<&str>) -> ArgMatches {
        build().get_matches_from(args)
    }

    #[test]
    fn test_new_command() {
        let matches = get_matches(vec![
            "nucleusflow",
            "new",
            "my-site",
            "--template",
            "blog",
        ]);
        let new_cmd = matches.subcommand_matches("new").unwrap();

        assert_eq!(
            new_cmd.get_one::<String>("name").unwrap(),
            "my-site"
        );
        assert_eq!(
            new_cmd.get_one::<String>("template").unwrap(),
            "blog"
        );

        assert!(execute_matches(&matches).is_ok());
    }

    #[test]
    fn test_new_command_empty_name() {
        let matches = get_matches(vec!["nucleusflow", "new", ""]);
        let result = execute_matches(&matches);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_command() {
        let temp_dir = std::env::temp_dir().join(format!(
            "nucleusflow_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();
        let content_file = temp_dir.join("content.md");
        fs::write(&content_file, "hello   world\n\nrust").unwrap();

        let matches = get_matches(vec![
            "nucleusflow",
            "build",
            "--content",
            content_file.to_str().unwrap(),
            "--output",
            temp_dir.to_str().unwrap(),
            "--minify",
        ]);
        let build_cmd = matches.subcommand_matches("build").unwrap();

        assert_eq!(
            build_cmd.get_one::<PathBuf>("content").unwrap().as_path(),
            content_file.as_path()
        );
        assert!(build_cmd.get_flag("minify"));

        let res = execute_matches(&matches);
        assert!(res.is_ok());

        let output_file = temp_dir.join("output.html");
        assert!(output_file.exists());
        let output_content = fs::read_to_string(&output_file).unwrap();
        assert_eq!(output_content, "HELLO WORLD RUST");

        // Now test without minify
        let matches_no_min = get_matches(vec![
            "nucleusflow",
            "build",
            "--content",
            content_file.to_str().unwrap(),
            "--output",
            temp_dir.to_str().unwrap(),
        ]);
        let res_no_min = execute_matches(&matches_no_min);
        assert!(res_no_min.is_ok());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_build_command_missing_content() {
        let non_existent =
            PathBuf::from("/path/that/does/not/exist/ever/12345");
        let matches = get_matches(vec![
            "nucleusflow",
            "build",
            "--content",
            non_existent.to_str().unwrap(),
        ]);
        let result = execute_matches(&matches);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_matches_unknown_subcommand() {
        // Build an empty clap command to test None subcommand
        let cmd = Command::new("test");
        let matches = cmd.get_matches_from(vec!["test"]);
        let result = execute_matches(&matches);
        assert!(result.is_err());
    }

    #[test]
    fn test_minify_content_helper() {
        let raw = "  hello   world \n \t foo   bar  ";
        let minified = minify_content(raw);
        assert_eq!(minified, "hello world foo bar");
    }

    #[test]
    fn test_print_banner_fn() {
        print_banner();
    }
}
