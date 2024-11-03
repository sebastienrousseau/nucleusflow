// Copyright © 2024 NucleusFlow. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # NucleusFlow CLI
//!
//! This is the main entry point for the NucleusFlow command-line interface.
//! It initializes the logger, displays a banner, and runs the main CLI process.

use anyhow::Context;
use clap::{Parser, Subcommand};
use log::info;
use nucleusflow::{
    FileContentProcessor, HtmlOutputGenerator, HtmlTemplateRenderer,
    NucleusFlow, NucleusFlowConfig,
};
use std::path::PathBuf;

/// Main command-line interface for NucleusFlow.
#[derive(Parser)]
#[command(
    name = "NucleusFlow",
    version = "0.0.1",
    about = "A Static Site Generator"
)]
struct Cli {
    /// Verbose mode (-v, -vv, etc.)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// The action to perform, such as `build` or `generate-assets`
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the site from content, templates, and config
    Build {
        /// Path to the content directory
        #[arg(short, long, default_value = "content")]
        content_dir: String,

        /// Path to the output directory
        #[arg(short, long, default_value = "public")]
        output_dir: String,

        /// Path to the template directory
        #[arg(short, long, default_value = "templates")]
        template_dir: String,
    },

    /// Generate static assets without building content
    GenerateAssets,
}

/// Initializes and runs the NucleusFlow static site generator.
///
/// This function performs the following steps:
/// 1. Sets up the logger and displays the CLI banner.
/// 2. Configures the `NucleusFlowConfig` for content, output, and template directories.
/// 3. Initializes the main processing components: `FileContentProcessor`, `HtmlTemplateRenderer`, and `HtmlOutputGenerator`.
/// 4. Creates an instance of `NucleusFlow` and processes the content.
///
/// # Errors
///
/// This function will return an error if:
/// - The configuration directories are invalid or inaccessible.
/// - Any of the main components fail during initialization or content processing.
///
/// # Returns
///
/// `Ok(())` if the SSG completes processing successfully.
pub fn run(
    content_dir: &str,
    output_dir: &str,
    template_dir: &str,
) -> Result<(), anyhow::Error> {
    // Initialize logger
    env_logger::init();
    info!("Starting NucleusFlow SSG...");

    // Configure directories
    let config =
        NucleusFlowConfig::new(content_dir, output_dir, template_dir)
            .context("Failed to create configuration for NucleusFlow")?;

    // Initialize processing components
    let content_processor =
        FileContentProcessor::new(PathBuf::from(content_dir));
    let template_renderer =
        HtmlTemplateRenderer::new(PathBuf::from(template_dir));
    let output_generator =
        HtmlOutputGenerator::new(PathBuf::from(output_dir));

    // Create and run the NucleusFlow instance
    let nucleus = NucleusFlow::new(
        config,
        Box::new(content_processor),
        Box::new(template_renderer),
        Box::new(output_generator),
    );

    nucleus.process().context("Failed to process content")?;

    info!("NucleusFlow SSG completed successfully");
    Ok(())
}

/// The main entry point for the NucleusFlow CLI.
fn main() {
    let cli = Cli::parse();

    if let Err(err) = match &cli.command {
        Some(Commands::Build {
            content_dir,
            output_dir,
            template_dir,
        }) => run(content_dir, output_dir, template_dir),
        Some(Commands::GenerateAssets) => {
            // Placeholder for generating assets
            println!("Generating assets...");
            Ok(())
        }
        None => {
            println!(
                "No command provided. Use --help for more information."
            );
            Ok(())
        }
    } {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
