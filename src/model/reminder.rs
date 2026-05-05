use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub id: String,
    pub task_id: String,
    pub trigger_at: DateTime<Local>,
    pub recurrence: Option<String>,
    pub acknowledged: bool,
}

#[allow(dead_code)]
impl Reminder {
    pub fn new(id: String, task_id: String, trigger_at: DateTime<Local>) -> Self {
        Self {
            id,
            task_id,
            trigger_at,
            recurrence: None,
            acknowledged: false,
        }
    }
}
