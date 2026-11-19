use std::{collections::HashMap, io, path::Path};

use serde::{Deserialize, Serialize};

use crate::{domain::task::Task, infrastructure::storage};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Registry {
    pub tasks: HashMap<String, Task>,
}

impl Registry {
    pub fn load(path: &Path) -> io::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)?;

        serde_json::from_str(&content).map_err(|e| {
            io::Error::other(format!("無法解析 {}：{e}", path.display()))
        })
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let content = serde_json::to_string_pretty(self).map_err(io::Error::other)?;

        storage::write_atomic(path, content.as_bytes())
    }
}
