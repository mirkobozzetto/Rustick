use chrono::Local;
use rustick::model::filter::{extract_tags, fuzzy_match, Filter, FilterKind};
use rustick::model::reminder::Reminder;
use rustick::model::task::{Task, TaskStatus};
use rustick::event::scheduler::{check_reminders, format_relative_time, next_reminder};
use rustick::store::Store;
use tempfile::TempDir;

fn create_temp_store() -> (Store, TempDir) {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let store = Store::new(db_path).expect("failed to create store");
    (store, temp_dir)
}

#[test]
fn test_task_creation() {
    let task = Task::new("task-1".to_string(), "Fix login bug".to_string());
    assert_eq!(task.id, "task-1");
    assert_eq!(task.title, "Fix login bug");
    assert_eq!(task.status, TaskStatus::Todo);
    assert_eq!(task.priority, 0);
    assert!(task.body.is_none());
    assert!(task.due_at.is_none());
    assert!(task.tags.is_empty());
    assert!(task.reminders.is_empty());
}

#[test]
fn test_task_created_at_is_set() {
    let before = Local::now();
    let task = Task::new("task-1".to_string(), "Test".to_string());
    let after = Local::now();
    assert!(task.created_at >= before);
    assert!(task.created_at <= after);
}

#[test]
fn test_task_status_transitions() {
    let mut task = Task::new("task-1".to_string(), "Test".to_string());
    assert_eq!(task.status, TaskStatus::Todo);

    task.status = TaskStatus::Done;
    assert_eq!(task.status, TaskStatus::Done);

    task.status = TaskStatus::Archived;
    assert_eq!(task.status, TaskStatus::Archived);

    task.status = TaskStatus::Todo;
    assert_eq!(task.status, TaskStatus::Todo);
}

#[test]
fn test_task_status_to_str() {
    assert_eq!(TaskStatus::Todo.to_str(), "todo");
    assert_eq!(TaskStatus::Done.to_str(), "done");
    assert_eq!(TaskStatus::Archived.to_str(), "archived");
}

#[test]
fn test_task_status_from_str() {
    assert_eq!(TaskStatus::from_str("todo"), TaskStatus::Todo);
    assert_eq!(TaskStatus::from_str("done"), TaskStatus::Done);
    assert_eq!(TaskStatus::from_str("archived"), TaskStatus::Archived);
    assert_eq!(TaskStatus::from_str("unknown"), TaskStatus::Todo);
}

#[test]
fn test_store_save_and_load_task() {
    let (store, _temp_dir) = create_temp_store();

    let mut task = Task::new("task-1".to_string(), "Fix login bug".to_string());
    task.status = TaskStatus::Done;
    task.priority = 5;
    task.body = Some("Need to handle edge case".to_string());
    task.tags = vec!["urgent".to_string(), "backend".to_string()];

    store.save_task(&task).expect("failed to save task");

    let loaded = store.load_tasks().expect("failed to load tasks");
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, "task-1");
    assert_eq!(loaded[0].title, "Fix login bug");
    assert_eq!(loaded[0].status, TaskStatus::Done);
    assert_eq!(loaded[0].priority, 5);
    assert_eq!(loaded[0].body, Some("Need to handle edge case".to_string()));
    assert_eq!(loaded[0].tags, vec!["urgent".to_string(), "backend".to_string()]);
}

#[test]
fn test_store_update_task() {
    let (store, _temp_dir) = create_temp_store();

    let mut task = Task::new("task-1".to_string(), "Original title".to_string());
    store.save_task(&task).expect("failed to save task");

    task.title = "Updated title".to_string();
    task.status = TaskStatus::Done;
    task.priority = 8;
    store.update_task(&task).expect("failed to update task");

    let loaded = store.load_tasks().expect("failed to load tasks");
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].title, "Updated title");
    assert_eq!(loaded[0].status, TaskStatus::Done);
    assert_eq!(loaded[0].priority, 8);
}

#[test]
fn test_store_delete_task() {
    let (store, _temp_dir) = create_temp_store();

    let task = Task::new("task-1".to_string(), "Delete me".to_string());
    store.save_task(&task).expect("failed to save task");

    let loaded = store.load_tasks().expect("failed to load tasks");
    assert_eq!(loaded.len(), 1);

    store.delete_task("task-1").expect("failed to delete task");

    let loaded = store.load_tasks().expect("failed to load tasks");
    assert_eq!(loaded.len(), 0);
}

#[test]
fn test_store_save_and_load_reminders() {
    let (store, _temp_dir) = create_temp_store();

    let mut task = Task::new("task-1".to_string(), "Task with reminder".to_string());
    let reminder = Reminder::new(
        "rem-1".to_string(),
        "task-1".to_string(),
        Local::now(),
    );
    task.reminders.push(reminder);

    store.save_task(&task).expect("failed to save task");

    let loaded = store.load_tasks().expect("failed to load tasks");
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].reminders.len(), 1);
    assert_eq!(loaded[0].reminders[0].id, "rem-1");
}

