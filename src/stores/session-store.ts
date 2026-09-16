import { reconcileDisk, savedDocument, type DocumentState } from "../document-state";
import { create } from "zustand";
import { useLayoutStore } from "./layout-store";
import { reorderTabs } from "../workspace-layout";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { ask } from "@tauri-apps/plugin-dialog";

export type SessionKind = "editor" | "shell" | "agent";
export interface Session {
  id: number;
  kind: SessionKind;
  title: string;
  path?: string;
  document?: DocumentState;
  saving?: boolean;
  diskError?: string;
  exited: boolean;
  exitCode?: number | null;
}
interface NativeSession {
  id: number; shell: string; args: string[]; cwd: string;
  owner_type: "User" | "Agent" | "Editor"; exited: boolean;
}
interface Sessions {
  sessions: Session[];
  active: Record<SessionKind, number | null>;
  focusRequest: { id: number; revision: number } | null;
  busy: boolean;
  error: string | null;
  vim: boolean;
  toggleVim: () => void;
  edit: (id: number, text: string) => void;
  save: (id: number) => Promise<boolean>;
  reload: (id: number) => Promise<void>;
  checkDisk: () => Promise<void>;
  restore: () => Promise<void>;
  start: (kind: SessionKind, options?: { program?: string; args?: string[]; path?: string; title?: string }) => Promise<void>;
  select: (kind: SessionKind, id: number) => void;
  moveTab: (id: number, target: number) => void;
  close: (id: number) => Promise<void>;
  markExited: (id: number, code: number | null) => void;
  setError: (error: string | null) => void;
}
let nextDocumentId = -1;
let checkingDisk = false;
export const useSessionStore = create<Sessions>((set, get) => ({
  focusRequest: null, sessions: [], active: { editor: null, shell: null, agent: null }, busy: false, error: null,
  vim: localStorage.getItem("ginger-vim") !== "false",
  toggleVim: () => { const vim = !get().vim; localStorage.setItem("ginger-vim", String(vim)); set({ vim }); },
  edit: (id, text) => set(s => ({ sessions: s.sessions.map(tab => tab.id === id && tab.document ? { ...tab, document: { ...tab.document, text } } : tab) })),
  save: async (id) => {
    const tab = get().sessions.find(s => s.id === id);
    if (!tab?.document || tab.saving) return false;
    const snapshot = tab.document.text;
    set(s => ({ sessions: s.sessions.map(t => t.id === id ? { ...t, saving: true } : t) }));
    try {
      await invoke("workspace_save_file", { path: tab.path, text: snapshot, expected: tab.document.baseline });
      set(s => ({ sessions: s.sessions.map(t => t.id === id && t.document ? { ...t, document: savedDocument(t.document, snapshot), diskError: undefined } : t), error: null }));
      window.dispatchEvent(new CustomEvent("ginger-file-saved", { detail: tab.path }));
      return true;
    } catch (e) { set({ error: String(e) }); return false; }
    finally { set(s => ({ sessions: s.sessions.map(t => t.id === id ? { ...t, saving: false } : t) })); }
  },
  reload: async (id) => {
    const tab = get().sessions.find(s => s.id === id);
    if (!tab?.document || tab.saving) return;
    if (tab.document.text !== tab.document.baseline && !await ask("Discard your unsaved edits and reload this file from disk?", { title: `Reload ${tab.title}`, kind: "warning" })) return;
    const before = get().sessions.find(s => s.id === id);
    try {
      const text = await invoke<string>("workspace_read_file", { path: tab.path });
      const current = get().sessions.find(s => s.id === id);
      if (current?.saving || current?.document?.text !== before?.document?.text) { set({ error: "File changed while reloading. Your edits are preserved; try again." }); return; }
      set(s => ({ sessions: s.sessions.map(t => t.id === id ? { ...t, document: { text, baseline: text, conflict: false }, diskError: undefined } : t), error: null }));
    } catch (e) { set({ error: String(e) }); }
  },
  checkDisk: async () => {
    if (checkingDisk) return;
    checkingDisk = true;
    try {
      for (const tab of get().sessions.filter(t => t.document && !t.saving)) {
        const baseline = tab.document!.baseline;
        try {
          const disk = await invoke<string>("workspace_read_file", { path: tab.path });
          set(s => ({ sessions: s.sessions.map(t => t.id === tab.id && t.document && !t.saving && t.document.baseline === baseline ? { ...t, document: reconcileDisk(t.document, disk), diskError: undefined } : t) }));
        } catch (e) {
          set(s => ({ sessions: s.sessions.map(t => t.id === tab.id ? { ...t, diskError: String(e) } : t) }));
        }
      }
    } finally { checkingDisk = false; }
  },
  setError: (error) => set({ error }),
  restore: async () => {
    if (!isTauri()) return;
    try {
      const native = await invoke<NativeSession[]>("terminal_list");
      const sessions: Session[] = native.filter(s => s.owner_type !== "Editor").map((s) => {
        const kind = s.owner_type === "Editor" ? "editor" : s.owner_type === "Agent" ? "agent" : "shell";
        const path = kind === "editor" ? s.args.at(-1)?.slice(s.cwd.length + 1) : undefined;
        return { id: s.id, kind, path, title: path?.split("/").at(-1) ?? s.shell.split("/").at(-1) ?? "shell", exited: s.exited };
      });
      set({ sessions, active: {
        editor: sessions.find((s) => s.kind === "editor")?.id ?? null,
        shell: sessions.find((s) => s.kind === "shell")?.id ?? null,
        agent: sessions.find((s) => s.kind === "agent")?.id ?? null,
      } });
    } catch (e) { set({ error: String(e) }); }
  },
  start: async (kind, options = {}) => {
    if (!isTauri()) { set({ error: "Open the Ginger desktop app to use files and terminal sessions." }); return; }
    const existing = kind === "editor" ? get().sessions.find((s) => s.path === options.path && !s.exited) : undefined;
    if (existing) { get().select(kind, existing.id); return; }
    if (get().busy) return;
    set({ busy: true, error: null });
    try {
      let document: DocumentState | undefined;
      if (kind === "editor") {
        const text = await invoke<string>("workspace_read_file", { path: options.path });
        document = { text, baseline: text, conflict: false };
      }
      const result = document ? { id: nextDocumentId-- } : await invoke<{ id: number }>("terminal_launch", { kind, program: options.program ?? null, args: options.args ?? null, path: options.path ?? null });
      useLayoutStore.getState().focusPane(kind);
      const title = options.title ?? options.path?.split("/").at(-1) ?? (kind === "shell" ? "shell" : options.program ?? "agent");
      set((state) => ({ sessions: [...state.sessions, { id: result.id, kind, title, path: options.path, document, exited: false }], focusRequest: { id: result.id, revision: (state.focusRequest?.revision ?? 0) + 1 }, active: { ...state.active, [kind]: result.id } }));
    } catch (e) { set({ error: String(e) }); }
    finally { set({ busy: false }); }
  },
  select: (kind, id) => {
    if (!get().sessions.some(s => s.id === id && s.kind === kind)) return;
    useLayoutStore.getState().focusPane(kind);
    set((state) => ({ focusRequest: { id, revision: (state.focusRequest?.revision ?? 0) + 1 }, active: { ...state.active, [kind]: id } }));
  },
  moveTab: (id, target) => set(s => ({ sessions: reorderTabs(s.sessions, id, target) })),
  close: async (id) => {
    const session = get().sessions.find((s) => s.id === id);
    if (!session) return;
    if (session.saving) { set({ error: "Wait for this file to finish saving." }); return; }
    if (session.document) {
      if (session.document.text !== session.document.baseline && !await ask("Discard unsaved changes and close this file?", { title: `Close ${session.title}`, kind: "warning" })) return;
    } else if (!session.exited && !await ask("This stops the running process in this tab. Close session?", { title: `Close ${session.title}`, kind: "warning" })) return;
    try {
      if (!session.document) await invoke("terminal_terminate", { id });
      const wasFocused = get().focusRequest?.id === id;
      set((state) => {
        const sessions = state.sessions.filter((s) => s.id !== id);
        const replacement = sessions.find((s) => s.kind === session.kind)?.id ?? null;
        const focusId = state.focusRequest?.id === id ? replacement ?? sessions.at(-1)?.id : state.focusRequest?.id;
        return { sessions,
          focusRequest: wasFocused ? (focusId ? { id: focusId, revision: (state.focusRequest?.revision ?? 0) + 1 } : null) : state.focusRequest,
          active: { ...state.active, [session.kind]: state.active[session.kind] === id ? replacement : state.active[session.kind] },
        };
      });
      if (wasFocused) {
        const replacement = get().sessions.find(s => s.id === get().focusRequest?.id);
        if (replacement) get().select(replacement.kind, replacement.id);
      }
    } catch (e) { set({ error: String(e) }); }
  },
  markExited: (id, code) => set((state) => ({ sessions: state.sessions.map((s) => s.id === id ? { ...s, exited: true, exitCode: code } : s) })),
}));
