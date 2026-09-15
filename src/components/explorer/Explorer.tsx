import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useWorkspaceStore } from "@/stores/workspace-store";
import { useSessionStore } from "@/stores/session-store";
import { useWorkbenchStore } from "@/stores/workbench-store";
import { treeTarget, type TreeMove } from "@/tree-navigation";
interface Entry { name: string; path: string; is_dir: boolean }
export function Explorer({ onOpenFolder }: { onOpenFolder: () => void }) {
  const workspace = useWorkspaceStore((s) => s.status.workspace);
  const sessions = useSessionStore((s) => s.sessions);
  const revision = useWorkbenchStore((s) => s.treeRevision);
  const refreshTree = useWorkbenchStore((s) => s.refreshTree);
  return <aside className="explorer">
    <header className="panel-header"><button className="tree-focus-button" onClick={() => useWorkbenchStore.getState().focusTree()} title="Focus file tree (⌘⇧E)">PROJECT <kbd>⌘⇧E</kbd></button><div className="header-actions"><button onClick={onOpenFolder} title="Open folder" aria-label="Open folder">⊞</button><button onClick={refreshTree} disabled={!workspace} title="Refresh file tree" aria-label="Refresh file tree">↻</button></div></header>
    <div className="file-tree">
      {workspace ? <><div className="tree-root"><span className="accent">⌄ ▣</span> {workspace.display_name}</div><FileTree key={workspace.root_path} revision={revision} /></> : <div className="explorer-empty"><span className="folder-outline">▱</span><p>Every good idea<br />starts somewhere.</p><button className="primary-button" onClick={onOpenFolder}>Open folder <span>↗</span></button><small>⌘ O</small></div>}
    </div>
    <div className="workspace-facts"><span className="section-label">WORKSPACE</span><dl><dt>Open files</dt><dd>{sessions.filter((s) => s.kind === "editor").length}</dd><dt>Live harnesses</dt><dd>{sessions.filter((s) => s.kind === "agent" && !s.exited).length} / 3</dd><dt>Shell sessions</dt><dd>{sessions.filter((s) => s.kind === "shell").length}</dd></dl><p className="tiny muted">Local files. Real processes.<br />A little company while you code.</p></div>
    <div className="explorer-bottom"><span className="accent">g.</span><span>GINGER CODE<br /><small>v0.1 · work in progress</small></span></div>
  </aside>;
}
function FileTree({ revision }: { revision: number }) {
  const tree = useRef<HTMLDivElement>(null);
  const remembered = useRef<string | null>(null);
  const [cursor, setCursor] = useState<string | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const request = useWorkbenchStore(s => s.treeFocusRequest);
  const rows = () => Array.from(tree.current?.querySelectorAll<HTMLElement>('[data-tree-path]') ?? []);
  const move = (direction: TreeMove) => {
    const next = treeTarget(rows().map(row => row.dataset.treePath!), remembered.current, direction);
    remembered.current = next;
    setCursor(next);
  };
  useEffect(() => {
    const element = tree.current;
    if (!element) return;
    const reconcile = () => {
      const next = treeTarget(rows().map(row => row.dataset.treePath!), remembered.current, 'restore');
      if (next) remembered.current = next;
      setCursor(next);
    };
    const observer = new MutationObserver(reconcile);
    observer.observe(element, { childList: true, subtree: true });
    reconcile();
    return () => observer.disconnect();
  }, []);
  useEffect(() => { if (request) tree.current?.focus(); }, [request]);
  useEffect(() => { setSelected(null); }, [revision]);
  useEffect(() => {
    if (document.activeElement === tree.current) rows().find(row => row.dataset.treePath === cursor)?.querySelector('.tree-entry')?.scrollIntoView({ block: 'nearest' });
  }, [cursor]);
  const activate = (entry: Entry) => {
    remembered.current = entry.path;
    setCursor(entry.path);
    tree.current?.focus();
    if (entry.is_dir) useWorkbenchStore.getState().toggleDirectory(entry.path);
    else if (!useSessionStore.getState().busy) void useSessionStore.getState().start('editor', { path: entry.path });
  };
  return <>
    <div ref={tree} className="keyboard-tree" role="tree" aria-label="Project files" aria-describedby="tree-key-help" tabIndex={0} aria-activedescendant={cursor ? treeId(cursor) : undefined} onFocus={() => move('restore')} onKeyDown={event => {
      if (event.metaKey || event.ctrlKey || event.altKey) return;
      const directions: Record<string, TreeMove> = { j: 'down', ArrowDown: 'down', k: 'up', ArrowUp: 'up', Home: 'first', End: 'last', g: 'first', G: 'last' };
      const direction = directions[event.key];
      if (direction) { event.preventDefault(); move(direction); return; }
      if (!['Enter', ' ', 'h', 'l', 'ArrowLeft', 'ArrowRight'].includes(event.key)) return;
      event.preventDefault();
      const row = rows().find(row => row.dataset.treePath === cursor);
      if (!row || !cursor) return;
      if (event.key === ' ') { setSelected(value => value === cursor ? null : cursor); return; }
      const isDirectory = row.dataset.treeDirectory === 'true';
      const expanded = useWorkbenchStore.getState().expanded.has(cursor);
      if (event.key === 'Enter') { activate({ path: cursor, name: '', is_dir: isDirectory }); return; }
      if (event.key === 'h' || event.key === 'ArrowLeft') {
        if (isDirectory && expanded) useWorkbenchStore.getState().toggleDirectory(cursor);
        else move('parent');
      } else if (isDirectory && !expanded) useWorkbenchStore.getState().toggleDirectory(cursor);
      else move('child');
    }}>
      <Directory path="" depth={0} revision={revision} cursor={cursor} selected={selected} activate={activate} />
    </div>
    <div id="tree-key-help" className="tree-key-help">j/k move · Enter open<br />h/l folders · Space select</div>
  </>;
}
function treeId(path: string) { return `tree-item-${encodeURIComponent(path)}`; }
interface DirectoryProps { path: string; depth: number; revision: number; cursor: string | null; selected: string | null; activate: (entry: Entry) => void }
function Directory({ path, depth, revision, cursor, selected, activate }: DirectoryProps) {
  const [entries, setEntries] = useState<Entry[]>([]);
  const expanded = useWorkbenchStore((s) => s.expanded);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
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
  if (loading) return <p className="tree-loading" role="status">Reading…</p>;
  if (!entries.length) return <p className="tree-loading">Empty folder</p>;
  return <ul className="tree-list" role="group">{entries.map((entry) => <li key={entry.path} id={treeId(entry.path)} role="treeitem" aria-label={entry.name} aria-level={depth + 1} aria-expanded={entry.is_dir ? expanded.has(entry.path) : undefined} aria-selected={entry.path === selected} aria-disabled={!entry.is_dir && busy || undefined} data-tree-path={entry.path} data-tree-directory={entry.is_dir}>
    <div className={`tree-entry ${entry.path === activePath ? "active" : ""} ${entry.path === cursor ? "tree-cursor" : ""} ${entry.path === selected ? "tree-selected" : ""}`} style={{ paddingLeft: 16 + depth * 14 }} title={entry.path} onClick={() => activate(entry)}><span className="tree-selection" aria-hidden="true">{entry.path === selected ? '◆' : ''}</span><span className="tree-chevron">{entry.is_dir ? expanded.has(entry.path) ? "⌄" : "›" : ""}</span><span className={entry.is_dir ? "folder-icon" : "file-icon"}>{entry.is_dir ? "▱" : fileIcon(entry.name)}</span><span className="entry-name">{entry.name}</span></div>
    {entry.is_dir && expanded.has(entry.path) && <Directory path={entry.path} depth={depth + 1} revision={revision} cursor={cursor} selected={selected} activate={activate} />}
  </li>)}</ul>;
}
function fileIcon(name: string) { const ext = name.split(".").at(-1); return ext === "ts" || ext === "tsx" ? "TS" : ext === "rs" ? "rs" : ext === "json" ? "{}" : ext === "md" ? "≡" : "·"; }
