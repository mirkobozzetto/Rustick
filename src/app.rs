use crate::error::{Result, RustickError};
use crate::event::Event;
use crate::model::Task;
use crate::ui;
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
        Self {
            mode: Mode::Normal,
            active_panel: Panel::Main,
            tasks: Vec::new(),
            selected_index: 0,
            running: true,
            tick_count: 0,
        }
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
