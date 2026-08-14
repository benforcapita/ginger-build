import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface ProjectInfo {
  root: string;
  capabilities: string[];
  detected_files: string[];
}

export interface Recommendation {
  package_id: string;
  reason: string;
}

interface DetectionStore {
  projectInfo: ProjectInfo | null;
  recommendations: Recommendation[];
  loading: boolean;
  error: string | null;
  scan: (root: string) => Promise<void>;
  recommend: (capabilities: string[]) => Promise<void>;
}

export const useDetectionStore = create<DetectionStore>((set) => ({
  projectInfo: null,
  recommendations: [],
  loading: false,
  error: null,
  scan: async (root) => {
    set({ loading: true, error: null });
    try {
      const projectInfo = await invoke<ProjectInfo>("detection_scan", { root });
      const recommendations = await invoke<Recommendation[]>("detection_recommend", {
        capabilities: projectInfo.capabilities,
      });
      set({ projectInfo, recommendations, loading: false });
    } catch (e) {
      set({ loading: false, error: String(e) });
    }
  },
  recommend: async (capabilities) => {
    try {
      const recommendations = await invoke<Recommendation[]>("detection_recommend", {
        capabilities,
      });
      set({ recommendations });
    } catch (e) {
      set({ error: String(e) });
    }
  },
}));