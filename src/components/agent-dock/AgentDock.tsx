import { useEffect } from "react";
import { useAgentStore, type AgentThread } from "@/stores/agent-store";
import "./agent-dock.css";

export function AgentDock() {
  const agents = useAgentStore((s) => s.agents);
  const refresh = useAgentStore((s) => s.refresh);

  useEffect(() => {
    refresh();
  }, [refresh]);

  if (agents.length === 0) {
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

  return (
    <div className="agent-dock">
      <div className="agent-dock-header">
        AGENTS
        <span className="agent-dock-count">{agents.length}</span>
      </div>
      <div className="agent-dock-list">
        {agents.map((agent) => (
          <AgentCard key={agent.id} agent={agent} />
        ))}
      </div>
    </div>
  );
}

function AgentCard({ agent }: { agent: AgentThread }) {
  const statusClass = `agent-status agent-status-${agent.status}`;
  const statusLabel = agent.status.charAt(0).toUpperCase() + agent.status.slice(1);

  return (
    <div className="agent-card">
      <div className="agent-card-header">
        <span className="agent-adapter">{agent.adapter_id}</span>
        <span className={statusClass}>{statusLabel}</span>
      </div>
      <div className="agent-card-title">{agent.title}</div>
      <div className="agent-card-meta">
        <span>Mode: {agent.mode}</span>
        <span>Isolation: {agent.isolation}</span>
      </div>
      {agent.worktree_branch && (
        <div className="agent-card-branch">🌿 {agent.worktree_branch}</div>
      )}
      {agent.started_at && (
        <div className="agent-card-time">
          Started: {new Date(agent.started_at).toLocaleTimeString()}
        </div>
      )}
    </div>
  );
}