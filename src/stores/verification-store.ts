import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface VerificationCommand {
  command: string;
  args: string[];
  cwd: string | null;
  timeout_seconds: number | null;
}

export interface CommandOutput {
  command: string;
  exit_code: number;
  stdout: string;
  stderr: string;
  success: boolean;
}

export interface VerificationResult {
  agent_id: number;
  commands_run: VerificationCommand[];
  success: boolean;
  outputs: CommandOutput[];
  duration_ms: number;
}

interface VerificationStore {
  result: VerificationResult | null;
  suggestions: VerificationCommand[];
  loading: boolean;
  error: string | null;
  run: (agentId: number, worktreePath: string, commands: VerificationCommand[]) => Promise<void>;
  suggest: (worktreePath: string) => Promise<void>;
}

export const useVerificationStore = create<VerificationStore>((set) => ({
  result: null,
  suggestions: [],
  loading: false,
  error: null,
  run: async (agentId, worktreePath, commands) => {
    set({ loading: true, error: null });
    try {
      const result = await invoke<VerificationResult>("verification_run", {
        agentId,
        worktreePath,
        commands,
      });
      set({ result, loading: false });
    } catch (e) {
      set({ loading: false, error: String(e) });
    }
  },
  suggest: async (worktreePath) => {
    try {
      const suggestions = await invoke<VerificationCommand[]>("verification_suggest", {
        worktreePath,
      });
      set({ suggestions });
    } catch (e) {
      set({ error: String(e) });
    }
  },
}));