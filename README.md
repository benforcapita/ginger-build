# Ginger Code

A macOS desktop IDE with a terminal aesthetic, real Neovim, a project tree, independent CLI harness tabs, and Ginger: an orange-haired text-art code companion.

## What works

- Open a local folder and expand its file tree. Directories load on demand; refresh picks up filesystem changes. Links outside the open workspace are excluded.
- Open files in Neovim tabs. Use your normal Vim controls and user configuration: `i` to insert, `Esc` to return to normal mode, `:w` to save, `:q` to quit. Each tab owns a Neovim process.
- Run login shells alongside the editor with live input/output and pane resizing.
- Start Claude Code, Codex, OpenCode, or a custom executable with an argument list in a separate harness tab. Installed programs are detected; each CLI handles its own authentication. Up to three live harnesses share the open workspace.
- Pet Ginger, mute commentary, or minimize the portrait while harnesses run. Session counts and exit indicators reflect real processes.
- Search files, folders, and every current workspace action with the command line (`⌘ K` or `⌘ P`). Live session closure and app exit warn before stopping processes. Close tabs before switching workspaces.

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

## Movable tabs and panes

The focused editor, agent, or terminal pane takes the large center area by default. The file tree stays on the left and the other panes stack on the right. Selecting a tab or clicking its terminal centers its pane without restarting the session.

- Drag a tab’s ⠿ grip onto another tab in the same pane to reorder it. With a tab button focused, Option+Shift+Left/Right also changes its order.
- Drag a pane’s ⠿ handle onto another pane to swap positions. A focused handle also accepts Left/Right to cycle positions.
- Use ⌘K to move panes to center/upper right/lower right, reorder tabs, disable automatic centering, or reset the arrangement. Manual placement lasts until another focus change when automatic centering is enabled.
- Layout and tab order are kept for the current app session. Session kinds stay grouped; editor tabs remain editors and harness tabs remain agents.

## Command line (⌘ K)

Type a filename, part of its path, or an action name. Fuzzy matching works, so `agdock` finds `AgentDock.tsx`. Use `>` to search only actions or `/` to search files and folders. Arrow keys select a result; Enter runs it; Escape returns to your workspace.

Available commands include opening a project, saving the active editor, starting shells and harnesses, focusing and closing individual sessions, refreshing or collapsing the tree, expanding individual folders, showing/minimizing/petting Ginger, muting commentary, dismissing errors, and quitting. Harness setup opens the existing keyboard-accessible executable/arguments form. Disabled commands explain what is needed to use them.

File search includes hidden files and is refreshed each time you open the command line. It excludes `.git`, `node_modules`, `target`, `dist`, `build`, `.next`, and `.venv` directories. Directory symlinks are not traversed; file symlinks must remain inside the workspace. Indexing is bounded to 20,000 entries, with a notice for incomplete results. The first 100 matching results are shown; keep typing to narrow them.

## Build a macOS installer

On a Mac with Xcode Command Line Tools, Node.js (22.18+ for tests), pnpm, and Rust installed:

```sh
./scripts/build-dmg.sh
# Faster development installer:
./scripts/build-dmg.sh --debug
# Equivalent package command:
pnpm build:dmg
```

The script installs locked dependencies, builds the app, packages it with an Applications shortcut, and verifies the DMG. The result is in `artifacts/`. Transfer the DMG to another Mac, open it, and drag **Ginger Code.app** into **Applications**. Install Neovim (`brew install neovim`) and your chosen agent CLIs on that Mac separately.

By default it builds for the current Mac's architecture. To build for another architecture, install the Rust target and pass it explicitly:

```sh
rustup target add x86_64-apple-darwin
./scripts/build-dmg.sh --target x86_64-apple-darwin
# Both Intel and Apple Silicon (requires both Rust targets):
rustup target add aarch64-apple-darwin x86_64-apple-darwin
./scripts/build-dmg.sh --target universal-apple-darwin
```

Cross-architecture builds require a compatible Xcode SDK and native dependencies. The app requires macOS 12 or newer. Without Apple signing/notarization credentials this is an unsigned development installer, which Gatekeeper may block. The script preserves Tauri signing environment configuration; it does not install certificates or change security settings.

## Build and verify

```sh
pnpm build
pnpm test
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
