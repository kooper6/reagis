use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use ollama_rs::generation::chat::ChatMessage;

#[derive(Serialize, Deserialize, Clone)]
pub struct Session {
    pub id: String,
    pub history: Vec<ChatMessage>,
}

impl Session {
    pub fn new(id: String) -> Self {
        Self {
            id,
            history: Vec::new(),
        }
    }

    pub fn load(id: &str) -> Option<Self> {
        let path = format!("sessions/{}.json", id);
        if Path::new(&path).exists() {
            let data = fs::read_to_string(path).ok()?;
            serde_json::from_str(&data).ok()
        } else {
            None
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        fs::create_dir_all("sessions")?;
        let data = serde_json::to_string_pretty(self)?;
        fs::write(format!("sessions/{}.json", self.id), data)
    }
}

pub fn list_sessions() -> std::io::Result<Vec<String>> {
    let mut sessions = Vec::new();
    let dir = Path::new("sessions");
    if !dir.exists() {
        return Ok(sessions);
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                sessions.push(name.to_string());
            }
        }
    }
    Ok(sessions)
}