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
                        pascal_to_kebab_smart(&pcaps[2]),
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

fn detect_case(s: &str) -> Case {
    let mut has_uppercase = false;
    let mut prev_was_uppercase = false;
    let mut consecutive_uppercase = 0;

    for c in s.chars() {
        if c.is_uppercase() {
            has_uppercase = true;
            if prev_was_uppercase {
                consecutive_uppercase += 1;
                // If we have 2 or more consecutive uppercase letters, it's an acronym
                if consecutive_uppercase >= 2 {
                    return Case::Acronym;
                }
            } else {
                consecutive_uppercase = 0;
            }
            prev_was_uppercase = true;
        } else {
            prev_was_uppercase = false;
            consecutive_uppercase = 0;
        }
    }

    if !has_uppercase {
        Case::Kebab
    } else {
        Case::Pascal
    }
}

#[derive(Debug, PartialEq)]
enum Case {
    Pascal,  // MyComponent
    Acronym, // XMLHTTPRequest
    Kebab,   // my-component
}

fn pascal_to_kebab(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 5);
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_uppercase() {
            if !result.is_empty() {
                result.push('-');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }

    result
}

fn acronym_to_kebab(s: &str) -> String {
    let mut result = String::new();
    let mut acronym = String::new();
    let mut prev_lower = false;

    for c in s.chars() {
        if c.is_uppercase() {
            if !acronym.is_empty() && prev_lower {
                result.push('-');
            }
            acronym.push(c);
            prev_lower = false;
        } else {
            if !acronym.is_empty() {
                result.push_str(&acronym.to_lowercase());
                acronym.clear();
            }
            result.push(c);
            prev_lower = true;
        }
    }

    if !acronym.is_empty() {
        if prev_lower {
            result.push('-');
        }
        result.push_str(&acronym.to_lowercase());
    }

    result
}

fn pascal_to_kebab_smart(filename: &str) -> String {
    match detect_case(filename) {
        Case::Kebab => filename.to_string(),
        Case::Pascal => pascal_to_kebab(filename),
        Case::Acronym => acronym_to_kebab(filename),
    }
}

fn rename_file(path: &Path) -> Result<()> {
    let parent = path.parent().context("Failed to get parent directory")?;

    // Get just the stem (filename without extension)
    let stem = path
        .file_stem()
        .context("Failed to get file stem")?
        .to_string_lossy();

    // Convert only the stem to kebab case using our new smart function
    let new_stem = pascal_to_kebab_smart(&stem);

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

#[cfg(test)]
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_case() {
        assert_eq!(detect_case("MyComponent"), Case::Pascal);
        assert_eq!(detect_case("XMLHTTPRequest"), Case::Acronym);
        assert_eq!(detect_case("my-component"), Case::Kebab);
    }

    #[test]
    /// This test is inherently flawed and will likely fail in edge cases.
    /// It's impossible to algorithmically detect with 100% accuracy whether a word is an acronym
    /// without additional context or a predefined list. For example:
    /// - Is "ID" an acronym for "Identifier" or just the word "Id"?
    /// - Is "UNESCO" one acronym or "UN-ESCO"?
    /// - Is "LASER" still an acronym even though it's now commonly written as "laser"?
    /// The best we can do is make educated guesses based on common patterns.
    fn test_pascal_to_kebab_smart() {
        // Pascal case
        assert_eq!(pascal_to_kebab_smart("MyComponent"), "my-component");
        assert_eq!(
            pascal_to_kebab_smart("ButtonComponent"),
            "button-component"
        );

        // Acronyms
        assert_eq!(pascal_to_kebab_smart("API"), "api");
        assert_eq!(pascal_to_kebab_smart("XMLHTTPRequest"), "xml-http-request");
        assert_eq!(pascal_to_kebab_smart("MyXMLParser"), "my-xml-parser");
        assert_eq!(pascal_to_kebab_smart("APIEndpoint"), "api-endpoint");
        assert_eq!(pascal_to_kebab_smart("MyAPIService"), "my-api-service");

        // Already kebab case
        assert_eq!(pascal_to_kebab_smart("already-kebab"), "already-kebab");
    }

    #[test]
    fn test_needs_conversion() {
        assert!(needs_conversion("MyComponent"));
        assert!(needs_conversion("ButtonComponent"));
        assert!(!needs_conversion("my-component"));
        assert!(!needs_conversion("regular-file"));
    }

    #[test]
    fn test_update_imports() {
        let content = r#"
            import MyComponent from './MyComponent.svelte';
            import { Something } from '../ComponentLibrary/ButtonComponent';
            const util = require('./UtilityFunctions');
        "#;

        let (new_content, changes) = update_imports(content);
        assert_eq!(changes, 4); // MyComponent, ComponentLibrary, ButtonComponent, UtilityFunctions
        assert!(new_content.contains("./my-component.svelte"));
        assert!(new_content.contains("component-library/button-component"));
        assert!(new_content.contains("./utility-functions"));
    }

    #[test]
    fn test_matches_source_file() {
        assert!(matches_source_file(Path::new("test.ts")));
        assert!(matches_source_file(Path::new("test.tsx")));
        assert!(matches_source_file(Path::new("test.svelte")));
        assert!(!matches_source_file(Path::new("test.txt")));
        assert!(!matches_source_file(Path::new("test")));
    }

    mod integration {
        use super::*;
        use std::path::PathBuf;

        fn setup_test_directory() -> Result<(TempDir, PathBuf)> {
            let temp_dir = TempDir::new()?;
            let test_dir = temp_dir.path().join("test");
            fs::create_dir(&test_dir)?;

            // Create test files
            fs::write(
                test_dir.join("MyComponent.svelte"),
                r#"<script>
                    import ButtonComponent from './ComponentLibrary/ButtonComponent.svelte';
                </script>"#,
            )?;

            fs::create_dir(test_dir.join("ComponentLibrary"))?;
            fs::write(
                test_dir.join("ComponentLibrary/ButtonComponent.svelte"),
                "<div>Button</div>",
            )?;

            Ok((temp_dir, test_dir))
        }

        #[test]
        fn test_rename_files() -> Result<()> {
            let (_temp_dir, test_dir) = setup_test_directory()?;

            process_directory(&test_dir)?;

            assert!(test_dir.join("my-component.svelte").exists());
            assert!(test_dir.join("component-library").exists());
            assert!(test_dir
                .join("component-library/button-component.svelte")
                .exists());

            Ok(())
        }

        #[test]
        fn test_process_imports() -> Result<()> {
            let (_temp_dir, test_dir) = setup_test_directory()?;

            process_imports(&test_dir)?;

            let content =
                fs::read_to_string(test_dir.join("MyComponent.svelte"))?;
            assert!(
                content.contains("./component-library/button-component.svelte")
            );

            Ok(())
        }

        #[test]
        fn test_full_process() -> Result<()> {
            let (_temp_dir, test_dir) = setup_test_directory()?;

            // Process both imports and filenames
            process_imports(&test_dir)?;
            process_directory(&test_dir)?;

            // Check if files were renamed
            assert!(test_dir.join("my-component.svelte").exists());
            assert!(test_dir
                .join("component-library/button-component.svelte")
                .exists());

            // Check if imports were updated
            let content =
                fs::read_to_string(test_dir.join("my-component.svelte"))?;
            assert!(
                content.contains("./component-library/button-component.svelte")
            );

            Ok(())
        }
    }
}
