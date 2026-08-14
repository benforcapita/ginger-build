// Ginger Code — TypeScript Domain Contracts (LLD 216)
// Shared contracts from a single source of truth where possible.
// DB models, IPC DTOs, and frontend types may differ only through explicit conversion.

// --- Typed IDs (LLD 214) ---
export type WorkspaceId = number;
export type SessionId = number;
export type TaskId = number;
export type AgentThreadId = number;
export type WorktreeId = number;
export type TerminalSessionId = number;
export type VerificationRunId = number;
export type ReviewSessionId = number;
export type ProcessId = number;

// --- Domain Events (LLD 214: domain.entity.action) ---
export type DomainEvent =
  | { type: "workspace.opened"; workspace_id: WorkspaceId; root: string }
  | { type: "workspace.closed"; workspace_id: WorkspaceId }
  | { type: "editor.ready"; session_id: SessionId }
  | { type: "editor.config.error"; error: string; safe_mode: boolean }
  | { type: "agent.thread.created"; agent_id: AgentThreadId; adapter: string }
  | { type: "agent.thread.started"; agent_id: AgentThreadId; worktree_path: string }
  | { type: "agent.thread.status.changed"; agent_id: AgentThreadId; status: string }
  | { type: "agent.thread.completed"; agent_id: AgentThreadId; success: boolean }
  | { type: "agent.thread.failed"; agent_id: AgentThreadId; error: string }
  | { type: "worktree.created"; worktree_id: WorktreeId; path: string; branch: string }
  | { type: "worktree.changed"; worktree_id: WorktreeId; files_changed: number }
  | { type: "worktree.applied"; worktree_id: WorktreeId; strategy: string }
  | { type: "verification.started"; run_id: VerificationRunId; agent_id: AgentThreadId }
  | { type: "verification.completed"; run_id: VerificationRunId; success: boolean }
  | { type: "package.recommendations.updated"; workspace_id: WorkspaceId; count: number }
  | { type: "package.install.started"; package_id: string }
  | { type: "package.install.completed"; package_id: string; success: boolean }
  | { type: "recovery.required"; reason: string }
  | { type: "recovery.completed"; worktrees_recovered: number; safe_mode: boolean }
  | { type: "job.progress"; job_id: string; phase: string; completed: number; total: number }
  | { type: "job.cancelled"; job_id: string };

// --- Stable IPC Error Codes (LLD 215) ---
export type GingerErrorCode =
  | "GINGER_GIT_WORKTREE_CREATE_FAILED"
  | "GINGER_GIT_WORKTREE_LIST_FAILED"
  | "GINGER_GIT_APPLY_FAILED"
  | "GINGER_GIT_CHERRY_PICK_FAILED"
  | "GINGER_GIT_BRANCH_SWITCH_FAILED"
  | "GINGER_EDITOR_RPC_HANDSHAKE_FAILED"
  | "GINGER_EDITOR_SPAWN_FAILED"
  | "GINGER_EDITOR_CONFIG_ERROR"
  | "GINGER_AGENT_SPAWN_FAILED"
  | "GINGER_AGENT_ADAPTER_NOT_FOUND"
  | "GINGER_AGENT_AT_CAPACITY"
  | "GINGER_PACKAGE_RESOLVE_FAILED"
  | "GINGER_PACKAGE_INSTALL_FAILED"
  | "GINGER_PACKAGE_ROLLBACK_FAILED"
  | "GINGER_WORKSPACE_OPEN_FAILED"
  | "GINGER_WORKSPACE_TRUST_REQUIRED"
  | "GINGER_WORKSPACE_MISSING"
  | "GINGER_RECOVERY_REQUIRED"
  | "GINGER_IPC_VERSION_MISMATCH"
  | "GINGER_INTERNAL";

export interface GingerError {
  code: GingerErrorCode;
  detail: string;
}

// --- State Machines (LLD 135-142) ---
export type WorkspaceOpenState =
  | "idle" | "opening" | "loading" | "ready" | "editor-starting" | "editor-ready" | "failed";
export type WorkspaceCloseState =
  | "ready" | "closing" | "editor-saving" | "process-reconciling" | "persistence-flushing" | "closed";
export type AgentState =
  | "pending" | "starting" | "running" | "completed" | "failed" | "interrupted";
export type TaskState =
  | "pending" | "active" | "review" | "completed" | "failed" | "cancelled";
export type ReviewState =
  | "pending" | "open" | "reviewing" | "applying" | "applied" | "rejected";
export type VerificationState =
  | "pending" | "running" | "completed" | "failed";
export type PackageEnvState =
  | "unresolved" | "resolving" | "resolved" | "installing" | "ready" | "degraded";
export type RecoveryState =
  | "detecting" | "reconciling" | "restoring" | "ready" | "safe-mode";

// --- Background Jobs (LLD 63) ---
export type BackgroundJobKind =
  | "project-scan"
  | "package-resolve"
  | "package-install"
  | "git-refresh"
  | "verification"
  | "runtime-validation"
  | "workspace-reconcile";

