import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export type GingerState = "idle" | "listening" | "thinking" | "coding" | "testing" | "reviewing" | "success" | "warning" | "failure";
export type Personality = "standard" | "quiet" | "extra";
export type MessageTier = "full" | "medium" | "compact";

export interface GingerConfig {
  personality: Personality;
  commentary: boolean;
}

export interface GingerMessage {
  text: string;
  tier: MessageTier;
}

interface PresenceStore {
  state: GingerState;
  config: GingerConfig;
  message: GingerMessage | null;
  setState: (s: GingerState) => Promise<void>;
  setConfig: (c: GingerConfig) => Promise<void>;
  refreshMessage: () => Promise<void>;
  toggleCommentary: () => Promise<void>;
  cyclePersonality: () => Promise<void>;
}

export const usePresenceStore = create<PresenceStore>((set) => ({
  state: "idle",
  config: { personality: "standard", commentary: true },
  message: null,
  setState: async (s) => {
    await invoke("presence_set_state", { state: s });
    set({ state: s });
  },
  setConfig: async (c) => {
    await invoke("presence_set_config", { config: c });
    set({ config: c });
  },
  refreshMessage: async () => {
    const message = await invoke<GingerMessage | null>("presence_message");
    set({ message });
  },
  toggleCommentary: async () => {
    await invoke("presence_toggle_commentary");
    const config = await invoke<GingerConfig>("presence_config");
    set({ config });
  },
  cyclePersonality: async () => {
    const personality = await invoke<Personality>("presence_cycle_personality");
    set((s) => ({ config: { ...s.config, personality } }));
  },
}));