#[test]
fn test_store_session_save_load() {
    let (store, _temp_dir) = create_temp_store();

    store.save_session("key1", "value1").expect("failed to save session");

    let loaded = store.load_session("key1").expect("failed to load session");
    assert_eq!(loaded, Some("value1".to_string()));
}

#[test]
fn test_store_session_update() {
    let (store, _temp_dir) = create_temp_store();

    store.save_session("key1", "value1").expect("failed to save session");
    store.save_session("key1", "value2").expect("failed to save session");

    let loaded = store.load_session("key1").expect("failed to load session");
    assert_eq!(loaded, Some("value2".to_string()));
}

#[test]
fn test_store_session_nonexistent() {
    let (store, _temp_dir) = create_temp_store();

    let loaded = store.load_session("nonexistent").expect("failed to load session");
    assert_eq!(loaded, None);
}

#[test]
fn test_fuzzy_match_exact() {
    assert!(fuzzy_match("fix", "fix login bug"));
    assert!(fuzzy_match("Fix", "Fix login bug"));
}

#[test]
fn test_fuzzy_match_fuzzy() {
    assert!(fuzzy_match("flb", "Fix login bug"));
    assert!(fuzzy_match("fb", "Fix login bug"));
    assert!(fuzzy_match("fib", "Fix login bug"));
}

#[test]
fn test_fuzzy_match_no_match() {
    assert!(!fuzzy_match("xyz", "Fix login bug"));
    assert!(!fuzzy_match("bug fix", "Fix login bug"));
}

#[test]
fn test_fuzzy_match_empty_query() {
    assert!(fuzzy_match("", "anything"));
    assert!(fuzzy_match("", ""));
}

#[test]
fn test_fuzzy_match_case_insensitive() {
    assert!(fuzzy_match("FLB", "fix login bug"));
    assert!(fuzzy_match("FlB", "Fix Login Bug"));
}

#[test]
fn test_extract_tags() {
    let tags = extract_tags("Fix #urgent #backend bug");
    assert_eq!(tags, vec!["urgent", "backend"]);
}

#[test]
fn test_extract_tags_empty() {
    let tags = extract_tags("No tags here");
    assert!(tags.is_empty());
}

#[test]
fn test_extract_tags_single() {
    let tags = extract_tags("Fix #urgent");
    assert_eq!(tags, vec!["urgent"]);
}

#[test]
fn test_extract_tags_with_special_chars() {
    let tags = extract_tags("Fix #bug-fix #v2");
    assert_eq!(tags, vec!["bug-fix", "v2"]);
}

#[test]
fn test_filter_all() {
    let filter = Filter::new();
    let task = Task::new("1".to_string(), "Test".to_string());
    assert!(filter.matches(&task));
}

#[test]
fn test_filter_inactive() {
    let filter = Filter {
        active: false,
        kind: FilterKind::Priority(5),
    };
    let task = Task::new("1".to_string(), "Test".to_string());
    assert!(filter.matches(&task));
}

#[test]
fn test_filter_priority() {
    let filter = Filter {
        active: true,
        kind: FilterKind::Priority(5),
    };
    let mut task1 = Task::new("1".to_string(), "Test".to_string());
    task1.priority = 5;

    let mut task2 = Task::new("2".to_string(), "Test".to_string());
    task2.priority = 3;

    assert!(filter.matches(&task1));
    assert!(!filter.matches(&task2));
}

#[test]
fn test_filter_tag() {
    let filter = Filter {
        active: true,
        kind: FilterKind::Tag("urgent".to_string()),
    };
    let mut task1 = Task::new("1".to_string(), "Test".to_string());
    task1.tags = vec!["urgent".to_string()];

    let task2 = Task::new("2".to_string(), "Test".to_string());

    assert!(filter.matches(&task1));
    assert!(!filter.matches(&task2));
}

#[test]
fn test_filter_status() {
    let filter = Filter {
        active: true,
        kind: FilterKind::Status(TaskStatus::Done),
    };
    let mut task1 = Task::new("1".to_string(), "Test".to_string());
    task1.status = TaskStatus::Done;

    let task2 = Task::new("2".to_string(), "Test".to_string());

    assert!(filter.matches(&task1));
    assert!(!filter.matches(&task2));
}

#[test]
fn test_scheduler_check_reminders_past() {
    let mut task = Task::new("task-1".to_string(), "Test".to_string());
    let past = Local::now() - chrono::Duration::hours(1);
    let reminder = Reminder::new("rem-1".to_string(), "task-1".to_string(), past);
    task.reminders.push(reminder);

    let triggered = check_reminders(&[task]);
    assert_eq!(triggered, vec!["task-1"]);
}

