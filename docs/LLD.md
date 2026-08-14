# Ginger Code v0.1 — Low-Level Design

Status: Design candidate for implementation
Platform: macOS first
Product: Complete installable desktop application
Primary UX: Bundled Neovim + terminal-backed multi-agent workspace
Visual direction: Dark terminal-first UI, amber/orange accents, persistent ASCII/ANSI Ginger mascot

## 1. Product Definition

Ginger Code is a macOS desktop development workspace combining a bundled managed Neovim runtime, Vim-native editing, a Zed-inspired global Action Registry and Cmd+P command palette, a dedicated multi-agent dock, terminal-backed coding agents such as Claude Code and Codex, Ollama local helper/reviewer support, Git worktree isolation, explicit diff/review/apply workflows, a LazyVim-inspired package/tool manager, project-context-aware package recommendations, persistent workspace recovery, and a visible sarcastic Ginger mascot.

Ginger is an application, not a Neovim configuration, plugin, or terminal wrapper. A fresh supported Mac must be able to install Ginger Code.app, open a repository, edit immediately, run terminals, configure packages, launch multiple agents, review isolated changes, apply approved work, and restore the workspace after restart.

### 1.1 Product principles
1. Code remains primary; chat never displaces the editor by default.
2. Neovim is real; Ginger embeds a bundled Neovim runtime.
3. Agents remain independent; Claude Code remains Claude Code, Codex remains Codex.
4. Coding agents are isolated by default in Git worktrees.
5. Every meaningful application operation is an Action.
6. Agent claims are not proof; Git diff + verification + explicit apply are the trust boundary.
7. Good defaults, full escape hatch; UI configuration and Lua customization coexist.
8. Curated and custom packages coexist.
9. Offline editing always works.
10. Ginger is a product identity, not merely a logo.

## 2. Scope

### 2.1 v0.1 includes
- macOS Apple Silicon desktop app packaging
- opinionated workspace layout
- bundled Neovim runtime and protected Ginger core
- user Lua customization
- project explorer
- Action Registry
- Cmd+P universal command palette
- configurable keybindings
- PTY terminal subsystem
- Agent Dock
- Claude Code, Codex, custom terminal-agent adapters
- Ollama local helper/reviewer
- concurrent agents
- Git worktree isolation for coding agents
- durable tasks and agent threads
- deterministic verification
- Git status/diff/review/apply
- unified and side-by-side diffs
- file- and hunk-level review
- second-agent review
- curated + custom Neovim packages
- curated + custom LSP/formatter/linter/debugger/CLI tools
- shared package cache + workspace manifests/locks
- project capability detection + package recommendations
- SQLite persistence
- session restore + crash recovery
- macOS Keychain and notifications
- safe mode
- Ginger mascot state system
- structured logs and migrations

### 2.2 Not required in v0.1
- Windows/Linux
- browser app
- collaboration/cloud sync
- plugin marketplace
- arbitrary draggable pane graph
- debugger UI
- SSH/remote workspaces
- built-in AI autocomplete
- autonomous multi-agent swarm
- ACP as a required integration path
- universal structured review extraction
- AI merge-conflict resolution

## 3. Technology Stack
- Desktop: Tauri 2 + Rust
- UI: React + TypeScript + Zustand
- Editor: bundled Neovim launched with nvim --embed; Msgpack-RPC bridge
- Terminal: Rust PTY host + xterm.js rendering
- Persistence: SQLite + filesystem + macOS Keychain
- Git: git executable behind a strict Rust GitService
- Package/runtime management: Ginger-managed cache and workspace-specific resolved environments

## 4. High-Level Architecture
```
┌───────────────────────────────────────────────────────────────┐
│ Ginger Code UI                                                │
│ Explorer │ Neovim │ Agent Dock │ Terminal │ Diff │ Packages  │
└──────────────────────────────┬────────────────────────────────┘
                               │ Action Registry
                               │
   ┌──────────────┬─────────┼──────────┬──────────────┐
   │              │         │          │              │
 Editor Host  Agent Sup  Terminal   Git Service   Package Mgr
   │              │ Host    │          │              │
 Neovim      Agent Adapters PTYs   Worktrees    Catalog/Tools
   │
 Claude Code │ Codex │ Ollama │ Custom CLI

 Persistence Service
   │
 SQLite + filesystem

 Platform Services
   │
 Keychain │ Notifications │ Runtime │ Updates

 Ginger Presence
   │
 mascot state + reactions + copy
```

Architectural rule: React components never directly spawn processes, mutate Git, install packages, write secrets, or manage Neovim. Those operations go through explicit service contracts and Tauri IPC.

