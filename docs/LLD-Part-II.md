# Ginger Code v0.1 — LLD Part II

Status: Continuation of ginger-code-v0.1-lld.md
Platform: macOS first
Purpose: Complete the application design beyond the core editor/agent/worktree/package architecture.

## 61. Application Shell Lifecycle

The desktop shell owns the lifetime of the workspace and every hosted subsystem.

Startup sequence:
```
process start → initialize logging → resolve application directories → open/migrate SQLite → initialize platform services → initialize Action Registry → initialize package/runtime services → render application shell → restore or open workspace → start editor host → start background capability scan → detect agent adapters → reconcile previous sessions/worktrees
```

The shell must remain responsive if a non-critical background service fails.

## 62. Application State Model

Frontend state is divided into domain stores: appStore, workspaceStore, editorStore, terminalStore, agentStore, gitStore, packageStore, paletteStore, settingsStore, gingerStore, and recoveryStore.

Each store owns presentation state only. Persistent business state remains owned by Rust services and SQLite.

## 63. Background Worker Model

Long-running background work is represented as jobs:
```
type BackgroundJobKind = "project-scan" | "package-resolve" | "package-install" | "git-refresh" | "verification" | "runtime-validation" | "workspace-reconcile"
```
Jobs emit progress events and may be cancelled where safe.

## 64. Process Supervisor

All child processes launched by Ginger are registered with a central process supervisor. Process categories: editor, terminal, agent, package tool, verification. Responsibilities: process creation, stdout/stderr routing, cancellation, child cleanup, abnormal-exit reporting, graceful shutdown, orphan reconciliation. No subsystem spawns unmanaged processes directly.

## 65. Agent Adapter Discovery

Known terminal agents are defined through descriptors with IDs, display names, command candidates, arguments, worktree support, and read-only capabilities. Detection checks configured overrides, Ginger-managed paths, user PATH, and common macOS install locations. Missing agents are visible in Settings but are not application errors.

## 66. Custom Agent Definitions

Users may define terminal agents in config:
```toml
[[agents.custom]]
id = "opencode"
name = "OpenCode"
command = "opencode"
default_mode = "coding"
default_isolation = "worktree"
```

## 67. Agent Startup Prompt Strategy

Adapters declare how task prompts are injected: argument, stdin, or manual. v0.1 prefers native supported startup mechanisms.

## 68. Agent Permissions UX

Terminal-backed agents own their native permission models and run with local user permissions. Ginger must disclose this clearly.

## 69. Native Agent Abstraction

Future Ginger-native agents use the same durable AgentThread model as terminal agents. v0.1 may use this interface only for a lightweight Ollama helper/reviewer.

## 70. Model Provider Layer

Native AI features are provider-agnostic through a ModelProvider interface. v0.1 requires only Ollama support at this layer.

## 71. Ollama Native Helper

Ollama may power optional local features: diff summaries, selected-code explanations, commit-message drafts, lightweight reviews, agent-output summaries. If unavailable, no editor functionality is blocked.

## 72. Context Collection for Native Features

Native helper context is explicit and bounded: current file, selection, current diff, task metadata, Git status, detected capabilities, and user-selected files.

## 73. Project Explorer

Explorer provides file tree, Git decorations, active-buffer indication, agent-worktree change indication, and create/rename/delete actions. Keyboard navigable, never replaces Neovim-native navigation.

## 74. Search

Workspace search uses ripgrep. Supports literal/regex, globs, case sensitivity, opening results in Neovim, and sending selected results to an agent.

## 75. Symbols

Symbol search comes from Neovim/LSP when available. Falls back gracefully to file search.

## 76. LSP Management

LSP servers installed through package manager. Resolution: project-local → workspace-pinned → global/system fallback. Neovim receives generated activation config from Ginger core.

## 77. Formatter and Linter Management

Same resolution model as LSP. Existing project conventions win over Ginger recommendations.

## 78. Treesitter

Treesitter parsers are versioned with workspace/runtime environments. Missing parsers never block opening a file.

## 79-82. Package Recommendation Engine

Recommendations derive from deterministic project evidence. Badges: Core, Curated, Community/Custom, Local. Install plan displayed before installation.

## 83-84. Package Rollback and Cache Management

Environment updates are staged, validated, and atomically activated. Shared cache supports size reporting, verification, pruning, and repair.

## 85. Workspace Trust

New repositories begin untrusted. Reading/editing and safe scans are allowed, but project-defined executable behavior is blocked until trust is granted.

## 86. Project .ginger Configuration

Optional `.ginger/workspace.toml` may declare verification commands, recommended package IDs, preferred agent templates, and display metadata. May not silently execute startup commands or store secrets.

## 87-89. Settings Storage and UI

Settings layers: defaults → global user → workspace user → safe project-shared config. Settings are searchable and action-addressable.

## 90. Keybinding Conflict Detection

Assigning a global Ginger shortcut checks existing Ginger mappings, reserved macOS conflicts, and known Neovim conflicts.

## 91-92. Command Palette Architecture

Palette providers: actions, files, symbols, agents, tasks, worktrees, packages, settings, recent items. Cmd+P opens instantly from cached indexes.

