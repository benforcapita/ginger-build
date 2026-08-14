import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface TerminalInfo {
  id: number;
  cwd: string;
  shell: string;
  owner_type: "User" | "Agent";
  owner_id: number | null;
}

interface TerminalStore {
  terminals: TerminalInfo[];
  activeId: number | null;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  create: (cwd: string, shell?: string) => Promise<number>;
  write: (id: number, data: Uint8Array) => Promise<void>;
  resize: (id: number, rows: number, cols: number) => Promise<void>;
  terminate: (id: number) => Promise<void>;
  setActive: (id: number | null) => void;
}

export const useTerminalStore = create<TerminalStore>((set, get) => ({
  terminals: [],
  activeId: null,
  loading: false,
  error: null,
  refresh: async () => {
    try {
      const terminals = await invoke<TerminalInfo[]>("terminal_list");
      set({ terminals, error: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },
  create: async (cwd, shell) => {
    set({ loading: true, error: null });
    try {
      const result = await invoke<{ id: number }>("terminal_create", {
        args: { cwd, shell: shell ?? null, owner_type: "user" },
      });
      await get().refresh();
      set({ activeId: result.id, loading: false });
      return result.id;
    } catch (e) {
      set({ loading: false, error: String(e) });
      throw e;
    }
  },
  write: async (id, data) => {
    await invoke("terminal_write", { id, data: Array.from(data) });
  },
  resize: async (id, rows, cols) => {
    await invoke("terminal_resize", { id, rows, cols });
  },
  terminate: async (id) => {
    await invoke("terminal_terminate", { id });
    const activeId = get().activeId;
    if (activeId === id) set({ activeId: null });
    await get().refresh();
  },
  setActive: (id) => set({ activeId: id }),
}));