import { useCallback, useEffect, useRef, useState } from "react";
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
import { useWorkbenchStore } from "@/stores/workbench-store";
import { useSessionStore } from "@/stores/session-store";

export default function App() {
  const workspace = useWorkspaceStore((s) => s.status.workspace);
  const workspaceError = useWorkspaceStore((s) => s.error);
  const sessions = useSessionStore((s) => s.sessions);
  const error = useSessionStore((s) => s.error);
  const busy = useSessionStore((s) => s.busy);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const paletteReturnFocus = useRef<HTMLElement | null>(null);
  const togglePalette = useCallback(() => {
    if (!paletteOpen) paletteReturnFocus.current = document.activeElement as HTMLElement | null;
    setPaletteOpen(!paletteOpen);
  }, [paletteOpen]);
  const workbench = useWorkbenchStore();
  const activeEditor = useSessionStore((s) => s.active.editor);
  const openHarness = () => useWorkbenchStore.setState({ creatingHarness: true });
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
    const quitFromCommand = () => { void requestQuit(); };
    window.addEventListener("ginger-request-quit", quitFromCommand);
    const cleanup = getCurrentWindow().onCloseRequested((event) => { event.preventDefault(); void requestQuit(); });
    const quitCleanup = listen("ginger-quit-requested", () => { void requestQuit(); });
    void quitCleanup.then((unlisten) => { if (disposed) unlisten(); });
    void cleanup.then((unlisten) => { if (disposed) unlisten(); });
    return () => { disposed = true; window.removeEventListener("ginger-request-quit", quitFromCommand); void cleanup.then((unlisten) => unlisten()); void quitCleanup.then((unlisten) => unlisten()); };
  }, [native]);

  const openFolder = useCallback(async () => {
    const state = useSessionStore.getState();
    if (!native) { state.setError("This is the browser preview. Run pnpm tauri dev to open a local folder in Ginger."); return; }
    if (state.busy || opening) return;
    if (state.sessions.length) { state.setError("Close your editor, shell, and harness tabs before switching folders. Your current sessions are preserved."); return; }
    setOpening(true);
    try {
      const path = await chooseFolder({ directory: true, multiple: false, title: "Open a project in Ginger" });
      if (typeof path === "string") { state.setError(null); await useWorkspaceStore.getState().open(path); useWorkbenchStore.getState().collapseTree(); }
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
      if (key === "p" || key === "k") { e.preventDefault(); togglePalette(); }
      if (key === "o") { e.preventDefault(); void openFolder(); }
      if (key === "s" && e.metaKey) { e.preventDefault(); save(); }
      if (e.shiftKey && key === "t" && workspace) { e.preventDefault(); void useSessionStore.getState().start("shell"); }
      if (e.shiftKey && key === "n" && workspace) { e.preventDefault(); useWorkbenchStore.setState({ creatingHarness: true }); }
    };
    window.addEventListener("keydown", handle);
    return () => window.removeEventListener("keydown", handle);
  }, [openFolder, save, workspace, togglePalette]);
  const dismissError = () => { useSessionStore.getState().setError(null); useWorkspaceStore.setState({ error: null }); };
  const actions: PaletteAction[] = [
    { id: "folder", title: "Open folder", keywords: "project workspace switch", shortcut: "⌘ O", run: openFolder, disabled: opening || busy, reason: "Wait for the current operation" },
    { id: "save", title: "Save active Neovim file", shortcut: "⌘ S", run: save, disabled: !sessions.some((s) => s.id === activeEditor && !s.exited), reason: "Open an editor first" },
    { id: "shell", title: "New terminal", keywords: "shell command", shortcut: "⌘ ⇧ T", run: () => useSessionStore.getState().start("shell"), disabled: !workspace || busy, reason: "Open a folder and wait for session startup" },
    { id: "agent", title: "Start an agent harness", keywords: "claude codex opencode custom executable arguments", shortcut: "⌘ ⇧ N", run: openHarness, disabled: !workspace || busy, reason: "Open a folder and wait for session startup" },
    { id: "cancel-agent", title: "Cancel harness setup", run: () => useWorkbenchStore.setState({ creatingHarness: false }), disabled: !workbench.creatingHarness, reason: "No harness setup is open" },
    { id: "refresh", title: "Refresh file tree and search", keywords: "reload rescan files", run: workbench.refreshTree, disabled: !workspace, reason: "Open a folder first" },
    { id: "collapse", title: "Collapse all folders", keywords: "tree explorer", run: workbench.collapseTree, disabled: !workspace, reason: "Open a folder first" },
    { id: "companion", title: workbench.showCompanion ? "Minimize Ginger" : "Show Ginger", keywords: "mascot companion portrait", run: () => useWorkbenchStore.setState({ showCompanion: !workbench.showCompanion }) },
    { id: "quiet", title: workbench.quiet ? "Unmute Ginger commentary" : "Mute Ginger commentary", keywords: "quiet mascot", run: () => useWorkbenchStore.setState({ quiet: !workbench.quiet }) },
    { id: "pet", title: "Pet Ginger", keywords: "mascot head pat", run: workbench.pet },
    { id: "dismiss", title: "Dismiss error", run: dismissError, disabled: !error && !workspaceError, reason: "No error to dismiss" },
    ...sessions.flatMap((s): PaletteAction[] => [
      { id: `session-${s.id}`, title: `Focus ${s.kind}: ${s.title}`, keywords: s.path, run: () => useSessionStore.getState().select(s.kind, s.id) },
      { id: `close-${s.id}`, title: `Close ${s.kind}: ${s.title}`, keywords: `stop terminate tab ${s.path ?? ""}`, run: () => useSessionStore.getState().close(s.id) },
    ]),
    { id: "quit", title: "Quit Ginger Code", keywords: "exit application", shortcut: "⌘ Q", run: () => { window.dispatchEvent(new Event("ginger-request-quit")); }, disabled: !native, reason: "Available in the desktop app" },
  ];
  const liveAgents = sessions.filter((s) => s.kind === "agent" && !s.exited).length;
  return <main className="app">
    <header className="workspace-bar"><div className="workspace-location"><span className="brand-symbol">g.</span><span className="accent">ginger</span><span className="muted">:</span><span className="workspace-path" title={workspace?.root_path}>{workspace?.root_path ?? "~/your-next-idea"}</span><span className="prompt-cursor">▌</span></div><div className="workspace-actions"><span className="environment-badge">{native ? "LOCAL WORKSPACE" : "BROWSER PREVIEW"}</span><button onClick={togglePalette}>Commands <kbd>⌘ K / ⌘ P</kbd></button><button disabled={opening || busy} onClick={() => void openFolder()}>{opening ? "Opening…" : "Open folder ↗"}</button></div></header>
    {(error || workspaceError) && <div className="error-banner" role="alert"><span>{error ?? workspaceError}</span><button aria-label="Dismiss error" onClick={dismissError}>×</button></div>}
    <div className="workspace-grid"><Explorer onOpenFolder={() => { void openFolder(); }} /><AgentDock /><div className="code-column"><Editor /><Terminal /></div></div>
    <footer className="status-bar"><div><span className="status-brand">GINGER</span><span>{workspace ? workspace.display_name : "No workspace"}</span><span className="status-divider">/</span><span>NEOVIM + CLI HARNESSES</span></div><div><span><i className="dot" /> {liveAgents} live {liveAgents === 1 ? "harness" : "harnesses"}</span><span>{sessions.length} sessions</span><span className="accent">{busy ? "STARTING…" : "LET’S BUILD SOMETHING."}</span></div></footer>
    {paletteOpen && <CommandPalette returnFocus={paletteReturnFocus.current} actions={actions} onClose={() => setPaletteOpen(false)} />}
  </main>;
}
