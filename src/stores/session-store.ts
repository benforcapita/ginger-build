import { create } from "zustand";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { ask } from "@tauri-apps/plugin-dialog";

export type SessionKind = "editor" | "shell" | "agent";
export interface Session {
  id: number;
  kind: SessionKind;
  title: string;
  path?: string;
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
  busy: boolean;
  error: string | null;
  restore: () => Promise<void>;
  start: (kind: SessionKind, options?: { program?: string; args?: string[]; path?: string; title?: string }) => Promise<void>;
  select: (kind: SessionKind, id: number) => void;
  close: (id: number) => Promise<void>;
  markExited: (id: number, code: number | null) => void;
  setError: (error: string | null) => void;
}
export const useSessionStore = create<Sessions>((set, get) => ({
  sessions: [], active: { editor: null, shell: null, agent: null }, busy: false, error: null,
  setError: (error) => set({ error }),
  restore: async () => {
    if (!isTauri()) return;
    try {
      const native = await invoke<NativeSession[]>("terminal_list");
      const sessions: Session[] = native.map((s) => {
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
    if (!isTauri()) { set({ error: "Open the Ginger desktop app to use files, Neovim, and terminal sessions." }); return; }
    const existing = kind === "editor" ? get().sessions.find((s) => s.path === options.path && !s.exited) : undefined;
    if (existing) { get().select(kind, existing.id); return; }
    if (get().busy) return;
    set({ busy: true, error: null });
    try {
      const result = await invoke<{ id: number }>("terminal_launch", { kind, program: options.program ?? null, args: options.args ?? null, path: options.path ?? null });
      const title = options.title ?? options.path?.split("/").at(-1) ?? (kind === "shell" ? "shell" : options.program ?? "agent");
      set((state) => ({ sessions: [...state.sessions, { id: result.id, kind, title, path: options.path, exited: false }], active: { ...state.active, [kind]: result.id } }));
    } catch (e) { set({ error: String(e) }); }
    finally { set({ busy: false }); }
  },
  select: (kind, id) => set((state) => ({ active: { ...state.active, [kind]: id } })),
  close: async (id) => {
    const session = get().sessions.find((s) => s.id === id);
    if (!session) return;
    if (!session.exited) {
      const message = session.kind === "editor" ? "Closing this editor stops Neovim. Unsaved changes may be lost. Save with :w first, or use :q to close safely. Close anyway?" : "This stops the running process in this tab. Close session?";
      if (!await ask(message, { title: `Close ${session.title}`, kind: "warning" })) return;
    }
    try {
      await invoke("terminal_terminate", { id });
      set((state) => {
        const sessions = state.sessions.filter((s) => s.id !== id);
        return { sessions, active: { ...state.active, [session.kind]: state.active[session.kind] === id ? sessions.find((s) => s.kind === session.kind)?.id ?? null : state.active[session.kind] } };
      });
    } catch (e) { set({ error: String(e) }); }
  },
  markExited: (id, code) => set((state) => ({ sessions: state.sessions.map((s) => s.id === id ? { ...s, exited: true, exitCode: code } : s) })),
}));
