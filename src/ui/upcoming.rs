use crate::app::App;
use crate::event::scheduler::format_relative_time;
use crate::model::task::TaskStatus;
use chrono::Local;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.active_panel == crate::app::Panel::Sidebar {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let now = Local::now();

    let mut upcoming_tasks: Vec<_> = app
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Todo && t.due_at.is_some())
        .filter(|t| t.due_at.unwrap() > now)
        .collect();

    upcoming_tasks.sort_by_key(|t| t.due_at);

    let count = upcoming_tasks.len();
    let title = format!("Upcoming ({})", count);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 2 {
        return;
    }

    if upcoming_tasks.is_empty() {
        let text = Paragraph::new("No upcoming tasks")
            .alignment(Alignment::Center)
            .style(Style::default().dim());
        frame.render_widget(text, inner);
        return;
    }

    let mut y = inner.y;
    let max_tasks = 5;

    for task in upcoming_tasks.iter().take(max_tasks) {
        if y >= inner.y + inner.height {
            break;
        }

        let due_at = task.due_at.unwrap();
        let time_str = format_relative_time(due_at);

        let priority_color = match task.priority {
            1 => Color::Red,
            2 => Color::Yellow,
            3 => Color::Blue,
            _ => Color::DarkGray,
        };

        let dot_style = Style::default().fg(priority_color);
        let time_style = Style::default().dim();

        let mut spans = Vec::new();
        spans.push(Span::styled("● ", dot_style));
        spans.push(Span::styled(
            time_str.clone(),
            time_style,
        ));
        spans.push(Span::raw("  "));
        spans.push(Span::raw(task.title.clone()));

        frame.render_widget(
            Paragraph::new(Line::from(spans)),
            Rect {
                x: inner.x,
                y,
                width: inner.width,
                height: 1,
            },
        );

        y += 1;
    }
}
