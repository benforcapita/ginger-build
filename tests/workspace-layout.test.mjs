import { test } from 'node:test';
import assert from 'node:assert/strict';
import { placePane, reorderTabs } from '../src/workspace-layout.ts';

test('focusing a side pane swaps it with center without losing another pane', () => {
  const order = ['editor', 'agent', 'shell'];
  assert.deepEqual(placePane(order, 'shell', 0), ['shell', 'agent', 'editor']);
  assert.deepEqual(order, ['editor', 'agent', 'shell']);
});
test('manual placement can swap the two side panes and no-op safely', () => {
  assert.deepEqual(placePane(['editor', 'agent', 'shell'], 'shell', 1), ['editor', 'shell', 'agent']);
  const order = ['editor', 'agent', 'shell'];
  assert.equal(placePane(order, 'editor', 0), order);
});
test('tab movement changes order without changing identity or other groups', () => {
  const tabs = [{id: 1, kind: 'editor'}, {id: 2, kind: 'agent'}, {id: 3, kind: 'editor'}, {id: 4, kind: 'editor'}];
  const reordered = reorderTabs(tabs, 1, 4);
  assert.deepEqual(reordered.map(s => s.id), [3, 2, 4, 1]);
  assert.equal(reordered[3], tabs[0]);
  assert.deepEqual(reorderTabs(reordered, 1, 3).map(s => s.id), [1, 2, 3, 4]);
  assert.equal(reorderTabs(tabs, 1, 2), tabs);
  assert.equal(reorderTabs(tabs, 99, 1), tabs);
});

test('centering can be disabled without losing the focused pane, then re-enabled', async () => {
  const { focusLayout } = await import('../src/workspace-layout.ts');
  const original = { order: ['editor', 'agent', 'shell'], focused: 'editor', autoCenter: false };
  const manual = focusLayout(original, 'shell');
  assert.deepEqual(manual.order, original.order);
  assert.equal(manual.focused, 'shell');
  const automatic = focusLayout({ ...manual, autoCenter: true }, manual.focused);
  assert.deepEqual(automatic.order, ['shell', 'agent', 'editor']);
});
