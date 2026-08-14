import { useEffect, useState, useMemo } from "react";
import { useActionStore, type ActionDef } from "@/stores/action-store";
import { useAgentStore } from "@/stores/agent-store";
import "./command-palette.css";

interface PaletteItem {
  type: "action" | "agent" | "file";
  id: string;
  title: string;
  subtitle: string;
  icon: string;
}

export function CommandPalette({ onClose }: { onClose: () => void }) {
  const actions = useActionStore((s) => s.actions);
  const invokeAction = useActionStore((s) => s.invokeAction);
  const agents = useAgentStore((s) => s.agents);
  const [query, setQuery] = useState("");

  const items = useMemo<PaletteItem[]>(() => {
    const actionItems: PaletteItem[] = actions.map((a) => ({
      type: "action",
      id: a.id,
      title: a.title,
      subtitle: a.category,
      icon: "⚡",
    }));
    const agentItems: PaletteItem[] = agents.map((a) => ({
      type: "agent",
      id: String(a.id),
      title: a.title,
      subtitle: a.adapter_id,
      icon: "🤖",
    }));
    return [...actionItems, ...agentItems];
  }, [actions, agents]);

  const filtered = useMemo(() => {
    if (!query) return items.slice(0, 20);
    const q = query.toLowerCase();
    return items.filter((i) =>
      i.title.toLowerCase().includes(q) || i.subtitle.toLowerCase().includes(q)
    ).slice(0, 20);
  }, [items, query]);

  const handleSelect = (item: PaletteItem) => {
    if (item.type === "action") {
      invokeAction(item.id);
    }
    onClose();
  };

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [onClose]);

  return (
    <div className="palette-overlay" onClick={onClose}>
      <div className="palette" onClick={(e) => e.stopPropagation()}>
        <input
          className="palette-input"
          placeholder="Search actions, agents, files..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          autoFocus
        />
        <div className="palette-results">
          {filtered.map((item) => (
            <button
              key={`${item.type}:${item.id}`}
              className="palette-item"
              onClick={() => handleSelect(item)}
            >
              <span className="palette-icon">{item.icon}</span>
              <div className="palette-item-text">
                <span className="palette-item-title">{item.title}</span>
                <span className="palette-item-sub">{item.subtitle}</span>
              </div>
            </button>
          ))}
          {filtered.length === 0 && (
            <div className="palette-empty">No results</div>
          )}
        </div>
      </div>
    </div>
  );
}