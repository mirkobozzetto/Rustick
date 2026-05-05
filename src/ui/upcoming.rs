use crate::app::App;
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

    let block = Block::default()
        .title("Upcoming")
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 2 {
        return;
    }

    let now = Local::now();

    let mut upcoming_tasks: Vec<_> = app
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Todo && t.due_at.is_some())
        .filter(|t| t.due_at.unwrap() > now)
        .collect();

    upcoming_tasks.sort_by_key(|t| t.due_at);

    if upcoming_tasks.is_empty() {
        let text = Paragraph::new("No upcoming tasks").alignment(Alignment::Center);
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
        let duration = due_at.signed_duration_since(now);

        let time_str = if duration.num_days() > 0 {
            if duration.num_days() == 1 {
                "tomorrow".to_string()
            } else {
                format!("in {} days", duration.num_days())
            }
        } else if duration.num_hours() > 0 {
            format!("in {}h", duration.num_hours())
        } else {
            format!("in {}m", (duration.num_minutes()).max(1))
        };

        let _color = match task.priority {
            1 => Color::Red,
            2 => Color::Yellow,
            3 => Color::Cyan,
            _ => Color::DarkGray,
        };

        let time_style = Style::default().dim();
        let title_style = Style::default();

        let first_line = format!("● in {}", if time_str.starts_with("in ") {
            &time_str[3..]
        } else {
            &time_str
        });

        frame.render_widget(
            Paragraph::new(first_line).style(if time_str.starts_with("in ") {
                time_style
            } else {
                Style::default()
            }),
            Rect {
                x: inner.x,
                y,
                width: inner.width,
                height: 1,
            },
        );

        y += 1;

        if y < inner.y + inner.height {
            let title_display = task.title.chars().take(inner.width as usize - 3).collect::<String>();
            frame.render_widget(
                Paragraph::new(format!("   {}", title_display)).style(title_style),
                Rect {
                    x: inner.x,
                    y,
                    width: inner.width,
                    height: 1,
                },
            );

            y += 1;

            if y < inner.y + inner.height {
                y += 1;
            }
        }
    }
}
