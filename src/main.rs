mod empty;
mod list;
mod remove;
mod restore;
mod trash;
mod ui;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "toss",
    version,
    about = "A fast, beautiful TUI trash manager",
    long_about = "toss — FreeDesktop.org compliant trash CLI with a beautiful TUI.\nPut, list, restore, empty, or remove trashed files."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Move files or directories to the trash
    Put {
        #[arg(required = true)]
        files: Vec<String>,
    },
    /// Browse trashed files in a TUI
    List,
    /// Interactively restore trashed files via TUI
    Restore {
        /// Overwrite existing files at original path
        #[arg(long)]
        overwrite: bool,
    },
    /// Empty the trash (optionally only files older than N days)
    Empty {
        /// Delete files older than this many days
        days: Option<u64>,
    },
    /// Remove trashed files matching a glob pattern
    Rm {
        /// Glob pattern, e.g. "*.log"
        pattern: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Put { files }) => trash::put(&files),
        Some(Command::List) => list::run(),
        Some(Command::Restore { overwrite }) => restore::run(overwrite),
        Some(Command::Empty { days }) => empty::run(days),
        Some(Command::Rm { pattern }) => remove::run(&pattern),
        None => list::run(),
    }
}
