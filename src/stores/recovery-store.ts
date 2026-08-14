import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface RecoveryWorktree {
  path: string;
  branch: string;
  status: string;
  has_unapplied_changes: boolean;
  owner_agent_id: number | null;
}

export interface RecoveryAction {
  action: string;
  target: string;
  result: string;
}

export interface RecoveryReport {
  crashed_sessions: number[];
  recovered_worktrees: RecoveryWorktree[];
  actions: RecoveryAction[];
  safe_mode: boolean;
}

interface RecoveryStore {
  report: RecoveryReport | null;
  safeMode: boolean;
  loading: boolean;
  heartbeat: () => Promise<void>;
  isStale: () => Promise<boolean>;
  enterSafeMode: () => Promise<void>;
  exitSafeMode: () => Promise<void>;
  run: (dataRoot: string) => Promise<void>;
}

export const useRecoveryStore = create<RecoveryStore>((set) => ({
  report: null,
  safeMode: false,
  loading: false,
  heartbeat: async () => { await invoke("recovery_heartbeat"); },
  isStale: async () => { return await invoke<boolean>("recovery_is_stale"); },
  enterSafeMode: async () => {
    await invoke("recovery_enter_safe_mode");
    set({ safeMode: true });
  },
  exitSafeMode: async () => {
    await invoke("recovery_exit_safe_mode");
    set({ safeMode: false });
  },
  run: async (dataRoot) => {
    set({ loading: true });
    try {
      const report = await invoke<RecoveryReport>("recovery_run", { dataRoot });
      set({ report, safeMode: report.safe_mode, loading: false });
    } catch (e) {
      set({ loading: false });
    }
  },
}));