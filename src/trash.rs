use anyhow::{bail, Context, Result};
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};

/// Returns the XDG Trash directory: ~/.local/share/Trash
pub fn trash_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not resolve home directory")?;
    Ok(home.join(".local/share/Trash"))
}

pub fn files_dir() -> Result<PathBuf> {
    Ok(trash_dir()?.join("files"))
}

pub fn info_dir() -> Result<PathBuf> {
    Ok(trash_dir()?.join("info"))
}

/// Represents a single entry in the trash
#[derive(Debug, Clone)]
pub struct TrashEntry {
    pub name: String,
    pub original_path: PathBuf,
    pub deleted_at: chrono::DateTime<Local>,
    pub trash_path: PathBuf,
    pub info_path: PathBuf,
    pub size: u64,
}

/// Move each path into the FreeDesktop trash
pub fn put(files: &[String]) -> Result<()> {
    let files_dir = files_dir()?;
    let info_dir = info_dir()?;
    fs::create_dir_all(&files_dir)?;
    fs::create_dir_all(&info_dir)?;

    for raw in files {
        let src = PathBuf::from(raw)
            .canonicalize()
            .with_context(|| format!("'{}' not found", raw))?;

        if !src.exists() {
            bail!("'{}' does not exist", src.display());
        }

        let stem = src
            .file_name()
            .context("Invalid file name")?
            .to_string_lossy()
            .to_string();

        let dest_name = unique_name(&files_dir, &stem);
        let dest = files_dir.join(&dest_name);
        let info_file = info_dir.join(format!("{}.trashinfo", dest_name));

        fs::rename(&src, &dest)
            .with_context(|| format!("Failed to move '{}' to trash", src.display()))?;

        write_trashinfo(&info_file, &src)?;

        eprintln!("Trashed: {}", src.display());
    }
    Ok(())
}

fn unique_name(dir: &Path, stem: &str) -> String {
    let mut name = stem.to_string();
    let mut counter = 2;
    while dir.join(&name).exists() {
        name = format!("{}_{}", stem, counter);
        counter += 1;
    }
    name
}

fn write_trashinfo(info_path: &Path, original: &Path) -> Result<()> {
    let now = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let content = format!(
        "[Trash Info]\nPath={}\nDeletionDate={}\n",
        original.display(),
        now
    );
    fs::write(info_path, content)?;
    Ok(())
}

/// Parse all .trashinfo files and return TrashEntry list
pub fn list_entries() -> Result<Vec<TrashEntry>> {
    let info_dir = info_dir()?;
    let files_dir = files_dir()?;

    if !info_dir.exists() {
        return Ok(vec![]);
    }

    let mut entries = Vec::new();

    for entry in fs::read_dir(&info_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("trashinfo") {
            continue;
        }

        if let Ok(parsed) = parse_trashinfo(&path, &files_dir) {
            entries.push(parsed);
        }
    }

    entries.sort_by(|a, b| b.deleted_at.cmp(&a.deleted_at));
    Ok(entries)
}

fn parse_trashinfo(info_path: &Path, files_dir: &Path) -> Result<TrashEntry> {
    let content = fs::read_to_string(info_path)?;
    let mut original_path = None;
    let mut deleted_at = None;

    for line in content.lines() {
        if let Some(val) = line.strip_prefix("Path=") {
            original_path = Some(PathBuf::from(val));
        }
        if let Some(val) = line.strip_prefix("DeletionDate=") {
            deleted_at = chrono::NaiveDateTime::parse_from_str(val, "%Y-%m-%dT%H:%M:%S")
                .ok()
                .map(|dt| dt.and_local_timezone(Local).single())
                .flatten();
        }
    }

    let original_path = original_path.context("Missing Path in trashinfo")?;
    let deleted_at = deleted_at.unwrap_or_else(|| Local::now());

    let stem = info_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let trash_path = files_dir.join(&stem);
    let size = entry_size(&trash_path);

    Ok(TrashEntry {
        name: stem,
        original_path,
        deleted_at,
        trash_path,
        info_path: info_path.to_path_buf(),
        size,
    })
}

fn entry_size(path: &Path) -> u64 {
    if path.is_file() {
        fs::metadata(path).map(|m| m.len()).unwrap_or(0)
    } else if path.is_dir() {
        dir_size(path)
    } else {
        0
    }
}

fn dir_size(path: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    entries
        .filter_map(|e| e.ok())
        .map(|e| entry_size(&e.path()))
        .sum()
}
