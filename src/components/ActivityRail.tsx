import { useState } from "react";

type RailItem = "explorer" | "search" | "git" | "agents" | "packages" | "settings";

const ITEMS: { id: RailItem; label: string; icon: string }[] = [
  { id: "explorer", label: "Explorer", icon: "📁" },
  { id: "search", label: "Search", icon: "🔍" },
  { id: "git", label: "Source Control", icon: "🌿" },
  { id: "agents", label: "Agents", icon: "🤖" },
  { id: "packages", label: "Packages", icon: "📦" },
  { id: "settings", label: "Settings", icon: "⚙️" },
];

export function ActivityRail() {
  const [active, setActive] = useState<RailItem>("explorer");
  return (
    <div className="activity-rail">
      {ITEMS.map((item) => (
        <button
          key={item.id}
          className={`rail-item ${active === item.id ? "active" : ""}`}
          onClick={() => setActive(item.id)}
          title={item.label}
        >
          <span className="rail-icon">{item.icon}</span>
        </button>
      ))}
    </div>
  );
}