## 93-94. Task Model

Task is independent of agent. One task may have multiple implementation agents, reviewers, verification runs, and an applied result.

## 95-96. Verification Runs

Verification runs are durable objects. Commands suggested from package.json, Cargo, Go, .NET, Python, Makefile, and justfile conventions.

## 97-101. Diff, Review, Apply

Diff data includes file status, hunks, and lines. Hunk IDs are deterministic. Review session stores agent thread, base/target revisions, diff fingerprint. Apply flow: refresh → verify fingerprint → construct patch → dry-run → apply → validate → persist. Undo appears only when verified safe.

## 102. Worktree Cleanup Policy

Applied clean worktrees become cleanup-eligible after seven days. Unapplied worktrees never auto-delete. Orphaned worktrees require review.

## 103-104. macOS File Permissions and Notifications

Lost permissions surface Reauthorize Folder. Notifications carry workspace/thread/task routing metadata.

## 105-108. App Menu, Deep Links, Single Instance, CLI Launcher

Native menus expose File, View, Agents, Packages, Help. Reserve `ginger://workspace/<id>` routes. Ship `ginger .` launcher.

## 109-111. First-Run Validation and Onboarding

Validate bundled Neovim, Ginger core, writable data directories, SQLite before first editor start. Agents panel shows detected status.

## 112-113. Ginger Welcome Screen and Empty Agent Dock

Full ASCII Ginger portrait when no workspace is open. Medium Ginger art for empty Agent Dock.

## 114-116. Ginger State Arbitration and Commentary

Priority: failure → warning → reviewing → testing → coding → thinking → listening → success → idle. Commentary is deterministic, event-driven, offline-capable.

## 117. Accessibility

All controls keyboard reachable; focus visible; icons have labels; mascot art has text alternatives; reduced motion disables nonessential animation.

## 118-119. Theme System and Font Strategy

v0.1 ships canonical Ginger dark theme. Editor font is configurable.

## 120. Telemetry and Privacy

Ginger works with telemetry disabled. Never collect source code, raw terminal transcripts, prompts, diffs, secrets, or repo names by default.

## 121-124. Update Channels, Catalog Updates, Supply-Chain Safety, Runtime Integrity

Bundled baseline catalog works offline. Remote catalog updates require signature/hash verification. Bundled runtime has an integrity manifest.

## 125-127. Diagnostics Bundle, Health Screen, Developer Mode

Export Diagnostics includes versions, sanitized settings, structured logs. Developer mode exposes event inspector, IPC logs, context keys.

## 128-129. Action Inspector and Event Inspector

Developer views list action ID, title, category, context predicate, enabled state, keybinding, and event history.

## 130. IPC Versioning

Frontend and Rust backend negotiate an IPC protocol version. Mismatch triggers Restart/Repair.

## 131. Database Migration Testing

Migrations tested from empty DB, previous release DB, and representative DB with workspaces/agents/worktrees.

## 132-134. Reconciliation Algorithms

Worktree: load recorded worktrees, parse git worktree list --porcelain, canonicalize paths, validate branches, detect missing/unmanaged. Agent: check surviving processes, mark interrupted, inspect worktree. Package: read manifest/lock, verify runtime compatibility, check project-local tools.

## 135-142. State Machines

- Workspace Open: idle → opening → loading → ready → editor-starting → editor-ready
- Workspace Close: ready → closing → editor-saving → process-reconciling → persistence-flushing → closed
- Agent: pending → starting → running → completed/failed/interrupted
- Task: pending → active → review → completed/failed/cancelled
- Review: pending → open → reviewing → applying → applied/rejected
- Verification: pending → running → completed/failed
- Package Environment: unresolved → resolving → resolved → installing → ready/degraded
- Recovery: detecting → reconciling → restoring → ready/safe-mode

## 143-144. Performance and Resource Budgets

No busy polling. Git refresh is debounced/event-driven. Project scans are cancellable. Inactive terminal rendering may throttle.

## 145. Large Repository Strategy

Skip ignored/vendor paths, progressively populate Explorer, use optimized Git/ripgrep commands.

## 146-147. Terminal Scrollback and Rendering States

Bounded visible scrollback. Rendering states: starting, connected, exited, disconnected, recovered-metadata-only.

## 148-151. Clipboard, Drag and Drop, File Watcher, External Changes

Normal explicit clipboard interactions. Drop folder to open workspace. Watcher normalizes/debounces file changes. Neovim remains authoritative for buffer change handling.

## 152-153. Agent Change Attribution and Git Remote Safety

Attribute modifications to agent-owned worktree. Ginger-managed push/force push/PR creation out of default v0.1 scope.

## 154-155. Commit Workflow and Branch Switching

Applied changes may be committed through Ginger composer. Inspect dirty state and active agents before branch switches.

## 156-160. Agent Worktree Update, Review, Concurrency, Queue

Stale agent worktrees expose Update Agent. Default max active coding agents is 3. Queued starts are durable.

## 161. Package Recommendations from Agent Context

