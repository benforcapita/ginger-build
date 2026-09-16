import { open as openLink } from "@tauri-apps/plugin-shell";
import { useEffect, useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { useSessionStore } from "@/stores/session-store";
import { useWorkspaceStore } from "@/stores/workspace-store";
import { GingerMascot } from "@/components/presence/GingerMascot";
import { SessionTabs } from "@/components/terminal/SessionTabs";
import { useWorkbenchStore } from "@/stores/workbench-store";
interface Harness { name: string; program: string; available: boolean; executable: string | null }
const setupLinks: Record<string, string> = {
  claude: 'https://code.claude.com/docs/en/setup',
  codex: 'https://developers.openai.com/codex/cli/',
  opencode: 'https://opencode.ai/docs/',
  agy: 'https://antigravity.google/docs/cli/getting-started/',
  pi: 'https://pi.dev/docs/latest',
};
const quoteArgument = (value: string) => "'" + value.replaceAll("'", "'\"'\"'") + "'";
export function AgentDock() {
  const sessions = useSessionStore((s) => s.sessions);
  const start = useSessionStore((s) => s.start);
  const busy = useSessionStore((s) => s.busy);
  const open = useWorkspaceStore((s) => s.status.open);
  const creating = useWorkbenchStore((s) => s.creatingHarness);
  const setCreating = (creatingHarness: boolean) => useWorkbenchStore.setState({ creatingHarness });
  const showCompanion = useWorkbenchStore((s) => s.showCompanion);
  const setShowCompanion = (showCompanion: boolean) => useWorkbenchStore.setState({ showCompanion });
  const [harnesses, setHarnesses] = useState<Harness[]>([]);
  const [selected, setSelected] = useState("custom");
  const [program, setProgram] = useState("");
  const [args, setArgs] = useState("");
  const [revision, setRevision] = useState(0);
  const [checking, setChecking] = useState(false);
  const harness = harnesses.find(h => h.program === selected);
  const executable = selected === 'custom' ? program.trim() : harness?.executable ?? selected;
  const launchArgs = args.split('\n').filter(value => value.length > 0);
  const ready = selected === 'custom' ? Boolean(program.trim()) : Boolean(harness?.available);
  const hasAgents = sessions.some((s) => s.kind === "agent");
  useEffect(() => {
    if (!isTauri()) return;
    let cancelled = false;
    setChecking(true);
    void invoke<Harness[]>("terminal_harnesses").then(items => {
      if (cancelled) return;
      setHarnesses(items);
    }).catch(e => { if (!cancelled) useSessionStore.getState().setError(String(e)); })
      .finally(() => { if (!cancelled) setChecking(false); });
    return () => { cancelled = true; };
  }, [revision, creating]);
  return <section className="agent-dock" aria-label="Ginger and agent harnesses">
    {showCompanion && <GingerMascot onMinimize={hasAgents ? () => setShowCompanion(false) : undefined} />}
    {!showCompanion && <div className="ginger-compact"><span className="accent">g.</span> Ginger’s here. Let’s build.<button onClick={() => setShowCompanion(true)}>Show Ginger</button></div>}
    <header className="panel-header"><span>AGENT HARNESSES <span className="count">{sessions.filter((s) => s.kind === "agent").length}</span></span><button onClick={() => setCreating(!creating)} disabled={!open || busy} aria-label="New agent harness">{creating ? "−" : "+"}</button></header>
    {creating && <form className="harness-form" onSubmit={async (event) => {
      event.preventDefault();
      if (!ready || checking) return;
      await start("agent", { program: executable, args: launchArgs, title: harnesses.find((h) => h.program === selected)?.name ?? executable.split("/").at(-1) });
      if (!useSessionStore.getState().error) setCreating(false);
    }}>
      <label>Harness<select value={selected} onChange={(e) => { setSelected(e.target.value); setArgs(""); }}>{harnesses.map((h) => <option key={h.program} value={h.program}>{h.name}{h.available ? "" : " · not installed"}</option>)}<option value="custom">Custom executable</option></select></label>
      {selected === "custom" ? <label>Executable<input autoFocus placeholder="my-agent or /path/to/agent" required value={program} onChange={(e) => setProgram(e.target.value)} /></label> : <div className="harness-status">
        <span role="status">{checking ? 'Checking installation…' : harness?.available ? `Installed · ${harness.executable}` : `${selected} is not installed or not on PATH`}</span>
        <button type="button" onClick={() => void openLink(setupLinks[selected]).catch(e => useSessionStore.getState().setError(String(e)))}>Setup guide ↗</button>
      </div>}
      <button type="button" disabled={checking} onClick={() => setRevision(value => value + 1)}>{checking ? 'Checking…' : 'Refresh harness detection'}</button>
      <label>Arguments <span className="muted">· one per line, no shell quoting needed</span><textarea rows={2} value={args} onChange={(e) => setArgs(e.target.value)} placeholder={'--model\nmy-model'} /></label>
      <div className="harness-preview"><span className="section-label">COMMAND PREVIEW</span><code>{[executable || 'executable', ...launchArgs].map(quoteArgument).join(' ')}</code></div>
      <p>Runs in this folder with access to your files. Each harness uses its own CLI login and configuration.</p>
      <div className="form-actions"><button type="button" onClick={() => setCreating(false)}>Cancel</button><button className="primary-button" disabled={!open || busy || checking || !ready}>{busy ? "Starting…" : "Start harness ↗"}</button></div>
    </form>}
    {hasAgents ? <SessionTabs kind="agent" /> : !creating && <div className="agent-empty"><span className="terminal-symbol">[ &gt;_ ]</span><h2>Bring your own agent.</h2><p>Claude Code, Codex, OpenCode, Antigravity, Pi.<br />Or any CLI that feels like home.</p><button disabled={!open || busy} onClick={() => setCreating(true)}>+ Start a harness</button><small>{open ? "One tab per harness. Your code alongside." : "Open a folder to get started."}</small></div>}
    <footer className="agent-footer"><span className="dot" /> SHARED WORKSPACE <span>CLI / provider agnostic</span></footer>
  </section>;
}
