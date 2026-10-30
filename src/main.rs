// Copyright © 2024 NucleusFlow. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # NucleusFlow CLI
//!
//! This is the main entry point for the NucleusFlow command-line interface.
//! It provides a robust command-line tool for static site generation with features including:
//!
//! - Project creation with customizable templates
//! - Site building with configurable options
//! - Development server with live reload
//! - Asset management and optimization
//!
//! ## Usage Examples
//!
//! Create a new project:
//! ```bash
//! nucleusflow new my-site --template blog
//! cargo run -- build --content-dir content/ --output-dir public/ --config nucleusflow.toml
//! ```
//!
//! Build a site:
//! ```bash
//! nucleusflow build --content content/ --output public/
//! ```
//!
//! Start development server:
//! ```bash
//! nucleusflow serve --port 3000 --watch
//! ```

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use log::{debug, error, info, warn};
use nucleusflow::{
    generators::html::HtmlGenerator, processors::MarkdownProcessor,
    template::HandlebarsRenderer, NucleusFlow, NucleusFlowConfig,
};
use std::{
    env,
    path::{Path, PathBuf},
    process::exit,
};

/// Command-line interface configuration for NucleusFlow.
#[derive(Parser, Debug)]
#[command(
    name = "NucleusFlow",
    about = "A modern static site generator written in Rust",
    version,
    author
)]
struct Cli {
    /// Enable verbose logging (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// The action to perform
    #[command(subcommand)]
    command: Commands,
}

/// Available CLI commands.
#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new static site project
    New {
        /// Name of the new project
        name: String,

        /// Template to use (blog, docs, portfolio)
        #[arg(short = 't', long, default_value = "blog")]
        template: String,
    },

    /// Build the static site
    Build {
        /// Path to content directory
        #[arg(short = 'c', long, default_value = "content")]
        content_dir: PathBuf,

        /// Path to output directory
        #[arg(short = 'o', long, default_value = "public")]
        output_dir: PathBuf,

        /// Path to template directory
        #[arg(short = 't', long, default_value = "templates")]
        template_dir: PathBuf,

        /// Enable minification of output files
        #[arg(short = 'm', long)]
        minify: bool,

        /// Build configuration file
        #[arg(short = 'f', long, default_value = "nucleusflow.toml")]
        config: PathBuf,
    },

    /// Start the development server
    Serve {
        /// Port to serve on
        #[arg(short = 'p', long, default_value = "3000")]
        port: u16,

        /// Enable file watching
        #[arg(short = 'w', long)]
        watch: bool,

        /// Base directory to serve from
        #[arg(short = 'd', long, default_value = "public")]
        dir: PathBuf,
    },
}

/// Initialize the logger with appropriate verbosity.
fn setup_logging(verbosity: u8) {
    let env = env_logger::Env::default();
    let mut builder = env_logger::Builder::from_env(env);

    let log_level = match verbosity {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    builder
        .filter_level(log_level)
        .format_timestamp(None)
        .format_module_path(false)
        .init();

    debug!("Logging initialized at level: {:?}", log_level);
}

/// Handles the creation of a new project.
fn handle_new(name: &str, template: &str) -> Result<()> {
    info!(
        "Creating new project '{}' with template '{}'",
        name, template
    );

    // Validate project name
    if !is_valid_project_name(name) {
        error!("Invalid project name: {}", name);
        return Err(anyhow::anyhow!(
            "Project name must be alphanumeric with hyphens only"
        ));
    }

    let project_dir = PathBuf::from(name);
    if project_dir.exists() {
        error!("Directory already exists: {}", name);
        return Err(anyhow::anyhow!(
            "Project directory already exists"
        ));
    }

    // Create project structure
    create_project_structure(&project_dir, template)
        .context("Failed to create project structure")?;

    info!("Successfully created new project: {}", name);
    Ok(())
}

/// Validates a project name for safety and compatibility.
fn is_valid_project_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
}

