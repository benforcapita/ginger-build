import { test } from 'node:test';
import assert from 'node:assert/strict';
import { rankResults } from '../src/command-search.ts';
const items = [
  { title: 'Start agent harness', keywords: 'claude codex custom', kind: 'action' },
  { title: 'src/components/AgentDock.tsx', kind: 'file' },
  { title: 'docs/agent.md', kind: 'file' },
  { title: 'src/components', kind: 'directory' },
];
test('search finds ordered fuzzy file paths', () => {
  assert.equal(rankResults(items, '/ agdock')[0]?.title, 'src/components/AgentDock.tsx');
  assert.equal(rankResults(items, '/ no-such-file').length, 0);
});
test('prefixes select commands or files and keywords find actions', () => {
  assert.deepEqual(rankResults(items, '> codex'), [items[0]]);
  assert.deepEqual(rankResults(items, '/ agent'), [items[2], items[1]]);
});
test('multiple words match independently and whitespace resets search', () => {
  assert.deepEqual(rankResults(items, 'harness start'), [items[0]]);
  assert.equal(rankResults(items, '   ').length, 4);
});
