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

pub fn run() -> Result<()> {
    let mut entries = trash::list_entries()?;

    if entries.is_empty() {
        println!("Trash is empty 🗑️");
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, &mut entries);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    entries: &mut Vec<TrashEntry>,
) -> Result<()> {
    let mut state = TableState::default();
    if !entries.is_empty() {
        state.select(Some(0));
    }
    let mut filter = String::new();
    let mut searching = false;
    let mut selected_indices: HashSet<usize> = HashSet::new();

    loop {
        let filtered_indices: Vec<usize> = entries
            .iter()
            .enumerate()
            .filter_map(|(idx, e)| {
                if filter.is_empty()
                    || e.original_path
                        .to_string_lossy()
                        .to_lowercase()
                        .contains(&filter.to_lowercase())
                {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(5),
                    Constraint::Length(3),
                ])
                .split(f.area());

            let total_size: u64 = filtered_indices.iter().map(|&idx| entries[idx].size).sum();
            let header_text = format!(
                " 🗑️  TOSS — Trashed Items ({}) | Selected: {} | Total Size: {}",
                filtered_indices.len(),
                selected_indices.len(),
                format_size(total_size, BINARY)
            );

            let header = Paragraph::new(header_text)
                .style(ui::style_header())
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            let rows: Vec<Row> = filtered_indices
                .iter()
                .map(|&idx| {
                    let e = &entries[idx];
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

            let status_text = if searching {
                format!(" Search: {} (Press Enter to confirm, Esc to clear)", filter)
            } else {
                format!(
                    " [Space] Toggle | [r] Restore | [d] Delete | [/] Filter | [a] All | [q] Quit"
                )
            };

            let footer = Paragraph::new(status_text)
                .style(ui::style_dimmed())
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            if searching {
                match key.code {
                    KeyCode::Esc => {
                        filter.clear();
                        searching = false;
                    }
                    KeyCode::Enter => {
                        searching = false;
                    }
                    KeyCode::Backspace => {
                        filter.pop();
                    }
                    KeyCode::Char(c) => {
                        filter.push(c);
                    }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Char('/') => {
                        searching = true;
                    }
                    KeyCode::Char(' ') => {
                        if let Some(i) = state.selected() {
                            if i < filtered_indices.len() {
                                let real_idx = filtered_indices[i];
                                if selected_indices.contains(&real_idx) {
                                    selected_indices.remove(&real_idx);
                                } else {
                                    selected_indices.insert(real_idx);
                                }
                            }
                        }
                    }
                    KeyCode::Char('a') => {
                        if selected_indices.len() == filtered_indices.len() {
                            selected_indices.clear();
                        } else {
                            selected_indices = filtered_indices.iter().copied().collect();
                        }
                    }
                    KeyCode::Char('r') | KeyCode::Enter => {
                        let to_restore: Vec<usize> = if !selected_indices.is_empty() {
                            selected_indices.iter().copied().collect()
                        } else if let Some(i) = state.selected() {
                            if i < filtered_indices.len() {
                                vec![filtered_indices[i]]
                            } else {
                                vec![]
                            }
                        } else {
                            vec![]
                        };

                        for idx in to_restore {
                            let entry = &entries[idx];
                            restore_entry(entry, true)?;
                        }

                        *entries = trash::list_entries()?;
                        selected_indices.clear();
                        if entries.is_empty() {
                            break;
                        }
                        state.select(Some(0));
                    }
                    KeyCode::Char('d') | KeyCode::Delete => {
                        let to_delete: Vec<usize> = if !selected_indices.is_empty() {
                            selected_indices.iter().copied().collect()
                        } else if let Some(i) = state.selected() {
                            if i < filtered_indices.len() {
                                vec![filtered_indices[i]]
                            } else {
                                vec![]
                            }
                        } else {
                            vec![]
                        };

                        for idx in to_delete {
                            let entry = &entries[idx];
                            delete_entry(entry)?;
                        }

                        *entries = trash::list_entries()?;
                        selected_indices.clear();
                        if entries.is_empty() {
                            break;
                        }
                        state.select(Some(0));
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let i = match state.selected() {
                            Some(i) => {
                                if filtered_indices.is_empty() {
                                    0
                                } else if i >= filtered_indices.len() - 1 {
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
                                if filtered_indices.is_empty() {
                                    0
                                } else if i == 0 {
                                    filtered_indices.len().saturating_sub(1)
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

    Ok(())
}

fn restore_entry(entry: &TrashEntry, overwrite: bool) -> Result<()> {
    let target = &entry.original_path;

    if target.exists() && !overwrite {
        return Ok(());
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::rename(&entry.trash_path, target)
        .with_context(|| format!("Failed to restore to {}", target.display()))?;

    let _ = fs::remove_file(&entry.info_path);
    Ok(())
}

fn delete_entry(entry: &TrashEntry) -> Result<()> {
    if entry.trash_path.exists() {
        if entry.trash_path.is_dir() {
            let _ = fs::remove_dir_all(&entry.trash_path);
        } else {
            let _ = fs::remove_file(&entry.trash_path);
        }
    }
    let _ = fs::remove_file(&entry.info_path);
    Ok(())
}
