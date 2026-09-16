export type ServerId = 'typescript' | 'python' | 'rust' | 'json';
export const serverNames: Record<ServerId, string> = { typescript: 'TypeScript / JavaScript', python: 'Python (Pyright)', rust: 'Rust (rust-analyzer)', json: 'JSON' };
export function languageFor(path: string): { server: ServerId; language: string } | null {
  const ext = path.split('.').at(-1)?.toLowerCase();
  if (['ts', 'tsx', 'js', 'jsx', 'mts', 'cts', 'mjs', 'cjs'].includes(ext ?? '')) return { server: 'typescript', language: ext === 'tsx' ? 'typescriptreact' : ext === 'jsx' ? 'javascriptreact' : ['ts', 'mts', 'cts'].includes(ext!) ? 'typescript' : 'javascript' };
  if (ext === 'py' || ext === 'pyi') return { server: 'python', language: 'python' };
  if (ext === 'rs') return { server: 'rust', language: 'rust' };
  if (ext === 'json' || ext === 'jsonc') return { server: 'json', language: ext };
  return null;
}
export function fileUri(root: string, path = '') { return 'file://' + (root.replace(/\/$/, '') + (path ? '/' + path : '/')).split('/').map(encodeURIComponent).join('/'); }
export function relativeUri(root: string, uri: string): string | null {
  try {
    const url = new URL(uri);
    if (url.protocol !== 'file:' || url.host || url.search || url.hash) return null;
    const path = decodeURIComponent(url.pathname), prefix = root.replace(/\/$/, '') + '/';
    if (!path.startsWith(prefix)) return null;
    const relative = path.slice(prefix.length);
    return relative && !relative.split('/').some(p => p === '..' || p === '.') ? relative : null;
  } catch { return null; }
}
export function prepareLspMessage(message: string, server?: ServerId) {
  const value = JSON.parse(message);
  if (value.method === 'initialize') {
    // CodeMirror currently advertises pull diagnostics but its diagnostics extension consumes push notifications.
    delete value.params.capabilities.textDocument.diagnostic;
    value.params.capabilities.workspace = { ...value.params.capabilities.workspace, configuration: true, workspaceFolders: true };
    if (server === 'json') value.params.initializationOptions = { ...value.params.initializationOptions, provideFormatter: true };
  }
  return JSON.stringify(value);
}