## 5. Proposed Repository Layout
```
ginger-code/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── action/          # Action Registry
│   │   ├── editor/          # Neovim host
│   │   ├── agent/           # Agent supervisor + adapters
│   │   ├── terminal/        # PTY host
│   │   ├── git/             # Git service
│   │   ├── persistence/     # SQLite + filesystem
│   │   ├── platform/        # macOS services
│   │   ├── presence/        # Ginger mascot
│   │   ├── package/         # Package manager
│   │   └── workspace/       # Workspace management
│   ├── migrations/
│   ├── resources/
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── components/
│   │   ├── explorer/
│   │   ├── editor/
│   │   ├── agent-dock/
│   │   ├── terminal/
│   │   ├── diff/
│   │   ├── packages/
│   │   ├── settings/
│   │   └── presence/
│   ├── stores/              # Zustand stores
│   ├── hooks/
│   ├── actions/             # Action Registry TS bindings
│   ├── ipc/                 # Tauri IPC wrappers
│   ├── types/
│   ├── utils/
│   ├── styles/
│   ├── App.tsx
│   └── main.tsx
├── docs/
├── assets/
├── scripts/
├── crates/                  # Shared Rust crates
├── package.json
├── tsconfig.json
├── vite.config.ts
└── README.md
```

## 6. Workspace UI

Default layout is opinionated. v0.1 supports resizing/hiding Explorer, Agent Dock and bottom panel, switching bottom tabs, and maximizing editor/agent/diff. Arbitrary pane rearrangement is deferred.

Activity rail: Explorer, Search, Source Control, Agents, Packages, Settings.

## 7. Action Registry

Every meaningful capability is an Action. Buttons, menus, keybindings, Neovim commands, plugins and Ginger all invoke Actions rather than duplicating logic.

Context keys include workspace, editor, agent, terminal, Git and recommendation state.

## 8. Command Palette

Cmd+P searches: Actions, Files, Symbols, Agents, Tasks, Worktrees, Packages, Settings, Recent items.

Optional prefixes: action:, file:, symbol:, agent:, task:, git:, package:, setting:

Ranking combines fuzzy score, context relevance, recency, frequency and exact-prefix boosts.

## 9. Keybindings

Global Ginger shortcuts map only to Action IDs. Vim keys remain Neovim-owned. Ginger exposes :Ginger... commands and a Lua action API for advanced users.

## 10. Bundled Neovim

Startup order: select runtime → start nvim --embed → Msgpack-RPC handshake → protected Ginger core → resolved package environment → generated workspace config → user Lua → restore Neovim session → editor ready

Protected core contains only what Ginger needs: RPC bridge, workspace hooks, buffer/selection/diagnostics events, Action invocation, package activation, session lifecycle and required commands.

If user Lua fails, Ginger keeps the editor usable with the user layer disabled and shows exact recovery actions.

## 11. Terminal Host

Rust owns PTYs. Operations: create, write, resize, terminate, subscribe output, observe exit.

User terminals and agent terminals share PTY infrastructure but not lifecycle semantics.

## 12. Agent Supervisor

Adapter types: terminal adapter, native adapter.
v0.1 adapters: Claude Code terminal, Codex terminal, custom terminal, Ollama native helper/reviewer.

Durable thread: create intent → allocate worktree → start PTY/agent → stream output → detect completion → verify → review → apply/discard.

Coding defaults to isolated worktree. Review/research defaults to read-only current workspace or target worktree.

## 13. Worktree Isolation

Agent creation: create worktree → record base revision → start agent in worktree cwd.
Branch naming: ginger/<adapter>/<task-slug>.
Worktree root: ~/.ginger/worktrees/<workspace-id>/<worktree-id>.
Never auto-delete a worktree containing unapplied changes.
Worktree states: active, review, applied, discarded, stale, orphaned.

## 14. Multi-Agent Workflow

The primary tree stays user-owned while multiple agents work in independent trees.

Agent Dock displays adapter, task, elapsed time, status, isolation, branch/worktree, changed files and review state.

Completion does not imply correctness. Verification is deterministic and workspace-configured.

Second-agent review receives original task, base revision, target diff and verification output, and runs read-only by default.

## 15. Git Service

Conceptual contract: open repository → validate → read status → read diff → create worktree → commit → cherry-pick → merge → apply patch → close.

Repository mutations use an async repository lock; reads may run concurrently.

## 16. Diff, Review and Apply

Diff modes: unified, side-by-side.
Keyboard actions: next/previous file, next/previous hunk, accept/reject hunk, open in editor.

