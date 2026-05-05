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
                    Mode::Search => handle_search_mode(app, key.code),
                    Mode::TimeInput => handle_time_input_mode(app, key.code),
                    Mode::Focus => {}
                }
            }
        }
        Event::Tick => {
            app.tick_count = app.tick_count.wrapping_add(1);

            let triggered = crate::event::scheduler::check_reminders(&app.tasks);
            if !triggered.is_empty() {
                print!("\x07");

                if let Some(task_id) = triggered.first() {
                    if let Some(task) = app.tasks.iter().find(|t| t.id == *task_id) {
                        app.alert_task = Some(task.title.clone());
                        app.alert_tick = app.tick_count;

                        app.popup_message = format!("REMINDER: {}", task.title);
                        app.popup_visible = true;
                    }
                }

                for task in &mut app.tasks {
                    for reminder in &mut task.reminders {
                        if !reminder.acknowledged && reminder.trigger_at <= chrono::Local::now() {
                            reminder.acknowledged = true;
                        }
                    }
                }
            }
        }
        Event::Key(_) => {}
        Event::Resize(_, _) => {}
        Event::Reminder(_) => {}
    }
}

fn handle_normal_mode(app: &mut App, code: KeyCode) {
    match code {
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
        KeyCode::Char('q') => {
            app.running = false;
        }
        KeyCode::Char('?') => {
            app.popup_message = "j/↓: Down | k/↑: Up | h/←: Left | l/→: Right | Tab: Next Panel\nn: New | e: Edit | Enter: Edit Body | t: Set Time\nSpace: Toggle | d: Delete | 1-4: Priority | /: Search | q: Quit".to_string();
            app.popup_visible = true;
        }
        KeyCode::Char('/') => {
            app.mode = Mode::Search;
            app.input_buffer.clear();
            app.input_cursor = 0;
        }
        _ => match app.active_panel {
            Panel::Main => handle_main_panel(app, code),
            Panel::Sidebar => handle_sidebar_panel(app, code),
            Panel::Timeline => handle_timeline_panel(app, code),
        },
    }
}

fn handle_main_panel(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('j') | KeyCode::Down => {
            app.selected_index = (app.selected_index + 1).min(app.tasks.len().saturating_sub(1));
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.selected_index = app.selected_index.saturating_sub(1);
        }
        KeyCode::Char('n') => {
            app.mode = Mode::Insert;
            app.input_buffer.clear();
            app.input_cursor = 0;
            app.editing_task = None;
            app.editing_body = false;
        }
        KeyCode::Char('e') => {
            if app.selected_index < app.tasks.len() {
                app.mode = Mode::Insert;
                app.input_buffer = app.tasks[app.selected_index].title.clone();
                app.input_cursor = app.input_buffer.len();
                app.editing_task = Some(app.selected_index);
                app.editing_body = false;
            }
        }
        KeyCode::Char('t') => {
            if app.selected_index < app.tasks.len() {
                app.mode = Mode::TimeInput;
                app.time_input_buffer.clear();
                app.time_input_cursor = 0;
            }
        }
        KeyCode::Enter => {
            if app.selected_index < app.tasks.len() {
                app.mode = Mode::Insert;
                app.editing_body = true;
                app.input_buffer = app.tasks[app.selected_index].body.clone().unwrap_or_default();
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
        KeyCode::Char(c @ '1'..='4') => {
            if app.selected_index < app.tasks.len() {
                app.tasks[app.selected_index].priority = c.to_digit(10).unwrap_or(3) as u8;
                let _ = app.update_task(app.selected_index).ok();
            }
        }
        _ => {}
    }
}

fn handle_sidebar_panel(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('j') | KeyCode::Down => {
            app.sidebar_index = (app.sidebar_index + 1).min(6);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.sidebar_index = app.sidebar_index.saturating_sub(1);
        }
        KeyCode::Enter => {
            app.active_panel = Panel::Main;
        }
        _ => {}
    }
}

