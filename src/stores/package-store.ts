import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export type PackageKind = "neovim-plugin" | "lsp-server" | "formatter" | "linter" | "debugger" | "cli-tool";
export type OwnershipLane = "CURATED" | "CUSTOM";

export interface CatalogEntry {
  id: string;
  name: string;
  description: string;
  kind: PackageKind;
  source: string;
  version: string;
  ownership: OwnershipLane;
  install_mechanism: string;
  runtime_compatibility: string[];
  detection_rules: string[];
  recommendation_rules: string[];
}

export interface InstalledPackage {
  catalog_id: string;
  version: string;
  installed_at: string;
  path: string;
}

interface PackageStore {
  catalog: CatalogEntry[];
  searchResults: CatalogEntry[];
  loading: boolean;
  error: string | null;
  loadCatalog: () => Promise<void>;
  search: (query: string) => Promise<void>;
  install: (id: string) => Promise<InstalledPackage>;
}

export const usePackageStore = create<PackageStore>((set, get) => ({
  catalog: [],
  searchResults: [],
  loading: false,
  error: null,
  loadCatalog: async () => {
    try {
      const catalog = await invoke<CatalogEntry[]>("package_list_catalog");
      set({ catalog, error: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },
  search: async (query) => {
    try {
      const searchResults = await invoke<CatalogEntry[]>("package_search", { query });
      set({ searchResults });
    } catch (e) {
      set({ error: String(e) });
    }
  },
  install: async (id) => {
    set({ loading: true, error: null });
    try {
      const result = await invoke<InstalledPackage>("package_install", { id });
      set({ loading: false });
      return result;
    } catch (e) {
      set({ loading: false, error: String(e) });
      throw e;
    }
  },
}));