#[test]
fn test_scheduler_check_reminders_future() {
    let mut task = Task::new("task-1".to_string(), "Test".to_string());
    let future = Local::now() + chrono::Duration::hours(1);
    let reminder = Reminder::new("rem-1".to_string(), "task-1".to_string(), future);
    task.reminders.push(reminder);

    let triggered = check_reminders(&[task]);
    assert!(triggered.is_empty());
}

#[test]
fn test_scheduler_check_reminders_acknowledged() {
    let mut task = Task::new("task-1".to_string(), "Test".to_string());
    let past = Local::now() - chrono::Duration::hours(1);
    let mut reminder = Reminder::new("rem-1".to_string(), "task-1".to_string(), past);
    reminder.acknowledged = true;
    task.reminders.push(reminder);

    let triggered = check_reminders(&[task]);
    assert!(triggered.is_empty());
}

#[test]
fn test_scheduler_next_reminder_exists() {
    let mut task = Task::new("task-1".to_string(), "Test".to_string());
    let future = Local::now() + chrono::Duration::hours(1);
    let reminder = Reminder::new("rem-1".to_string(), "task-1".to_string(), future);
    task.reminders.push(reminder);

    let result = next_reminder(&[task.clone()]);
    assert!(result.is_some());
    let (task_id, _time) = result.unwrap();
    assert_eq!(task_id, "task-1");
}

#[test]
fn test_scheduler_next_reminder_empty() {
    let task = Task::new("task-1".to_string(), "Test".to_string());
    let result = next_reminder(&[task]);
    assert!(result.is_none());
}

#[test]
fn test_scheduler_next_reminder_past() {
    let mut task = Task::new("task-1".to_string(), "Test".to_string());
    let past = Local::now() - chrono::Duration::hours(1);
    let reminder = Reminder::new("rem-1".to_string(), "task-1".to_string(), past);
    task.reminders.push(reminder);

    let result = next_reminder(&[task]);
    assert!(result.is_none());
}

#[test]
fn test_scheduler_next_reminder_earliest() {
    let mut task1 = Task::new("task-1".to_string(), "Test".to_string());
    let future1 = Local::now() + chrono::Duration::hours(2);
    let reminder1 = Reminder::new("rem-1".to_string(), "task-1".to_string(), future1);
    task1.reminders.push(reminder1);

    let mut task2 = Task::new("task-2".to_string(), "Test".to_string());
    let future2 = Local::now() + chrono::Duration::hours(1);
    let reminder2 = Reminder::new("rem-2".to_string(), "task-2".to_string(), future2);
    task2.reminders.push(reminder2);

    let result = next_reminder(&[task1, task2]);
    assert!(result.is_some());
    let (task_id, _time) = result.unwrap();
    assert_eq!(task_id, "task-2");
}

#[test]
fn test_scheduler_format_relative_time_seconds_ago() {
    let past = Local::now() - chrono::Duration::seconds(30);
    let formatted = format_relative_time(past);
    assert_eq!(formatted, "30s ago");
}

#[test]
fn test_scheduler_format_relative_time_minutes_ago() {
    let past = Local::now() - chrono::Duration::minutes(5);
    let formatted = format_relative_time(past);
    assert_eq!(formatted, "5m ago");
}

#[test]
fn test_scheduler_format_relative_time_hours_ago() {
    let past = Local::now() - chrono::Duration::hours(2);
    let formatted = format_relative_time(past);
    assert_eq!(formatted, "2h ago");
}

#[test]
fn test_scheduler_format_relative_time_days_ago() {
    let past = Local::now() - chrono::Duration::days(3);
    let formatted = format_relative_time(past);
    assert_eq!(formatted, "3d ago");
}

#[test]
fn test_scheduler_format_relative_time_seconds_future() {
    let future = Local::now() + chrono::Duration::seconds(30);
    let formatted = format_relative_time(future);
    assert!(formatted.contains("in ") && formatted.contains("s"));
}

#[test]
fn test_scheduler_format_relative_time_minutes_future() {
    let future = Local::now() + chrono::Duration::minutes(5);
    let formatted = format_relative_time(future);
    assert!(formatted.contains("in ") && formatted.contains("m"));
}

#[test]
fn test_scheduler_format_relative_time_hours_future() {
    let future = Local::now() + chrono::Duration::hours(2);
    let formatted = format_relative_time(future);
    assert!(formatted.contains("in ") && formatted.contains("h"));
}

#[test]
fn test_scheduler_format_relative_time_tomorrow() {
    let future = Local::now() + chrono::Duration::days(1) + chrono::Duration::hours(2);
    let formatted = format_relative_time(future);
    assert_eq!(formatted, "tomorrow");
}

#[test]
fn test_scheduler_format_relative_time_days_future() {
    let future = Local::now() + chrono::Duration::days(7);
    let formatted = format_relative_time(future);
    assert!(formatted.contains("in ") && formatted.contains("d"));
}