export type JobStatus = "pending" | "running" | "completed" | "failed" | "cancelled";

export interface BackgroundJob {
  id: string;
  kind: BackgroundJobKind;
  status: JobStatus;
  phase: string;
  completed: number;
  total: number;
  message: string;
  cancellable: boolean;
}

// --- Process Supervisor (LLD 64) ---
export type ProcessCategory =
  | "editor" | "terminal" | "agent" | "package-tool" | "verification";

export interface ProcessInfo {
  id: ProcessId;
  category: ProcessCategory;
  label: string;
  pid: number;
  started_at: number;
  exit_code: number | null;
  abnormal_exit: boolean;
}

// --- Agent Adapter (LLD 65-66) ---
export type AgentMode = "coding" | "review";
export type AgentIsolation = "worktree" | "none";

export interface AgentDescriptor {
  id: string;
  name: string;
  command_candidates: string[];
  args: string[];
  worktree_support: boolean;
  read_only: boolean;
  default_mode: AgentMode;
  default_isolation: AgentIsolation;
}

export interface AgentDetection {
  descriptor_id: string;
  name: string;
  found: boolean;
  path: string | null;
  version: string | null;
}

// --- Workspace Trust (LLD 85) ---
export type TrustLevel = "untrusted" | "trusted";

export interface WorkspaceTrust {
  workspace_id: WorkspaceId;
  root: string;
  level: TrustLevel;
  granted_at: number | null;
}

// --- Settings (LLD 87) ---
export type SettingsSource = "defaults" | "global-user" | "workspace-user" | "project-shared";

export interface SettingValue {
  value: string;
  source: SettingsSource;
}

// --- Search (LLD 74) ---
export interface SearchQuery {
  pattern: string;
  root: string;
  regex: boolean;
  case_sensitive: boolean;
  globs: string[];
  max_results: number;
}

export interface SearchResult {
  path: string;
  line: number;
  column: number;
  text: string;
}

// --- Verification (LLD 95-96) ---
export interface VerificationRun {
  id: VerificationRunId;
  agent_id: AgentThreadId;
  worktree: string;
  command: string;
  status: VerificationState;
  started_at: number;
  finished_at: number | null;
  exit_code: number | null;
  output_path: string | null;
  diff_fingerprint: string;
}

export interface SuggestedCommand {
  command: string;
  source: string;
  confidence: number;
}

// --- Review (LLD 98-99) ---
export interface ReviewSession {
  id: ReviewSessionId;
  agent_id: AgentThreadId;
  base_revision: string;
  target_revision: string;
  diff_fingerprint: string;
  status: ReviewState;
  accepted_hunks: string[];
  rejected_hunks: string[];
}

// --- Apply (LLD 100) ---
export type ApplyStrategy = "patch" | "cherry-pick" | "merge";

export interface ApplyRecord {
  id: number;
  agent_id: AgentThreadId;
  task_id: TaskId | null;
  target_branch: string;
  pre_head: string;
  post_head: string;
  strategy: ApplyStrategy;
  timestamp: number;
}

// --- Task (LLD 93) ---
export interface Task {
  id: TaskId;
  title: string;
  status: TaskState;
  agent_ids: AgentThreadId[];
  reviewer_ids: AgentThreadId[];
  verification_run_ids: VerificationRunId[];
  applied_result: string | null;
  created_at: number;
}

// --- Package Trust & Plan (LLD 81-82) ---
export type PackageTrust = "core" | "curated" | "community" | "local";

export interface InstallPlanItem {
  package_id: string;
  version: string;
  source: string;
  command: string;
  environment_changes: string[];
}

export interface InstallPlan {
  items: InstallPlanItem[];
  custom_commands_explicit: boolean;
}

// --- Recommendation (LLD 79-80) ---
export type RecommendationState = "new" | "shown" | "installed" | "ignored" | "dismissed";

export interface Recommendation {
  package_id: string;
  capability: string;
  confidence: number;
  evidence: string[];
  state: RecommendationState;
  reason: string;
}

// --- Diagnostics (LLD 125-127) ---
export interface HealthStatus {
  component: string;
  ok: boolean;
  detail: string;
}

export interface DiagnosticsBundle {
  app_version: string;
  runtime_version: string;
  db_schema_version: number;
  ipc_protocol_version: number;
  catalog_version: string;
  health: HealthStatus[];
  agent_detection: string[];
  crash_markers: string[];
  generated_at: number;
}

// --- IPC Versioning (LLD 130) ---
export const IPC_PROTOCOL_VERSION = 1;

export type IpcNegotiation =
  | { status: "compatible" }
  | { status: "mismatch"; frontend: number; backend: number };

// --- Progress (LLD 222) ---
export interface ProgressUpdate {
  operation_id: string;
  phase: string;
  completed: number;
  total: number;
  message: string;
}
