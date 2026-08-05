use crate::trash::{self, TrashEntry};
use crate::ui;
use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use humansize::{format_size, BINARY};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph, Row, Table, TableState},
    Terminal,
};
use std::collections::HashSet;
use std::fs;
use std::io;

pub fn run(overwrite: bool) -> Result<()> {
    let entries = trash::list_entries()?;

    if entries.is_empty() {
        println!("Trash is empty 🗑️");
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let selected_indices = run_restore_tui(&mut terminal, &entries)?;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if selected_indices.is_empty() {
        println!("No files selected for restoration.");
        return Ok(());
    }

    for idx in selected_indices {
        let entry = &entries[idx];
        restore_entry(entry, overwrite)?;
    }

    Ok(())
}

fn restore_entry(entry: &TrashEntry, overwrite: bool) -> Result<()> {
    let target = &entry.original_path;

    if target.exists() && !overwrite {
        println!("Skipped (already exists): {}", target.display());
        return Ok(());
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::rename(&entry.trash_path, target)
        .with_context(|| format!("Failed to restore to {}", target.display()))?;

    let _ = fs::remove_file(&entry.info_path);

    println!("Restored: {}", target.display());
    Ok(())
}

fn run_restore_tui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    entries: &[TrashEntry],
) -> Result<HashSet<usize>> {
    let mut state = TableState::default();
    if !entries.is_empty() {
        state.select(Some(0));
    }
    let mut selected_indices: HashSet<usize> = HashSet::new();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(5),
                    Constraint::Length(3),
                ])
                .split(f.area());

            let header_text = format!(
                " ♻️  RESTORE — Select items to restore ({}/{} selected)",
                selected_indices.len(),
                entries.len()
            );

            let header = Paragraph::new(header_text)
                .style(ui::style_header())
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            let rows: Vec<Row> = entries
                .iter()
                .enumerate()
                .map(|(idx, e)| {
                    let mark = if selected_indices.contains(&idx) {
                        "[✓]"
                    } else {
                        "[ ]"
                    };
                    Row::new(vec![
                        mark.to_string(),
                        e.deleted_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                        format_size(e.size, BINARY),
                        e.original_path.to_string_lossy().to_string(),
                    ])
                })
                .collect();

            let table = Table::new(
                rows,
                [
                    Constraint::Length(5),
                    Constraint::Length(20),
                    Constraint::Length(12),
                    Constraint::Min(30),
                ],
            )
            .header(
                Row::new(vec!["Sel", "Date Trashed", "Size", "Original Path"])
                    .style(ui::style_header()),
            )
            .block(Block::default().borders(Borders::ALL))
            .row_highlight_style(ui::style_selected());

            f.render_stateful_widget(table, chunks[1], &mut state);

            let footer = Paragraph::new(
                " [Space] Toggle | [a] Select All | [Enter] Restore Selected | [q] Cancel ",
            )
            .style(ui::style_dimmed())
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(HashSet::new()),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(HashSet::new())
                }
                KeyCode::Enter => return Ok(selected_indices),
                KeyCode::Char(' ') => {
                    if let Some(i) = state.selected() {
                        if selected_indices.contains(&i) {
                            selected_indices.remove(&i);
                        } else {
                            selected_indices.insert(i);
                        }
                    }
                }
                KeyCode::Char('a') => {
                    if selected_indices.len() == entries.len() {
                        selected_indices.clear();
                    } else {
                        selected_indices = (0..entries.len()).collect();
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let i = match state.selected() {
                        Some(i) => {
                            if entries.is_empty() {
                                0
                            } else if i >= entries.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    state.select(Some(i));
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let i = match state.selected() {
                        Some(i) => {
                            if entries.is_empty() {
                                0
                            } else if i == 0 {
                                entries.len().saturating_sub(1)
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    state.select(Some(i));
                }
                _ => {}
            }
        }
    }
}
