import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface AppVersion {
  app_version: string;
  runtime_version: string;
  neovim_version: string;
  catalog_version: string;
  build_date: string;
}

export interface UpdateCandidate {
  version: string;
  download_url: string;
  sha256: string;
  signature: string;
}

export interface ValidationResult {
  candidate: UpdateCandidate;
  signature_valid: boolean;
  hash_valid: boolean;
  neovim_check_passed: boolean;
  core_check_passed: boolean;
  packages_check_passed: boolean;
  overall_passed: boolean;
}

interface PackagingStore {
  version: AppVersion | null;
  validating: boolean;
  lastValidation: ValidationResult | null;
  error: string | null;
  loadVersion: () => Promise<void>;
  validateUpdate: (candidate: UpdateCandidate, downloadedPath: string) => Promise<void>;
}

export const usePackagingStore = create<PackagingStore>((set) => ({
  version: null,
  validating: false,
  lastValidation: null,
  error: null,
  loadVersion: async () => {
    try {
      const version = await invoke<AppVersion | null>("packaging_version");
      set({ version });
    } catch (e) {
      set({ error: String(e) });
    }
  },
  validateUpdate: async (candidate, downloadedPath) => {
    set({ validating: true, error: null });
    try {
      const result = await invoke<ValidationResult>("packaging_validate_update", {
        candidate,
        downloadedPath,
      });
      set({ lastValidation: result, validating: false });
    } catch (e) {
      set({ validating: false, error: String(e) });
    }
  },
}));