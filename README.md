# Ginger Code

A macOS desktop IDE with a terminal aesthetic, a built-in editor with Vim keybindings, a project tree, independent CLI harness tabs, and Ginger: an orange-haired text-art code companion.

## What works

- Open a local folder and expand its file tree. Directories load on demand; refresh picks up filesystem changes. Links outside the open workspace are excluded.
- Open files in Ginger’s CodeMirror editor. Vim mode is enabled by default: `i` to insert, `Esc` for Normal mode, `v` for Visual mode, `:w` to save, `:q` to close, and `:wq` to save and close. Neovim is not required; Neovim configuration and plugins do not apply. Toggle Vim mode through ⌘K.
- Run login shells alongside the editor with live input/output and pane resizing.
- Start Claude Code, Codex, OpenCode, or a custom executable with an argument list in a separate harness tab. Installed programs are detected; each CLI handles its own authentication. Up to three live harnesses share the open workspace.
- Pet Ginger, mute commentary, or minimize the portrait while harnesses run. Session counts and exit indicators reflect real processes.
- Search files, folders, and every current workspace action with the command line (`⌘ K` or `⌘ P`). Live session closure and app exit warn before stopping processes. Close tabs before switching workspaces.

## Develop

Requires macOS, Node.js, pnpm, a Rust toolchain. The Tauri launcher locates an installed rustup toolchain even when Cargo is absent from PATH. Ginger also checks common Homebrew and user executable directories when started from Finder.

```sh
pnpm install
pnpm tauri dev
```

`pnpm dev` runs the browser-only visual preview. Local files and processes require the desktop app.

| Shortcut | Action |
| --- | --- |
| ⌘ O | Open folder |
| ⌘ P / ⌘ K | Command palette |
| ⌘ S | Save the active file |
| ⌘ ⇧ T | New shell |
| ⌘ ⇧ N | New harness |

Ctrl keys use Vim bindings in the editor and pass through to terminal programs.

## Keyboard file tree

Press **⌘⇧E**, click **PROJECT**, or run **Focus file tree** from ⌘K. The amber row is your keyboard cursor.

- `j` / `k` or Down / Up move through visible entries.
- Enter expands or collapses a folder; on a file it opens the file and moves focus to the editor.
- `l` / Right expands a folder, then moves into its first visible child. `h` / Left collapses a folder or moves to its parent.
- Space toggles a single selection marker without opening the entry. Moving the cursor leaves that selection marked; Space on another entry replaces it. Refresh clears the selection.
- `g` / Home jumps to the first entry; `G` / End jumps to the last.

Press ⌘⇧E again to return from an editor or harness. The file tree stays on the left.

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

The script installs locked dependencies, builds the app, packages it with an Applications shortcut, and verifies the DMG. The result is in `artifacts/`. Transfer the DMG to another Mac, open it, and drag **Ginger Code.app** into **Applications**. Install your chosen agent CLIs on that Mac separately; the file editor is included.

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

This is a working development build, not a release candidate. Harnesses currently share the primary workspace; automatic worktree isolation, review/apply workflows, restart persistence, native filesystem watching remain unfinished. The pre-existing Part II service modules remain in the repository, but nonexistent IPC commands are no longer registered as if implemented. Legacy modules still emit unused-code warnings.

Editor files must be UTF-8 text up to 5 MiB. LF and CRLF line endings are preserved; mixed line endings are normalized to the first line’s style when edited. Unsaved changes show a dot on the tab and require confirmation before closing or quitting. Disk changes are checked every three seconds and on window focus: clean buffers reload, modified buffers show a conflict. Saves check disk contents again and use an atomic replacement. External writers can still race the final check/rename; this is not a filesystem transaction. Use “Reload active file from disk” to discard local edits after confirmation. Find and replace is available through ⌘K or ⌘F. Tab buffers and undo history are not persisted across app restarts. Session cleanup stops the original and foreground process groups; separately detached processes remain outside that guarantee. Terminal input is bounded; a single paste above 64 KiB or a full input queue produces an error rather than blocking every session.

UI and stream integration follow the [Tauri channel API](https://v2.tauri.app/develop/calling-frontend/) and [xterm.js terminal API](https://xtermjs.org/docs/api/terminal/classes/terminal/).

## License

MIT


## Language servers and completion

Syntax highlighting loads automatically for recognized extensions. Plain-text word suggestions work without installing a server, including in Vim Insert mode. Use **⌘K → Show autocomplete suggestions** (or Ctrl-Space) to request suggestions explicitly.

Open **⌘K → Language servers: status and setup** to see installed tools and connection status. Click **Install in terminal** for the language you need, wait for the command to finish, then **Restart language servers**. Node-based servers need Node.js/npm. Rust needs a rustup-managed toolchain. These tools are separate from the DMG.

| Files | Server | Installation |
| --- | --- | --- |
| JS / JSX / TS / TSX / MJS / CJS / MTS / CTS | typescript-language-server | `npm install --prefix ~/.ginger/language-servers typescript@6.0.3 typescript-language-server` |
| Python / PYI | Pyright | `npm install --prefix ~/.ginger/language-servers pyright` |
| Rust | rust-analyzer | `rustup component add rust-analyzer rust-src` |
| JSON / JSONC | VS Code JSON language server | `npm install --prefix ~/.ginger/language-servers vscode-langservers-extracted` |

A compatible project TypeScript installation takes precedence over the managed TypeScript 6.0.3 fallback. TypeScript 7 packages without `tsserver.js` cannot supply the fallback used by this server. Rust projects should include Cargo.toml; Python projects can use pyrightconfig.json or pyproject.toml. JSON schema completion uses a file's `$schema` declaration when provided.

The command palette exposes diagnostics, definitions, references, cross-file rename, formatting, and signature help. Standard shortcuts include F12 for definition, Shift-F12 for references, F2 for rename, Shift-Option-F for formatting, and ⌘⇧Space for signature help. Availability depends on the server: Pyright, for example, supplies analysis but no formatter. Hover over a symbol for documentation.

Rename and format edit buffers without writing to disk. Save each changed tab with ⌘S or :w. Cross-file edits are restricted to the current workspace; unsupported file creation/renaming operations are rejected. Completion and diagnostics synchronize unsaved edits. One server per language family is shared across tabs, and connections stop on workspace changes or app close. Servers with missing dependencies or startup failures leave the editor usable; inspect their message in the setup panel and restart after repair.

Optional installed-server smoke tests: `node scripts/smoke-language-servers.mjs`. These create and remove a temporary workspace and check real completions and diagnostics for all four servers.
