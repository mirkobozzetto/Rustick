use crate::app::App;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap, Clear};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let popup_width = (area.width as f32 * 0.5) as u16;
    let popup_height = (area.height as f32 * 0.3).max(8.0) as u16;

    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect {
        x: area.x + popup_x,
        y: area.y + popup_y,
        width: popup_width,
        height: popup_height,
    };

    frame.render_widget(Clear, popup_area);

    let is_help = app.popup_message.starts_with("j/");
    let is_reminder = app.popup_message.starts_with("REMINDER:");
    let is_confirmation = app.pending_delete.is_some();

    let (title, border_color, animated) = if is_reminder {
        ("Reminder!", Color::Red, true)
    } else if is_help {
        ("Help", Color::Blue, false)
    } else {
        ("Confirm", Color::Yellow, false)
    };

    let border_style = if animated {
        Style::default().fg(border_color).italic()
    } else {
        Style::default().fg(border_color)
    };

    let overlay = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(Style::default().bg(Color::Black));

    let inner = overlay.inner(popup_area);
    frame.render_widget(overlay, popup_area);

    let message_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: inner.height.saturating_sub(2),
    };

    let footer_area = Rect {
        x: inner.x,
        y: inner.y + message_area.height,
        width: inner.width,
        height: 2,
    };

    let display_message = if is_help {
        app.popup_message.trim_start_matches("j/").to_string()
    } else if is_reminder {
        app.popup_message.trim_start_matches("REMINDER:").trim().to_string()
    } else {
        app.popup_message.clone()
    };

    let message = Paragraph::new(display_message)
        .wrap(Wrap { trim: true });
    frame.render_widget(message, message_area);

    let footer_text = if is_confirmation {
        "[Enter] OK  [Esc] Cancel"
    } else {
        "[Esc] Dismiss"
    };

    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, footer_area);
}
