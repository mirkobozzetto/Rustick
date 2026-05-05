pub mod popup;
pub mod search;
pub mod task_input;
pub mod task_list;
pub mod timeline;
pub mod upcoming;

use crate::app::App;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, BorderType, Paragraph};

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let block = Block::default()
        .title("Rustick")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mode_text = format!("Mode: {:?} | Ticks: {}", app.mode, app.tick_count);
    let paragraph = Paragraph::new(mode_text).alignment(Alignment::Left);
    frame.render_widget(paragraph, inner);
}
