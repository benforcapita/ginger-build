use super::{pty_error, TerminalError};
use parking_lot::Mutex;
use portable_pty::{Child, MasterPty};
use std::sync::Arc;

pub(super) type SharedMaster = Arc<Mutex<Box<dyn MasterPty + Send>>>;

pub(super) struct OwnedProcess {
    child: Box<dyn Child + Send + Sync>,
    master: SharedMaster,
    pub complete: bool,
    pub code: Option<u32>,
}

impl OwnedProcess {
    pub fn new(child: Box<dyn Child + Send + Sync>, master: SharedMaster) -> Self {
        Self { child, master, complete: false, code: None }
    }

    pub fn poll(&mut self) -> Result<bool, TerminalError> {
        if self.complete { return Ok(true); }
        #[cfg(unix)]
        {
            let pid = self.child.process_id().ok_or_else(|| pty_error("child has no process id"))?;
            // Observe completion WITHOUT reaping. The unreaped child reserves its
            // PID/PGID until we clean its group; it cannot be reused by another job.
            let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
            let result = unsafe { libc::waitid(libc::P_PID, pid, &mut info, libc::WEXITED | libc::WNOHANG | libc::WNOWAIT) };
            if result != 0 {
                let error = std::io::Error::last_os_error();
                if error.kind() == std::io::ErrorKind::Interrupted { return Ok(false); }
                return Err(pty_error(format!("waitid {pid}: {error}")));
            }
            if unsafe { info.si_pid() } == 0 { return Ok(false); }
            self.shutdown()?;
        }
        #[cfg(not(unix))]
        if let Some(status) = self.child.try_wait().map_err(pty_error)? {
            self.code = Some(status.exit_code());
            self.complete = true;
        }
        Ok(self.complete)
    }

    pub fn shutdown(&mut self) -> Result<(), TerminalError> {
        if self.complete { return Ok(()); }
        #[cfg(unix)]
        {
            let leader = self.child.process_id().map(|id| id as i32);
            let foreground = self.master.lock().process_group_leader();
            for group in [foreground, leader].into_iter().flatten().filter(|pid| *pid > 1) {
                // SAFETY: these groups belong to this PTY; the child has not been
                // reaped, so its original group identity is still reserved.
                let result = unsafe { libc::kill(-group, libc::SIGKILL) };
                if result != 0 {
                    let error = std::io::Error::last_os_error();
                    if error.raw_os_error() == Some(libc::ESRCH) { continue; }
                    // macOS reports EPERM for a group containing only its zombie
                    // leader. Confirm the leader exited without releasing its PID.
                    if error.raw_os_error() == Some(libc::EPERM) && Some(group) == leader {
                        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
                        let waited = unsafe { libc::waitid(libc::P_PID, group as u32, &mut info, libc::WEXITED | libc::WNOHANG | libc::WNOWAIT) };
                        if waited == 0 && unsafe { info.si_pid() } == group { continue; }
                    }
                    return Err(pty_error(format!("kill group {group}: {error}")));
                }
            }
        }
        #[cfg(not(unix))]
        self.child.kill().map_err(pty_error)?;
        self.code = Some(self.child.wait().map_err(pty_error)?.exit_code());
        self.complete = true;
        Ok(())
    }
}
