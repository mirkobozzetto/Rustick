use crate::app::{App, Mode};
use crate::event::Event;
use crossterm::event::KeyCode;

pub fn handle_event(app: &mut App, event: Event) {
    match event {
        Event::Key(key) => match app.mode {
            Mode::Normal => match key.code {
                KeyCode::Char('q') => {
                    app.running = false;
                }
                _ => {}
            },
            Mode::Insert => {}
            Mode::Search => {}
            Mode::Focus => {}
        },
        Event::Tick => {
            app.tick_count = app.tick_count.wrapping_add(1);
        }
        Event::Resize(_, _) => {}
        Event::Reminder(_) => {}
    }
}
