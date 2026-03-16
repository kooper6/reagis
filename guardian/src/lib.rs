use std::path::{Path, PathBuf};
use std::fs;
use std::io;

pub struct ReagisGuard {
    root_path: PathBuf,
}

impl ReagisGuard {
    pub fn new(workspace_path: &str) -> Result<Self> {
        let root = Path::new(workspace_path).canonicalize()?;
        Ok(Self {root_path: root})
    }

    fn resolve_path(&self, agent_path: &str) -> Result<PathBuf, String> {
        let relative_path = agent_path.trim_start_matches('/');
        let full_path = self.root_path.join(relative_path);

        let actual_path = full_path.canonicalize()
            .map_err(|_| format!("Access denied, path doesn't exist or is invalid {}", agent_path))?;

        if actual_path.starts_with(&self.root_path) {
            Ok(actual_path)
        } else {
            Err(format!("Access denied, beach attempted {}", agent_path))
        }
    }

    pub fn read_file(&self, agent_path: &str) -> Result<String, String> {
        let safe_path = self.resolve_path(agent_path)?;
        fs::read_to_strint(safe_path).map_err(|e| e.to_string())
    }

    pub fn write_file(&self, agent_path: &str, content: &str) -> Result<(), String> {
        let path = Path::new(agent_path);
        let parent = path.parent().unwrap_or(Path::new(""));

        let safe_parent_dif = self.root_path.join(parent);

        if !safe_parent_dif.starts_with(&self.root_path) {
            fs::create_dir_all(&safe_parent_dif)
                .map_err(|e| format!("Failed to create directory structure {}", e))?;
        }

        let canonical_parent = safe_parent_dif.caninicalize()
            .map_err(|_| "Invalid directory path")?;

        if !canonical_parent.starts_with(&self.root_path) {
            return Err("Security breach".to_string());
        }

        let file_ame = path.file_name()
            .ok_or("Invalid filename")?;
        let final_path = canonical_parent.join(file_ame);

        fs::write(&final_path, content)
            .map_err(|e| format!("Failed to wirte file: {}", e))?;

        Ok(format!("Successful write to {}", agent_path))
    }
}