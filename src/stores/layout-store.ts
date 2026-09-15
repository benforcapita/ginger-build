import { create } from 'zustand';
import { focusLayout, placePane, type Pane } from '../workspace-layout';
interface Layout {
  order: Pane[];
  autoCenter: boolean;
  focused: Pane;
  focusPane: (pane: Pane) => void;
  movePane: (pane: Pane, slot: number) => void;
  toggleAutoCenter: () => void;
  reset: () => void;
}
export const useLayoutStore = create<Layout>((set) => ({
  order: ['editor', 'agent', 'shell'], autoCenter: true, focused: 'editor',
  focusPane: (pane) => set(s => focusLayout(s, pane)),
  movePane: (pane, slot) => set(s => ({ order: placePane(s.order, pane, slot) })),
  toggleAutoCenter: () => set(s => focusLayout({ ...s, autoCenter: !s.autoCenter }, s.focused)),
  reset: () => set(s => ({ autoCenter: true, order: placePane(['editor', 'agent', 'shell'], s.focused, 0) })),
}));
