use crate::error::{Result, RustickError};
use crate::event::Event;
use crate::model::Task;
use crate::store::Store;
use crate::ui;
use chrono::{Datelike, Local};
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
    DatePick,
    TimeInput,
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
    pub store: Store,
    pub input_buffer: String,
    pub input_cursor: usize,
    pub editing_task: Option<usize>,
    pub editing_body: bool,
    pub popup_visible: bool,
    pub popup_message: String,
    pub _popup_confirm: bool,
    pub pending_delete: Option<usize>,
    pub sidebar_index: usize,
    pub _timeline_scroll: usize,
    pub time_input_buffer: String,
    pub time_input_cursor: usize,
    pub alert_task: Option<String>,
    pub alert_tick: u64,
    pub calendar_year: i32,
    pub calendar_month: u32,
    pub calendar_day: u32,
}

impl App {
    pub fn new() -> Result<Self> {
        let proj_dirs = directories::ProjectDirs::from("dev", "rustick", "rustick").ok_or(
            RustickError::ConfigError("Failed to determine project directories".into()),
        )?;

        let data_dir = proj_dirs.data_dir();
        std::fs::create_dir_all(data_dir)?;
        let db_path = data_dir.join("rustick.db");

        let store = Store::new(db_path)?;
        let tasks = store.load_tasks()?;

        let selected_index = store
            .load_session("selected_index")
            .ok()
            .and_then(|v| v)
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);

        let mode_str = store
            .load_session("mode")
            .ok()
            .and_then(|v| v)
            .unwrap_or_else(|| "Normal".to_string());

        let mode = match mode_str.as_str() {
            "Insert" => Mode::Insert,
            "Search" => Mode::Search,
            "Focus" => Mode::Focus,
            "DatePick" => Mode::DatePick,
            "TimeInput" => Mode::TimeInput,
            _ => Mode::Normal,
        };

        let panel_str = store
            .load_session("active_panel")
            .ok()
            .and_then(|v| v)
            .unwrap_or_else(|| "Main".to_string());

        let active_panel = match panel_str.as_str() {
            "Sidebar" => Panel::Sidebar,
            "Timeline" => Panel::Timeline,
            _ => Panel::Main,
        };

        let now = Local::now();
        Ok(Self {
            mode,
            active_panel,
            tasks,
            selected_index,
            running: true,
            tick_count: 0,
            store,
            input_buffer: String::new(),
            input_cursor: 0,
            editing_task: None,
            editing_body: false,
            popup_visible: false,
            popup_message: String::new(),
            _popup_confirm: false,
            pending_delete: None,
            sidebar_index: 0,
            _timeline_scroll: 0,
            time_input_buffer: String::new(),
            time_input_cursor: 0,
            alert_task: None,
            alert_tick: 0,
            calendar_year: now.year(),
            calendar_month: now.month(),
            calendar_day: now.day(),
        })
    }

    pub fn add_task(&mut self, task: Task) -> Result<()> {
        self.store.save_task(&task)?;
        self.tasks.push(task);
        Ok(())
    }

    pub fn update_task(&mut self, index: usize) -> Result<()> {
        if index < self.tasks.len() {
            self.store.update_task(&self.tasks[index])?;
        }
        Ok(())
    }

    pub fn delete_task(&mut self, index: usize) -> Result<()> {
        if index < self.tasks.len() {
            let task = self.tasks.remove(index);
            self.store.delete_task(&task.id)?;
        }
        Ok(())
    }

    pub fn toggle_task_status(&mut self, index: usize) -> Result<()> {
        if index < self.tasks.len() {
            use crate::model::task::TaskStatus;
            self.tasks[index].status = match self.tasks[index].status {
                TaskStatus::Todo => TaskStatus::Done,
                TaskStatus::Done => TaskStatus::Archived,
                TaskStatus::Archived => TaskStatus::Todo,
            };
            self.store.update_task(&self.tasks[index])?;
        }
        Ok(())
    }

    pub fn save_session(&self) -> Result<()> {
        self.store
            .save_session("selected_index", &self.selected_index.to_string())?;
        self.store.save_session(
            "active_panel",
            match self.active_panel {
                Panel::Sidebar => "Sidebar",
                Panel::Main => "Main",
                Panel::Timeline => "Timeline",
            },
        )?;
        self.store.save_session(
            "mode",
            match self.mode {
                Mode::Normal => "Normal",
                Mode::Insert => "Insert",
                Mode::Search => "Search",
                Mode::Focus => "Focus",
                Mode::DatePick => "DatePick",
                Mode::TimeInput => "TimeInput",
            },
        )?;
        Ok(())
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

    let mut app = App::new()?;

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

    app.save_session()?;
    Ok(())
}
