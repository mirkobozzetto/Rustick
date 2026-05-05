use crate::app::{App, Mode};
use chrono::{Local, Datelike, Timelike};
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
        .title(if app.mode == Mode::DatePick { "Calendar" } else { "Timeline" })
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 2 {
        return;
    }

    if app.mode == Mode::DatePick {
        render_calendar(frame, app, inner);
    } else {
        render_timeline(frame, app, inner);
    }
}

fn render_timeline(frame: &mut Frame, app: &App, area: Rect) {
    if area.height < 2 {
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

    let start_line = if current_hour > (area.height as usize / 2) {
        current_hour - (area.height as usize / 2)
    } else {
        0
    };
    let end_line = (start_line + area.height as usize).min(24);

    for (i, y) in (0..area.height as usize).enumerate() {
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
            Paragraph::new(display_text).style(style),
            Rect {
                x: area.x,
                y: area.y + y as u16,
                width: area.width,
                height: 1,
            },
        );
    }
}

fn render_calendar(frame: &mut Frame, app: &App, area: Rect) {
    if area.height < 10 {
        frame.render_widget(
            Paragraph::new("Calendar too small"),
            area,
        );
        return;
    }

    let mut y = 0;

    let header = format!("< {} {} >", month_name(app.calendar_month), app.calendar_year);
    let header_spans = vec![
        Span::styled(
            header,
            Style::default().bold().fg(Color::Cyan),
        ),
    ];
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
        let now = Local::now();

        let mut week_str = String::new();
        for _ in 1..first_weekday {
            week_str.push_str("   ");
        }

        for day in 1..=days_in_month {
            let day_str = format!("{:2}", day);

            let is_selected = day == app.calendar_day;
            let is_today = day == now.day() && app.calendar_month == now.month() && app.calendar_year == now.year();

            let day_text = if is_selected {
                format!("[{}]", day_str.trim())
            } else if is_today {
                format!(" {}*", day_str.trim())
            } else {
                format!(" {} ", day_str.trim())
            };

            week_str.push_str(&day_text);

            let weekday = (first_weekday + (day as u8 - 1) - 1) % 7;
            if weekday == 6 || day == days_in_month {
                frame.render_widget(
                    Paragraph::new(week_str.clone()),
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

    while y < area.height as u16 {
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
