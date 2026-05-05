use crate::error::{Result, RustickError};
use crate::event::Event;
use crate::model::Task;
use crate::ui;
use chrono::Local;
use crossterm::event::EventStream;
use ratatui::prelude::*;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Search,
    Focus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Sidebar,
    Main,
    Timeline,
}

pub struct App {
    pub mode: Mode,
    pub active_panel: Panel,
    pub tasks: Vec<Task>,
    pub selected_index: usize,
    pub running: bool,
    pub tick_count: u64,
}

impl App {
    pub fn new() -> Self {
        let now = Local::now();
        let mut tasks = vec![
            Task::new(ulid::Ulid::new().to_string(), "Fix login bug".into()),
            Task::new(ulid::Ulid::new().to_string(), "Write documentation".into()),
            Task::new(ulid::Ulid::new().to_string(), "Deploy to staging".into()),
            Task::new(ulid::Ulid::new().to_string(), "Review PR #42".into()),
        ];

        tasks[0].due_at = Some(now - chrono::Duration::hours(5));
        tasks[0].priority = 1;

        tasks[1].due_at = Some(now + chrono::Duration::hours(2));
        tasks[1].priority = 2;

        tasks[2].due_at = Some(now + chrono::Duration::days(3));
        tasks[2].priority = 1;

        tasks[3].due_at = Some(now + chrono::Duration::days(7));
        tasks[3].priority = 3;

        Self {
            mode: Mode::Normal,
            active_panel: Panel::Main,
            tasks,
            selected_index: 0,
            running: true,
            tick_count: 0,
        }
    }

    pub fn visible_tasks(&self) -> (Vec<&Task>, Vec<&Task>, Vec<&Task>, Vec<&Task>) {
        let now = Local::now();
        let today_date = now.date_naive();

        let mut overdue = Vec::new();
        let mut today = Vec::new();
        let mut upcoming = Vec::new();
        let mut no_date = Vec::new();

        for task in &self.tasks {
            if let Some(due_at) = task.due_at {
                if due_at < now {
                    overdue.push(task);
                } else if due_at.date_naive() == today_date {
                    today.push(task);
                } else {
                    upcoming.push(task);
                }
            } else {
                no_date.push(task);
            }
        }

        (overdue, today, upcoming, no_date)
    }
}

pub async fn run(terminal: &mut Terminal<impl Backend>) -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<Event>(32);

    let tick_tx = tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(250));
        loop {
            interval.tick().await;
            let _ = tick_tx.send(Event::Tick).await;
        }
    });

    let input_tx = tx.clone();
    tokio::spawn(async move {
        let mut reader = EventStream::new();
        loop {
            match reader.next().await {
                Some(Ok(crossterm::event::Event::Key(key))) => {
                    let _ = input_tx.send(Event::Key(key)).await;
                }
                Some(Ok(crossterm::event::Event::Resize(width, height))) => {
                    let _ = input_tx.send(Event::Resize(width, height)).await;
                }
                Some(Err(_)) => break,
                None => break,
                _ => {}
            }
        }
    });

    let mut app = App::new();

    loop {
        terminal
            .draw(|frame| ui::draw(frame, &app))
            .map_err(|e| RustickError::TerminalError(e.to_string()))?;

        if !app.running {
            break;
        }

        if let Ok(event) = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
            if let Some(event) = event {
                crate::event::handler::handle_event(&mut app, event);
            }
        }
    }

    Ok(())
}
