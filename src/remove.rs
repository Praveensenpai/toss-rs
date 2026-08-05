use crate::trash;
use anyhow::Result;
use glob::Pattern;
use std::fs;

pub fn run(pattern_str: &str) -> Result<()> {
    let pattern = Pattern::new(pattern_str)?;
    let entries = trash::list_entries()?;

    let mut removed = 0;

    for entry in entries {
        let original_name = entry
            .original_path
            .file_name()
            .map(|f| f.to_string_lossy())
            .unwrap_or_default();

        if pattern.matches(&original_name)
            || pattern.matches(&entry.name)
            || pattern.matches(&entry.original_path.to_string_lossy())
        {
            if entry.trash_path.exists() {
                if entry.trash_path.is_dir() {
                    let _ = fs::remove_dir_all(&entry.trash_path);
                } else {
                    let _ = fs::remove_file(&entry.trash_path);
                }
            }
            let _ = fs::remove_file(&entry.info_path);
            println!("Removed: {}", entry.original_path.display());
            removed += 1;
        }
    }

    if removed == 0 {
        println!("No trashed items matched pattern '{}'", pattern_str);
    } else {
        println!("Successfully removed {} item(s).", removed);
    }

    Ok(())
}
