type Tab = { id: number; kind: string };
export function cycleTab(tabs: Tab[], current: number | null, direction: 1 | -1): number | null {
  const ordered = ['editor', 'agent', 'shell'].flatMap(kind => tabs.filter(tab => tab.kind === kind));
  if (!ordered.length) return null;
  const index = ordered.findIndex(tab => tab.id === current);
  return ordered[index < 0 ? (direction === 1 ? 0 : ordered.length - 1) : (index + direction + ordered.length) % ordered.length].id;
}
export type FocusAction = 'tree' | 'editor' | 'agent' | 'shell' | 'next' | 'previous';
export function focusShortcut(event: { key: string; metaKey: boolean; ctrlKey: boolean; altKey: boolean; shiftKey: boolean }): FocusAction | null {
  if (event.altKey) return null;
  if (event.ctrlKey && !event.metaKey && event.key === 'Tab') return event.shiftKey ? 'previous' : 'next';
  if (event.metaKey && !event.ctrlKey && !event.shiftKey) return ({'1': 'tree', '2': 'editor', '3': 'agent', '4': 'shell'} as Record<string, FocusAction>)[event.key] ?? null;
  return null;
}