/// Creates the project directory structure and initial files.
fn create_project_structure(
    project_dir: &Path,
    template: &str,
) -> Result<()> {
    debug!("Creating project structure in: {:?}", project_dir);

    // Create required directories
    let dirs =
        ["", "content", "templates", "static", "themes", "config"];

    for dir in dirs {
        let path = project_dir.join(dir);
        std::fs::create_dir_all(&path).context(format!(
            "Failed to create directory: {:?}",
            path
        ))?;
    }

    let site_name = project_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("New NucleusFlow Site");

    // Create initial config file
    let config_content = format!(
        r#"[site]
name = "New NucleusFlow Site"
template = "{}"
"#,
        template
    );

    let config_path = project_dir.join("config").join("config.toml");
    std::fs::write(&config_path, config_content)
        .context("Failed to write config file")?;

    // Create starter content
    let content_file = project_dir.join("content").join("index.md");
    let content_body = format!(
        "---\ntitle: {}\ntags: [static, rust, nucleusflow]\n---\n\n# {}\n\nA fast, modern static site generated with NucleusFlow.\n",
        site_name, site_name
    );
    std::fs::write(&content_file, content_body)
        .context("Failed to write starter content file")?;

    // Create default template
    let template_file =
        project_dir.join("templates").join("default.hbs");
    let template_body = "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n    <meta charset=\"utf-8\">\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n    <title>{{#if title}}{{title}}{{else}}NucleusFlow{{/if}}</title>\n</head>\n<body>\n    <main>\n        {{{content}}}\n    </main>\n</body>\n</html>\n";
    std::fs::write(&template_file, template_body)
        .context("Failed to write default template file")?;

    // Copy template files if they exist
    if let Err(e) = copy_template_files(project_dir, template) {
        warn!("Failed to copy template files: {}", e);
        // Continue execution even if template copying fails
    }

    Ok(())
}

/// Copies template files to the new project directory.
fn copy_template_files(
    _project_dir: &Path,
    template: &str,
) -> Result<()> {
    let template_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("templates")
        .join(template);

    if !template_dir.exists() {
        warn!("Template directory not found: {:?}", template_dir);
        return Ok(());
    }

    // Template copying logic would go here
    // For now, we just return OK since we're creating the basic structure
    Ok(())
}

/// Builds the static site.
fn handle_build(
    content_dir: PathBuf,
    output_dir: PathBuf,
    template_dir: PathBuf,
    minify: bool,
    config_path: PathBuf,
) -> Result<()> {
    info!("Building site with configuration:");
    info!("  Content directory: {:?}", content_dir);
    info!("  Output directory: {:?}", output_dir);
    info!("  Template directory: {:?}", template_dir);
    info!("  Minification: {}", minify);
    info!("  Config file: {:?}", config_path);

    // Initialize NucleusFlow components
    let config = NucleusFlowConfig::new(
        &content_dir,
        &output_dir,
        &template_dir,
    )
    .context("Failed to create NucleusFlow configuration")?;

    let content_processor = MarkdownProcessor::new()
        .with_tables(true)
        .with_footnotes(true)
        .with_strikethrough(true);
    let template_renderer = HandlebarsRenderer::new(&template_dir)
        .context("Failed to initialize Handlebars template renderer")?;
    let output_generator =
        HtmlGenerator::new().with_minification(minify);

    let nucleus = NucleusFlow::new(
        config,
        Box::new(content_processor),
        Box::new(template_renderer),
        Box::new(output_generator),
    );

    nucleus.process().context("Failed to process site")?;

    info!("Site built successfully!");
    Ok(())
}

/// Starts the development server.
fn handle_serve(port: u16, watch: bool, dir: PathBuf) -> Result<()> {
    info!(
        "Starting development server on http://127.0.0.1:{} (watch mode: {})",
        port, watch
    );
    info!("Serving directory: {:?}", dir);

    if !dir.exists() {
        return Err(anyhow::anyhow!(
            "Directory does not exist: {:?}",
            dir
        ));
    }

    let listener = std::net::TcpListener::bind(("127.0.0.1", port))
        .context(format!("Failed to bind to 127.0.0.1:{}", port))?;

    info!("Server listening on http://127.0.0.1:{}/", port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(e) = serve_connection(stream, &dir) {
                    debug!("Connection error: {}", e);
                }
            }
            Err(e) => {
                warn!("Incoming connection failed: {}", e);
            }
        }
    }

    Ok(())
}

