use super::WorkspaceService;
use std::io::{Read, Write};
const LIMIT: u64 = 5 * 1024 * 1024;

pub fn read(svc: &WorkspaceService, path: &str) -> Result<String, String> {
    let path = svc.resolve_path(path).map_err(|e| e.to_string())?;
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() { return Err("Choose a text file".into()); }
    let mut bytes = Vec::new();
    file.take(LIMIT + 1).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > LIMIT { return Err("Editor supports files up to 5 MiB".into()); }
    if bytes.contains(&0) { return Err("Binary files cannot be edited".into()); }
    String::from_utf8(bytes).map_err(|_| "Editor supports UTF-8 text files".into())
}

pub fn save(svc: &WorkspaceService, path: &str, text: &str, expected: &str) -> Result<(), String> {
    if text.len() as u64 > LIMIT { return Err("Editor supports files up to 5 MiB".into()); }
    let resolved = svc.resolve_path(path).map_err(|e| e.to_string())?;
    if read(svc, path)? != expected { return Err("File changed on disk. Reload it before saving; your edits are preserved.".into()); }
    let permissions = std::fs::metadata(&resolved).map_err(|e| e.to_string())?.permissions();
    if permissions.readonly() { return Err("File is read-only".into()); }
    let mut staged = tempfile::NamedTempFile::new_in(resolved.parent().ok_or("Missing parent folder")?).map_err(|e| e.to_string())?;
    staged.as_file().set_permissions(permissions).map_err(|e| e.to_string())?;
    staged.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
    staged.as_file().sync_all().map_err(|e| e.to_string())?;
    if svc.resolve_path(path).map_err(|e| e.to_string())? != resolved || read(svc, path)? != expected {
        return Err("File changed during save. Your edits are preserved.".into());
    }
    staged.persist(resolved).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn workspace_read_file(svc: tauri::State<'_, WorkspaceService>, path: String) -> Result<String, String> { read(&svc, &path) }
#[tauri::command]
pub fn workspace_save_file(svc: tauri::State<'_, WorkspaceService>, path: String, text: String, expected: String) -> Result<(), String> { save(&svc, &path, &text, &expected) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip_and_conflict() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("file.txt");
        std::fs::write(&path, "hello\r\n世界").unwrap();
        let svc = WorkspaceService::new(); svc.open(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(read(&svc, "file.txt").unwrap(), "hello\r\n世界");
        save(&svc, "file.txt", "new\r\n", "hello\r\n世界").unwrap();
        assert!(save(&svc, "file.txt", "lost", "hello\r\n世界").is_err());
        assert_eq!(std::fs::read_to_string(path).unwrap(), "new\r\n");
        assert!(read(&svc, "../outside").is_err());
    }
    #[test]
    fn rejects_binary_and_large_files() {
        let dir = tempfile::tempdir().unwrap();
        let svc = WorkspaceService::new(); svc.open(dir.path().to_str().unwrap()).unwrap();
        std::fs::write(dir.path().join("binary"), [0, 1, 2]).unwrap();
        assert!(read(&svc, "binary").is_err());
        std::fs::write(dir.path().join("large"), vec![b'a'; LIMIT as usize + 1]).unwrap();
        assert!(read(&svc, "large").is_err());
    }
}
