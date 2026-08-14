import { useActionStore } from "@/stores/action-store";

export function StatusBar() {
  const ctx = useActionStore((s) => s.context);
  return (
    <div className="status-bar">
      <div className="status-left">
        <span className="status-item">🫚 Ginger Code v0.1.0</span>
        {ctx.workspace_open && <span className="status-item">Workspace: Open</span>}
        {ctx.safe_mode && <span className="status-item status-warning">SAFE MODE</span>}
      </div>
      <div className="status-right">
        <span className="status-item">{ctx.agent_count} agents</span>
        <span className="status-item">{ctx.terminal_count} terminals</span>
      </div>
    </div>
  );
}