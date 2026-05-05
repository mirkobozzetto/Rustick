use crate::app::App;
use crate::model::filter::fuzzy_match;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let width = (area.width * 60) / 100;
    let height = (area.height * 50) / 100;
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;

    let overlay_area = Rect {
        x: area.x + x,
        y: area.y + y,
        width,
        height,
    };

    let block = Block::default()
        .title("Search")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let query = &app.input_buffer;
    let matches: Vec<&crate::model::Task> = app
        .tasks
        .iter()
        .filter(|task| {
            fuzzy_match(query, &task.title)
                || task
                    .body
                    .as_ref()
                    .map(|b| fuzzy_match(query, b))
                    .unwrap_or(false)
        })
        .take(10)
        .collect();

    let mut items = vec![ListItem::new(Line::from(vec![
        Span::raw("> "),
        Span::styled(query.clone(), Style::default().fg(Color::Yellow)),
        if app.mode == crate::app::Mode::Search {
            Span::styled("│", Style::default().fg(Color::Cyan))
        } else {
            Span::raw("")
        },
    ]))];

    items.push(ListItem::new(""));

    if matches.is_empty() {
        items.push(ListItem::new(Span::styled(
            "No results",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for task in matches {
            items.push(ListItem::new(Span::raw(format!("● {}", task.title))));
        }
    }

    let list = List::new(items);
    frame.render_widget(list, inner);
}
