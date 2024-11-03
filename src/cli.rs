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
        .subcommand(
            Command::new("serve")
                .about("Start development server")
                .arg(
                    Arg::new("port")
                        .short('p')
                        .long("port")
                        .help("Port to serve on")
                        .value_parser(value_parser!(u16))
                        .default_value(DEFAULT_PORT.to_string())
                )
                .arg(
                    Arg::new("watch")
                        .short('w')
                        .long("watch")
                        .help("Watch for changes")
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
        Some(("serve", sub_matches)) => {
            let port = *sub_matches.get_one::<u16>("port").unwrap();
            let watch = sub_matches.get_flag("watch");
            serve_site(port, watch)
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

/// Starts the development server on the specified port.
fn serve_site(port: u16, watch: bool) -> Result<()> {
    info!(
        "Starting development server on port {} with watch mode: {}",
        port, watch
    );

    Ok(())
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
    }

    #[test]
    fn test_build_command() {
        let matches = get_matches(vec![
            "nucleusflow",
            "build",
            "--content",
            "content",
            "--output",
            "public",
            "--minify",
        ]);
        let build_cmd = matches.subcommand_matches("build").unwrap();

        assert_eq!(
            build_cmd.get_one::<PathBuf>("content").unwrap().as_path(),
            PathBuf::from("content").as_path()
        );
        assert!(build_cmd.get_flag("minify"));
    }

    #[test]
    fn test_serve_command() {
        let matches = get_matches(vec![
            "nucleusflow",
            "serve",
            "--port",
            "8080",
            "--watch",
        ]);
        let serve_cmd = matches.subcommand_matches("serve").unwrap();

        assert_eq!(serve_cmd.get_one::<u16>("port").unwrap(), &8080);
        assert!(serve_cmd.get_flag("watch"));
    }
}
