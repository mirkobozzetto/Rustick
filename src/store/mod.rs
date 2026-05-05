use crate::error::Result;
use crate::model::Task;
use serde::{Deserialize, Serialize};

pub trait Store {
    fn save_task(&mut self, task: Task) -> Result<()>;
    fn load_tasks(&self) -> Result<Vec<Task>>;
    fn delete_task(&mut self, id: &str) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonStore {
    pub path: std::path::PathBuf,
}

impl JsonStore {
    pub fn new(path: std::path::PathBuf) -> Self {
        Self { path }
    }
}

impl Store for JsonStore {
    fn save_task(&mut self, _task: Task) -> Result<()> {
        Ok(())
    }

    fn load_tasks(&self) -> Result<Vec<Task>> {
        Ok(Vec::new())
    }

    fn delete_task(&mut self, _id: &str) -> Result<()> {
        Ok(())
    }
}
