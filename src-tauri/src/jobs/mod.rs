/// Ginger Code — Background Worker Model
/// Long-running background work is represented as jobs.
/// Jobs emit progress events and may be cancelled where safe.

use crate::events::DomainEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackgroundJobKind {
    ProjectScan,
    PackageResolve,
    PackageInstall,
    GitRefresh,
    Verification,
    RuntimeValidation,
    WorkspaceReconcile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundJob {
    pub id: String,
    pub kind: BackgroundJobKind,
    pub status: JobStatus,
    pub phase: String,
    pub completed: usize,
    pub total: usize,
    pub message: String,
    pub cancellable: bool,
}

pub struct JobManager {
    jobs: Mutex<HashMap<String, BackgroundJob>>,
    next_id: Mutex<u64>,
}

impl JobManager {
    pub fn new() -> Self {
        Self {
            jobs: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
        }
    }

    pub fn start(&self, kind: BackgroundJobKind, cancellable: bool) -> String {
        let mut next = self.next_id.lock().unwrap();
        let id = format!("job-{}", *next);
        *next += 1;

        let job = BackgroundJob {
            id: id.clone(),
            kind,
            status: JobStatus::Running,
            phase: "starting".to_string(),
            completed: 0,
            total: 0,
            message: String::new(),
            cancellable,
        };
        self.jobs.lock().unwrap().insert(id.clone(), job);
        id
    }

    pub fn progress(&self, id: &str, phase: &str, completed: usize, total: usize) -> Option<DomainEvent> {
        let mut map = self.jobs.lock().unwrap();
        if let Some(job) = map.get_mut(id) {
            job.phase = phase.to_string();
            job.completed = completed;
            job.total = total;
            Some(DomainEvent::JobProgress {
                job_id: id.to_string(),
                phase: phase.to_string(),
                completed,
                total,
            })
        } else {
            None
        }
    }

    pub fn complete(&self, id: &str) {
        if let Some(job) = self.jobs.lock().unwrap().get_mut(id) {
            job.status = JobStatus::Completed;
            job.phase = "completed".to_string();
        }
    }

    pub fn fail(&self, id: &str, message: &str) {
        if let Some(job) = self.jobs.lock().unwrap().get_mut(id) {
            job.status = JobStatus::Failed;
            job.message = message.to_string();
        }
    }

    pub fn cancel(&self, id: &str) -> bool {
        let mut map = self.jobs.lock().unwrap();
        if let Some(job) = map.get_mut(id) {
            if job.cancellable {
                job.status = JobStatus::Cancelled;
                return true;
            }
        }
        false
    }

    pub fn list(&self) -> Vec<BackgroundJob> {
        self.jobs.lock().unwrap().values().cloned().collect()
    }

    pub fn get(&self, id: &str) -> Option<BackgroundJob> {
        self.jobs.lock().unwrap().get(id).cloned()
    }
}

impl Default for JobManager {
    fn default() -> Self {
        Self::new()
    }
}