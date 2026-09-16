import { cycleTab, focusShortcut, type FocusAction } from "@/tab-navigation";
import { LanguageServers } from '@/components/editor/LanguageServers';
import { useLanguageStore } from '@/stores/language-store';
import { restartLanguageServers } from '@/editor/language-client';
import { flushSync } from "react-dom";
import { useCallback, useEffect, useRef, useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { open as chooseFolder, ask } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Explorer } from "@/components/explorer/Explorer";
import { WorkspacePanes, paneNames, slotNames } from "@/components/workspace/WorkspacePanes";
import { useLayoutStore } from "@/stores/layout-store";
import { type Pane } from "@/workspace-layout";
import { CommandPalette, type PaletteAction } from "@/components/CommandPalette";
import { useWorkspaceStore } from "@/stores/workspace-store";
import { useWorkbenchStore } from "@/stores/workbench-store";
import { useSessionStore } from "@/stores/session-store";

export default function App() {
  const languagePanel = useLanguageStore(s => s.open);
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
  const layout = useLayoutStore();
  const vimEnabled = useSessionStore(s => s.vim);
  const activeEditor = useSessionStore((s) => s.active.editor);
  const previousTab = useRef<number | null>(null);
  useEffect(() => useSessionStore.subscribe((state, before) => {
    if (state.focusRequest?.id !== before.focusRequest?.id && before.focusRequest) previousTab.current = before.focusRequest.id;
  }), []);
  const focusTab = (id: number | null) => {
    const state = useSessionStore.getState();
    const tab = state.sessions.find(tab => tab.id === id);
    if (tab) state.select(tab.kind, tab.id);
  };
  const navigate = (action: FocusAction) => {
    const state = useSessionStore.getState();
    if (action === 'tree') { useWorkbenchStore.getState().focusTree(); return; }
    if (action === 'next' || action === 'previous') {
      focusTab(cycleTab(state.sessions, state.focusRequest?.id ?? null, action === 'next' ? 1 : -1));
      return;
    }
    const active = state.sessions.find(tab => tab.id === state.active[action]) ?? state.sessions.find(tab => tab.kind === action);
    if (active) state.select(action, active.id);
    else {
      useLayoutStore.getState().focusPane(action);
      document.querySelector<HTMLElement>(`[data-pane="${action}"]`)?.focus();
    }
  };
  const openHarness = () => useWorkbenchStore.setState({ creatingHarness: true });
  const [opening, setOpening] = useState(false);
  const native = isTauri();
  useEffect(() => {
    if (!native) return;
    void useWorkspaceStore.getState().refresh();
    void useSessionStore.getState().restore();
    const check = () => { void useSessionStore.getState().checkDisk(); };
    const poll = window.setInterval(check, 3000);
    window.addEventListener("focus", check);
    let disposed = false;
    let closing = false;
    const requestQuit = async () => {
      if (closing) return;
      closing = true;
      try {
        const tabs = useSessionStore.getState().sessions;
        if (tabs.some(s => s.saving)) { useSessionStore.getState().setError("Wait for files to finish saving before quitting."); return; }
        if (tabs.some(s => s.document ? s.document.text !== s.document.baseline : !s.exited) && !await ask("Quit Ginger? Unsaved file changes will be discarded and running sessions will stop.", { title: "Quit Ginger Code", kind: "warning" })) return;
        await invoke("language_servers_stop_all");
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
    return () => { window.clearInterval(poll); window.removeEventListener("focus", check); disposed = true; window.removeEventListener("ginger-request-quit", quitFromCommand); void cleanup.then((unlisten) => unlisten()); void quitCleanup.then((unlisten) => unlisten()); };
  }, [native]);

  const openFolder = useCallback(async () => {
    const state = useSessionStore.getState();
    if (!native) { state.setError("This is the browser preview. Run pnpm tauri dev to open a local folder in Ginger."); return; }
    if (state.busy || opening) return;
    if (state.sessions.length) { state.setError("Close your editor, shell, and harness tabs before switching folders. Your current sessions are preserved."); return; }
    setOpening(true);
    try {
      const path = await chooseFolder({ directory: true, multiple: false, title: "Open a project in Ginger" });
      if (typeof path === "string") { await restartLanguageServers(); state.setError(null); await useWorkspaceStore.getState().open(path); useWorkbenchStore.getState().collapseTree(); }
    } catch (e) { state.setError(String(e)); }
    finally { setOpening(false); }
  }, [native, opening]);
  const save = useCallback(() => {
    const state = useSessionStore.getState();
    if (state.active.editor !== null) void state.save(state.active.editor);
  }, []);
  useEffect(() => {
    const handle = (e: KeyboardEvent) => {
      const key = e.key.toLowerCase();
      const focusAction = focusShortcut(e);
      if (focusAction && !useLanguageStore.getState().open) {
        e.preventDefault(); e.stopPropagation();
        if (paletteOpen) flushSync(() => setPaletteOpen(false));
        navigate(focusAction);
        return;
      }
      if (!e.metaKey) return;
      if (key === "p" || key === "k") { e.preventDefault(); togglePalette(); }
      if (key === "e" && e.shiftKey && workspace) {
        e.preventDefault();
        if (paletteOpen) flushSync(() => setPaletteOpen(false));
        useWorkbenchStore.getState().focusTree();
      }
      if (key === "q") { e.preventDefault(); window.dispatchEvent(new Event("ginger-request-quit")); }
      if (key === "o") { e.preventDefault(); void openFolder(); }
      if (key === "s" && e.metaKey) { e.preventDefault(); save(); }
      if (e.shiftKey && key === "t" && workspace) { e.preventDefault(); void useSessionStore.getState().start("shell"); }
      if (e.shiftKey && key === "n" && workspace) { e.preventDefault(); useWorkbenchStore.setState({ creatingHarness: true }); }
    };
    window.addEventListener("keydown", handle, true);
    return () => window.removeEventListener("keydown", handle, true);
  }, [openFolder, save, workspace, togglePalette, paletteOpen]);
  const dismissError = () => { useSessionStore.getState().setError(null); useWorkspaceStore.setState({ error: null }); };
  const actions: PaletteAction[] = [
    { id: "folder", title: "Open folder", keywords: "project workspace switch", shortcut: "⌘ O", run: openFolder, disabled: opening || busy, reason: "Wait for the current operation" },
    { id: "save", title: "Save active file", shortcut: "⌘ S", run: save, disabled: !sessions.some((s) => s.id === activeEditor && !s.exited), reason: "Open an editor first" },
    { id: "language-servers", title: "Language servers: status and setup", keywords: "lsp autocomplete diagnostics install typescript python rust json", run: () => useLanguageStore.setState({ open: true }), disabled: !workspace, reason: "Open a folder first" },
    { id: "language-restart", title: "Restart language servers", keywords: "lsp reconnect", run: restartLanguageServers, disabled: !workspace, reason: "Open a folder first" },
    ...Object.entries({ complete: 'Show autocomplete suggestions', diagnostics: 'Show errors and warnings', definition: 'Go to definition', references: 'Find references', rename: 'Rename symbol across files', format: 'Format document', signature: 'Show function signature' }).map(([id, title]): PaletteAction => ({ id: `language-${id}`, title, keywords: 'editor language server code intelligence', run: () => { window.dispatchEvent(new CustomEvent('ginger-editor-action', { detail: id })); }, disabled: activeEditor === null, reason: 'Open a file first' })),
    { id: "vim-mode", title: vimEnabled ? "Disable Vim mode" : "Enable Vim mode", keywords: "editor keybindings normal insert visual", run: () => useSessionStore.getState().toggleVim() },
    { id: "editor-search", title: "Find and replace in active file", keywords: "search text", run: () => { window.dispatchEvent(new Event("ginger-editor-search")); }, disabled: activeEditor === null, reason: "Open a file first" },
    { id: "editor-reload", title: "Reload active file from disk", keywords: "external changes conflict discard", run: () => { if (activeEditor !== null) return useSessionStore.getState().reload(activeEditor); }, disabled: activeEditor === null, reason: "Open a file first" },
    { id: "shell", title: "New terminal", keywords: "shell command", shortcut: "⌘ ⇧ T", run: () => useSessionStore.getState().start("shell"), disabled: !workspace || busy, reason: "Open a folder and wait for session startup" },
    { id: "agent", title: "Start an agent harness", keywords: "claude codex opencode antigravity agy pi custom executable arguments", shortcut: "⌘ ⇧ N", run: openHarness, disabled: !workspace || busy, reason: "Open a folder and wait for session startup" },
    { id: "cancel-agent", title: "Cancel harness setup", run: () => useWorkbenchStore.setState({ creatingHarness: false }), disabled: !workbench.creatingHarness, reason: "No harness setup is open" },
    { id: "refresh", title: "Refresh file tree and search", keywords: "reload rescan files", run: workbench.refreshTree, disabled: !workspace, reason: "Open a folder first" },
    { id: "focus-tree", title: "Focus file tree", keywords: "explorer project vim navigation", shortcut: "⌘ 1 / ⌘ ⇧ E", run: workbench.focusTree, disabled: !workspace, reason: "Open a folder first" },
    ...(['editor', 'agent', 'shell'] as const).map((pane, index): PaletteAction => ({ id: `focus-${pane}`, title: `Focus ${paneNames[pane]}`, keywords: 'pane tab switch navigate', shortcut: `⌘ ${index + 2}`, run: () => navigate(pane) })),
    { id: 'next-tab', title: 'Focus next tab', shortcut: 'Ctrl Tab', run: () => navigate('next'), disabled: !sessions.length },
    { id: 'previous-tab', title: 'Focus previous tab in order', shortcut: 'Ctrl ⇧ Tab', run: () => navigate('previous'), disabled: !sessions.length },
    { id: 'last-tab', title: 'Focus previously used tab', keywords: 'back last recent switch', run: () => focusTab(previousTab.current), disabled: !sessions.some(tab => tab.id === previousTab.current) },
    { id: "collapse", title: "Collapse all folders", keywords: "tree explorer", run: workbench.collapseTree, disabled: !workspace, reason: "Open a folder first" },
    { id: "companion", title: workbench.showCompanion ? "Minimize Ginger" : "Show Ginger", keywords: "mascot companion portrait", run: () => useWorkbenchStore.setState({ showCompanion: !workbench.showCompanion }) },
    { id: "quiet", title: workbench.quiet ? "Unmute Ginger commentary" : "Mute Ginger commentary", keywords: "quiet mascot", run: () => useWorkbenchStore.setState({ quiet: !workbench.quiet }) },
    { id: "pet", title: "Pet Ginger", keywords: "mascot head pat", run: workbench.pet },
    { id: "dismiss", title: "Dismiss error", run: dismissError, disabled: !error && !workspaceError, reason: "No error to dismiss" },
    { id: "auto-center", title: layout.autoCenter ? "Disable automatic tab centering" : "Enable automatic tab centering", keywords: "layout focus pane position", run: layout.toggleAutoCenter },
    { id: "reset-layout", title: "Reset pane arrangement", keywords: "layout positions default", run: layout.reset },
    ...(['editor', 'agent', 'shell'] as Pane[]).flatMap(pane => slotNames.map((slot, index): PaletteAction => ({ id: `place-${pane}-${index}`, title: `Move ${paneNames[pane]} pane to ${slot}`, keywords: "layout arrange position", run: () => layout.movePane(pane, index) }))),
    ...sessions.flatMap((s): PaletteAction[] => [
      { id: `session-${s.id}`, title: `Focus ${s.kind}: ${s.title}`, keywords: s.path, run: () => useSessionStore.getState().select(s.kind, s.id) },
      ...([-1, 1] as const).map(direction => {
        const group = sessions.filter(item => item.kind === s.kind);
        const target = group[group.findIndex(item => item.id === s.id) + direction];
        return { id: `move-${s.id}-${direction}`, title: `Move ${s.kind} tab ${s.title} ${direction < 0 ? 'left' : 'right'}`, keywords: 'reorder arrange position', disabled: !target, reason: 'Already at the edge', run: () => { if (target) useSessionStore.getState().moveTab(s.id, target.id); } };
      }),
      { id: `close-${s.id}`, title: `Close ${s.kind}: ${s.title}`, keywords: `stop terminate tab ${s.path ?? ""}`, run: () => useSessionStore.getState().close(s.id) },
    ]),
    { id: "quit", title: "Quit Ginger Code", keywords: "exit application", shortcut: "⌘ Q", run: () => { window.dispatchEvent(new Event("ginger-request-quit")); }, disabled: !native, reason: "Available in the desktop app" },
  ];
  const liveAgents = sessions.filter((s) => s.kind === "agent" && !s.exited).length;
  return <main className="app">
    <header className="workspace-bar"><div className="workspace-location"><span className="brand-symbol">g.</span><span className="accent">ginger</span><span className="muted">:</span><span className="workspace-path" title={workspace?.root_path}>{workspace?.root_path ?? "~/your-next-idea"}</span><span className="prompt-cursor">▌</span></div><div className="workspace-actions"><span className="environment-badge">{native ? "LOCAL WORKSPACE" : "BROWSER PREVIEW"}</span><button onClick={togglePalette}>Commands <kbd>⌘ K / ⌘ P</kbd></button><button disabled={opening || busy} onClick={() => void openFolder()}>{opening ? "Opening…" : "Open folder ↗"}</button></div></header>
    {(error || workspaceError) && <div className="error-banner" role="alert"><span>{error ?? workspaceError}</span><button aria-label="Dismiss error" onClick={dismissError}>×</button></div>}
    <div className="workspace-grid"><Explorer onOpenFolder={() => { void openFolder(); }} /><WorkspacePanes /></div>
    <footer className="status-bar"><div><span className="status-brand">GINGER</span><span>{workspace ? workspace.display_name : "No workspace"}</span><span className="status-divider">/</span><span>FILE EDITOR + CLI HARNESSES</span></div><div><span><i className="dot" /> {liveAgents} live {liveAgents === 1 ? "harness" : "harnesses"}</span><span>{sessions.length} sessions</span><span className="accent">{busy ? "STARTING…" : "LET’S BUILD SOMETHING."}</span></div></footer>
    {languagePanel && <LanguageServers />}
    {paletteOpen && <CommandPalette returnFocus={paletteReturnFocus.current} actions={actions} onClose={() => setPaletteOpen(false)} />}
  </main>;
}
