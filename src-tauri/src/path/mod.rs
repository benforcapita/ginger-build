/// Ginger Code — Path Handling (LLD 219)
/// Backend canonicalizes paths, resolves symlinks where needed, validates
/// roots, and never uses raw string concatenation for security decisions.

use std::path::{Path, PathBuf};

pub struct PathValidator;

impl PathValidator {
    /// Canonicalize a path, resolving symlinks. Returns None on failure.
    pub fn canonicalize(path: &Path) -> Option<PathBuf> {
        path.canonicalize().ok()
    }

    /// Check that `candidate` is inside `root` (after canonicalization).
    /// This is the security boundary for path traversal rejection.
    pub fn is_within(root: &Path, candidate: &Path) -> bool {
        let root_c = match root.canonicalize() {
            Ok(r) => r,
            Err(_) => return false,
        };
        let cand_c = match candidate.canonicalize() {
            Ok(c) => c,
            Err(_) => return false,
        };
        cand_c.starts_with(&root_c)
    }

    /// Join a relative path onto a root, rejecting traversal (..) escapes.
    pub fn safe_join(root: &Path, relative: &str) -> Option<PathBuf> {
        let rel = Path::new(relative);
        if rel.is_absolute() {
            return None;
        }
        let joined = root.join(rel);
        if Self::is_within(root, &joined) {
            Some(joined)
        } else {
            None
        }
    }
}

impl Default for PathValidator {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_traversal() {
        let sandbox = tempfile::tempdir().unwrap();
        let root = sandbox.path().join("project");
        std::fs::create_dir(&root).unwrap();
        std::fs::write(sandbox.path().join("outside"), "outside").unwrap();
        assert!(PathValidator::safe_join(&root, "../outside").is_none());
    }

    #[test]
    fn accepts_inside_path() {
        let sandbox = tempfile::tempdir().unwrap();
        let root = sandbox.path();
        std::fs::create_dir(root.join("src")).unwrap();
        // Canonicalization requires an existing target, not only its parent.
        std::fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
        assert!(PathValidator::safe_join(root, "src/main.rs").is_some());
    }

    #[test]
    fn rejects_absolute() {
        let sandbox = tempfile::tempdir().unwrap();
        assert!(PathValidator::safe_join(sandbox.path(), "/etc/passwd").is_none());
    }
}
