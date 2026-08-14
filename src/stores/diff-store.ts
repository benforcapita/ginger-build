import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface DiffLine {
  line_type: "context" | "addition" | "deletion";
  content: string;
  old_line: number | null;
  new_line: number | null;
}

export interface DiffHunk {
  header: string;
  old_start: number;
  old_count: number;
  new_start: number;
  new_count: number;
  lines: DiffLine[];
}

export interface DiffFile {
  path: string;
  status: string;
  hunks: DiffHunk[];
}

export interface HunkRef {
  file: string;
  hunk_index: number;
}

interface DiffStore {
  files: DiffFile[];
  acceptedHunks: HunkRef[];
  loading: boolean;
  error: string | null;
  getDiff: (repo: string, base: string, head: string) => Promise<void>;
  toggleHunk: (file: string, hunkIndex: number) => void;
  applyPatch: (repo: string, patch: string) => Promise<void>;
  buildPatch: () => Promise<string>;
}

export const useDiffStore = create<DiffStore>((set, get) => ({
  files: [],
  acceptedHunks: [],
  loading: false,
  error: null,
  getDiff: async (repo, base, head) => {
    set({ loading: true, error: null });
    try {
      const files = await invoke<DiffFile[]>("diff_get", { repo, base, head });
      set({ files, loading: false });
    } catch (e) {
      set({ loading: false, error: String(e) });
    }
  },
  toggleHunk: (file, hunkIndex) => {
    const ref: HunkRef = { file, hunk_index: hunkIndex };
    set((s) => {
      const exists = s.acceptedHunks.some(
        (h) => h.file === ref.file && h.hunk_index === ref.hunk_index
      );
      return {
        acceptedHunks: exists
          ? s.acceptedHunks.filter((h) => !(h.file === ref.file && h.hunk_index === ref.hunk_index))
          : [...s.acceptedHunks, ref],
      };
    });
  },
  applyPatch: async (repo, patch) => {
    await invoke("diff_apply", { repo, patch });
  },
  buildPatch: async () => {
    const { files, acceptedHunks } = get();
    return await invoke<string>("diff_build_patch", { files, acceptedHunks });
  },
}));