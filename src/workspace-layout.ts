export type Pane = 'editor' | 'agent' | 'shell';
export function placePane(order: Pane[], pane: Pane, slot: number): Pane[] {
  const source = order.indexOf(pane);
  if (source < 0 || slot < 0 || slot >= order.length || source === slot) return order;
  const next = [...order];
  [next[source], next[slot]] = [next[slot], next[source]];
  return next;
}
export function reorderTabs<T extends { id: number; kind: string }>(tabs: T[], id: number, target: number): T[] {
  const source = tabs.find(s => s.id === id);
  const destination = tabs.find(s => s.id === target);
  if (!source || !destination || source.kind !== destination.kind || id === target) return tabs;
  const group = tabs.filter(s => s.kind === source.kind);
  const from = group.indexOf(source), to = group.indexOf(destination);
  group.splice(from, 1);
  group.splice(to, 0, source);
  let index = 0;
  return tabs.map(s => s.kind === source.kind ? group[index++] : s);
}

export function focusLayout<T extends { order: Pane[]; focused: Pane; autoCenter: boolean }>(state: T, pane: Pane): T {
  return { ...state, focused: pane, order: state.autoCenter ? placePane(state.order, pane, 0) : state.order };
}
