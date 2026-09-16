import { Channel, invoke } from '@tauri-apps/api/core';
import { LSPClient, LSPPlugin, serverCompletion, hoverTooltips, signatureHelp, serverDiagnostics, jumpToDefinitionKeymap, findReferencesKeymap, formatKeymap } from '@codemirror/lsp-client';
import { keymap, type EditorView } from '@codemirror/view';
import { Text, type Extension } from '@codemirror/state';
import DOMPurify from 'dompurify';
import { fileUri, relativeUri, languageFor, prepareLspMessage, type ServerId } from '../language-support';
import { useLanguageStore } from '../stores/language-store';
import { useSessionStore } from '../stores/session-store';
import { useWorkspaceStore } from '../stores/workspace-store';

type Connection = { client: LSPClient; id?: number; closed: boolean; pending: number; ready: Promise<Connection> };
const connections = new Map<string, Connection>();
const editors = new Map<string, EditorView>();
const waiters = new Map<string, Set<(view: EditorView) => void>>();
let opening = Promise.resolve();
let generation = 0;
function report(server: ServerId, state: string, detail?: string) { useLanguageStore.getState().report(server, state, detail); }
export function registerLanguageView(uri: string, view: EditorView) {
  editors.set(uri, view);
  waiters.get(uri)?.forEach(resolve => resolve(view));
  waiters.delete(uri);
  return () => { if (editors.get(uri) === view) editors.delete(uri); };
}
async function openFile(root: string, uri: string): Promise<EditorView | null> {
  const path = relativeUri(root, uri);
  if (!path || useWorkspaceStore.getState().status.workspace?.root_path !== root) throw new Error('Language server target is outside the current workspace');
  const requestedGeneration = generation;
  const valid = () => requestedGeneration === generation && useWorkspaceStore.getState().status.workspace?.root_path === root;
  const task = opening.then(async () => {
    if (!valid()) throw new Error('Workspace changed during navigation');
    await useSessionStore.getState().start('editor', { path });
    if (!valid()) throw new Error('Workspace changed during navigation');
    const current = editors.get(uri);
    if (current) return current;
    if (!useSessionStore.getState().sessions.some(s => s.kind === 'editor' && s.path === path)) throw new Error('Could not open language server target');
    return new Promise<EditorView>((resolve, reject) => {
      const ready = (view: EditorView) => { clearTimeout(timer); if (valid()) resolve(view); else reject(new Error('Workspace changed during navigation')); };
      const timer = setTimeout(() => { waiters.get(uri)?.delete(ready); reject(new Error('Timed out opening language server target')); }, 5000);
      const set = waiters.get(uri) ?? new Set(); set.add(ready); waiters.set(uri, set);
    });
  });
  opening = task.then(() => {}, () => {});
  return task;
}
function fail(connection: Connection, server: ServerId, error: unknown) {
  if (connection.closed) return;
  connection.closed = true;
  connection.client.disconnect();
  if (connection.id !== undefined) void invoke('language_server_stop', { id: connection.id });
  report(server, 'Unavailable', error && typeof error === 'object' && 'message' in error ? String(error.message) : String(error));
}
async function connect(root: string, server: ServerId): Promise<Connection> {
  const key = root + ':' + server;
  const existing = connections.get(key);
  if (existing) return existing.ready;
  report(server, 'Starting');
  const client = new LSPClient({
    rootUri: fileUri(root), timeout: 15000, sanitizeHTML: html => DOMPurify.sanitize(html),
    extensions: [serverCompletion(), hoverTooltips(), signatureHelp(), serverDiagnostics(), keymap.of([...jumpToDefinitionKeymap, ...findReferencesKeymap, ...formatKeymap])],
    notificationHandlers: {
      'window/showMessage': (_client, params) => { report(server, 'Ready', String(params.message)); return true; },
    },
  });
  client.workspace.displayFile = uri => openFile(root, uri);
  client.workspace.requestFile = async uri => { if (!client.workspace.getFile(uri)) await openFile(root, uri); return client.workspace.getFile(uri); };
  const connection = { client, closed: false, pending: 0 } as Connection;
  connections.set(key, connection);
  connection.ready = (async () => {
    try {
      const handlers = new Set<(message: string) => void>();
      let writes = Promise.resolve();
      const send = (message: string) => {
        message = prepareLspMessage(message, server);
        if (connection.closed) throw new Error('Language server is disconnected');
        if (++connection.pending > 32) { connection.pending--; fail(connection, server, 'Language server input queue is full. Restart it from Commands.'); throw new Error('Language server input queue is full'); }
        writes = writes.then(async () => { if (!connection.closed) await invoke('language_server_send', { id: connection.id, message }); }).catch(error => fail(connection, server, error)).finally(() => { connection.pending--; });
      };
      const events = new Channel<{ id: number; kind: string; data: string }>();
      events.onmessage = event => {
        if (event.kind === 'message') void invoke('language_server_ack', { id: event.id }).catch(() => {});
        if (connection.closed) return;
        if (event.kind === 'exit') { fail(connection, server, event.data); return; }
        try {
          const msg = JSON.parse(event.data);
          // Configuration and progress are server-initiated requests. Never allow implicit disk edits or arbitrary commands.
          if (msg.method && msg.id !== undefined) {
            let result: unknown = null;
            let error: unknown;
            if (msg.method === 'workspace/configuration') result = (msg.params?.items ?? []).map((item: { section?: string }) => item.section === 'python' ? { analysis: { typeCheckingMode: 'basic', autoSearchPaths: true, useLibraryCodeForTypes: true } } : item.section === 'python.analysis' ? { typeCheckingMode: 'basic', autoSearchPaths: true, useLibraryCodeForTypes: true } : item.section === 'json' ? { validate: { enable: true } } : {});
            else if (msg.method === 'workspace/workspaceFolders') result = [{ uri: fileUri(root), name: root.split('/').at(-1) }];
            else if (!['client/registerCapability', 'client/unregisterCapability', 'window/workDoneProgress/create'].includes(msg.method)) error = { code: -32601, message: 'Server-initiated edits and commands are not supported' };
            send(JSON.stringify({ jsonrpc: '2.0', id: msg.id, ...(error ? { error } : { result }) }));
          } else handlers.forEach(handler => handler(event.data));
        } catch (error) { fail(connection, server, error); }
      };
      connection.id = await invoke<number>('language_server_start', { server, root, onEvent: events });
      if (connection.closed) { await invoke('language_server_stop', { id: connection.id }); throw new Error('Language server start cancelled'); }
      client.connect({ send, subscribe: h => { handlers.add(h); }, unsubscribe: h => { handlers.delete(h); } });
      await client.initializing;
      if (connection.closed) throw new Error('Language server disconnected');
      client.notification('workspace/didChangeConfiguration', { settings: { python: { analysis: { typeCheckingMode: 'basic', autoSearchPaths: true, useLibraryCodeForTypes: true } }, json: { validate: { enable: true } } } });
      report(server, 'Ready');
      return connection;
    } catch (error) { fail(connection, server, error); throw error; }
  })();
  return connection.ready;
}
export async function languageExtension(root: string, path: string): Promise<Extension> {
  const language = languageFor(path);
  if (!language) return [];
  const { client } = await connect(root, language.server);
  return client.plugin(fileUri(root, path), language.language);
}
export async function restartLanguageServers() {
  generation++;
  for (const connection of connections.values()) { connection.closed = true; connection.client.disconnect(); }
  connections.clear();
  await invoke('language_servers_stop_all');
  useLanguageStore.setState(s => ({ statuses: {}, revision: s.revision + 1 }));
}
export function documentSaved(path: string) {
  const root = useWorkspaceStore.getState().status.workspace?.root_path;
  const language = languageFor(path);
  if (!root || !language) return;
  const connection = connections.get(root + ':' + language.server);
  if (!connection || connection.closed || !connection.client.serverCapabilities) return;
  connection.client.sync();
  connection.client.notification('textDocument/didSave', { textDocument: { uri: fileUri(root, path) } });
}

