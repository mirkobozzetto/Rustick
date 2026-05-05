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
use ratatui::widgets::{Block, Borders, Paragraph};

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

    render_sidebar(frame, app, sidebar_area);
    task_list::render(frame, app, main_area);
    timeline::render(frame, app, timeline_area);
    render_status_bar(frame, app, status_area);
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

    let sidebar_text = "All Tasks\nToday\nThis Week\n\nTags\nViews\nFilters";
    let paragraph = Paragraph::new(sidebar_text);
    frame.render_widget(paragraph, inner);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mode_str = match app.mode {
        Mode::Normal => "[NOR]",
        Mode::Insert => "[INS]",
        Mode::Search => "[SRC]",
        Mode::Focus => "[FOC]",
    };

    let task_count = app.tasks.len();
    let done_count = app
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Done)
        .count();

    let time_str = Local::now().format("%H:%M").to_string();

    let left_text = format!("{} Tasks: {} | Done: {}", mode_str, task_count, done_count);
    let right_text = time_str;

    let status_spans = vec![
        Span::raw(left_text.clone()),
        Span::raw(
            " ".repeat(
                inner
                    .width
                    .saturating_sub((left_text.len() + right_text.len()) as u16) as usize,
            ),
        ),
        Span::raw(right_text),
    ];

    let status_line = Line::from(status_spans);
    let paragraph = Paragraph::new(status_line);
    frame.render_widget(paragraph, inner);
}
