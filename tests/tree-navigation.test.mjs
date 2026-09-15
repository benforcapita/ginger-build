import { test } from 'node:test';
import assert from 'node:assert/strict';
import { treeTarget } from '../src/tree-navigation.ts';
const rows = ['src', 'src/auth', 'src/auth/login.ts', 'src/main.rs', 'README.md'];
test('tree movement follows visible order and clamps at both ends', () => {
  assert.equal(treeTarget(rows, 'src', 'down'), 'src/auth');
  assert.equal(treeTarget(rows, 'src/main.rs', 'up'), 'src/auth/login.ts');
  assert.equal(treeTarget(rows, 'src', 'up'), 'src');
  assert.equal(treeTarget(rows, 'README.md', 'down'), 'README.md');
  assert.equal(treeTarget([], null, 'down'), null);
});
test('collapsed or removed cursor falls back to closest visible parent', () => {
  assert.equal(treeTarget(['src', 'README.md'], 'src/auth/login.ts', 'restore'), 'src');
  assert.equal(treeTarget(rows, 'removed.ts', 'restore'), 'src');
});
test('parent, child and boundary navigation never escapes the visible tree', () => {
  assert.equal(treeTarget(rows, 'src/auth/login.ts', 'parent'), 'src/auth');
  assert.equal(treeTarget(rows, 'src', 'child'), 'src/auth');
  assert.equal(treeTarget(rows, 'README.md', 'child'), 'README.md');
  assert.equal(treeTarget(['sr', 'src'], 'src', 'parent'), 'src');
  assert.equal(treeTarget(rows, 'src/main.rs', 'first'), 'src');
  assert.equal(treeTarget(rows, 'src', 'last'), 'README.md');
});