Task context may raise ranking only through deterministic capabilities and explicit task metadata.

## 162-165. Future Boundaries

ACP maps into existing models. Future plugins may register Actions, Palette providers, Agent adapters. Built-ins use registries.

## 166-168. Build Configuration, CI Pipeline, Release Pipeline

Build modes: development, test, release. CI: format, lint, Rust tests, TS tests, integration tests, build frontend, build Tauri app, runtime validation, package artifact.

## 169-170. Version Compatibility and Crash Reporting

Maintain explicit compatibility between App, Runtime, DB schema, IPC protocol, and Catalog versions.

## 171-178. Test Fixtures and Strategies

Deterministic fixtures: simple Git repo, conflict repo, submodule repo, React/Tailwind project, Rust project, broken user Lua config, fake terminal agent. CI never depends on real Claude/Codex.

## 179. Threat Model Summary

Primary threats: malicious repository, malicious package, dangerous terminal agent, unsafe Git automation, secret leakage, path escape, corrupt update. Mitigations: trust, explicit approval, worktrees, structured commands, Keychain, path validation, signed/hashed runtime metadata.

## 180. Product Analytics Events — If Enabled

Allowed aggregate events: workspace open, palette use, agent adapter type, package recommendation acceptance, recovery use. No repo/file/prompt/diff/terminal content.

## 181-182. UX Copy and Error Severity

Information first, sarcasm second. Only blocking/destructive flows interrupt central work.

## 183-188. Toast Policy, Status Bar, Agent Dock, Task/Package/Runtime/Workspace Actions

Status bar: Neovim mode, branch, filetype, diagnostics, active agent count, compact Ginger state. Agent dock groups: Working, Waiting, Review, Recent.

## 189-192. Search Actions, Neovim Package Compatibility, User Lua Compatibility, Runtime Pinning

Curated plugins declare compatible Ginger runtime ranges. Runtime candidate validation loads user config before activation.

## 193-203. Future Boundaries (Multi-Repo, Remote, Multi-Window, Plugin Marketplace, MCP, Native Agent Tools, Planning, AI Boundary, Personality)

v0.1 has one primary local root. Filesystem/terminal/Git operations remain behind interfaces. One main window. Neovim/editor packages, developer tools, and future Ginger plugins remain separate.

## 204-206. Definition of Done and Implementation Order

Every slice includes domain interface, backend implementation, typed IPC, frontend state, UI, error states, tests, Action Registry entries, logging, persistence migration if required, and recovery impact analysis.

## 207-212. Milestones A-F

- A: Ginger boots and edits
- B: Isolated fake agent
- C: Real terminal agents
- D: Managed developer environment
- E: Recovery-grade application
- F: Release candidate

## 213-222. ADRs, Naming, Error Conventions, Domain Contracts, Serialization, Time, Path, Logging, Cancellation, Progress, Concurrency

Use typed IDs. Events use domain.entity.action. Internal errors retain technical causes; IPC exposes stable codes. Persist UTC timestamps; render local time. Backend canonicalizes paths.

## 223-229. Concurrency, File Locking, Atomic Writes, Backward Compatibility, Import/Export, Uninstall, Data Retention

Per-resource locks. Partial artifacts never appear at final cache keys. Generated manifests written temp → synced → atomically renamed. v0.1.x preserves IDs, user Lua, custom agents, keybindings.

## 230-232. Privacy of Agent Data, Documentation Set, In-App Help

User docs and developer docs. Cmd+P exposes local bundled help.

## 233-237. Golden Paths

Keyboard-first, mouse, package, recovery, and safe mode golden paths defined.

## 238-239. Product Differentiation and Success Metric

A sophisticated developer should be able to replace separate Neovim, terminal tabs, Claude/Codex windows, manual worktrees, manual diff review, and separate tooling setup with one coherent workspace.

## 240. Final Architectural Invariants

1. Ginger Code is a complete desktop application.
2. Neovim remains the editing engine.
3. The app owns/versions its Neovim runtime.
4. User Lua is a first-class escape hatch.
5. Actions are the stable command API.
6. Cmd+P indexes the whole workspace.
7. Terminal agents stay native where possible.
8. Coding agents get isolated worktrees by default.
9. Agent work enters the primary branch only through explicit review/apply.
10. Tasks, agents, and processes are separate concepts.
11. Agent threads survive process death.
12. Unapplied work is never silently deleted.
13. Recommendations derive from deterministic project evidence.
14. Curated and custom package paths coexist.
15. Project-local tooling wins.
16. Editor readiness precedes optional scans/AI work.
17. Workspace trust gates executable project behavior.
18. Bad user config cannot brick the app.
19. Third-party terminal agents are an explicit security boundary.
20. Ginger Presence is part of the product identity.
21. Personality never overrides safety or clarity.
22. macOS is the required v0.1 platform.
23. Architecture remains portable for later Linux/Windows.
24. Releases are recoverable, diagnosable, and migration-safe.

## 241. Next Artifact

After approval of both LLD documents, create the implementation plan. It should decompose the application into testable vertical slices with exact files, interfaces, TDD steps, verification commands, and commit checkpoints.