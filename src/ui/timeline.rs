use crate::app::App;
use chrono::{Local, Timelike};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.active_panel == crate::app::Panel::Timeline {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title("Timeline")
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 2 {
        return;
    }

    let now = Local::now();
    let current_hour = now.hour() as usize;

    let mut lines = Vec::new();
    for hour in 0..24 {
        let hour_str = format!("{:02}:00", hour);

        let hour_tasks: Vec<_> = app
            .tasks
            .iter()
            .filter(|t| {
                if let Some(due_at) = t.due_at {
                    due_at.hour() as usize == hour
                        && due_at.date_naive() == now.date_naive()
                } else {
                    false
                }
            })
            .collect();

        if !hour_tasks.is_empty() {
            let task = hour_tasks[0];
            let title_preview = task.title.chars().take(15).collect::<String>();
            let task_color = if task.due_at.unwrap() < now {
                Color::Red
            } else {
                match task.priority {
                    1 => Color::Red,
                    2 => Color::Yellow,
                    3 => Color::Cyan,
                    _ => Color::DarkGray,
                }
            };

            let line = format!("{:02}:00  █ {}", hour, title_preview);

            if hour == current_hour {
                lines.push((line, Some(task_color), true));
            } else {
                lines.push((line, Some(task_color), false));
            }
        } else {
            if hour == current_hour {
                lines.push((format!("{:02}:00", hour), None, true));
            } else {
                lines.push((hour_str, None, false));
            }
        }
    }

    let start_line = if current_hour > (inner.height as usize / 2) {
        current_hour - (inner.height as usize / 2)
    } else {
        0
    };
    let end_line = (start_line + inner.height as usize).min(24);

    for (i, y) in (0..inner.height as usize).enumerate() {
        if start_line + i >= end_line {
            break;
        }

        let (text, color, is_current) = &lines[start_line + i];
        let style = if *is_current {
            if let Some(c) = color {
                Style::default().fg(*c).bold()
            } else {
                Style::default().bold()
            }
        } else if let Some(c) = color {
            Style::default().fg(*c)
        } else {
            Style::default()
        };

        let prefix = if *is_current { "▶" } else { " " };
        let display_text = if *is_current {
            format!("{}{}", prefix, text)
        } else {
            format!("  {}", text)
        };

        frame.render_widget(
            ratatui::widgets::Paragraph::new(display_text).style(style),
            Rect {
                x: inner.x,
                y: inner.y + y as u16,
                width: inner.width,
                height: 1,
            },
        );
    }
}
