import { useEffect } from "react";
import { useActionStore } from "@/stores/action-store";
import { Explorer } from "@/components/explorer/Explorer";
import { Editor } from "@/components/editor/Editor";
import { AgentDock } from "@/components/agent-dock/AgentDock";
import { Terminal } from "@/components/terminal/Terminal";
import { StatusBar } from "@/components/presence/StatusBar";
import { ActivityRail } from "@/components/ActivityRail";

export default function App() {
  const loadActions = useActionStore((s) => s.loadActions);

  useEffect(() => {
    loadActions();
  }, [loadActions]);

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
    </div>
  );
}