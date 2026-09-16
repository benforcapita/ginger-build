import { useSessionStore } from "@/stores/session-store";
import { SessionTabs } from "@/components/terminal/SessionTabs";
export function Editor() {
  const sessions = useSessionStore((s) => s.sessions);
  const active = useSessionStore((s) => s.active.editor);
  const editor = sessions.find((s) => s.id === active);
  return <section className="editor-panel" aria-label="Ginger file editor">
    <header className="panel-header"><span><span className="accent">g.</span> FILE EDITOR</span><span className="muted">Your files. Your editor.</span></header>
    {sessions.some((s) => s.kind === "editor") ? <SessionTabs kind="editor" /> : <div className="editor-welcome">
      <div className="vim-mark">[ <span>g.</span> ]</div>
      <h2>A little less clicking.<br />A little more building.</h2>
      <p>Pick a file from the tree or search with ⌘K.<br />Edit, save, and build in Ginger.</p>
      <div className="vim-keys"><span><kbd>⌘K</kbd> commands</span><span><kbd>⌘S</kbd> save</span><span><kbd>⌘2</kbd> focus editor</span></div>
    </div>}
    <footer className="editor-footer"><span>{editor?.path ?? "No file open"}</span><span>{editor ? "GINGER EDITOR" : "READY"}</span></footer>
  </section>;
}
