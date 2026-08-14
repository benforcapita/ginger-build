import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface E2ETest {
  id: string;
  name: string;
  description: string;
  slices_required: string[];
}

interface StabilizationStore {
  tests: E2ETest[];
  wiringIssues: string[];
  loadTests: () => Promise<void>;
  verifyWiring: () => Promise<void>;
}

export const useStabilizationStore = create<StabilizationStore>((set) => ({
  tests: [],
  wiringIssues: [],
  loadTests: async () => {
    const tests = await invoke<E2ETest[]>("e2e_tests");
    set({ tests });
  },
  verifyWiring: async () => {
    const wiringIssues = await invoke<string[]>("e2e_verify_wiring");
    set({ wiringIssues });
  },
}));