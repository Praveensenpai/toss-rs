use ratatui::style::{Color, Modifier, Style};

pub const COLOR_ACCENT: Color = Color::Cyan;
pub const COLOR_MUTED: Color = Color::DarkGray;
pub const COLOR_SUCCESS: Color = Color::Green;
pub const COLOR_WARN: Color = Color::Yellow;
pub const COLOR_ALERT: Color = Color::Red;

#[derive(Clone, Copy, Debug)]
pub enum FeedbackKind {
    Success,
    Warn,
    Alert,
    Info,
}

pub fn style_header() -> Style {
    Style::default()
        .fg(COLOR_ACCENT)
        .add_modifier(Modifier::BOLD)
}

pub fn style_selected() -> Style {
    Style::default()
        .fg(Color::Black)
        .bg(COLOR_ACCENT)
        .add_modifier(Modifier::BOLD)
}

pub fn style_dimmed() -> Style {
    Style::default().fg(COLOR_MUTED)
}

pub fn style_success() -> Style {
    Style::default()
        .fg(COLOR_SUCCESS)
        .add_modifier(Modifier::BOLD)
}

pub fn style_warn() -> Style {
    Style::default()
        .fg(COLOR_WARN)
        .add_modifier(Modifier::BOLD)
}

pub fn style_alert() -> Style {
    Style::default()
        .fg(COLOR_ALERT)
        .add_modifier(Modifier::BOLD)
}
