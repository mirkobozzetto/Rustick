use crate::app::App;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let input_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(3),
        width: area.width,
        height: 3,
    };

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(input_area);
    frame.render_widget(block, input_area);

    let label = if app.editing_task.is_some() {
        "Edit: "
    } else {
        "New:  "
    };

    let display_text = format!("{}{}", label, app.input_buffer);
    let cursor_pos = label.len() + app.input_cursor;

    let mut spans = Vec::new();
    for (i, c) in display_text.chars().enumerate() {
        if i == cursor_pos {
            spans.push(Span::styled(
                c.to_string(),
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black),
            ));
        } else {
            spans.push(Span::raw(c.to_string()));
        }
    }

    if cursor_pos >= display_text.len() {
        spans.push(Span::styled(
            " ",
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black),
        ));
    }

    let paragraph = Paragraph::new(Line::from(spans));
    frame.render_widget(paragraph, inner);
}
