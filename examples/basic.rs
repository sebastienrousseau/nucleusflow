//! Basic example of using NucleusFlow as an embedded static site library.

use nucleusflow::{
    FileContentProcessor, HtmlOutputGenerator, HtmlTemplateRenderer,
    NucleusFlow, NucleusFlowConfig,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("NucleusFlow basic pipeline example");

    let temp_dir = tempfile::tempdir()?;
    let content_dir = temp_dir.path().join("content");
    let output_dir = temp_dir.path().join("public");
    let template_dir = temp_dir.path().join("templates");

    std::fs::create_dir_all(&content_dir)?;
    std::fs::create_dir_all(&template_dir)?;
    std::fs::create_dir_all(&output_dir)?;

    std::fs::write(
        content_dir.join("index.md"),
        "---\ntitle: Home\n---\n# Welcome to NucleusFlow\nFast and secure static site generation.",
    )?;

    std::fs::write(
        template_dir.join("default.html"),
        "<!DOCTYPE html><html><head><title>{{title}}</title></head><body>{{{content}}}</body></html>",
    )?;

    let config = NucleusFlowConfig::new(
        &content_dir,
        &output_dir,
        &template_dir,
    )?;

    let content_processor = FileContentProcessor::new(content_dir);
    let template_renderer = HtmlTemplateRenderer::new(template_dir);
    let output_generator = HtmlOutputGenerator::new(output_dir);

    let flow = NucleusFlow::new(
        config,
        Box::new(content_processor),
        Box::new(template_renderer),
        Box::new(output_generator),
    );

    flow.process()?;
    println!("Pipeline executed successfully!");

    Ok(())
}
