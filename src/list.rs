use crate::trash::{self, TrashEntry};
use crate::ui::{self, FeedbackKind};
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
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Row, Table, TableState},
    Terminal,
};
use std::collections::HashSet;
use std::fs;
use std::io;
use std::time::{Duration, Instant};

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

    let mut restored_count = 0;
    let mut deleted_count = 0;

    let res = run_app(&mut terminal, &mut entries, &mut restored_count, &mut deleted_count);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if restored_count > 0 {
        println!("✔ Restored {} item(s)", restored_count);
    }
    if deleted_count > 0 {
        println!("🗑️ Permanently deleted {} item(s)", deleted_count);
    }

    res
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    entries: &mut Vec<TrashEntry>,
    restored_count: &mut usize,
    deleted_count: &mut usize,
) -> Result<()> {
    let mut state = TableState::default();
    if !entries.is_empty() {
        state.select(Some(0));
    }
    let mut filter = String::new();
    let mut searching = false;
    let mut selected_indices: HashSet<usize> = HashSet::new();
    let mut feedback: Option<(String, FeedbackKind, Instant)> = None;

    loop {
        if let Some((_, _, time)) = &feedback {
            if time.elapsed() >= Duration::from_secs(3) {
                feedback = None;
            }
        }

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

            let footer = if searching {
                let match_count = filtered_indices.len();
                let spans = vec![
                    Span::styled(" 🔍 Search: ", ui::style_header()),
                    Span::styled(
                        if filter.is_empty() {
                            "type to filter..."
                        } else {
                            &filter
                        },
                        ui::style_warn(),
                    ),
                    Span::styled(
                        format!(
                            " ({} match{}) ",
                            match_count,
                            if match_count == 1 { "" } else { "es" }
                        ),
                        ui::style_dimmed(),
                    ),
                    Span::styled("[Enter] Done | [Esc] Clear", ui::style_dimmed()),
                ];
                Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::ALL))
            } else {
                Paragraph::new(
                    " [Space/Enter] Toggle | [r] Restore | [d] Delete | [/] Filter | [a] All | [q] Quit",
                )
                .style(ui::style_dimmed())
                .block(Block::default().borders(Borders::ALL))
            };

            f.render_widget(footer, chunks[2]);

            if let Some((msg, kind, _)) = &feedback {
                let (border_color, text_style) = match kind {
                    FeedbackKind::Success => (
                        ui::COLOR_SUCCESS,
                        ui::style_success(),
                    ),
                    FeedbackKind::Warn => (
                        ui::COLOR_WARN,
                        ui::style_warn(),
                    ),
                    FeedbackKind::Alert => (
                        ui::COLOR_ALERT,
                        ui::style_alert(),
                    ),
                    FeedbackKind::Info => (
                        ui::COLOR_ACCENT,
                        ui::style_header(),
                    ),
                };

                let toast_width = (msg.chars().count() as u16 + 4)
                    .max(24)
                    .min(f.area().width.saturating_sub(4));
                let toast_height = 3;
                let toast_x = f.area().width.saturating_sub(toast_width + 2);
                let toast_y = chunks[0].y + chunks[0].height;

                let toast_area =
                    ratatui::layout::Rect::new(toast_x, toast_y, toast_width, toast_height);

                f.render_widget(Clear, toast_area);
                let toast_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(ratatui::style::Style::default().fg(border_color));
                let toast_p = Paragraph::new(format!(" {}", msg))
                    .style(text_style)
                    .block(toast_block);
                f.render_widget(toast_p, toast_area);
            }
        })?;

        if event::poll(Duration::from_millis(100))? {
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
                        feedback = None;
                    }
                    KeyCode::Char(' ') | KeyCode::Enter => {
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
                            feedback = Some((
                                "Deselected all items".to_string(),
                                FeedbackKind::Info,
                                Instant::now(),
                            ));
                        } else {
                            selected_indices = filtered_indices.iter().copied().collect();
                            feedback = Some((
                                format!("Selected all {} item(s)", selected_indices.len()),
                                FeedbackKind::Info,
                                Instant::now(),
                            ));
                        }
                    }
                    KeyCode::Char('r') => {
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

                        let mut restored_names = Vec::new();
                        let mut errors = Vec::new();

                        for idx in to_restore {
                            let entry = &entries[idx];
                            let name = entry
                                .original_path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| {
                                    entry.original_path.to_string_lossy().to_string()
                                });

                            match restore_entry(entry, true) {
                                Ok(_) => {
                                    *restored_count += 1;
                                    restored_names.push(name);
                                }
                                Err(e) => {
                                    errors.push(format!("{}: {}", name, e));
                                }
                            }
                        }

                        if !restored_names.is_empty() {
                            if restored_names.len() == 1 {
                                feedback = Some((
                                    format!("✔ Restored '{}'", restored_names[0]),
                                    FeedbackKind::Success,
                                    Instant::now(),
                                ));
                            } else {
                                feedback = Some((
                                    format!("✔ Restored {} items", restored_names.len()),
                                    FeedbackKind::Success,
                                    Instant::now(),
                                ));
                            }
                        } else if !errors.is_empty() {
                            feedback = Some((
                                format!("❌ Failed: {}", errors[0]),
                                FeedbackKind::Alert,
                                Instant::now(),
                            ));
                        }

                        *entries = trash::list_entries()?;
                        selected_indices.clear();
                        if entries.is_empty() {
                            break;
                        }
                        if let Some(i) = state.selected() {
                            state.select(Some(i.min(entries.len().saturating_sub(1))));
                        }
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

                        let mut deleted_names = Vec::new();
                        let mut errors = Vec::new();

                        for idx in to_delete {
                            let entry = &entries[idx];
                            let name = entry
                                .original_path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| {
                                    entry.original_path.to_string_lossy().to_string()
                                });

                            match delete_entry(entry) {
                                Ok(_) => {
                                    *deleted_count += 1;
                                    deleted_names.push(name);
                                }
                                Err(e) => {
                                    errors.push(format!("{}: {}", name, e));
                                }
                            }
                        }

                        if !deleted_names.is_empty() {
                            if deleted_names.len() == 1 {
                                feedback = Some((
                                    format!("🗑️ Deleted '{}'", deleted_names[0]),
                                    FeedbackKind::Warn,
                                    Instant::now(),
                                ));
                            } else {
                                feedback = Some((
                                    format!(
                                        "🗑️ Permanently deleted {} items",
                                        deleted_names.len()
                                    ),
                                    FeedbackKind::Warn,
                                    Instant::now(),
                                ));
                            }
                        } else if !errors.is_empty() {
                            feedback = Some((
                                format!("❌ Failed: {}", errors[0]),
                                FeedbackKind::Alert,
                                Instant::now(),
                            ));
                        }

                        *entries = trash::list_entries()?;
                        selected_indices.clear();
                        if entries.is_empty() {
                            break;
                        }
                        if let Some(i) = state.selected() {
                            state.select(Some(i.min(entries.len().saturating_sub(1))));
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let i = match state.selected() {
                            Some(i) => {
                                if filtered_indices.is_empty() || i >= filtered_indices.len() - 1 {
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
