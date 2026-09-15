export type TreeMove = 'up' | 'down' | 'first' | 'last' | 'parent' | 'child' | 'restore';
export function treeTarget(paths: string[], current: string | null, move: TreeMove): string | null {
  if (!paths.length) return null;
  let cursor = current ?? paths[0];
  while (!paths.includes(cursor) && cursor.includes('/')) cursor = cursor.slice(0, cursor.lastIndexOf('/'));
  if (!paths.includes(cursor)) cursor = paths[0];
  const index = paths.indexOf(cursor);
  switch (move) {
    case 'up': return paths[Math.max(0, index - 1)];
    case 'down': return paths[Math.min(paths.length - 1, index + 1)];
    case 'first': return paths[0];
    case 'last': return paths[paths.length - 1];
    case 'parent': { if (!cursor.includes('/')) return cursor; const parent = cursor.slice(0, cursor.lastIndexOf('/')); return paths.includes(parent) ? parent : cursor; }
    case 'child': return paths[index + 1]?.startsWith(cursor + '/') ? paths[index + 1] : cursor;
    default: return cursor;
  }
}
