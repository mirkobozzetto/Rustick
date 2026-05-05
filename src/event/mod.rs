pub mod handler;
pub mod scheduler;

#[derive(Debug, Clone)]
pub enum Event {
    Key(crossterm::event::KeyEvent),
    Tick,
    Reminder(String),
    Resize(u16, u16),
}
