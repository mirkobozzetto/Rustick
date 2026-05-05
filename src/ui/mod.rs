pub mod popup;
pub mod search;
pub mod task_input;
pub mod task_list;
pub mod timeline;
pub mod upcoming;

use crate::app::{App, Mode, Panel};
use crate::model::task::TaskStatus;
use chrono::Local;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(area);

    let content_area = chunks[0];
    let status_area = chunks[1];

    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(50),
            Constraint::Percentage(30),
        ])
        .split(content_area);

    let sidebar_area = horizontal_chunks[0];
    let main_area = horizontal_chunks[1];
    let timeline_area = horizontal_chunks[2];

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(timeline_area);

    render_sidebar(frame, app, sidebar_area);
    task_list::render(frame, app, main_area);
    timeline::render(frame, app, right_chunks[0]);
    upcoming::render(frame, app, right_chunks[1]);
    render_status_bar(frame, app, status_area);

    if app.mode == Mode::Insert || app.mode == Mode::DatePick || app.mode == Mode::TimeInput {
        task_input::render(frame, app, main_area);
    }
    if app.mode == Mode::Search {
        search::render(frame, app, frame.area());
    }
    if app.popup_visible {
        popup::render(frame, app, frame.area());
    }
}

fn render_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.active_panel == Panel::Sidebar {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title("Rustick")
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let items = vec![
        "All Tasks",
        "Today",
        "This Week",
        "",
        "P1 High",
        "P2 Medium",
        "P3 Low",
        "P4 None",
    ];

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let content = if i == app.sidebar_index && app.active_panel == Panel::Sidebar {
                format!("▸ {}", label)
            } else {
                format!("  {}", label)
            };

            let style = if i == app.sidebar_index && app.active_panel == Panel::Sidebar {
                Style::default().reversed()
            } else {
                Style::default()
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.sidebar_index));

    let list = List::new(list_items).block(Block::default());
    frame.render_stateful_widget(list, inner, &mut list_state);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let (mode_label, mode_color) = match app.mode {
        Mode::Normal => ("NORMAL", Color::Green),
        Mode::Insert => ("INSERT", Color::Yellow),
        Mode::Search => ("SEARCH", Color::Cyan),
        Mode::Focus => ("FOCUS", Color::Magenta),
        Mode::DatePick => ("DATE", Color::Cyan),
        Mode::TimeInput => ("TIME", Color::Yellow),
    };

    let mode_span = Span::styled(mode_label, Style::default().fg(mode_color).bold());

    let context_hint = match app.mode {
        Mode::Normal => "? help",
        Mode::Insert => "Enter save │ Esc cancel",
        Mode::Search => "Enter jump │ Esc close",
        Mode::DatePick => "↑↓←→ navigate │ Enter select │ Esc cancel",
        Mode::TimeInput => "HH:MM │ Enter save │ Esc cancel",
        Mode::Focus => "? help",
    };

    let task_count = app.tasks.len();
    let done_count = app
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Done)
        .count();

    let time_str = Local::now().format("%H:%M").to_string();

    let left_content = format!("{}  Tasks: {} | Done: {} {}", mode_label, task_count, done_count, context_hint);
    let padding_len = inner
        .width
        .saturating_sub((left_content.len() + time_str.len()) as u16) as usize;

    let status_spans = vec![
        mode_span,
        Span::raw("  "),
        Span::raw(format!("Tasks: {} | Done: {}", task_count, done_count)),
        Span::raw("  "),
        Span::raw(context_hint),
        Span::raw(" ".repeat(padding_len)),
        Span::raw(time_str),
    ];

    let status_line = Line::from(status_spans);
    let paragraph = Paragraph::new(status_line);
    frame.render_widget(paragraph, inner);
}