Review selection is non-destructive until Apply.
Apply strategies: uncommitted work → patch; clean agent commits → cherry-pick default; explicit Keep Branch/Merge → merge workflow.
Default apply does not auto-commit.

If primary and agent changes overlap, automatic apply is blocked. No silent stash, rebase or AI conflict resolution.

## 17. Package Manager

Two ownership lanes: CURATED, CUSTOM.
Package kinds: neovim plugin, LSP server, formatter, linter, debugger, CLI tool.
Shared downloads are deduplicated; workspace resolution/pinning remains independent.

## 18. Tool and Package Resolution

Tool precedence: project-local → workspace resolved → curated catalog → not installed.
Ginger respects existing project-local tooling before suggesting duplicates.

## 19. Project Detection and Recommendations

Project scanner inspects deterministic artifacts: package.json, lock files, pyproject.toml, Cargo.toml, go.mod, *.csproj, Docker files, framework/test configs.

Capabilities map to curated packages. Detection runs asynchronously and never blocks editor startup.

## 20. Runtime Versioning

Runtime identity: Ginger runtime version, Neovim version, Ginger core revision, compatible catalog state.
Upgrade flow: resolve candidate → build side-by-side → validate → switch active pointer. Failure leaves previous runtime active.
Safe mode: bundled Neovim + protected core + minimal required packages, no user Lua or optional packages.

## 21. Persistence and Database

SQLite stores structured metadata; filesystem stores worktrees, logs, packages, generated configs, runtime assets; Keychain stores Ginger-owned secrets.

Data root: ~/.ginger/ with cache/, workspaces/, worktrees/, logs/, backups/, data/ginger.sqlite.

Core tables: workspaces, sessions, pane_state, editor_state, terminal_sessions, tasks, agent_threads, agent_events, worktrees, package_environments, package_recommendations, recent_actions.

## 22. Session Lifecycle and Recovery

Graceful close: mark session closing → persist panes → save Neovim session → persist terminal metadata → snapshot agent state → flush logs → stop editor → reconcile managed processes → mark session closed.

Heartbeat identifies abnormal shutdown. Recovery reconciles workspace root, Git repository, worktrees, agent processes, package environment and editor session.

Never auto-delete recovered work.

## 23. Platform Services — macOS

Keychain secrets, Notification Center, app support/cache paths, native file/folder dialogs, bundle resources, application metadata.

Third-party terminal agents retain their own auth; Ginger does not copy their credentials.

## 24. Ginger Presence Layer

Ginger is a UI/presence subsystem, not the business-logic owner.

States: idle, listening, thinking, coding, testing, reviewing, success, warning, failure.

Asset tiers: full ANSI portrait, medium portrait, compact portrait/status glyph.

Copy rules: sarcasm may target bugs, flaky tests, dependencies, build systems and complexity; never insult the user; never obscure dangerous/security decisions; serious states override humor; commentary is optional.

Settings: Standard / Quiet / Extra personality, and commentary On/Off.

## 25. Visual Design

- near-black/charcoal foundation
- amber/orange primary brand accent
- restrained green success and red conflict/destructive states
- compact developer density
- monospace for terminal/technical surfaces
- native sans where settings/descriptive readability wins
- thin low-contrast borders
- subtle, short animations

## 26-38. [Full details in original LLD]

### 37. Implementation Decomposition

1. Desktop foundation + Action Registry
2. Bundled Neovim host + protected/user config
3. Workspace persistence + editor restore
4. PTY terminal
5. Git repository/worktree service
6. Terminal-agent supervisor
7. Agent Dock
8. Diff/review/apply pipeline
9. Verification + second-agent review
10. Package/tool manager
11. Project detection + recommendations
12. Ginger presence/visual system
13. Crash recovery + safe mode
14. macOS packaging/signing/update safety
15. End-to-end stabilization

### 38. Non-Negotiable Design Invariants

1. No core feature exists only as a React click handler; use Actions.
2. No coding agent writes directly to the primary tree by default.
3. No agent result auto-merges because the agent says it succeeded.
4. No unapplied worktree is auto-deleted.
5. No repository config auto-executes on open.
6. No user Lua failure bricks the app.
7. No recommendation invents an unknown installable package.
8. No editor startup depends on remote AI/package APIs.
9. No external-agent output is falsely attributed to Ginger.
10. No UI claims sandboxing Ginger cannot enforce.
11. No secret goes into normal SQLite/log data.
12. No mascot copy obstructs security/destructive decisions.
13. Ginger remains useful as an editor without AI configured.
14. The Ginger mascot remains a deliberate, visible part of the product identity.