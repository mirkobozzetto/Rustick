use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub id: String,
    pub trigger_at: DateTime<Local>,
    pub recurrence: Option<String>,
    pub acknowledged: bool,
}

impl Reminder {
    pub fn new(id: String, trigger_at: DateTime<Local>) -> Self {
        Self {
            id,
            trigger_at,
            recurrence: None,
            acknowledged: false,
        }
    }
}
