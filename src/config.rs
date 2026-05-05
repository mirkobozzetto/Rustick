use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybinding {
    pub key: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub keybindings: Vec<Keybinding>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            keybindings: Vec::new(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
