import { useEffect, useRef, useState } from "react";
export interface PaletteAction { id: string; title: string; shortcut?: string; run: () => void; disabled?: boolean }
export function CommandPalette({ actions, onClose }: { actions: PaletteAction[]; onClose: () => void }) {
  const [query, setQuery] = useState("");
  const [index, setIndex] = useState(0);
  const element = useRef<HTMLDivElement>(null);
  const filtered = actions.filter((a) => a.title.toLowerCase().includes(query.toLowerCase()) && !a.disabled);
  const choose = (action?: PaletteAction) => { if (action) { onClose(); action.run(); } };
  useEffect(() => {
    const previous = document.activeElement as HTMLElement | null;
    return () => { previous?.focus(); };
  }, []);
  return <div className="palette-overlay" onClick={onClose}>
    <div ref={element} className="palette" role="dialog" aria-modal="true" aria-label="Command palette" onClick={(e) => e.stopPropagation()} onKeyDown={(e) => {
      if (e.key === "Escape") { e.stopPropagation(); onClose(); }
      if (e.key === "ArrowDown") { e.preventDefault(); setIndex((i) => Math.min(i + 1, filtered.length - 1)); }
      if (e.key === "ArrowUp") { e.preventDefault(); setIndex((i) => Math.max(i - 1, 0)); }
      if (e.key === "Enter") { e.preventDefault(); choose(filtered[index]); }
      if (e.key === "Tab") {
        const nodes = Array.from(element.current?.querySelectorAll<HTMLElement>("input, button") ?? []);
        const first = nodes[0], last = nodes.at(-1);
        if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last?.focus(); }
        else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first?.focus(); }
      }
    }}>
      <div className="palette-search"><span>›</span><input autoFocus aria-label="Search commands" value={query} placeholder="What would you like to do?" onChange={(e) => { setQuery(e.target.value); setIndex(0); }} /><kbd>esc</kbd></div>
      <div className="palette-results">{filtered.map((action, i) => <button key={action.id} className={`palette-item ${i === index ? "selected" : ""}`} onClick={() => choose(action)}><span>{action.title}</span><kbd>{action.shortcut}</kbd></button>)}{!filtered.length && <p className="muted">No matching commands.</p>}</div>
    </div>
  </div>;
}
