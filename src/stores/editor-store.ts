import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface EditorStatus {
  alive: boolean;
  runtime_path: string;
  safe_mode: boolean;
}

interface EditorStore {
  status: EditorStatus | null;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  start: () => Promise<void>;
  stop: () => Promise<void>;
}

export const useEditorStore = create<EditorStore>((set) => ({
  status: null,
  loading: false,
  error: null,
  refresh: async () => {
    try {
      const status = await invoke<EditorStatus>("editor_status");
      set({ status, error: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },
  start: async () => {
    set({ loading: true, error: null });
    try {
      await invoke("editor_start");
      const status = await invoke<EditorStatus>("editor_status");
      set({ status, loading: false });
    } catch (e) {
      set({ loading: false, error: String(e) });
    }
  },
  stop: async () => {
    set({ loading: true });
    try {
      await invoke("editor_stop");
      const status = await invoke<EditorStatus>("editor_status");
      set({ status, loading: false });
    } catch (e) {
      set({ loading: false, error: String(e) });
    }
  },
}));