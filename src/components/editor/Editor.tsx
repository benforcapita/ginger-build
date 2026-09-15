import { useSessionStore } from "@/stores/session-store";
import { SessionTabs } from "@/components/terminal/SessionTabs";
export function Editor() {
  const sessions = useSessionStore((s) => s.sessions);
  const active = useSessionStore((s) => s.active.editor);
  const editor = sessions.find((s) => s.id === active);
  return <section className="editor-panel" aria-label="Ginger file editor">
    <header className="panel-header"><span><span className="accent">g.</span> FILE EDITOR</span><span className="muted">Vim motions. Ginger editor.</span></header>
    {sessions.some((s) => s.kind === "editor") ? <SessionTabs kind="editor" /> : <div className="editor-welcome">
      <div className="vim-mark">[ <span>vim</span> ]</div>
      <h2>A little less clicking.<br />A little more building.</h2>
      <p>Pick a file from the tree or search with ⌘K.<br />Vim motions, with a built-in file editor.</p>
      <div className="vim-keys"><span><kbd>i</kbd> insert</span><span><kbd>:w</kbd> save</span><span><kbd>:q</kbd> quit</span></div>
    </div>}
    <footer className="editor-footer"><span>{editor?.path ?? "No file open"}</span><span>{editor ? "GINGER EDITOR" : "READY"}</span></footer>
  </section>;
}
