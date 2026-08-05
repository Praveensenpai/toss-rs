use crate::trash;
use anyhow::Result;
use chrono::{Duration, Local};
use std::fs;

pub fn run(days: Option<u64>) -> Result<()> {
    let entries = trash::list_entries()?;

    if entries.is_empty() {
        println!("Trash is already empty.");
        return Ok(());
    }

    let cutoff = days.map(|d| Local::now() - Duration::days(d as i64));
    let mut removed_count = 0;

    for entry in entries {
        if let Some(cutoff_date) = cutoff {
            if entry.deleted_at > cutoff_date {
                continue;
            }
        }

        if entry.trash_path.exists() {
            if entry.trash_path.is_dir() {
                let _ = fs::remove_dir_all(&entry.trash_path);
            } else {
                let _ = fs::remove_file(&entry.trash_path);
            }
        }
        let _ = fs::remove_file(&entry.info_path);
        removed_count += 1;
    }

    if let Some(d) = days {
        println!("Emptied {} item(s) older than {} day(s).", removed_count, d);
    } else {
        println!("Emptied trash ({} item(s) removed).", removed_count);
    }

    Ok(())
}
