use super::*;
use std::time::{Duration, Instant};

fn wait_until(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !condition() {
        assert!(Instant::now() < deadline, "terminal operation timed out");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn preserves_early_output_and_exit_code() {
    let host = TerminalHost::default();
    let dir = tempfile::tempdir().unwrap();
    let id = host.launch(dir.path(), "/bin/sh", &["-c".into(), "printf ginger-ready; exit 7".into()], TerminalOwner::User, None).unwrap();
    wait_until(|| host.list().iter().any(|s| s.id == id && s.exited));
    let sessions = host.sessions.lock();
    let output = sessions[&id].output.lock();
    assert!(String::from_utf8_lossy(&output.history.iter().copied().collect::<Vec<_>>()).contains("ginger-ready"));
    assert_eq!(output.exit_code, Some(7));
}

#[test]
fn writes_input_and_terminates_session() {
    let host = TerminalHost::default();
    let dir = tempfile::tempdir().unwrap();
    let id = host.launch(dir.path(), "/bin/cat", &[], TerminalOwner::User, None).unwrap();
    host.resize(id, 30, 100).unwrap();
    host.write(id, b"ginger-input\n").unwrap();
    wait_until(|| {
        let sessions = host.sessions.lock();
        let output = sessions[&id].output.lock();
        String::from_utf8_lossy(&output.history.iter().copied().collect::<Vec<_>>()).contains("ginger-input")
    });
    host.terminate(id).unwrap();
    assert!(host.list().is_empty());
    assert!(host.write(id, b"no").is_err());
}

#[test]
fn rejects_missing_executable() {
    let host = TerminalHost::default();
    let dir = tempfile::tempdir().unwrap();
    assert!(host.launch(dir.path(), "/ginger-missing-command", &[], TerminalOwner::Agent, None).is_err());
    assert!(host.list().is_empty());
}

#[test]
#[ignore = "requires Neovim installed; run explicitly for editor validation"]
fn neovim_edits_and_saves_a_file_with_spaces() {
    let host = TerminalHost::default();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("ginger's code sample.txt");
    std::fs::write(&file, "").unwrap();
    let id = host.launch(dir.path(), "nvim", &["-u".into(), "NONE".into(), "-i".into(), "NONE".into(), "-n".into(), "--".into(), file.to_string_lossy().to_string()], TerminalOwner::Editor, None).unwrap();
    host.write(id, b"iGinger saved this through real Neovim.\x1b:wq\r").unwrap();
    wait_until(|| host.list().iter().any(|s| s.id == id && s.exited));
    assert_eq!(std::fs::read_to_string(file).unwrap(), "Ginger saved this through real Neovim.\n");
}

#[test]
fn closes_a_hup_ignoring_process_without_blocking_other_sessions() {
    let host = TerminalHost::default();
    let dir = tempfile::tempdir().unwrap();
    let id = host.launch(dir.path(), "/bin/sh", &["-c".into(), "trap '' HUP; printf ready; while :; do sleep 1; done".into()], TerminalOwner::Agent, None).unwrap();
    let output = host.sessions.lock()[&id].output.clone();
    wait_until(|| !output.lock().history.is_empty());
    // More input than a nonreading PTY can accept must not block the caller.
    for _ in 0..64 { let _ = host.write(id, &vec![b'x'; 16 * 1024]); }
    let start = Instant::now();
    assert_eq!(host.list().len(), 1);
    host.terminate(id).unwrap();
    assert!(start.elapsed() < Duration::from_secs(2));
    wait_until(|| output.lock().exited);
}

#[test]
fn subscription_replays_output_before_exit() {
    let host = TerminalHost::default();
    let dir = tempfile::tempdir().unwrap();
    let id = host.launch(dir.path(), "/bin/sh", &["-c".into(), "printf replay-me; exit 3".into()], TerminalOwner::User, None).unwrap();
    wait_until(|| host.list()[0].exited);
    let received = Arc::new(Mutex::new(Vec::new()));
    let capture = received.clone();
    let channel = Channel::new(move |body| {
        if let tauri::ipc::InvokeResponseBody::Json(json) = body {
            capture.lock().push(serde_json::from_str::<serde_json::Value>(&json).unwrap());
        }
        Ok(())
    });
    host.subscribe(id, channel).unwrap();
    let events = received.lock();
    assert_eq!(events[0]["type"], "output");
    assert_eq!(events[0]["data"], serde_json::json!(b"replay-me".to_vec()));
    assert_eq!(events[1]["type"], "exit");
    assert_eq!(events[1]["code"], 3);
}

#[test]
#[cfg(unix)]
fn closes_descendants_after_the_wrapper_exits() {
    let host = TerminalHost::default();
    let dir = tempfile::tempdir().unwrap();
    let id = host.launch(dir.path(), "/bin/sh", &["-c".into(), "trap '' HUP; (while :; do sleep 1; done) & printf '%s' \"$!\" > child.pid; printf ready; exit 0".into()], TerminalOwner::Agent, None).unwrap();
    let output = host.sessions.lock()[&id].output.clone();
    wait_until(|| !output.lock().history.is_empty());
    let descendant: i32 = std::fs::read_to_string(dir.path().join("child.pid")).unwrap().parse().unwrap();
    // Process completion must clean the descendants before marking the tab exited.
    wait_until(|| output.lock().exited);
    host.terminate(id).unwrap();
    wait_until(|| unsafe { libc::kill(descendant, 0) } != 0);
}
