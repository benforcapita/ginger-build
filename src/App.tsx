import { useCallback, useEffect, useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { open as chooseFolder, ask } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Explorer } from "@/components/explorer/Explorer";
import { Editor } from "@/components/editor/Editor";
import { AgentDock } from "@/components/agent-dock/AgentDock";
import { Terminal } from "@/components/terminal/Terminal";
import { CommandPalette, type PaletteAction } from "@/components/CommandPalette";
import { useWorkspaceStore } from "@/stores/workspace-store";
import { useSessionStore } from "@/stores/session-store";

export default function App() {
  const workspace = useWorkspaceStore((s) => s.status.workspace);
  const workspaceError = useWorkspaceStore((s) => s.error);
  const sessions = useSessionStore((s) => s.sessions);
  const error = useSessionStore((s) => s.error);
  const busy = useSessionStore((s) => s.busy);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [launchRequest, setLaunchRequest] = useState(0);
  const [opening, setOpening] = useState(false);
  const native = isTauri();
  useEffect(() => {
    if (!native) return;
    void useWorkspaceStore.getState().refresh();
    void useSessionStore.getState().restore();
    let disposed = false;
    let closing = false;
    const requestQuit = async () => {
      if (closing) return;
      closing = true;
      try {
        if (useSessionStore.getState().sessions.some((s) => !s.exited) && !await ask("Quit Ginger? This stops running sessions. Unsaved Neovim buffers may be lost. Save your files first.", { title: "Quit Ginger Code", kind: "warning" })) return;
        await invoke("terminal_terminate_all");
        await getCurrentWindow().destroy();
      } catch (e) { useSessionStore.getState().setError(String(e)); }
      finally { closing = false; }
    };
    const cleanup = getCurrentWindow().onCloseRequested((event) => { event.preventDefault(); void requestQuit(); });
    const quitCleanup = listen("ginger-quit-requested", () => { void requestQuit(); });
    void quitCleanup.then((unlisten) => { if (disposed) unlisten(); });
    void cleanup.then((unlisten) => { if (disposed) unlisten(); });
    return () => { disposed = true; void cleanup.then((unlisten) => unlisten()); void quitCleanup.then((unlisten) => unlisten()); };
  }, [native]);

  const openFolder = useCallback(async () => {
    const state = useSessionStore.getState();
    if (!native) { state.setError("This is the browser preview. Run pnpm tauri dev to open a local folder in Ginger."); return; }
    if (state.busy || opening) return;
    if (state.sessions.length) { state.setError("Close your editor, shell, and harness tabs before switching folders. Your current sessions are preserved."); return; }
    setOpening(true);
    try {
      const path = await chooseFolder({ directory: true, multiple: false, title: "Open a project in Ginger" });
      if (typeof path === "string") { state.setError(null); await useWorkspaceStore.getState().open(path); }
    } catch (e) { state.setError(String(e)); }
    finally { setOpening(false); }
  }, [native, opening]);
  const save = useCallback(() => {
    const state = useSessionStore.getState();
    if (state.active.editor !== null) void invoke("terminal_write", { id: state.active.editor, data: Array.from(new TextEncoder().encode("\x1b:write\r")) }).catch((e) => state.setError(String(e)));
  }, []);
  useEffect(() => {
    const handle = (e: KeyboardEvent) => {
      const key = e.key.toLowerCase();
      if (!e.metaKey) return;
      if (key === "p" || key === "k") { e.preventDefault(); setPaletteOpen((v) => !v); }
      if (key === "o") { e.preventDefault(); void openFolder(); }
      if (key === "s" && e.metaKey) { e.preventDefault(); save(); }
      if (e.shiftKey && key === "t" && workspace) { e.preventDefault(); void useSessionStore.getState().start("shell"); }
      if (e.shiftKey && key === "n" && workspace) { e.preventDefault(); setLaunchRequest((v) => v + 1); }
    };
    window.addEventListener("keydown", handle);
    return () => window.removeEventListener("keydown", handle);
  }, [openFolder, save, workspace]);
  const actions: PaletteAction[] = [
    { id: "folder", title: "Open folder", shortcut: "⌘ O", run: () => { void openFolder(); }, disabled: opening || busy },
    { id: "save", title: "Save active Neovim file", shortcut: "⌘ S", run: save, disabled: !sessions.some((s) => s.kind === "editor" && !s.exited) },
    { id: "shell", title: "New terminal", shortcut: "⌘ ⇧ T", run: () => { void useSessionStore.getState().start("shell"); }, disabled: !workspace || busy },
    { id: "agent", title: "Start an agent harness", shortcut: "⌘ ⇧ N", run: () => setLaunchRequest((v) => v + 1), disabled: !workspace || busy },
    ...sessions.map((s) => ({ id: `session-${s.id}`, title: `Focus ${s.kind}: ${s.title}`, run: () => useSessionStore.getState().select(s.kind, s.id) })),
  ];
  const liveAgents = sessions.filter((s) => s.kind === "agent" && !s.exited).length;
  return <main className="app">
    <header className="workspace-bar"><div className="workspace-location"><span className="brand-symbol">g.</span><span className="accent">ginger</span><span className="muted">:</span><span className="workspace-path" title={workspace?.root_path}>{workspace?.root_path ?? "~/your-next-idea"}</span><span className="prompt-cursor">▌</span></div><div className="workspace-actions"><span className="environment-badge">{native ? "LOCAL WORKSPACE" : "BROWSER PREVIEW"}</span><button onClick={() => setPaletteOpen(true)}>Commands <kbd>⌘ K / ⌘ P</kbd></button><button disabled={opening || busy} onClick={() => void openFolder()}>{opening ? "Opening…" : "Open folder ↗"}</button></div></header>
    {(error || workspaceError) && <div className="error-banner" role="alert"><span>{error ?? workspaceError}</span><button aria-label="Dismiss error" onClick={() => { useSessionStore.getState().setError(null); useWorkspaceStore.setState({ error: null }); }}>×</button></div>}
    <div className="workspace-grid"><Explorer onOpenFolder={() => { void openFolder(); }} /><AgentDock launchRequest={launchRequest} /><div className="code-column"><Editor /><Terminal /></div></div>
    <footer className="status-bar"><div><span className="status-brand">GINGER</span><span>{workspace ? workspace.display_name : "No workspace"}</span><span className="status-divider">/</span><span>NEOVIM + CLI HARNESSES</span></div><div><span><i className="dot" /> {liveAgents} live {liveAgents === 1 ? "harness" : "harnesses"}</span><span>{sessions.length} sessions</span><span className="accent">{busy ? "STARTING…" : "LET’S BUILD SOMETHING."}</span></div></footer>
    {paletteOpen && <CommandPalette actions={actions} onClose={() => setPaletteOpen(false)} />}
  </main>;
}
