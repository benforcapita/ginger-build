import { useSessionStore } from "@/stores/session-store";
import { SessionTabs } from "@/components/terminal/SessionTabs";
export function Editor() {
  const sessions = useSessionStore((s) => s.sessions);
  const active = useSessionStore((s) => s.active.editor);
  const editor = sessions.find((s) => s.id === active);
  return <section className="editor-panel" aria-label="Neovim editor">
    <header className="panel-header"><span><span className="accent">N</span> NEOVIM</span><span className="muted">your editor. your muscle memory.</span></header>
    {sessions.some((s) => s.kind === "editor") ? <SessionTabs kind="editor" /> : <div className="editor-welcome">
      <div className="vim-mark">[ <span>vim</span> ]</div>
      <h2>A little less clicking.<br />A little more building.</h2>
      <p>Pick a file from the tree to open Neovim.<br />Real Vim motions. Your config. No imitation.</p>
      <div className="vim-keys"><span><kbd>i</kbd> insert</span><span><kbd>:w</kbd> save</span><span><kbd>:q</kbd> quit</span></div>
    </div>}
    <footer className="editor-footer"><span>{editor?.path ?? "No file open"}</span><span>{editor ? editor.exited ? "EXITED" : "NEOVIM" : "READY"}</span></footer>
  </section>;
}
