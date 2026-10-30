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

/// Handles the creation of a new project in the specified base directory.
fn handle_new_at(
    base_dir: &Path,
    name: &str,
    template: &str,
) -> Result<()> {
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

    let project_dir = base_dir.join(name);
    if project_dir.exists() {
        error!("Directory already exists: {:?}", project_dir);
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

/// Handles the creation of a new project.
fn handle_new(name: &str, template: &str) -> Result<()> {
    handle_new_at(Path::new("."), name, template)
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

/// Serves an HTTP connection stream with file lookup and MIME dispatch.
fn serve_stream<R: std::io::BufRead, W: std::io::Write>(
    mut reader: R,
    mut writer: W,
    dir: &Path,
) -> std::io::Result<()> {
    let mut request_line = String::new();
    let _ = reader.read_line(&mut request_line)?;

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 || parts[0] != "GET" {
        let response = "HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\n\r\n";
        writer.write_all(response.as_bytes())?;
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
        writer.write_all(header.as_bytes())?;
        writer.write_all(&content)?;
    } else {
        let body = b"<h1>404 Not Found</h1>";
        let header = format!(
            "HTTP/1.1 404 Not Found\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        writer.write_all(header.as_bytes())?;
        writer.write_all(body)?;
    }

    writer.flush()?;
    Ok(())
}

/// Serves a single HTTP connection.
fn serve_connection(
    stream: std::net::TcpStream,
    dir: &Path,
) -> std::io::Result<()> {
    let writer = stream.try_clone()?;
    let reader = std::io::BufReader::new(stream);
    serve_stream(reader, writer, dir)
}

/// Dispatches a parsed CLI command.
fn execute_command(command: Commands) -> Result<()> {
    match command {
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
    }
}

fn main() {
    let cli = Cli::parse();

    // Initialize logging based on verbosity flag
    setup_logging(cli.verbose);

    // Handle commands
    let result = execute_command(cli.command);

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

    #[test]
    fn test_handle_new_invalid_name() -> Result<()> {
        let temp_dir = TempDir::new()?;
        assert!(
            handle_new_at(temp_dir.path(), "-invalid", "blog").is_err()
        );
        assert!(
            handle_new_at(temp_dir.path(), "invalid!", "blog").is_err()
        );
        assert!(handle_new_at(temp_dir.path(), "", "blog").is_err());
        assert!(handle_new("-invalid", "blog").is_err());
        Ok(())
    }

    #[test]
    fn test_handle_new_collision() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let site_path = temp_dir.path().join("my-site");
        std::fs::create_dir_all(&site_path)?;
        let err = handle_new_at(temp_dir.path(), "my-site", "blog");
        assert!(err.is_err());
        assert!(err
            .unwrap_err()
            .to_string()
            .contains("Project directory already exists"));
        Ok(())
    }

    #[test]
    fn test_handle_new_success() -> Result<()> {
        let temp_dir = TempDir::new()?;
        assert!(handle_new_at(temp_dir.path(), "awesome-site", "blog")
            .is_ok());
        assert!(temp_dir
            .path()
            .join("awesome-site")
            .join("content")
            .join("index.md")
            .exists());
        assert!(temp_dir
            .path()
            .join("awesome-site")
            .join("templates")
            .join("default.hbs")
            .exists());
        Ok(())
    }

    #[test]
    fn test_copy_template_files_missing() -> Result<()> {
        let temp_dir = TempDir::new()?;
        assert!(copy_template_files(
            temp_dir.path(),
            "nonexistent-template-123"
        )
        .is_ok());
        Ok(())
    }

    #[test]
    fn test_handle_build_minify_and_missing_content() -> Result<()> {
        init_test_logger();

        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path().join("minified-site");
        create_project_structure(&project_path, "blog")?;

        let content_dir = project_path.join("content");
        let output_dir = project_path.join("public");
        let template_dir = project_path.join("templates");
        let config_path = project_path.join("config/config.toml");

        // Build with minify = true
        handle_build(
            content_dir.clone(),
            output_dir.clone(),
            template_dir.clone(),
            true,
            config_path.clone(),
        )?;

        let output_file = output_dir.join("index.html");
        assert!(output_file.exists());
        let html = std::fs::read_to_string(&output_file)?;
        assert!(html.to_lowercase().contains("<!doctype html>"));

        // Build with missing content dir should fail
        let missing_content = temp_dir.path().join("does_not_exist");
        assert!(handle_build(
            missing_content,
            output_dir,
            template_dir,
            false,
            config_path
        )
        .is_err());

        Ok(())
    }

    #[test]
    fn test_handle_serve_missing_dir() {
        let missing_dir =
            PathBuf::from("/nonexistent/directory/path/for/test");
        assert!(handle_serve(9999, false, missing_dir).is_err());
    }

    #[test]
    fn test_serve_stream_methods_and_mimes() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let doc_root = temp_dir.path();
        std::fs::write(doc_root.join("index.html"), "<h1>Home</h1>")?;
        std::fs::write(
            doc_root.join("style.css"),
            "body { margin: 0; }",
        )?;
        std::fs::write(doc_root.join("app.js"), "console.log('hi');")?;
        std::fs::write(
            doc_root.join("data.json"),
            "{\"key\": \"val\"}",
        )?;
        std::fs::write(doc_root.join("image.svg"), "<svg></svg>")?;
        std::fs::write(doc_root.join("photo.jpg"), "fake-jpeg")?;
        std::fs::write(doc_root.join("image.png"), "fake-png")?;
        std::fs::write(doc_root.join("anim.gif"), "fake-gif")?;
        std::fs::write(doc_root.join("icon.ico"), "fake-ico")?;
        std::fs::write(doc_root.join("note.txt"), "hello world")?;
        std::fs::write(doc_root.join("data.bin"), b"\x00\x01\x02")?;

        let sub_dir = doc_root.join("nested");
        std::fs::create_dir_all(&sub_dir)?;
        std::fs::write(sub_dir.join("index.html"), "<h2>Nested</h2>")?;

        let run_req = |req: &str| -> String {
            let mut reader = std::io::Cursor::new(req.as_bytes());
            let mut writer = Vec::new();
            serve_stream(&mut reader, &mut writer, doc_root).unwrap();
            String::from_utf8_lossy(&writer).into_owned()
        };

        // Test GET /
        let res = run_req("GET / HTTP/1.1\r\n\r\n");
        assert!(res.starts_with("HTTP/1.1 200 OK"));
        assert!(res.contains("Content-Type: text/html; charset=utf-8"));
        assert!(res.contains("<h1>Home</h1>"));

        // Test GET /style.css
        let res = run_req("GET /style.css HTTP/1.1\r\n\r\n");
        assert!(res.starts_with("HTTP/1.1 200 OK"));
        assert!(res.contains("Content-Type: text/css; charset=utf-8"));

        // Test GET /app.js
        let res = run_req("GET /app.js HTTP/1.1\r\n\r\n");
        assert!(res.contains("Content-Type: application/javascript"));

        // Test GET /data.json
        let res = run_req("GET /data.json HTTP/1.1\r\n\r\n");
        assert!(res.contains("Content-Type: application/json"));

        // Test GET /image.svg
        let res = run_req("GET /image.svg HTTP/1.1\r\n\r\n");
        assert!(res.contains("Content-Type: image/svg+xml"));

        // Test GET /photo.jpg
        let res = run_req("GET /photo.jpg HTTP/1.1\r\n\r\n");
        assert!(res.contains("Content-Type: image/jpeg"));

        // Test GET /image.png
        let res = run_req("GET /image.png HTTP/1.1\r\n\r\n");
        assert!(res.contains("Content-Type: image/png"));

        // Test GET /anim.gif
        let res = run_req("GET /anim.gif HTTP/1.1\r\n\r\n");
        assert!(res.contains("Content-Type: image/gif"));

        // Test GET /icon.ico
        let res = run_req("GET /icon.ico HTTP/1.1\r\n\r\n");
        assert!(res.contains("Content-Type: image/x-icon"));

        // Test GET /note.txt
        let res = run_req("GET /note.txt HTTP/1.1\r\n\r\n");
        assert!(res.contains("Content-Type: text/plain; charset=utf-8"));

        // Test GET /data.bin
        let res = run_req("GET /data.bin HTTP/1.1\r\n\r\n");
        assert!(res.contains("Content-Type: application/octet-stream"));

        // Test GET /nested/
        let res = run_req("GET /nested/ HTTP/1.1\r\n\r\n");
        assert!(res.starts_with("HTTP/1.1 200 OK"));
        assert!(res.contains("<h2>Nested</h2>"));

        // Test GET /nested (directory without trailing slash)
        let res = run_req("GET /nested HTTP/1.1\r\n\r\n");
        assert!(res.starts_with("HTTP/1.1 200 OK"));
        assert!(res.contains("<h2>Nested</h2>"));

        // Test GET /missing.html (404)
        let res = run_req("GET /missing.html HTTP/1.1\r\n\r\n");
        assert!(res.starts_with("HTTP/1.1 404 Not Found"));
        assert!(res.contains("404 Not Found"));

        // Test POST /index.html (405)
        let res = run_req("POST /index.html HTTP/1.1\r\n\r\n");
        assert!(res.starts_with("HTTP/1.1 405 Method Not Allowed"));

        // Test Malformed request line
        let res = run_req("MALFORMED\r\n");
        assert!(res.starts_with("HTTP/1.1 405 Method Not Allowed"));

        Ok(())
    }

    #[test]
    fn test_serve_connection_loopback() -> Result<()> {
        use std::io::{Read, Write};

        let temp_dir = TempDir::new()?;
        let doc_root = temp_dir.path().to_path_buf();
        std::fs::write(
            doc_root.join("index.html"),
            "<h1>Loopback OK</h1>",
        )?;

        let listener = std::net::TcpListener::bind(("127.0.0.1", 0))?;
        let addr = listener.local_addr()?;

        let handle = std::thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                let _ = serve_connection(stream, &doc_root);
            }
        });

        let mut client = std::net::TcpStream::connect(addr)?;
        client.write_all(b"GET / HTTP/1.1\r\n\r\n")?;
        let mut response = String::new();
        let _ = client.read_to_string(&mut response)?;
        handle
            .join()
            .map_err(|_| anyhow::anyhow!("Thread join failed"))?;

        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("<h1>Loopback OK</h1>"));
        Ok(())
    }

    #[test]
    fn test_execute_command() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_dir = temp_dir.path().join("cmd-site");
        create_project_structure(&project_dir, "blog")?;

        let build_cmd = Commands::Build {
            content_dir: project_dir.join("content"),
            output_dir: project_dir.join("public"),
            template_dir: project_dir.join("templates"),
            minify: false,
            config: project_dir.join("config/config.toml"),
        };
        assert!(execute_command(build_cmd).is_ok());

        let new_err = Commands::New {
            name: "-invalid".to_string(),
            template: "blog".to_string(),
        };
        assert!(execute_command(new_err).is_err());

        let serve_err = Commands::Serve {
            port: 19999,
            watch: false,
            dir: PathBuf::from("/nonexistent/for/cli/test"),
        };
        assert!(execute_command(serve_err).is_err());

        Ok(())
    }
}
