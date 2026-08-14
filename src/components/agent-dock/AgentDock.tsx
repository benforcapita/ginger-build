export function AgentDock() {
  return (
    <div className="agent-dock">
      <div className="agent-dock-header">AGENTS</div>
      <div className="agent-dock-empty">
        <p>No agents running</p>
        <p className="agent-dock-hint">Press Cmd+Shift+N to start one</p>
      </div>
    </div>
  );
}