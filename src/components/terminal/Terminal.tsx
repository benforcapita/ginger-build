import { useSessionStore } from "@/stores/session-store";
import { useWorkspaceStore } from "@/stores/workspace-store";
import { SessionTabs } from "./SessionTabs";
export function Terminal() {
  const sessions = useSessionStore((s) => s.sessions);
  const start = useSessionStore((s) => s.start);
  const busy = useSessionStore((s) => s.busy);
  const open = useWorkspaceStore((s) => s.status.open);
  const hasShell = sessions.some((s) => s.kind === "shell");
  return <section className="shell-panel" aria-label="Shell terminal">
    <header className="panel-header"><span><span className="accent">›_</span> TERMINAL</span><button disabled={!open || busy} onClick={() => void start("shell")} title="New terminal">+ New shell</button></header>
    {hasShell ? <SessionTabs kind="shell" /> : <div className="empty-terminal"><span className="prompt">ginger<span className="muted"> $</span></span><p>{open ? "Your shell, right beside your code." : "Open a folder to start a shell."}</p><button disabled={!open || busy} onClick={() => void start("shell")}>Start terminal <span>↵</span></button></div>}
  </section>;
}
