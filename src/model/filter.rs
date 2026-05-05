use crate::model::task::{Task, TaskStatus};

#[derive(Debug, Clone)]
pub enum FilterKind {
    All,
    Priority(u8),
    Tag(String),
    Status(TaskStatus),
    Today,
    ThisWeek,
}

#[derive(Debug, Clone)]
pub struct Filter {
    pub active: bool,
    pub kind: FilterKind,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            active: false,
            kind: FilterKind::All,
        }
    }

    pub fn matches(&self, task: &Task) -> bool {
        if !self.active {
            return true;
        }
        match &self.kind {
            FilterKind::All => true,
            FilterKind::Priority(p) => task.priority == *p,
            FilterKind::Tag(tag) => task.tags.iter().any(|t| t == tag),
            FilterKind::Status(status) => task.status == *status,
            FilterKind::Today => {
                use chrono::Local;
                task.due_at
                    .map(|d| d.date_naive() == Local::now().date_naive())
                    .unwrap_or(false)
            }
            FilterKind::ThisWeek => {
                use chrono::{Datelike, Local};
                task.due_at
                    .map(|d| {
                        let now = Local::now();
                        let task_week = d.iso_week().week();
                        let current_week = now.iso_week().week();
                        task_week == current_week && d.year() == now.year()
                    })
                    .unwrap_or(false)
            }
        }
    }
}

pub fn extract_tags(title: &str) -> Vec<String> {
    title
        .split_whitespace()
        .filter(|w| w.starts_with('#') && w.len() > 1)
        .map(|w| w[1..].to_string())
        .collect()
}

pub fn fuzzy_match(query: &str, text: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let query_lower = query.to_lowercase();
    let text_lower = text.to_lowercase();

    let mut query_chars = query_lower.chars().peekable();
    for c in text_lower.chars() {
        if query_chars.peek() == Some(&c) {
            query_chars.next();
        }
        if query_chars.peek().is_none() {
            return true;
        }
    }
    query_chars.peek().is_none()
}
