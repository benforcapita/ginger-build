use super::WorkspaceError;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
pub struct FileIndex {
    pub entries: Vec<super::FileEntry>,
    pub truncated: bool,
    pub skipped: usize,
}

pub fn index(root: &Path, limit: usize) -> Result<FileIndex, WorkspaceError> {
    let root = root.canonicalize().map_err(|e| WorkspaceError::Inner(e.to_string()))?;
    let mut result = FileIndex { entries: vec![], truncated: false, skipped: 0 };
    let mut pending = std::collections::VecDeque::from([root.clone()]);
    let mut visited = 0;
    while let Some(directory) = pending.pop_front() {
        let read = match std::fs::read_dir(&directory) {
            Ok(read) => read,
            Err(_) => { result.skipped += 1; continue; }
        };
        for entry in read {
            visited += 1;
            if result.entries.len() >= limit || visited > limit.saturating_mul(4).max(1) {
                result.truncated = true;
                break;
            }
            let entry = match entry { Ok(entry) => entry, Err(_) => { result.skipped += 1; continue; } };
            let name = entry.file_name().to_string_lossy().to_string();
            if name == ".git" { continue; }
            let kind = match entry.file_type() { Ok(kind) => kind, Err(_) => { result.skipped += 1; continue; } };
            if kind.is_dir() && ["node_modules", "target", "dist", "build", ".next", ".venv"].contains(&name.as_str()) { continue; }
            // Never recurse through directory links (including cycles), and only
            // include file links whose canonical target remains in this workspace.
            let is_dir = kind.is_dir();
            if kind.is_symlink() {
                let Ok(target) = entry.path().canonicalize() else { continue; };
                if !target.starts_with(&root) || !target.is_file() { continue; }
            } else if !is_dir && !kind.is_file() { continue; }
            let path = entry.path().strip_prefix(&root).unwrap().to_string_lossy().to_string();
            if is_dir {
                if Path::new(&path).components().count() < 64 { pending.push_back(entry.path()); }
                else { result.skipped += 1; }
            }
            result.entries.push(super::FileEntry { name, path, is_dir });
        }
        if result.truncated { break; }
    }
    result.entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finds_nested_and_hidden_files_excluding_generated_folders() {
        let root = tempfile::tempdir().unwrap();
        for path in ["src/deep", "node_modules/pkg", ".git", "target/debug"] {
            std::fs::create_dir_all(root.path().join(path)).unwrap();
        }
        for path in ["src/deep/a file.rs", ".env.example", "node_modules/pkg/index.js", ".git/config", "target/debug/app"] {
            std::fs::write(root.path().join(path), "test").unwrap();
        }
        let result = index(root.path(), 100).unwrap();
        let paths: Vec<_> = result.entries.iter().map(|entry| entry.path.as_str()).collect();
        assert!(paths.contains(&"src/deep/a file.rs"));
        assert!(paths.contains(&".env.example"));
        assert!(!paths.contains(&"node_modules/pkg/index.js"));
        assert!(!paths.contains(&".git/config"));
        assert!(!result.truncated);
    }
    #[test]
    fn reports_bounded_results() {
        let root = tempfile::tempdir().unwrap();
        for i in 0..6 { std::fs::write(root.path().join(format!("{i}.rs")), "test").unwrap(); }
        let result = index(root.path(), 3).unwrap();
        assert_eq!(result.entries.len(), 3);
        assert!(result.truncated);
    }
    #[test]
    #[cfg(unix)]
    fn does_not_follow_external_links_or_directory_cycles() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("secret"), "test").unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("outside")).unwrap();
        std::os::unix::fs::symlink(root.path(), root.path().join("cycle")).unwrap();
        assert!(index(root.path(), 100).unwrap().entries.is_empty());
    }
}
