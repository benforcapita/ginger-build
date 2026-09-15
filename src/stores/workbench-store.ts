import { create } from 'zustand';
interface Workbench {
  treeRevision: number;
  expanded: Set<string>;
  showCompanion: boolean;
  quiet: boolean;
  pets: number;
  creatingHarness: boolean;
  refreshTree: () => void;
  toggleDirectory: (path: string) => void;
  collapseTree: () => void;
  pet: () => void;
}
export const useWorkbenchStore = create<Workbench>((set) => ({
  treeRevision: 0, expanded: new Set(), showCompanion: true, quiet: false, pets: 0, creatingHarness: false,
  refreshTree: () => set((s) => ({ treeRevision: s.treeRevision + 1 })),
  collapseTree: () => set({ expanded: new Set() }),
  toggleDirectory: (path) => set((s) => {
    const expanded = new Set(s.expanded);
    if (expanded.has(path)) expanded.delete(path);
    else {
      const parts = path.split('/');
      for (let i = 1; i <= parts.length; i++) expanded.add(parts.slice(0, i).join('/'));
    }
    return { expanded };
  }),
  pet: () => set((s) => ({ pets: s.pets + 1, showCompanion: true })),
}));
