# Integrated editor implementation plan

Goal: Replace terminal Neovim with CodeMirror 6, Vim enabled by default.
Architecture: Editor tabs retain the shared session tab arrangement and focus routing, but use negative document IDs and document state instead of PTYs. Rust provides bounded UTF-8 reads and atomic saves with an expected disk-content check. Shell and harness sessions retain PTYs.

- Add document reconciliation tests: clean buffers reload external edits; dirty buffers retain text and report conflicts; saves preserve edits made during IO.
- Add workspace file IO tests: read/save UTF-8 and CRLF, reject binary/oversize and outside-root paths, reject stale saves without changing disk.
- Implement Rust read/save commands and register IPC; stage writes in the same directory and preserve permissions.
- Integrate document open/save/reload/check operations into the session store. Serialize saves, prevent closing while saving, and confirm dirty closes and quit.
- Render CodeMirror views with Vim, language loading, search, mode indicator, stable view lifetime, and existing focus/layout behavior. Connect :w, :q and command palette actions.
- Run frontend tests/build and Rust tests, independent review, build macOS installer and smoke-check editor interactions.

Limits: UTF-8 text files up to 5 MiB; no Neovim plugin compatibility or language-server integration in this change. External edits are checked periodically and on focus, and verified again at save time. Disk check and atomic rename are best effort against concurrent external writers, not a filesystem transaction.
