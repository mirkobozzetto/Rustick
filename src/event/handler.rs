use crate::app::{App, Mode, Panel};
use crate::event::Event;
use crate::model::Task;
use crossterm::event::{KeyCode, KeyEventKind};
use ulid::Ulid;

pub fn handle_event(app: &mut App, event: Event) {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            if app.popup_visible {
                handle_popup_mode(app, key.code);
            } else {
                match app.mode {
                    Mode::Normal => handle_normal_mode(app, key.code),
                    Mode::Insert => handle_insert_mode(app, key.code),
                    Mode::Search => {}
                    Mode::Focus => {}
                }
            }
        }
        Event::Tick => {
            app.tick_count = app.tick_count.wrapping_add(1);
        }
        Event::Key(_) => {}
        Event::Resize(_, _) => {}
        Event::Reminder(_) => {}
    }
}

fn handle_normal_mode(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('j') | KeyCode::Down => {
            app.selected_index = (app.selected_index + 1).min(app.tasks.len().saturating_sub(1));
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.selected_index = app.selected_index.saturating_sub(1);
        }
        KeyCode::Char('h') | KeyCode::Left => {
            app.active_panel = match app.active_panel {
                Panel::Sidebar => Panel::Timeline,
                Panel::Main => Panel::Sidebar,
                Panel::Timeline => Panel::Main,
            };
        }
        KeyCode::Char('l') | KeyCode::Right | KeyCode::Tab => {
            app.active_panel = match app.active_panel {
                Panel::Sidebar => Panel::Main,
                Panel::Main => Panel::Timeline,
                Panel::Timeline => Panel::Sidebar,
            };
        }
        KeyCode::Char('n') => {
            app.mode = Mode::Insert;
            app.input_buffer.clear();
            app.input_cursor = 0;
            app.editing_task = None;
        }
        KeyCode::Char('e') => {
            if app.selected_index < app.tasks.len() {
                app.mode = Mode::Insert;
                app.input_buffer = app.tasks[app.selected_index].title.clone();
                app.input_cursor = app.input_buffer.len();
                app.editing_task = Some(app.selected_index);
            }
        }
        KeyCode::Char(' ') => {
            let _ = app.toggle_task_status(app.selected_index).ok();
        }
        KeyCode::Char('d') => {
            if app.selected_index < app.tasks.len() {
                let title = app.tasks[app.selected_index].title.clone();
                app.popup_message = format!("Delete task '{}'?", title);
                app.popup_visible = true;
                app.pending_delete = Some(app.selected_index);
            }
        }
        KeyCode::Char('1') => {
            if app.selected_index < app.tasks.len() {
                app.tasks[app.selected_index].priority = 1;
                let _ = app.update_task(app.selected_index).ok();
            }
        }
        KeyCode::Char('2') => {
            if app.selected_index < app.tasks.len() {
                app.tasks[app.selected_index].priority = 2;
                let _ = app.update_task(app.selected_index).ok();
            }
        }
        KeyCode::Char('3') => {
            if app.selected_index < app.tasks.len() {
                app.tasks[app.selected_index].priority = 3;
                let _ = app.update_task(app.selected_index).ok();
            }
        }
        KeyCode::Char('4') => {
            if app.selected_index < app.tasks.len() {
                app.tasks[app.selected_index].priority = 4;
                let _ = app.update_task(app.selected_index).ok();
            }
        }
        KeyCode::Char('q') => {
            app.running = false;
        }
        KeyCode::Char('?') => {
            app.popup_message = "j/↓: Down | k/↑: Up | h/←: Left | l/→: Right | Tab: Next Panel\nn: New | e: Edit | Space: Toggle | d: Delete\n1-4: Priority | q: Quit | ?: Help".to_string();
            app.popup_visible = true;
        }
        _ => {}
    }
}

fn handle_insert_mode(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char(c) => {
            app.input_buffer.insert(app.input_cursor, c);
            app.input_cursor += 1;
        }
        KeyCode::Backspace => {
            if app.input_cursor > 0 {
                app.input_cursor -= 1;
                app.input_buffer.remove(app.input_cursor);
            }
        }
        KeyCode::Delete => {
            if app.input_cursor < app.input_buffer.len() {
                app.input_buffer.remove(app.input_cursor);
            }
        }
        KeyCode::Left => {
            app.input_cursor = app.input_cursor.saturating_sub(1);
        }
        KeyCode::Right => {
            app.input_cursor = (app.input_cursor + 1).min(app.input_buffer.len());
        }
        KeyCode::Home => {
            app.input_cursor = 0;
        }
        KeyCode::End => {
            app.input_cursor = app.input_buffer.len();
        }
        KeyCode::Enter => {
            if let Some(idx) = app.editing_task {
                if idx < app.tasks.len() {
                    app.tasks[idx].title = app.input_buffer.clone();
                    let _ = app.update_task(idx).ok();
                }
            } else {
                let task = Task::new(Ulid::new().to_string(), app.input_buffer.clone());
                let _ = app.add_task(task).ok();
            }
            app.mode = Mode::Normal;
            app.input_buffer.clear();
            app.input_cursor = 0;
            app.editing_task = None;
        }
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.input_buffer.clear();
            app.input_cursor = 0;
            app.editing_task = None;
        }
        _ => {}
    }
}

fn handle_popup_mode(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter | KeyCode::Char('y') => {
            if let Some(idx) = app.pending_delete {
                let _ = app.delete_task(idx).ok();
                if app.selected_index >= app.tasks.len() && app.selected_index > 0 {
                    app.selected_index -= 1;
                }
            }
            app.popup_visible = false;
            app.popup_message.clear();
            app.pending_delete = None;
        }
        KeyCode::Esc | KeyCode::Char('n') => {
            app.popup_visible = false;
            app.popup_message.clear();
            app.pending_delete = None;
        }
        _ => {}
    }
}
