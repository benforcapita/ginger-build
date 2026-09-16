import { useEffect, useMemo, useRef, useState } from "react";
import { flushSync } from "react-dom";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { rankResults, type Searchable } from "@/command-search";
import { useWorkspaceStore } from "@/stores/workspace-store";
import { useSessionStore } from "@/stores/session-store";
import { useWorkbenchStore } from "@/stores/workbench-store";
export interface PaletteAction extends Searchable { id: string; shortcut?: string; run: () => void | Promise<void>; disabled?: boolean; reason?: string }
interface FileIndex { entries: { name: string; path: string; is_dir: boolean }[]; truncated: boolean; skipped: number }
export function CommandPalette({ actions, onClose, returnFocus }: { actions: PaletteAction[]; onClose: () => void; returnFocus: HTMLElement | null }) {
  const [query, setQuery] = useState("");
  const [index, setIndex] = useState(0);
  const [files, setFiles] = useState<FileIndex | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const element = useRef<HTMLDivElement>(null);
  const input = useRef<HTMLInputElement>(null);
  const selectedElement = useRef<HTMLButtonElement>(null);
  const root = useWorkspaceStore((s) => s.status.workspace?.root_path);
  const revision = useWorkbenchStore((s) => s.treeRevision);
  const expanded = useWorkbenchStore((s) => s.expanded);
  const busy = useSessionStore((s) => s.busy);
  useEffect(() => {
    let cancelled = false;
    setFiles(null); setError(null);
    if (!root || !isTauri()) return;
    setLoading(true);
    void invoke<FileIndex>("workspace_file_index").then((value) => { if (!cancelled) setFiles(value); })
      .catch((e) => { if (!cancelled) setError(String(e)); })
      .finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, [root, revision]);
  const entries = useMemo<PaletteAction[]>(() => [
    ...actions,
    ...(files?.entries ?? []).map((entry): PaletteAction => ({
      id: `path:${entry.path}`, title: entry.path, kind: entry.is_dir ? "directory" : "file",
      shortcut: entry.is_dir ? expanded.has(entry.path) ? "Collapse folder" : "Expand folder" : "Open in editor",
      disabled: !entry.is_dir && busy, reason: "A session is starting",
      run: () => entry.is_dir ? useWorkbenchStore.getState().toggleDirectory(entry.path) : useSessionStore.getState().start("editor", { path: entry.path }),
    })),
  ], [actions, files, expanded, busy]);
  const matches = useMemo(() => rankResults(entries, query), [entries, query]);
  const filtered = matches.slice(0, 100);
  const selected = Math.max(0, Math.min(index, filtered.length - 1));
  const choose = (action?: PaletteAction) => {
    if (!action || action.disabled) return;
    flushSync(onClose);
    void Promise.resolve().then(action.run).catch((e) => useSessionStore.getState().setError(String(e)));
  };
  useEffect(() => {
    input.current?.focus();
    return () => { if (returnFocus?.isConnected) returnFocus.focus(); };
  }, [returnFocus]);
  useEffect(() => { selectedElement.current?.scrollIntoView({ block: "nearest" }); }, [selected, query]);
  return <div className="palette-overlay" onClick={onClose}>
    <div ref={element} className="palette" role="dialog" aria-modal="true" aria-label="Command line" onClick={(e) => e.stopPropagation()} onKeyDown={(e) => {
      if (e.key === "Escape") { e.stopPropagation(); onClose(); }
      if (e.key === "ArrowDown") { e.preventDefault(); setIndex(Math.min(selected + 1, filtered.length - 1)); }
      if (e.key === "ArrowUp") { e.preventDefault(); setIndex(Math.max(selected - 1, 0)); }
      if (e.key === "Enter" && e.target instanceof HTMLInputElement) { e.preventDefault(); choose(filtered[selected]); }
      if (e.key === "Tab") {
        const nodes = Array.from(element.current?.querySelectorAll<HTMLElement>("input, button:not(:disabled)") ?? []);
        const first = nodes[0], last = nodes.at(-1);
        if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last?.focus(); }
        else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first?.focus(); }
      }
    }}>
      <div className="palette-search"><span>›</span><input ref={input} autoFocus role="combobox" aria-expanded="true" aria-controls="command-results" aria-activedescendant={filtered[selected] ? `command-${selected}` : undefined} aria-label="Search files and actions" value={query} placeholder="Search files and actions…" onChange={(e) => { setQuery(e.target.value); setIndex(0); }} /><kbd>esc</kbd></div>
      <div className="palette-help"><span><kbd>&gt;</kbd> actions only · <kbd>/</kbd> files & folders</span><span>↑ ↓ select · ↵ run</span></div>
      <div className="palette-results" id="command-results" role="listbox" aria-label="Search results">{filtered.map((action, i) => <button ref={i === selected ? selectedElement : undefined} id={`command-${i}`} key={action.id} role="option" aria-selected={i === selected} aria-disabled={action.disabled || undefined} className={`palette-item ${i === selected ? "selected" : ""} ${action.disabled ? "unavailable" : ""}`} onClick={() => choose(action)}>
        <span className="palette-result-title"><span className="muted">{action.kind === "file" ? "·" : action.kind === "directory" ? "▱" : ">"}</span> {action.title}</span><small>{action.disabled ? action.reason ?? "Unavailable right now" : action.shortcut ?? "Action"}</small>
      </button>)}{!filtered.length && <p className="muted">{loading ? "Searching project files…" : "No matching files or actions."}</p>}</div>
      <div className="palette-status" role="status">{error ? `File search failed: ${error}` : loading ? "Indexing project files…" : !root ? "Open a folder to search files." : `${matches.length} results${matches.length > 100 ? " · showing first 100, keep typing" : ""}${files?.truncated ? " · file index limited to 20,000 entries" : ""}${files?.skipped ? ` · ${files.skipped} unreadable or deep folders skipped` : ""}`}</div>
    </div>
  </div>;
}
