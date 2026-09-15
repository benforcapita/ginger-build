import { test } from 'node:test';
import assert from 'node:assert/strict';
import { reconcileDisk, savedDocument } from '../src/document-state.ts';
const clean = { text: 'one', baseline: 'one', conflict: false };
test('clean documents reload external edits', () => {
  assert.deepEqual(reconcileDisk(clean, 'two'), { text: 'two', baseline: 'two', conflict: false });
});
test('dirty documents preserve edits and identify disk conflict', () => {
  assert.deepEqual(reconcileDisk({ ...clean, text: 'mine' }, 'theirs'), { text: 'mine', baseline: 'one', conflict: true });
});
test('saving a snapshot does not clear newer unsaved edits', () => {
  assert.deepEqual(savedDocument({ ...clean, text: 'newer' }, 'saved'), { text: 'newer', baseline: 'saved', conflict: false });
});
test('line-ending changes keep separate editor lines', async () => {
  const { editorText } = await import('../src/document-state.ts');
  assert.deepEqual(editorText('a\r\nb\r\n'), { separator: '\r\n', lines: ['a', 'b', ''], normalized: 'a\r\nb\r\n' });
  assert.deepEqual(editorText('a\nb\n'), { separator: '\n', lines: ['a', 'b', ''], normalized: 'a\nb\n' });
  assert.deepEqual(editorText('a\r\nb\n'), { separator: '\r\n', lines: ['a', 'b', ''], normalized: 'a\r\nb\r\n' });
});