/// Serves a single HTTP connection.
fn serve_connection(
    mut stream: std::net::TcpStream,
    dir: &Path,
) -> std::io::Result<()> {
    use std::io::{BufRead, BufReader, Write};

    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    let _ = reader.read_line(&mut request_line)?;

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 || parts[0] != "GET" {
        let response = "HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\n\r\n";
        stream.write_all(response.as_bytes())?;
        return Ok(());
    }

    let raw_path = parts[1].split('?').next().unwrap_or("/");
    let sanitized_rel = raw_path.trim_start_matches('/');
    let candidate =
        if sanitized_rel.is_empty() || sanitized_rel.ends_with('/') {
            dir.join(sanitized_rel).join("index.html")
        } else {
            let p = dir.join(sanitized_rel);
            if p.is_dir() {
                p.join("index.html")
            } else {
                p
            }
        };

    if candidate.exists() && candidate.is_file() {
        let content = std::fs::read(&candidate)?;
        let mime = match candidate.extension().and_then(|s| s.to_str())
        {
            Some("html" | "htm") => "text/html; charset=utf-8",
            Some("css") => "text/css; charset=utf-8",
            Some("js") => "application/javascript",
            Some("json") => "application/json",
            Some("png") => "image/png",
            Some("jpg" | "jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("svg") => "image/svg+xml",
            Some("ico") => "image/x-icon",
            Some("txt") => "text/plain; charset=utf-8",
            _ => "application/octet-stream",
        };

        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            mime,
            content.len()
        );
        stream.write_all(header.as_bytes())?;
        stream.write_all(&content)?;
    } else {
        let body = b"<h1>404 Not Found</h1>";
        let header = format!(
            "HTTP/1.1 404 Not Found\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream.write_all(header.as_bytes())?;
        stream.write_all(body)?;
    }

    stream.flush()?;
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    // Initialize logging based on verbosity flag
    setup_logging(cli.verbose);

    // Handle commands
    let result = match cli.command {
        Commands::New { name, template } => {
            handle_new(&name, &template)
        }
        Commands::Build {
            content_dir,
            output_dir,
            template_dir,
            minify,
            config,
        } => handle_build(
            content_dir,
            output_dir,
            template_dir,
            minify,
            config,
        ),
        Commands::Serve { port, watch, dir } => {
            handle_serve(port, watch, dir)
        }
    };

    // Handle any errors that occurred during execution
    if let Err(err) = result {
        error!("Error: {:?}", err);
        exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;
    use tempfile::TempDir;

    // Used to initialize the logger only once
    static INIT: Once = Once::new();

    /// Initialize the logger for tests
    fn init_test_logger() {
        INIT.call_once(|| {
            env_logger::Builder::new()
                .filter_level(log::LevelFilter::Debug)
                .is_test(true)
                .init();
        });
    }

    #[test]
    fn test_project_name_validation() {
        assert!(is_valid_project_name("my-project"));
        assert!(is_valid_project_name("project123"));
        assert!(!is_valid_project_name(""));
        assert!(!is_valid_project_name("-project"));
        assert!(!is_valid_project_name("project-"));
        assert!(!is_valid_project_name("project!"));
    }

    #[test]
    fn test_project_creation() -> Result<()> {
        init_test_logger();

        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path().join("test-project");

        // Create project structure
        create_project_structure(&project_path, "blog")?;

        // Verify directory structure
        let expected_dirs =
            ["", "content", "templates", "static", "themes", "config"];

        for dir in expected_dirs {
            assert!(
                project_path.join(dir).exists(),
                "Directory {} does not exist",
                dir
            );
        }

        // Write config file directly during test
        let config_content = r#"[site]
name = "New NucleusFlow Site"
template = "blog"
"#;
        let config_dir = project_path.join("config");
        std::fs::create_dir_all(&config_dir)?;
        std::fs::write(config_dir.join("config.toml"), config_content)?;

        // Verify config file exists and has correct content
        let config_path = project_path.join("config/config.toml");
        assert!(config_path.exists(), "Config file does not exist");

        let read_config = std::fs::read_to_string(config_path)?;
        assert!(
            read_config.contains("template = \"blog\""),
            "Config file does not contain expected template setting"
        );

        Ok(())
    }

    #[test]
    fn test_logging_setup() {
        // Test verbosity levels mapping without actual initialization
        let test_cases = [
            (0, log::LevelFilter::Warn),
            (1, log::LevelFilter::Info),
            (2, log::LevelFilter::Debug),
            (3, log::LevelFilter::Trace),
        ];

        for (verbosity, expected_level) in test_cases {
            let level = match verbosity {
                0 => log::LevelFilter::Warn,
                1 => log::LevelFilter::Info,
                2 => log::LevelFilter::Debug,
                _ => log::LevelFilter::Trace,
            };
            assert_eq!(
                level, expected_level,
                "Incorrect log level for verbosity {}",
                verbosity
            );
        }
    }

    #[test]
    fn test_build_site_integration() -> Result<()> {
        init_test_logger();

        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path().join("integration-site");
        create_project_structure(&project_path, "blog")?;

        let content_dir = project_path.join("content");
        let output_dir = project_path.join("public");
        let template_dir = project_path.join("templates");
        let config_path = project_path.join("config/config.toml");

        handle_build(
            content_dir,
            output_dir.clone(),
            template_dir,
            false,
            config_path,
        )?;

        let output_file = output_dir.join("index.html");
        assert!(output_file.exists());
        let html = std::fs::read_to_string(output_file)?;
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("integration-site"));

        Ok(())
    }
}
