# Ginger desktop workspace

The user approved retaining the Tauri desktop app and the reference layout on September 15, 2026.

Implement the first usable loop: choose a folder, expand its file tree, open a file in real Neovim, edit and save using Vim, and run a shell alongside it. The shell and editor use Rust-owned PTYs rendered by xterm.js. Neovim receives file arguments directly, without shell interpolation. Each open file has its own editor tab/session for this first implementation.

Keep the dark charcoal and amber terminal aesthetic. Place the explorer left, Ginger and harness tabs in the middle, and editor above shell on the right. Ginger is an original text-art companion with messy orange hair, glasses, and a dark hoodie. Its copy reflects actual workspace/session state; do not fabricate tests, progress, or token usage.

Harness tabs run user-selected installed CLI executables (Claude, Codex, OpenCode, or a custom executable and argument list). Each tab owns a separate PTY in the selected workspace. Existing worktree/review services remain available as scaffolding; this iteration clearly labels harnesses as sharing the workspace and does not claim automatic isolation.

Filesystem listing is lazy, directories first, constrained to the canonical open workspace; symlinks escaping the root cannot be opened. Refresh updates the tree. Workspace changes must not terminate active editors implicitly: require closing sessions before switching. Closing a live editor must warn about unsaved buffers.

PTY output is buffered so the first prompt is not lost before the UI attaches. Output and exit are ordered in one stream. Each session is resized, receives keyboard input, and kills its process on explicit close or app exit. Hidden tabs stay mounted to preserve terminal state. Browser preview explains that native functions require the desktop app.

Validation: dependency installation and frontend build; Rust check and tests for workspace boundaries and PTY input/output/exit; UI inspection at reference-like dimensions. Run a real Neovim save round-trip in a temporary workspace when Neovim is available. Report any remaining verification limits.
