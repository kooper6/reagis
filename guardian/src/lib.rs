use std::path::{Path, PathBuf};
use std::fs;
use std::io;

pub struct ReagisGuard {
    root_path: PathBuf,
}

impl ReagisGuard {
    pub fn new(workspace_path: &str) -> io::Result<Self> {
        let root = Path::new(workspace_path);
        if !root.exists() {
            fs::create_dir_all(root)?;
        }
        let canonical_root = root.canonicalize()?;
        Ok(Self { root_path: canonical_root })
    }

    fn resolve_path(&self, agent_path: &str) -> Result<PathBuf, String> {
        if agent_path.starts_with('/') || agent_path.contains("..") {
             // Let canonicalize handle specifics, but simple heuristic first
        }

        let relative_path = Path::new(agent_path);

        let relative_path = if relative_path.has_root() {
            relative_path.strip_prefix("/").unwrap_or(relative_path)
        } else {
            relative_path
        };

        let full_path = self.root_path.join(relative_path);

        if full_path.exists() {
            let actual_path = full_path.canonicalize()
                .map_err(|e| format!("Path error: {}", e))?;

            if actual_path.starts_with(&self.root_path) {
                Ok(actual_path)
            } else {
                Err(format!("Access denied: {} escapes root", agent_path))
            }
        } else {
             Err(format!("Path does not exist: {}", agent_path))
        }
    }

    pub fn read_file(&self, agent_path: &str) -> Result<String, String> {
        let safe_path = self.resolve_path(agent_path)?;
        fs::read_to_string(safe_path).map_err(|e| e.to_string())
    }

    pub fn write_file(&self, agent_path: &str, content: &str) -> Result<String, String> {
        let relative_path = Path::new(agent_path)
            .strip_prefix("/").unwrap_or(Path::new(agent_path));

        let full_path = self.root_path.join(relative_path);

        // Ensure parent directory exists and is safe
        if let Some(parent) = full_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }
            // Canonicalize parent to check jail
            let real_parent = parent.canonicalize()
                .map_err(|e| format!("Invalid path: {}", e))?;

            if !real_parent.starts_with(&self.root_path) {
                 return Err(format!("Access denied: {} is outside sandbox", agent_path));
            }
        } else {
             return Err("Invalid path".to_string());
        }

        fs::write(&full_path, content)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(format!("Successfully wrote to {}", agent_path))
    }
}