type Position = { line: number; character: number };
type Edit = { range: { start: Position; end: Position }; newText: string };
type RenameResult = { changes?: Record<string, Edit[]>; documentChanges?: { textDocument?: { uri: string; version?: number | null }; edits?: Edit[]; kind?: string }[] } | null;
function editGroups(result: RenameResult) {
  const groups: Record<string, Edit[]> = { ...result?.changes };
  for (const change of result?.documentChanges ?? []) {
    if (change.kind || !change.textDocument || !change.edits) throw new Error('This rename requires file operations that Ginger does not support');
    groups[change.textDocument.uri] = [...(groups[change.textDocument.uri] ?? []), ...change.edits];
  }
  return groups;
}
function offset(doc: Text, pos: Position) {
  if (pos.line < 0 || pos.line >= doc.lines) throw new Error('Invalid language-server edit');
  const line = doc.line(pos.line + 1);
  if (pos.character < 0 || pos.character > line.length) throw new Error('Invalid language-server edit');
  return line.from + pos.character;
}
export async function renameAcrossFiles(view: EditorView, newName: string) {
  const plugin = LSPPlugin.get(view);
  const root = useWorkspaceStore.getState().status.workspace?.root_path;
  if (!plugin || !root || !newName.trim()) throw new Error('A ready language server and a symbol name are required');
  const requestedGeneration = generation;
  const stillCurrent = () => requestedGeneration === generation && useWorkspaceStore.getState().status.workspace?.root_path === root;
  const position = plugin.toPosition(view.state.selection.main.head);
  const request = () => plugin.client.request('textDocument/rename', { textDocument: { uri: plugin.uri }, position, newName }) as Promise<RenameResult>;
  plugin.client.sync();
  const originalDoc = view.state.doc;
  const discovery = editGroups(await request());
  if (!stillCurrent() || view.state.doc !== originalDoc) throw new Error('Document changed during rename. Try again.');
  for (const uri of Object.keys(discovery)) { if (!relativeUri(root, uri)) throw new Error('Rename affects a file outside the workspace'); await openFile(root, uri); }
  if (!stillCurrent() || view.state.doc !== originalDoc) throw new Error('Document changed during rename. Try again.');
  plugin.client.sync();
  const snapshots = new Map([...editors].map(([uri, editor]) => [uri, editor.state.doc]));
  const result = editGroups(await request());
  if (!stillCurrent()) throw new Error('Workspace or language server changed during rename');
  const prepared = Object.entries(result).map(([uri, edits]) => {
    const target = editors.get(uri), snapshot = snapshots.get(uri);
    if (!relativeUri(root, uri) || !target || !snapshot || target.state.doc !== snapshot) throw new Error('Affected files changed during rename. No edits applied; try again.');
    const changes = edits.map(edit => ({ from: offset(snapshot, edit.range.start), to: offset(snapshot, edit.range.end), insert: edit.newText }));
    return { target, transaction: target.state.update({ changes, userEvent: 'input.rename' }) };
  });
  for (const { target, transaction } of prepared) target.dispatch(transaction);
  await openFile(root, plugin.uri);
}
