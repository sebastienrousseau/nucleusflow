# Migrating from Hugo to NucleusFlow

A practical guide for migrating static websites from Hugo to NucleusFlow.

## Key Differences

| Feature | Hugo | NucleusFlow |
| :--- | :--- | :--- |
| **Language** | Go | Rust |
| **Safety Guarantees** | Runtime memory managed | Compiler-enforced `#![forbid(unsafe_code)]` |
| **Templating Engine** | Go templates (`text/template`) | Handlebars (`{{{content}}}`) |
| **Frontmatter** | TOML, YAML, JSON | YAML (via `noyalib`), JSON, TOML |
| **Asset Pipeline** | Hugo Pipes | Embedded minification and asset processing |
| **Binary Size** | ~60MB | ~10MB stripped |

## Migration Steps

### 1. Directory Structure

Reorganize your site directories according to NucleusFlow conventions:

```text
hugo-site/                     nucleusflow-site/
├── content/           -->     ├── content/
├── layouts/           -->     ├── templates/
├── static/            -->     ├── public/assets/
└── config.toml        -->     └── nucleusflow.toml
```

### 2. Configuration File

Convert your Hugo `config.toml` to `nucleusflow.toml`:

```toml
# nucleusflow.toml
[site]
title = "My Migrated Site"
description = "Fast static site built with NucleusFlow"
base_url = "https://example.com"
author = "Site Author"

[build]
content_dir = "content"
output_dir = "public"
template_dir = "templates"
minify = true
```

### 3. Template Syntax Adaptation

Convert Go template constructs to Handlebars syntax:

- Page title: `{{ .Title }}` -> `{{title}}`
- Page content: `{{ .Content }}` -> `{{{content}}}`
- Site title: `{{ .Site.Title }}` -> `{{site_title}}`
- Conditionals: `{{ if .Params.featured }}...{{ end }}` -> `{{#if featured}}...{{/if}}`

### 4. Build and Preview

Verify the migrated build locally:

```bash
nucleusflow build --minify
nucleusflow serve --port 3000
```
