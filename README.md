# Ginger Code

A macOS desktop IDE with a terminal aesthetic, real Neovim, a project tree, independent CLI harness tabs, and Ginger: an orange-haired text-art code companion.

## What works

- Open a local folder and expand its file tree. Directories load on demand; refresh picks up filesystem changes. Links outside the open workspace are excluded.
- Open files in Neovim tabs. Use your normal Vim controls and user configuration: `i` to insert, `Esc` to return to normal mode, `:w` to save, `:q` to quit. Each tab owns a Neovim process.
- Run login shells alongside the editor with live input/output and pane resizing.
- Start Claude Code, Codex, OpenCode, or a custom executable with an argument list in a separate harness tab. Installed programs are detected; each CLI handles its own authentication. Up to three live harnesses share the open workspace.
- Pet Ginger, mute commentary, or minimize the portrait while harnesses run. Session counts and exit indicators reflect real processes.
- Search working commands and open sessions with the command palette. Live session closure and app exit warn before stopping processes. Close tabs before switching workspaces.

## Develop

Requires macOS, Node.js, pnpm, a Rust toolchain, and Neovim. The Tauri launcher locates an installed rustup toolchain even when Cargo is absent from PATH. Ginger also checks common Homebrew and user executable directories when started from Finder.

```sh
pnpm install
# If Neovim is missing:
brew install neovim
pnpm tauri dev
```

`pnpm dev` runs the browser-only visual preview. Local files and processes require the desktop app.

| Shortcut | Action |
| --- | --- |
| ⌘ O | Open folder |
| ⌘ P / ⌘ K | Command palette |
| ⌘ S | Send `:write` to the active Neovim tab |
| ⌘ ⇧ T | New shell |
| ⌘ ⇧ N | New harness |

Ctrl keys pass through to Neovim and terminal programs.

## Build and verify

```sh
pnpm build
pnpm tauri build --debug --bundles app
cargo test --manifest-path src-tauri/Cargo.toml --lib
cargo test --manifest-path src-tauri/Cargo.toml --lib neovim_edits -- --ignored
```

If `cargo` is absent from PATH, add the directory reported by `dirname "$(rustup which cargo)"` to PATH. The optional editor test launches Neovim against a temporary file and verifies saved contents. Rust tests cover workspace path boundaries, PTY input/output, channel replay, process exit, and shutdown under backpressure.

The local app bundle is created at `src-tauri/target/debug/bundle/macos/Ginger Code.app`.

## Scope of this implementation

This is a working development build, not a release candidate. Harnesses currently share the primary workspace; automatic worktree isolation, review/apply workflows, restart persistence, filesystem watching, and bundled Neovim distribution remain unfinished. The pre-existing Part II service modules remain in the repository, but nonexistent IPC commands are no longer registered as if implemented. Legacy modules still emit unused-code warnings.

Neovim is resolved from `~/.ginger/runtime/bin/nvim` when provided, otherwise from the system. Closing an editor forcefully can discard unsaved buffers; use `:wq` or `:q` when possible. Session cleanup stops the original and foreground process groups; separately detached processes remain outside that guarantee. Terminal input is bounded; a single paste above 64 KiB or a full input queue produces an error rather than blocking every session.

UI and stream integration follow the [Tauri channel API](https://v2.tauri.app/develop/calling-frontend/) and [xterm.js terminal API](https://xtermjs.org/docs/api/terminal/classes/terminal/).

## License

MIT
