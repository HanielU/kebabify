use anyhow::{Context, Result};
use clap::Parser;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// CLI tool to convert PascalCase filenames to kebab-case
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// The directory path to process
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    process_directory(&args.path)?;
    Ok(())
}

fn process_directory(dir: &Path) -> Result<()> {
    // Collect paths first to avoid renaming issues during iteration
    let entries: Vec<_> = WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();

    // Process directories first (bottom-up) to handle nested paths correctly
    for entry in entries.iter().rev() {
        if entry.file_type().is_dir() {
            if let Some(dirname) = entry.file_name().to_str() {
                if needs_conversion(dirname) {
                    rename_file(entry.path())?;
                }
            }
        }
    }

    // Then process files
    for entry in entries.iter() {
        if entry.file_type().is_file() {
            if let Some(filename) = entry.file_name().to_str() {
                if needs_conversion(filename) {
                    rename_file(entry.path())?;
                }
            }
        }
    }
    Ok(())
}

fn needs_conversion(filename: &str) -> bool {
    // Check if the filename contains uppercase letters
    filename.chars().any(|c| c.is_uppercase())
}

fn pascal_to_kebab(filename: &str) -> String {
    let mut result = String::with_capacity(filename.len() + 5);
    let mut chars = filename.chars().peekable();

    while let Some(current) = chars.next() {
        if current.is_uppercase() {
            if !result.is_empty() {
                result.push('-');
            }
            result.push(current.to_lowercase().next().unwrap());
        } else {
            result.push(current);
        }
    }

    result
}

fn rename_file(path: &Path) -> Result<()> {
    let parent = path.parent().context("Failed to get parent directory")?;

    // Get just the stem (filename without extension)
    let stem = path
        .file_stem()
        .context("Failed to get file stem")?
        .to_string_lossy();

    // Convert only the stem to kebab case
    let new_stem = pascal_to_kebab(&stem);

    // Create new filename with original extension
    let new_filename = if let Some(ext) = path.extension() {
        format!("{}.{}", new_stem, ext.to_string_lossy())
    } else {
        new_stem
    };

    let new_path = parent.join(new_filename);

    println!(
        "Renaming: {} -> {}",
        path.display(),
        new_path.file_name().unwrap().to_string_lossy()
    );

    std::fs::rename(path, new_path).with_context(|| {
        format!("Failed to rename file: {}", path.display())
    })?;

    Ok(())
}
