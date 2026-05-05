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

    let mut items = Vec::new();
    let mut current_index = 0;

    if !overdue.is_empty() {
        items.push(ListItem::new(Span::styled(
            "── Overdue ──",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
        current_index += 1;

        for task in &overdue {
            items.push(render_task_item(task, current_index == app.selected_index));
            current_index += 1;
        }
    }

    if !today.is_empty() {
        items.push(ListItem::new(Span::styled(
            "── Today ──",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        current_index += 1;

        for task in &today {
            items.push(render_task_item(task, current_index == app.selected_index));
            current_index += 1;
        }
    }

    if !upcoming.is_empty() {
        items.push(ListItem::new(Span::styled(
            "── Upcoming ──",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));
        current_index += 1;

        for task in &upcoming {
            items.push(render_task_item(task, current_index == app.selected_index));
            current_index += 1;
        }
    }

    if !no_date.is_empty() {
        items.push(ListItem::new(Span::styled(
            "── No Date ──",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD),
        )));
        current_index += 1;

        for task in &no_date {
            items.push(render_task_item(task, current_index == app.selected_index));
            current_index += 1;
        }
    }

    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn render_task_item(task: &Task, selected: bool) -> ListItem {
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

    let mut content = vec![Span::styled(
        priority_str,
        Style::default().fg(priority_color),
    )];

    content.push(Span::raw(" "));

    let title_style = if task.status == TaskStatus::Done {
        Style::default().add_modifier(Modifier::DIM | Modifier::CROSSED_OUT)
    } else if selected {
        Style::default().add_modifier(Modifier::REVERSED)
    } else {
        Style::default()
    };

    content.push(Span::styled(task.title.clone(), title_style));

    ListItem::new(Line::from(content))
}
