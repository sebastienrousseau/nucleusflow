# Migrating from Zola to NucleusFlow

A practical guide for migrating static sites from Zola to NucleusFlow.

## Key Differences

| Feature | Zola | NucleusFlow |
| :--- | :--- | :--- |
| **Language** | Rust | Rust |
| **Safety Invariant** | Standard safe Rust | Strict `#![forbid(unsafe_code)]` at all roots |
| **Templating** | Tera (Jinja2-like) | Handlebars |
| **Frontmatter** | TOML | YAML, JSON, TOML |
| **Security Auditing** | Manual | Formal property-based testing and supply-chain gates |

## Migration Steps

### 1. Template Syntax Mapping

Convert Tera tags to Handlebars:

- Variable output: `{{ page.title }}` -> `{{title}}`
- Raw HTML output: `{{ page.content | safe }}` -> `{{{content}}}`
- Loops: `{% for item in items %}...{% endfor %}` -> `{{#each items}}...{{/each}}`
- Conditionals: `{% if condition %}...{% endif %}` -> `{{#if condition}}...{{/if}}`

### 2. Configuration Setup

Translate `config.toml`:

```toml
# nucleusflow.toml
[site]
title = "My Migrated Zola Site"
description = "High-performance static site"
base_url = "https://example.com"

[build]
content_dir = "content"
output_dir = "public"
template_dir = "templates"
minify = true
```

### 3. Build and Test

Execute a clean build:

```bash
nucleusflow build --minify
nucleusflow serve
```
