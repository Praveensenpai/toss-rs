use crate::trash::{self, TrashEntry};
use crate::ui;
use anyhow::Result;
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
use std::io;

pub fn run() -> Result<()> {
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

    let res = run_app(&mut terminal, entries);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    entries: Vec<TrashEntry>,
) -> Result<()> {
    let mut state = TableState::default();
    if !entries.is_empty() {
        state.select(Some(0));
    }
    let mut filter = String::new();
    let mut searching = false;

    loop {
        let filtered_entries: Vec<&TrashEntry> = entries
            .iter()
            .filter(|e| {
                if filter.is_empty() {
                    true
                } else {
                    e.original_path
                        .to_string_lossy()
                        .to_lowercase()
                        .contains(&filter.to_lowercase())
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

            let total_size: u64 = filtered_entries.iter().map(|e| e.size).sum();
            let header_text = format!(
                " 🗑️  TOSS — Trashed Items ({}) | Total Size: {}",
                filtered_entries.len(),
                format_size(total_size, BINARY)
            );

            let header = Paragraph::new(header_text)
                .style(ui::style_header())
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            let rows: Vec<Row> = filtered_entries
                .iter()
                .map(|e| {
                    Row::new(vec![
                        e.deleted_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                        format_size(e.size, BINARY),
                        e.original_path.to_string_lossy().to_string(),
                    ])
                })
                .collect();

            let table = Table::new(
                rows,
                [
                    Constraint::Length(20),
                    Constraint::Length(12),
                    Constraint::Min(30),
                ],
            )
            .header(
                Row::new(vec!["Date Trashed", "Size", "Original Path"])
                    .style(ui::style_header()),
            )
            .block(Block::default().borders(Borders::ALL))
            .row_highlight_style(ui::style_selected());

            f.render_stateful_widget(table, chunks[1], &mut state);

            let status_text = if searching {
                format!(" Search: {} (Press Enter to confirm, Esc to clear)", filter)
            } else {
                format!(
                    " Press [/] Search | [q] Quit | [j/k] Navigate | Filter: '{}'",
                    if filter.is_empty() { "None" } else { &filter }
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
                    KeyCode::Down | KeyCode::Char('j') => {
                        let i = match state.selected() {
                            Some(i) => {
                                if filtered_entries.is_empty() {
                                    0
                                } else if i >= filtered_entries.len() - 1 {
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
                                if filtered_entries.is_empty() {
                                    0
                                } else if i == 0 {
                                    filtered_entries.len().saturating_sub(1)
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
