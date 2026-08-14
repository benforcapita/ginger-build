import { useEffect, useState } from "react";
import { useActionStore } from "@/stores/action-store";
import { Explorer } from "@/components/explorer/Explorer";
import { Editor } from "@/components/editor/Editor";
import { AgentDock } from "@/components/agent-dock/AgentDock";
import { Terminal } from "@/components/terminal/Terminal";
import { StatusBar } from "@/components/presence/StatusBar";
import { ActivityRail } from "@/components/ActivityRail";
import { CommandPalette } from "@/components/CommandPalette";

export default function App() {
  const loadActions = useActionStore((s) => s.loadActions);
  const [paletteOpen, setPaletteOpen] = useState(false);

  useEffect(() => {
    loadActions();
  }, [loadActions]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "p") {
        e.preventDefault();
        setPaletteOpen((v) => !v);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  return (
    <div className="app">
      <ActivityRail />
      <div className="app-body">
        <Explorer />
        <Editor />
        <AgentDock />
      </div>
      <div className="app-bottom">
        <Terminal />
      </div>
      <StatusBar />
      {paletteOpen && <CommandPalette onClose={() => setPaletteOpen(false)} />}
    </div>
  );
}