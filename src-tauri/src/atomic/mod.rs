/// Ginger Code — Atomic Filesystem Writes (LLD 225)
/// Generated manifests/config are written temp → synced where appropriate →
/// atomically renamed. Partial artifacts never appear at final paths.

use std::io::Write;
use std::path::Path;

pub struct AtomicWriter;

impl AtomicWriter {
    /// Write `content` to `path` atomically: write to a temp file in the same
    /// directory, sync, then rename over the target.
    pub fn write_atomic(path: &Path, content: &[u8]) -> std::io::Result<()> {
        let dir = path.parent().unwrap_or(Path::new("."));
        let file_name = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "tmp".to_string());
        let temp_path = dir.join(format!(".{}.tmp-{}", file_name, std::process::id()));

        {
            let mut f = std::fs::File::create(&temp_path)?;
            f.write_all(content)?;
            f.sync_all()?;
        }

        std::fs::rename(&temp_path, path)?;
        Ok(())
    }
}

impl Default for AtomicWriter {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_atomically() {
        let dir = std::env::temp_dir().join(format!("ginger-atomic-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("config.toml");
        AtomicWriter::write_atomic(&target, b"key = \"value\"").unwrap();
        let content = std::fs::read_to_string(&target).unwrap();
        assert_eq!(content, "key = \"value\"");
        // No temp files left behind
        let leftovers: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp-"))
            .collect();
        assert!(leftovers.is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}