# Language servers implementation plan

Goal: Supply code intelligence in Ginger's integrated Vim-compatible editor for TS/JS, Python, Rust and JSON; word completion for other text files.
Architecture: A Rust stdio host owns bounded, framed LSP subprocesses. CodeMirror's LSP client owns protocol synchronization, diagnostics, completion, hover, signature help, navigation, rename and formatting. Connections are shared per workspace and server, and stopped on workspace switch/quit. A status/setup dialog exposes missing binaries and explicit installation actions; editing remains available without a server.

1. Test language mapping, URI boundaries and word completion. Implement pure helpers in src/language-support.ts.
2. Test framed stream parsing; implement src-tauri/src/language_servers.rs with allowlisted server commands, discovery, bounded writes/reads, process cleanup and channel delivery. Add IPC registrations and quit cleanup.
3. Implement shared frontend connections, safe markdown rendering, open-file navigation and guarded edits. Keep edits unsaved and use existing save/conflict protections.
4. Attach LSP compartment to editor and plain-text completion. Add command palette actions and status/setup dialog. Expose explicit completion, diagnostics, definitions, references, rename, format and restart.
5. Test with real installed servers on temporary fixture workspaces, including diagnostics, completion, multi-file navigation, failure and restart. Run frontend/Rust tests, build, independent code review and native smoke checks; rebuild DMG.

Constraints: local stdio only; no arbitrary server-command configuration in this iteration. Supported server installs are explicit and use ~/.ginger/language-servers for Node packages, rustup for rust-analyzer. Only files inside the current workspace may be opened/edited through server requests. No implicit disk writes for rename/format. Server protocol messages capped at 16 MiB and bounded queues. Language-server dependencies are installed separately from the DMG.