fn handle_timeline_panel(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('j') | KeyCode::Down => {
            app.timeline_scroll = app.timeline_scroll.saturating_add(1).min(23);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.timeline_scroll = app.timeline_scroll.saturating_sub(1);
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
                    if app.editing_body {
                        app.tasks[idx].body = if app.input_buffer.is_empty() { None } else { Some(app.input_buffer.clone()) };
                    } else {
                        app.tasks[idx].title = app.input_buffer.clone();
                    }
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
            app.editing_body = false;
        }
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.input_buffer.clear();
            app.input_cursor = 0;
            app.editing_task = None;
            app.editing_body = false;
        }
        _ => {}
    }
}

fn handle_search_mode(app: &mut App, code: KeyCode) {
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
        KeyCode::Enter => {
            let query = app.input_buffer.clone();
            if !query.is_empty() {
                for (i, task) in app.tasks.iter().enumerate() {
                    if task.title.to_lowercase().contains(&query.to_lowercase()) {
                        app.selected_index = i;
                        break;
                    }
                }
            }
            app.mode = Mode::Normal;
            app.input_buffer.clear();
            app.input_cursor = 0;
        }
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.input_buffer.clear();
            app.input_cursor = 0;
        }
        KeyCode::Left => {
            app.input_cursor = app.input_cursor.saturating_sub(1);
        }
        KeyCode::Right => {
            app.input_cursor = (app.input_cursor + 1).min(app.input_buffer.len());
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

fn handle_time_input_mode(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char(c) => {
            app.time_input_buffer.insert(app.time_input_cursor, c);
            app.time_input_cursor += 1;
        }
        KeyCode::Backspace => {
            if app.time_input_cursor > 0 {
                app.time_input_cursor -= 1;
                app.time_input_buffer.remove(app.time_input_cursor);
            }
        }
        KeyCode::Left => {
            app.time_input_cursor = app.time_input_cursor.saturating_sub(1);
        }
        KeyCode::Right => {
            app.time_input_cursor = (app.time_input_cursor + 1).min(app.time_input_buffer.len());
        }
        KeyCode::Enter => {
            if let Some(due) = parse_time_input(&app.time_input_buffer) {
                if app.selected_index < app.tasks.len() {
                    app.tasks[app.selected_index].due_at = Some(due);

                    let reminder = crate::model::reminder::Reminder::new(
                        ulid::Ulid::new().to_string(),
                        app.tasks[app.selected_index].id.clone(),
                        due,
                    );
                    app.tasks[app.selected_index].reminders.push(reminder.clone());
                    let _ = app.store.save_reminder(&app.tasks[app.selected_index].id, &reminder);
                    let _ = app.update_task(app.selected_index).ok();
                }
            }
            app.mode = Mode::Normal;
            app.time_input_buffer.clear();
            app.time_input_cursor = 0;
        }
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.time_input_buffer.clear();
            app.time_input_cursor = 0;
        }
        _ => {}
    }
}

fn parse_time_input(input: &str) -> Option<chrono::DateTime<chrono::Local>> {
    let input = input.trim().to_lowercase();
    let now = chrono::Local::now();

    if input.starts_with('+') {
        let rest = &input[1..];
        if let Some(hours) = rest.strip_suffix('h').and_then(|s| s.parse::<i64>().ok()) {
            return Some(now + chrono::Duration::hours(hours));
        }
        if let Some(mins) = rest.strip_suffix('m').and_then(|s| s.parse::<i64>().ok()) {
            return Some(now + chrono::Duration::minutes(mins));
        }
    }

    if let Some((h, m)) = input.split_once(':') {
        if let (Ok(hour), Ok(min)) = (h.parse::<u32>(), m.parse::<u32>()) {
            if hour < 24 && min < 60 {
                let today = now.date_naive();
                let time = chrono::NaiveTime::from_hms_opt(hour, min, 0)?;
                let naive = today.and_time(time);
                return Some(chrono::TimeZone::from_local_datetime(&chrono::Local, &naive).single()?);
            }
        }
    }

    if input == "tomorrow" || input == "demain" {
        return Some(now + chrono::Duration::days(1));
    }

    None
}
