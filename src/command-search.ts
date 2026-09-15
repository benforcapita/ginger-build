export interface Searchable { title: string; keywords?: string; kind?: "action" | "file" | "directory" }
function score(text: string, query: string): number {
  const contiguous = text.indexOf(query);
  if (contiguous >= 0) return contiguous + (text.length - query.length) * 0.01;
  let position = -1, gaps = 0;
  for (const character of query) {
    const next = text.indexOf(character, position + 1);
    if (next < 0) return Infinity;
    gaps += next - position - 1;
    position = next;
  }
  return 100 + gaps;
}
export function rankResults<T extends Searchable>(items: T[], raw: string): T[] {
  const query = raw.trim().toLowerCase();
  const prefix = query.startsWith('>') ? 'action' : query.startsWith('/') ? 'file' : null;
  const words = (prefix ? query.slice(1).trim() : query).split(/\s+/).filter(Boolean);
  return items.filter((item) => !prefix || (prefix === 'action' ? !item.kind || item.kind === 'action' : item.kind === 'file' || item.kind === 'directory'))
    .map((item, order) => ({ item, order, score: words.reduce((total, word) => total + score(`${item.title} ${item.keywords ?? ''}`.toLowerCase(), word), 0) }))
    .filter((result) => Number.isFinite(result.score))
    .sort((a, b) => a.score - b.score || a.order - b.order).map((result) => result.item);
}
