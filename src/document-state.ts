export interface DocumentState { text: string; baseline: string; conflict: boolean }
export function reconcileDisk(doc: DocumentState, disk: string): DocumentState {
  if (disk === doc.baseline) return { ...doc, conflict: false };
  if (doc.text === doc.baseline || doc.text === disk) return { text: disk, baseline: disk, conflict: false };
  return { ...doc, conflict: true };
}
export function savedDocument(doc: DocumentState, snapshot: string): DocumentState {
  return { ...doc, baseline: snapshot, conflict: false };
}
export function editorText(text: string) {
  const separator = text.match(/\r\n|\r|\n/)?.[0] ?? '\n';
  const lines = text.split(/\r\n|\r|\n/);
  return { separator, lines, normalized: lines.join(separator) };
}
