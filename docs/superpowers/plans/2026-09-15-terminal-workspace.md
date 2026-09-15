# Terminal workspace implementation plan

> Execute inline with the executing-plans skill; review each deliverable before moving on.

**Goal:** Make the existing desktop shell usable for file editing and CLI harness sessions.
**Architecture:** Rust PTY lifecycle with buffered output, xterm rendering, workspace-scoped file listing, and a three-column React workspace.
**Tech Stack:** Tauri 2, Rust, React 19, TypeScript, Zustand, xterm.js, Neovim.
**Spec:** docs/superpowers/specs/2026-09-15-terminal-workspace-design.md

## Global constraints

- Keep the Tauri desktop app and reference layout.
- Execute programs with argv, never interpolate file paths into shell commands.
- Keep session state truthful, surface errors, and preserve user files.
- Do not modify the existing untracked docs/PROGRESS.md.

## Tasks

- [x] Repair build dependencies and command registration. Remove the unused unavailable Yjs middleware, add missing Rust dependencies, expose actual command modules, and remove registrations for nonexistent Part II commands while retaining service sources. Verify with `pnpm build` and `cargo check`.
- [x] Implement terminal lifecycle in `src-tauri/src/terminal/{mod,commands}.rs`: create a PTY with executable/argv/cwd, attach a buffered output channel, accept input and resize, observe exit, kill on close. Add tests for early output, interactive input, process exit, and termination; run them before and after implementation.
- [x] Implement workspace file listing in `src-tauri/src/workspace/{mod,commands}.rs`: canonical root validation, lazy directory entries, directory-first sorting, and file validation. Test traversal, symlink escape, and invalid root rejection before implementing.
- [x] Connect `src/components/terminal/TerminalView.tsx` to a Tauri channel and the lifecycle commands. Keep tabs mounted, handle resizing and errors, and dispose listeners on unmount. Build shared UI session store in `src/stores/session-store.ts`.
- [x] Replace explorer/editor/agent placeholders with folder picker, recursive file tree, real Neovim tabs, shell tabs, and CLI executable chooser. Use direct executable and argument arrays; show shared-workspace scope. Preserve sessions until explicit close.
- [x] Restyle `src/App.tsx` and `src/styles/` around the reference. Replace the emoji mascot with original text art, keyboard-focusable pet interaction, and truthful state-driven copy. Wire command palette and shortcuts to implemented operations only.
- [x] Validate frontend build, Rust tests, native boot, PTY round-trip, and visual layout. Document actual functionality and remaining limits in README.md.

## Validation

- Frontend production build passed.
- Rust suite: 43 passed, 1 optional Neovim test ignored by default; the optional Neovim edit/save test passed when run explicitly.
- Native smoke checks: folder picker, nested tree, Neovim edit/save, shell output, custom executable launch, close confirmation, and Command-Q confirmation/cancellation.
- Review prompted fixes for Ctrl-key passthrough, bounded input, and macOS process-group cleanup. Detached process groups are outside the cleanup guarantee.
