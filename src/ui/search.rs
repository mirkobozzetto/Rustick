use crate::app::App;
use crate::model::filter::fuzzy_match;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem};

fn priority_color(priority: u8) -> Color {
    match priority {
        3 => Color::Red,
        2 => Color::Yellow,
        1 => Color::Green,
        _ => Color::Gray,
    }
}

fn find_match_indices(query: &str, text: &str) -> Vec<usize> {
    if query.is_empty() {
        return Vec::new();
    }
    let query_lower = query.to_lowercase();
    let text_lower = text.to_lowercase();
    let mut indices = Vec::new();
    let mut query_chars = query_lower.chars().peekable();

    for (i, c) in text_lower.chars().enumerate() {
        if query_chars.peek() == Some(&c) {
            indices.push(i);
            query_chars.next();
        }
    }
    indices
}

fn highlight_matches(text: &str, match_indices: &[usize]) -> Vec<Span<'static>> {
    if match_indices.is_empty() {
        return vec![Span::raw(text.to_string())];
    }

    let mut spans = Vec::new();
    let mut last_pos = 0;
    let chars: Vec<char> = text.chars().collect();

    for &idx in match_indices {
        if idx > last_pos {
            spans.push(Span::raw(chars[last_pos..idx].iter().collect::<String>()));
        }
        spans.push(Span::styled(
            chars[idx].to_string(),
            Style::default().bold(),
        ));
        last_pos = idx + 1;
    }

    if last_pos < chars.len() {
        spans.push(Span::raw(chars[last_pos..].iter().collect::<String>()));
    }

    spans
}

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

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .title("Search")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan).bold());

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

    let total_tasks = app.tasks.len();
    let result_count = matches.len();

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

    if query.is_empty() {
        items.push(ListItem::new(Span::styled(
            "Type to search...",
            Style::default().fg(Color::DarkGray),
        )));
    } else if matches.is_empty() {
        items.push(ListItem::new(Span::styled(
            "No results",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for task in &matches {
            let mut line_spans = vec![
                Span::styled(
                    format!("● "),
                    Style::default().fg(priority_color(task.priority)),
                ),
            ];

            let match_indices = find_match_indices(query, &task.title);
            let title_spans = highlight_matches(&task.title, &match_indices);
            line_spans.extend(title_spans);

            if let Some(due_at) = task.due_at {
                let time_str = due_at.format(" @ %H:%M").to_string();
                line_spans.push(Span::styled(
                    time_str,
                    Style::default().fg(Color::DarkGray),
                ));
            }

            items.push(ListItem::new(Line::from(line_spans)));
        }
    }

    items.push(ListItem::new(""));
    items.push(ListItem::new(Span::styled(
        format!("Results: {}/{}", result_count, total_tasks),
        Style::default().fg(Color::DarkGray),
    )));

    let list = List::new(items);
    frame.render_widget(list, inner);
}
