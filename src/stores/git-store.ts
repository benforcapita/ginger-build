import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface FileChange {
  path: string;
  status: string;
  staged: boolean;
}

export interface GitStatus {
  branch: string;
  clean: boolean;
  staged: FileChange[];
  unstaged: FileChange[];
  untracked: string[];
}

interface GitStore {
  status: GitStatus | null;
  isRepo: boolean;
  loading: boolean;
  error: string | null;
  refresh: (repo: string) => Promise<void>;
  checkRepo: (path: string) => Promise<boolean>;
  createWorktree: (repo: string, worktreePath: string, branch: string) => Promise<void>;
  applyPatch: (repo: string, patch: string) => Promise<void>;
  cherryPick: (repo: string, commit: string) => Promise<void>;
  diff: (repo: string, a: string, b: string) => Promise<string>;
}

export const useGitStore = create<GitStore>((set) => ({
  status: null,
  isRepo: false,
  loading: false,
  error: null,
  refresh: async (repo) => {
    set({ loading: true, error: null });
    try {
      const status = await invoke<GitStatus>("git_status", { repo });
      set({ status, loading: false });
    } catch (e) {
      set({ loading: false, error: String(e) });
    }
  },
  checkRepo: async (path) => {
    try {
      const isRepo = await invoke<boolean>("git_is_repo", { path });
      set({ isRepo });
      return isRepo;
    } catch {
      set({ isRepo: false });
      return false;
    }
  },
  createWorktree: async (repo, worktreePath, branch) => {
    await invoke("git_create_worktree", { repo, worktreePath, branch });
  },
  applyPatch: async (repo, patch) => {
    await invoke("git_apply_patch", { repo, patch });
  },
  cherryPick: async (repo, commit) => {
    await invoke("git_cherry_pick", { repo, commit });
  },
  diff: async (repo, a, b) => {
    return await invoke<string>("git_diff", { repo, a, b });
  },
}));