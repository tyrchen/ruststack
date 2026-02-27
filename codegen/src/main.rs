//! S3 model code generator.
//!
//! Reads the AWS S3 Smithy JSON AST model and generates Rust source files
//! for the `ruststack-s3-model` crate.

mod codegen;
mod model;
mod shapes;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let model_path = args
        .get(1)
        .map_or_else(|| PathBuf::from("smithy-model/s3.json"), PathBuf::from);

    let output_dir = args.get(2).map_or_else(
        || PathBuf::from("../crates/ruststack-s3-model/src"),
        PathBuf::from,
    );

    eprintln!("Reading Smithy model from: {}", model_path.display());
    eprintln!("Writing output to: {}", output_dir.display());

    // Read and parse the Smithy JSON model.
    let model_json = fs::read_to_string(&model_path)
        .with_context(|| format!("Failed to read model file: {}", model_path.display()))?;

    let smithy_model: model::SmithyModel =
        serde_json::from_str(&model_json).context("Failed to parse Smithy JSON model")?;

    eprintln!("Parsed model: {} shapes", smithy_model.shapes.len());

    // Resolve shapes and types.
    let resolved =
        shapes::resolve_model(&smithy_model).context("Failed to resolve model shapes")?;

    eprintln!(
        "Resolved: {} operations, {} enums, {} shared structs, {} input structs, {} output structs",
        resolved.operations.len(),
        resolved.enums.len(),
        resolved.shared_structs.len(),
        resolved.input_structs.len(),
        resolved.output_structs.len(),
    );

    // Generate code.
    let files = codegen::generate_all(&resolved).context("Failed to generate code")?;

    // Write output files.
    for (rel_path, content) in &files {
        let full_path = output_dir.join(rel_path);
        ensure_parent_dir(&full_path)?;
        fs::write(&full_path, content)
            .with_context(|| format!("Failed to write {}", full_path.display()))?;
        eprintln!("  Wrote: {}", full_path.display());
    }

    eprintln!("Code generation complete. {} files written.", files.len());

    Ok(())
}

/// Ensure the parent directory of a path exists.
fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
    Ok(())
}
