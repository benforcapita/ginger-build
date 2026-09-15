import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useWorkspaceStore } from "@/stores/workspace-store";
import { useSessionStore } from "@/stores/session-store";
interface Entry { name: string; path: string; is_dir: boolean }
export function Explorer({ onOpenFolder }: { onOpenFolder: () => void }) {
  const workspace = useWorkspaceStore((s) => s.status.workspace);
  const sessions = useSessionStore((s) => s.sessions);
  const [revision, setRevision] = useState(0);
  return <aside className="explorer">
    <header className="panel-header"><span>PROJECT</span><div className="header-actions"><button onClick={onOpenFolder} title="Open folder" aria-label="Open folder">⊞</button><button onClick={() => setRevision((n) => n + 1)} disabled={!workspace} title="Refresh file tree" aria-label="Refresh file tree">↻</button></div></header>
    <div className="file-tree">
      {workspace ? <><div className="tree-root"><span className="accent">⌄ ▣</span> {workspace.display_name}</div><Directory key={workspace.root_path} path="" depth={0} revision={revision} /></> : <div className="explorer-empty"><span className="folder-outline">▱</span><p>Every good idea<br />starts somewhere.</p><button className="primary-button" onClick={onOpenFolder}>Open folder <span>↗</span></button><small>⌘ O</small></div>}
    </div>
    <div className="workspace-facts"><span className="section-label">WORKSPACE</span><dl><dt>Open files</dt><dd>{sessions.filter((s) => s.kind === "editor").length}</dd><dt>Live harnesses</dt><dd>{sessions.filter((s) => s.kind === "agent" && !s.exited).length} / 3</dd><dt>Shell sessions</dt><dd>{sessions.filter((s) => s.kind === "shell").length}</dd></dl><p className="tiny muted">Local files. Real processes.<br />A little company while you code.</p></div>
    <div className="explorer-bottom"><span className="accent">g.</span><span>GINGER CODE<br /><small>v0.1 · work in progress</small></span></div>
  </aside>;
}
function Directory({ path, depth, revision }: { path: string; depth: number; revision: number }) {
  const [entries, setEntries] = useState<Entry[]>([]);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const start = useSessionStore((s) => s.start);
  const busy = useSessionStore((s) => s.busy);
  const active = useSessionStore((s) => s.active.editor);
  const activePath = useSessionStore((s) => s.sessions.find((v) => v.id === active)?.path);
  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    void invoke<Entry[]>("workspace_list_directory", { path }).then((data) => { if (!cancelled) { setEntries(data); setError(null); } }).catch((e) => { if (!cancelled) setError(String(e)); }).finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [path, revision]);
  if (error) return <p className="tree-error" role="alert">{error}</p>;
  if (loading) return <p className="tree-loading">Reading…</p>;
  if (!entries.length) return <p className="tree-loading">Empty folder</p>;
  return <ul className="tree-list">{entries.map((entry) => <li key={entry.path}>
    <button className={`tree-entry ${entry.path === activePath ? "active" : ""}`} style={{ paddingLeft: 16 + depth * 14 }} aria-expanded={entry.is_dir ? expanded.has(entry.path) : undefined} disabled={!entry.is_dir && busy} title={entry.path} onClick={() => {
      if (entry.is_dir) setExpanded((old) => { const next = new Set(old); if (next.has(entry.path)) next.delete(entry.path); else next.add(entry.path); return next; });
      else void start("editor", { path: entry.path });
    }}><span className="tree-chevron">{entry.is_dir ? expanded.has(entry.path) ? "⌄" : "›" : ""}</span><span className={entry.is_dir ? "folder-icon" : "file-icon"}>{entry.is_dir ? "▱" : fileIcon(entry.name)}</span><span className="entry-name">{entry.name}</span></button>
    {entry.is_dir && expanded.has(entry.path) && <Directory path={entry.path} depth={depth + 1} revision={revision} />}
  </li>)}</ul>;
}
function fileIcon(name: string) { const ext = name.split(".").at(-1); return ext === "ts" || ext === "tsx" ? "TS" : ext === "rs" ? "rs" : ext === "json" ? "{}" : ext === "md" ? "≡" : "·"; }
