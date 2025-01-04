use anyhow::{Context, Result};
use clap::Parser;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// CLI tool to convert PascalCase filenames to kebab-case
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// The directory path to process
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Process import statements in files
    #[arg(long, short = 'i')]
    imports: bool,

    /// Process both filenames and imports
    #[arg(long, short = 'a', conflicts_with = "imports")]
    all: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Process imports first to ensure paths are still valid
    if args.all || args.imports {
        process_imports(&args.path)?;
    }

    // Then rename files and directories
    if args.all || !args.imports {
        process_directory(&args.path)?;
    }

    Ok(())
}

fn process_directory(dir: &Path) -> Result<()> {
    // Collect paths first to avoid renaming issues during iteration
    let entries: Vec<_> = WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();

    // First, process files (top-down)
    for entry in entries.iter() {
        if entry.file_type().is_file() {
            if let Some(filename) = entry.file_name().to_str() {
                if needs_conversion(filename) {
                    rename_file(entry.path())?;
                }
            }
        }
    }

    // Then process directories (bottom-up)
    for entry in entries.iter().rev() {
        if entry.file_type().is_dir() {
            if let Some(dirname) = entry.file_name().to_str() {
                if needs_conversion(dirname) {
                    rename_file(entry.path())?;
                }
            }
        }
    }
    Ok(())
}

fn process_imports(dir: &Path) -> Result<()> {
    let entries: Vec<_> = WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && matches_source_file(e.path()))
        .collect();

    for entry in entries {
        process_file_imports(entry.path())?;
    }
    Ok(())
}

fn matches_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("js" | "jsx" | "ts" | "tsx" | "svelte" | "vue")
    )
}

fn process_file_imports(path: &Path) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let (new_content, changes) = update_imports(&content);

    if changes > 0 {
        println!("Updated {} imports in: {}", changes, path.display());
        fs::write(path, new_content)?;
    }

    Ok(())
}

fn update_imports(content: &str) -> (String, usize) {
    let mut changes = 0;

    // Match both ES6 imports and requires
    let import_regex = Regex::new(
        r#"(?x)
        (import\s+[^"']*?from\s*["']|require\(["'])  # import/require start
        ([^"']+)                                     # path capture
        (["'][\);]?)                                 # closing quote/paren
    "#,
    )
    .unwrap();

    // Updated regex to handle both path segments and filenames
    let path_regex =
        Regex::new(r"(^|[\\/])([A-Z][a-zA-Z0-9]+)([\\/]|\.[\w]+$|$)").unwrap();

    let result = import_regex.replace_all(content, |caps: &regex::Captures| {
        let prefix = &caps[1];
        let path = &caps[2];
        let suffix = &caps[3];

        // Keep replacing until no more changes are made
        let mut current_path = path.to_string();
        loop {
            let new_path = path_regex
                .replace_all(&current_path, |pcaps: &regex::Captures| {
                    changes += 1;
                    format!(
                        "{}{}{}",
                        &pcaps[1], // separator or start
                        pascal_to_kebab(&pcaps[2]),
                        &pcaps[3] // separator or extension
                    )
                })
                .to_string();

            if new_path == current_path {
                break;
            }
            current_path = new_path;
        }

        format!("{}{}{}", prefix, current_path, suffix)
    });

    (result.to_string(), changes)
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
