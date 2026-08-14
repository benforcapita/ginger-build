import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface ActionDef {
  id: string;
  title: string;
  category: string;
  keybinding: string | null;
  icon: string | null;
  when: string | null;
  args_schema: unknown | null;
}

export interface ActionContext {
  workspace_open: boolean;
  editor_ready: boolean;
  agent_count: number;
  terminal_count: number;
  git_clean: boolean;
  safe_mode: boolean;
}

interface ActionStore {
  actions: ActionDef[];
  context: ActionContext;
  loadActions: () => Promise<void>;
  invokeAction: (id: string, args?: unknown) => Promise<unknown>;
  refreshContext: () => Promise<void>;
}

export const useActionStore = create<ActionStore>((set, get) => ({
  actions: [],
  context: {
    workspace_open: false,
    editor_ready: false,
    agent_count: 0,
    terminal_count: 0,
    git_clean: true,
    safe_mode: false,
  },
  loadActions: async () => {
    const actions = await invoke<ActionDef[]>("list_actions");
    set({ actions });
  },
  invokeAction: async (id, args) => {
    return await invoke("invoke_action", { invocation: { id, args: args ?? null } });
  },
  refreshContext: async () => {
    const context = await invoke<ActionContext>("get_action_context");
    set({ context });
  },
}));