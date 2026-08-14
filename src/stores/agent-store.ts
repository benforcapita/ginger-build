import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export type AdapterId = "claude-code" | "codex" | "ollama" | string;
export type AgentMode = "coding" | "review" | "research";
export type AgentStatus = "pending" | "running" | "completed" | "failed";
export type IsolationMode = "worktree" | "read-only" | "primary";

export interface AgentThread {
  id: number;
  adapter_id: AdapterId;
  title: string;
  mode: AgentMode;
  status: AgentStatus;
  isolation: IsolationMode;
  worktree_path: string | null;
  worktree_branch: string | null;
  terminal_id: number | null;
  base_revision: string | null;
  started_at: string | null;
  finished_at: string | null;
}

interface AgentStore {
  agents: AgentThread[];
  activeCount: number;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  create: (req: { adapter_id: AdapterId; title: string; mode: AgentMode; isolation: IsolationMode }) => Promise<AgentThread>;
  start: (id: number, worktreePath?: string, worktreeBranch?: string, baseRevision?: string, terminalId?: number) => Promise<void>;
  complete: (id: number, success: boolean) => Promise<void>;
  remove: (id: number) => Promise<void>;
}

export const useAgentStore = create<AgentStore>((set, get) => ({
  agents: [],
  activeCount: 0,
  loading: false,
  error: null,
  refresh: async () => {
    try {
      const agents = await invoke<AgentThread[]>("agent_list");
      const activeCount = await invoke<number>("agent_active_count");
      set({ agents, activeCount, error: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },
  create: async (req) => {
    set({ loading: true, error: null });
    try {
      const agent = await invoke<AgentThread>("agent_create", { req });
      await get().refresh();
      set({ loading: false });
      return agent;
    } catch (e) {
      set({ loading: false, error: String(e) });
      throw e;
    }
  },
  start: async (id, worktreePath, worktreeBranch, baseRevision, terminalId) => {
    await invoke("agent_start", {
      id,
      worktreePath: worktreePath ?? null,
      worktreeBranch: worktreeBranch ?? null,
      baseRevision: baseRevision ?? null,
      terminalId: terminalId ?? null,
    });
    await get().refresh();
  },
  complete: async (id, success) => {
    await invoke("agent_complete", { id, success });
    await get().refresh();
  },
  remove: async (id) => {
    await invoke("agent_remove", { id });
    await get().refresh();
  },
}));