use crate::app::App;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let popup_width = (area.width as f32 * 0.4) as u16;
    let popup_height = (area.height as f32 * 0.2).max(5.0) as u16;

    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect {
        x: area.x + popup_x,
        y: area.y + popup_y,
        width: popup_width,
        height: popup_height,
    };

    let overlay = Block::default()
        .title("Confirm")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
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

    let message = Paragraph::new(app.popup_message.clone())
        .wrap(Wrap { trim: true });
    frame.render_widget(message, message_area);

    let footer_text = if app.pending_delete.is_some() {
        "[Enter] Confirm  [Esc] Cancel"
    } else {
        "[Esc] Close"
    };

    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, footer_area);
}
