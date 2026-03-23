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

    /// Resolves path for READING (expects existence)
    fn resolve_existing_path(&self, agent_path: &str) -> Result<PathBuf, String> {
        let path = Path::new(agent_path);
        // Strip leading separator to treat as relative
        let relative = path.strip_prefix("/").unwrap_or(path);
        let full_path = self.root_path.join(relative);

        if !full_path.exists() {
            return Err(format!("Path does not exist: {}", agent_path));
        }

        let canonical = full_path.canonicalize()
            .map_err(|e| format!("Failed to resolve path: {}", e))?;

        if !canonical.starts_with(&self.root_path) {
            return Err(format!("Access denied: {} escapes root", agent_path));
        }

        Ok(canonical)
    }

    pub fn read_file(&self, agent_path: &str) -> Result<String, String> {
        let safe_path = self.resolve_existing_path(agent_path)?;
        fs::read_to_string(safe_path).map_err(|e| e.to_string())
    }

    pub fn write_file(&self, agent_path: &str, content: &str) -> Result<String, String> {
        let path = Path::new(agent_path);
        let relative = path.strip_prefix("/").unwrap_or(path);
        let full_path = self.root_path.join(relative);

        // Security Check: Verify nearest existing ancestor is safe
        // This prevents writing through a symlink that points outside
        let mut ancestor = full_path.parent();
        while let Some(p) = ancestor {
            if p.exists() {
                let real_path = p.canonicalize().map_err(|e| e.to_string())?;
                if !real_path.starts_with(&self.root_path) {
                     return Err(format!("Access denied: Path escapes sandbox via {}", p.display()));
                }
                break;
            }
            ancestor = p.parent();
        }

        // Check if full_path itself exists and is a symlink?
        // If it exists, we overwrite it. If it's a symlink to outside, we overwrite target outside!
        // So we MUST check if it exists and is safe.
        if full_path.exists() {
             let real_path = full_path.canonicalize().map_err(|e| e.to_string())?;
             if !real_path.starts_with(&self.root_path) {
                  return Err("Access denied: Target file is outside sandbox".to_string());
             }
        }

        // Create parent directories
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directories: {}", e))?;

            // Re-verify parent after creation (paranoia check for race conditions/symlinks created)
            let real_parent = parent.canonicalize().map_err(|e| e.to_string())?;
            if !real_parent.starts_with(&self.root_path) {
                return Err("Security breach: Parent directory escaped sandbox".to_string());
            }
        }

        fs::write(&full_path, content).map_err(|e| format!("Failed to write: {}", e))?;

        Ok(format!("Successfully wrote to {}", agent_path))
    }
}