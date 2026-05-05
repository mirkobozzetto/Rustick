use std::path::PathBuf;
use rusqlite::{Connection, params};
use crate::error::Result;
use crate::model::task::{Task, TaskStatus};
use crate::model::reminder::Reminder;

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn new(path: PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)?;
        let store = Self { conn };
        store.init_tables()?;
        Ok(store)
    }

    fn init_tables(&self) -> Result<()> {
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                body TEXT,
                status TEXT NOT NULL DEFAULT 'todo',
                priority INTEGER NOT NULL DEFAULT 3,
                due_at TEXT,
                created_at TEXT NOT NULL,
                tags TEXT NOT NULL DEFAULT '[]'
            );
            CREATE TABLE IF NOT EXISTS reminders (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                trigger_at TEXT NOT NULL,
                recurrence TEXT,
                acknowledged INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS session (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
        ")?;
        Ok(())
    }

    pub fn load_tasks(&self) -> Result<Vec<Task>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, body, status, priority, due_at, created_at, tags FROM tasks ORDER BY created_at DESC"
        )?;

        let tasks = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let body: Option<String> = row.get(2)?;
            let status_str: String = row.get(3)?;
            let priority: u8 = row.get(4)?;
            let due_at_str: Option<String> = row.get(5)?;
            let created_at_str: String = row.get(6)?;
            let tags_json: String = row.get(7)?;

            let status = TaskStatus::from_str(&status_str);
            let due_at = due_at_str.as_ref().and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&chrono::Local))
            });
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Local))
                .unwrap_or_else(chrono::Local::now);

            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

            Ok((id, title, body, status, priority, due_at, created_at, tags))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        let mut result = Vec::new();
        for (id, title, body, status, priority, due_at, created_at, tags) in tasks {
            let reminders = self.load_reminders(&id)?;
            result.push(Task {
                id,
                title,
                body,
                status,
                priority,
                due_at,
                created_at,
                tags,
                reminders,
            });
        }

        Ok(result)
    }

    pub fn save_task(&self, task: &Task) -> Result<()> {
        let tags_json = serde_json::to_string(&task.tags)?;
        let due_at_str = task.due_at.as_ref().map(|dt| dt.to_rfc3339());
        let created_at_str = task.created_at.to_rfc3339();

        self.conn.execute(
            "INSERT INTO tasks (id, title, body, status, priority, due_at, created_at, tags)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &task.id,
                &task.title,
                &task.body,
                task.status.to_str(),
                task.priority,
                due_at_str,
                created_at_str,
                tags_json,
            ],
        )?;

        for reminder in &task.reminders {
            self.save_reminder(&task.id, reminder)?;
        }

        Ok(())
    }

    pub fn update_task(&self, task: &Task) -> Result<()> {
        let tags_json = serde_json::to_string(&task.tags)?;
        let due_at_str = task.due_at.as_ref().map(|dt| dt.to_rfc3339());
        let created_at_str = task.created_at.to_rfc3339();

        self.conn.execute(
            "UPDATE tasks SET title = ?1, body = ?2, status = ?3, priority = ?4, due_at = ?5, created_at = ?6, tags = ?7
             WHERE id = ?8",
            params![
                &task.title,
                &task.body,
                task.status.to_str(),
                task.priority,
                due_at_str,
                created_at_str,
                tags_json,
                &task.id,
            ],
        )?;

        self.conn.execute("DELETE FROM reminders WHERE task_id = ?1", params![&task.id])?;
        for reminder in &task.reminders {
            self.save_reminder(&task.id, reminder)?;
        }

        Ok(())
    }

    pub fn delete_task(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn load_reminders(&self, task_id: &str) -> Result<Vec<Reminder>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, task_id, trigger_at, recurrence, acknowledged FROM reminders WHERE task_id = ?1"
        )?;

        let reminders = stmt.query_map(params![task_id], |row| {
            let id: String = row.get(0)?;
            let task_id: String = row.get(1)?;
            let trigger_at_str: String = row.get(2)?;
            let recurrence: Option<String> = row.get(3)?;
            let acknowledged: i32 = row.get(4)?;

            let trigger_at = chrono::DateTime::parse_from_rfc3339(&trigger_at_str)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Local))
                .unwrap_or_else(chrono::Local::now);

            Ok(Reminder {
                id,
                task_id,
                trigger_at,
                recurrence,
                acknowledged: acknowledged != 0,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(reminders)
    }

    pub fn save_reminder(&self, task_id: &str, reminder: &Reminder) -> Result<()> {
        let trigger_at_str = reminder.trigger_at.to_rfc3339();

        self.conn.execute(
            "INSERT INTO reminders (id, task_id, trigger_at, recurrence, acknowledged)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                &reminder.id,
                task_id,
                trigger_at_str,
                &reminder.recurrence,
                if reminder.acknowledged { 1 } else { 0 },
            ],
        )?;

        Ok(())
    }

    pub fn update_reminder(&self, reminder: &Reminder) -> Result<()> {
        let trigger_at_str = reminder.trigger_at.to_rfc3339();

        self.conn.execute(
            "UPDATE reminders SET trigger_at = ?1, recurrence = ?2, acknowledged = ?3 WHERE id = ?4",
            params![
                trigger_at_str,
                &reminder.recurrence,
                if reminder.acknowledged { 1 } else { 0 },
                &reminder.id,
            ],
        )?;

        Ok(())
    }

    pub fn delete_reminder(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM reminders WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn save_session(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO session (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn load_session(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT value FROM session WHERE key = ?1")?;
        let value = stmt.query_row(params![key], |row| row.get(0)).ok();
        Ok(value)
    }
}
