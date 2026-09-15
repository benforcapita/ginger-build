import { useEffect, useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { useSessionStore } from "@/stores/session-store";
import { useWorkspaceStore } from "@/stores/workspace-store";
import { GingerMascot } from "@/components/presence/GingerMascot";
import { SessionTabs } from "@/components/terminal/SessionTabs";
interface Harness { name: string; program: string; available: boolean }
export function AgentDock({ launchRequest }: { launchRequest: number }) {
  const sessions = useSessionStore((s) => s.sessions);
  const start = useSessionStore((s) => s.start);
  const busy = useSessionStore((s) => s.busy);
  const open = useWorkspaceStore((s) => s.status.open);
  const [creating, setCreating] = useState(false);
  const [showCompanion, setShowCompanion] = useState(true);
  const [harnesses, setHarnesses] = useState<Harness[]>([]);
  const [selected, setSelected] = useState("custom");
  const [program, setProgram] = useState("");
  const [args, setArgs] = useState("");
  const hasAgents = sessions.some((s) => s.kind === "agent");
  useEffect(() => {
    if (!isTauri()) return;
    void invoke<Harness[]>("terminal_harnesses").then((items) => { setHarnesses(items); setSelected(items.find((h) => h.available)?.program ?? "custom"); }).catch((e) => useSessionStore.getState().setError(String(e)));
  }, []);
  useEffect(() => { if (launchRequest) setCreating(true); }, [launchRequest]);
  return <section className="agent-dock" aria-label="Ginger and agent harnesses">
    {showCompanion && <GingerMascot onMinimize={hasAgents ? () => setShowCompanion(false) : undefined} />}
    {!showCompanion && <div className="ginger-compact"><span className="accent">g.</span> Ginger’s here. Let’s build.<button onClick={() => setShowCompanion(true)}>Show Ginger</button></div>}
    <header className="panel-header"><span>AGENT HARNESSES <span className="count">{sessions.filter((s) => s.kind === "agent").length}</span></span><button onClick={() => setCreating((v) => !v)} disabled={!open || busy} aria-label="New agent harness">{creating ? "−" : "+"}</button></header>
    {creating && <form className="harness-form" onSubmit={async (event) => {
      event.preventDefault();
      const executable = selected === "custom" ? program.trim() : selected;
      await start("agent", { program: executable, args: selected === "custom" ? args.split("\n").filter((s) => s.length > 0) : [], title: harnesses.find((h) => h.program === selected)?.name ?? executable.split("/").at(-1) });
      if (!useSessionStore.getState().error) setCreating(false);
    }}>
      <label>Harness<select value={selected} onChange={(e) => setSelected(e.target.value)}>{harnesses.map((h) => <option key={h.program} value={h.program} disabled={!h.available}>{h.name}{h.available ? "" : " · not installed"}</option>)}<option value="custom">Custom executable</option></select></label>
      {selected === "custom" && <><label>Executable<input autoFocus placeholder="my-agent or /path/to/agent" required value={program} onChange={(e) => setProgram(e.target.value)} /></label><label>Arguments <span className="muted">· one per line</span><textarea rows={2} value={args} onChange={(e) => setArgs(e.target.value)} placeholder={'--model\nmy-model'} /></label></>}
      <p>Runs in this folder with access to your files. Each harness uses its own CLI login and configuration.</p>
      <div className="form-actions"><button type="button" onClick={() => setCreating(false)}>Cancel</button><button className="primary-button" disabled={!open || busy || (selected === "custom" && !program.trim())}>{busy ? "Starting…" : "Start harness ↗"}</button></div>
    </form>}
    {hasAgents ? <SessionTabs kind="agent" /> : !creating && <div className="agent-empty"><span className="terminal-symbol">[ &gt;_ ]</span><h2>Bring your own agent.</h2><p>Claude Code, Codex, OpenCode.<br />Or any CLI that feels like home.</p><button disabled={!open || busy} onClick={() => setCreating(true)}>+ Start a harness</button><small>{open ? "One tab per harness. Your code alongside." : "Open a folder to get started."}</small></div>}
    <footer className="agent-footer"><span className="dot" /> SHARED WORKSPACE <span>CLI / provider agnostic</span></footer>
  </section>;
}
