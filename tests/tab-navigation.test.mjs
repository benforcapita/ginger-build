import { test } from 'node:test';
import assert from 'node:assert/strict';
import { cycleTab, focusShortcut } from '../src/tab-navigation.ts';

test('cycles across pane groups in stable order and wraps in both directions', () => {
  const tabs = [{id: -1, kind: 'editor'}, {id: 3, kind: 'shell'}, {id: 2, kind: 'agent'}, {id: -2, kind: 'editor'}];
  assert.equal(cycleTab(tabs, -2, 1), 2);
  assert.equal(cycleTab(tabs, -1, -1), 3);
  assert.equal(cycleTab(tabs, 3, 1), -1);
  assert.equal(cycleTab(tabs, 99, 1), -1);
  assert.equal(cycleTab([], null, 1), null);
});
test('focus shortcuts distinguish navigation from normal terminal input', () => {
  const event = { key: 'Tab', ctrlKey: true, metaKey: false, altKey: false, shiftKey: false };
  assert.equal(focusShortcut(event), 'next');
  assert.equal(focusShortcut({...event, shiftKey: true}), 'previous');
  assert.equal(focusShortcut({...event, key: '2', ctrlKey: false, metaKey: true}), 'editor');
  assert.equal(focusShortcut({...event, key: '2', ctrlKey: false}), null);
  assert.equal(focusShortcut({...event, altKey: true}), null);
});
