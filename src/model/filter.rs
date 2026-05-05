use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub query: Option<String>,
    pub tags: Vec<String>,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            query: None,
            tags: Vec::new(),
        }
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::new()
    }
}
