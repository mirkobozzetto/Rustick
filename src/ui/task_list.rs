use crate::app::App;
use crate::model::Task;
use crate::model::task::TaskStatus;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.active_panel == crate::app::Panel::Main {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title("Tasks")
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let (overdue, today, upcoming, no_date) = app.visible_tasks();

    let done_tasks: Vec<&Task> = app.tasks.iter()
        .filter(|t| t.status == TaskStatus::Done)
        .collect();

    let mut items = Vec::new();
    let mut task_index = 0;

    if !overdue.is_empty() {
        items.push(ListItem::new(Span::styled(
            "── Overdue ──",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));

        for task in &overdue {
            items.push(render_task_item(task, task_index == app.selected_index));
            task_index += 1;
        }
    }

    if !today.is_empty() {
        items.push(ListItem::new(Span::styled(
            "── Today ──",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));

        for task in &today {
            items.push(render_task_item(task, task_index == app.selected_index));
            task_index += 1;
        }
    }

    if !upcoming.is_empty() {
        items.push(ListItem::new(Span::styled(
            "── Upcoming ──",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));

        for task in &upcoming {
            items.push(render_task_item(task, task_index == app.selected_index));
            task_index += 1;
        }
    }

    if !no_date.is_empty() {
        items.push(ListItem::new(Span::styled(
            "── No Date ──",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD),
        )));

        for task in &no_date {
            items.push(render_task_item(task, task_index == app.selected_index));
            task_index += 1;
        }
    }

    if !done_tasks.is_empty() {
        let done_header = format!("── Done ({}) ──", done_tasks.len());
        items.push(ListItem::new(Span::styled(
            done_header,
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )));
    }

    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn render_task_item<'a>(task: &'a Task, selected: bool) -> ListItem<'a> {
    let priority_color = match task.priority {
        1 => Color::Red,
        2 => Color::Yellow,
        3 => Color::Blue,
        _ => Color::DarkGray,
    };

    let priority_str = if task.priority > 0 {
        format!("[P{}]", task.priority)
    } else {
        "[P4]".to_string()
    };

    let indicator = if selected { "▸ " } else { "  " };

    let title_style = match (selected, task.status == TaskStatus::Done) {
        (true, true) => Style::default()
            .fg(Color::Black)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::CROSSED_OUT),
        (true, false) => Style::default().fg(Color::Black).bg(Color::White),
        (false, true) => Style::default().add_modifier(Modifier::DIM | Modifier::CROSSED_OUT),
        (false, false) => Style::default(),
    };

    let mut lines = Vec::new();

    let mut title_content = vec![
        Span::styled(indicator, if selected { Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD) } else { Style::default() }),
        Span::styled(priority_str, Style::default().fg(priority_color)),
        Span::raw(" "),
        Span::styled(task.title.clone(), title_style),
    ];

    if let Some(due) = task.due_at {
        let time_str = format!(" @ {}", due.format("%H:%M"));
        title_content.push(Span::styled(time_str, Style::default().fg(Color::Cyan).add_modifier(Modifier::DIM)));
    }

    lines.push(Line::from(title_content));

    if let Some(body) = &task.body {
        if !body.is_empty() {
            let preview = if body.len() > 30 {
                format!("{}…", &body[..30])
            } else {
                body.clone()
            };
            lines.push(Line::from(Span::styled(
                format!("  {}", preview),
                Style::default().add_modifier(Modifier::DIM),
            )));
        }
    }

    ListItem::new(lines)
}
