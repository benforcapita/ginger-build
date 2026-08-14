import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface Workspace {
  id: number | null;
  root_path: string;
  display_name: string;
  runtime_version: string;
  created_at: string;
  last_opened_at: string | null;
  active_session_id: number | null;
}

export interface PaneState {
  explorer_visible: boolean;
  explorer_width: number;
  agent_dock_visible: boolean;
  agent_dock_width: number;
  bottom_panel_visible: boolean;
  bottom_panel_height: number;
  bottom_panel_tab: string;
}

export interface WorkspaceStatus {
  open: boolean;
  workspace: Workspace | null;
  pane_state: PaneState;
  is_git: boolean;
}

interface WorkspaceStore {
  status: WorkspaceStatus;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  open: (path: string) => Promise<void>;
  close: () => Promise<void>;
  setPaneState: (state: PaneState) => Promise<void>;
}

export const useWorkspaceStore = create<WorkspaceStore>((set) => ({
  status: {
    open: false,
    workspace: null,
    pane_state: {
      explorer_visible: true,
      explorer_width: 240,
      agent_dock_visible: false,
      agent_dock_width: 300,
      bottom_panel_visible: false,
      bottom_panel_height: 240,
      bottom_panel_tab: "terminal",
    },
    is_git: false,
  },
  loading: false,
  error: null,
  refresh: async () => {
    try {
      const status = await invoke<WorkspaceStatus>("workspace_status");
      set({ status, error: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },
  open: async (path) => {
    set({ loading: true, error: null });
    try {
      await invoke("workspace_open", { path });
      const status = await invoke<WorkspaceStatus>("workspace_status");
      set({ status, loading: false });
    } catch (e) {
      set({ loading: false, error: String(e) });
    }
  },
  close: async () => {
    set({ loading: true });
    try {
      await invoke("workspace_close");
      const status = await invoke<WorkspaceStatus>("workspace_status");
      set({ status, loading: false });
    } catch (e) {
      set({ loading: false, error: String(e) });
    }
  },
  setPaneState: async (state) => {
    try {
      await invoke("workspace_set_pane_state", { state });
      const status = await invoke<WorkspaceStatus>("workspace_status");
      set({ status });
    } catch (e) {
      set({ error: String(e) });
    }
  },
}));