use crate::model::Reminder;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    Done,
    Archived,
}

impl TaskStatus {
    pub fn to_str(&self) -> &str {
        match self {
            TaskStatus::Todo => "todo",
            TaskStatus::Done => "done",
            TaskStatus::Archived => "archived",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "done" => TaskStatus::Done,
            "archived" => TaskStatus::Archived,
            _ => TaskStatus::Todo,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub body: Option<String>,
    pub status: TaskStatus,
    pub priority: u8,
    pub due_at: Option<DateTime<Local>>,
    pub created_at: DateTime<Local>,
    pub tags: Vec<String>,
    pub reminders: Vec<Reminder>,
}

impl Task {
    pub fn new(id: String, title: String) -> Self {
        Self {
            id,
            title,
            body: None,
            status: TaskStatus::Todo,
            priority: 0,
            due_at: None,
            created_at: Local::now(),
            tags: Vec::new(),
            reminders: Vec::new(),
        }
    }
}
