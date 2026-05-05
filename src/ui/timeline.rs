use crate::app::{App, Mode};
use chrono::{Datelike, Local};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use time::{Date, Month};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.active_panel == crate::app::Panel::Timeline {
        if app.mode == Mode::DatePick {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Cyan)
        }
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title("Calendar")
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 2 {
        return;
    }

    render_calendar(frame, app, inner);
}

fn render_calendar(frame: &mut Frame, app: &App, area: Rect) {
    if area.height < 15 {
        frame.render_widget(Paragraph::new("Calendar too small"), area);
        return;
    }

    let mut y = 0;

    let header = format!(
        "< {} {} >",
        month_name(app.calendar_month),
        app.calendar_year
    );
    let header_spans = vec![Span::styled(
        header,
        Style::default().bold().fg(Color::Cyan),
    )];
    frame.render_widget(
        Paragraph::new(Line::from(header_spans)),
        Rect {
            x: area.x,
            y: area.y + y,
            width: area.width,
            height: 1,
        },
    );
    y += 1;

    let now = Local::now();
    let now_time = now.format("%H:%M:%S").to_string();
    frame.render_widget(
        Paragraph::new(format!("⏰ {}", now_time)).style(Style::default().fg(Color::Yellow)),
        Rect {
            x: area.x,
            y: area.y + y,
            width: area.width,
            height: 1,
        },
    );
    y += 1;

    let weekdays = "Mo Tu We Th Fr Sa Su";
    frame.render_widget(
        Paragraph::new(weekdays),
        Rect {
            x: area.x,
            y: area.y + y,
            width: area.width,
            height: 1,
        },
    );
    y += 1;

    if let Ok(first_date) = Date::from_calendar_date(
        app.calendar_year,
        Month::try_from(app.calendar_month as u8).unwrap(),
        1,
    ) {
        let first_weekday = first_date.weekday().number_from_monday();
        let days_in_month = days_in_month(app.calendar_year, app.calendar_month);

        let mut week_str = String::new();
        for _ in 1..first_weekday {
            week_str.push_str("   ");
        }

        for day in 1..=days_in_month {
            let day_str = format!("{:2}", day);

            let is_selected = day == app.calendar_day;
            let is_today = day == now.day()
                && app.calendar_month == now.month()
                && app.calendar_year == now.year();

            let day_text = if is_selected {
                format!("({})", day_str.trim())
            } else if is_today {
                format!(" {}*", day_str.trim())
            } else {
                format!(" {} ", day_str.trim())
            };

            week_str.push_str(&day_text);

            let weekday = (first_weekday + (day as u8 - 1) - 1) % 7;
            if weekday == 6 || day == days_in_month {
                let line_style = if is_selected {
                    Style::default().fg(Color::Green).bold()
                } else if is_today {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                };

                frame.render_widget(
                    Paragraph::new(week_str.clone()).style(line_style),
                    Rect {
                        x: area.x,
                        y: area.y + y,
                        width: area.width,
                        height: 1,
                    },
                );
                y += 1;
                week_str.clear();
            }
        }
    }

    y += 1;

    let selected_day = app.calendar_day;
    let selected_month = app.calendar_month;
    let selected_year = app.calendar_year;

    let tasks_for_day: Vec<_> = app
        .tasks
        .iter()
        .filter(|t| {
            if let Some(due_at) = t.due_at {
                let due_date = due_at.date_naive();
                due_date.day() == selected_day
                    && due_date.month() == selected_month
                    && due_date.year() == selected_year
            } else {
                false
            }
        })
        .collect();

    frame.render_widget(
        Paragraph::new("Today's tasks:").style(Style::default().fg(Color::Gray)),
        Rect {
            x: area.x,
            y: area.y + y,
            width: area.width,
            height: 1,
        },
    );
    y += 1;

    for (_idx, task) in tasks_for_day.iter().take(3).enumerate() {
        if y >= area.height {
            break;
        }

        let time_str = if let Some(due_at) = task.due_at {
            due_at.format("%H:%M").to_string()
        } else {
            "??:??".to_string()
        };

        let task_color = if task.priority == 1 {
            Color::Red
        } else if task.priority == 2 {
            Color::Yellow
        } else {
            Color::DarkGray
        };

        let task_title = task.title.chars().take(20).collect::<String>();
        let task_line = format!("  {} • {}", time_str, task_title);

        frame.render_widget(
            Paragraph::new(task_line).style(Style::default().fg(task_color)),
            Rect {
                x: area.x,
                y: area.y + y,
                width: area.width,
                height: 1,
            },
        );
        y += 1;
    }

    while y < area.height.saturating_sub(3) as u16 {
        frame.render_widget(
            Paragraph::new(""),
            Rect {
                x: area.x,
                y: area.y + y,
                width: area.width,
                height: 1,
            },
        );
        y += 1;
    }

    let hints = "←→↑↓: Navigate  H/L: Month  Enter: Select  Esc: Cancel";
    frame.render_widget(
        Paragraph::new(hints).style(Style::default().fg(Color::DarkGray)),
        Rect {
            x: area.x,
            y: area.y + area.height.saturating_sub(1),
            width: area.width,
            height: 1,
        },
    );
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}
