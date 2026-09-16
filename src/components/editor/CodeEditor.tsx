import { completeAnyWord, startCompletion } from '@codemirror/autocomplete';
import { openLintPanel } from '@codemirror/lint';
import { LSPPlugin, jumpToDefinition, findReferences, formatDocument, showSignatureHelp } from '@codemirror/lsp-client';
import { languageExtension, registerLanguageView, renameAcrossFiles, documentSaved } from '@/editor/language-client';
import { languageFor, fileUri } from '@/language-support';
import { useWorkspaceStore } from '@/stores/workspace-store';
import { useLanguageStore } from '@/stores/language-store';
import { useEffect, useRef, useState } from 'react';
import { basicSetup } from 'codemirror';
import { EditorView } from '@codemirror/view';
import { Compartment, EditorState, Text } from '@codemirror/state';
import { languages } from '@codemirror/language-data';
import { LanguageDescription } from '@codemirror/language';
import { openSearchPanel } from '@codemirror/search';
import { vim, Vim, getCM, type ExParams } from '@replit/codemirror-vim';
import { editorText } from '@/document-state';
import { useSessionStore } from '@/stores/session-store';

const documents = new WeakMap<EditorView, number>();
function supportedWrite(params: ExParams) {
  if (params.argString?.trim()) {
    useSessionStore.getState().setError("Use :w to save the current file. Save-as paths and force writes are not supported.");
    return false;
  }
  return true;
}
Vim.defineEx('write', 'w', (cm, params) => { if (!supportedWrite(params)) return; const id = documents.get(cm.cm6); if (id !== undefined) void useSessionStore.getState().save(id); });
Vim.defineEx('quit', 'q', cm => { const id = documents.get(cm.cm6); if (id !== undefined) void useSessionStore.getState().close(id); });
Vim.defineEx('wq', 'wq', (cm, params) => { if (!supportedWrite(params)) return; const id = documents.get(cm.cm6); if (id !== undefined) void useSessionStore.getState().save(id).then(saved => { if (saved) void useSessionStore.getState().close(id); }); });
const theme = EditorView.theme({
  '&': { height: '100%', backgroundColor: '#101411', color: '#d5d7c5', fontSize: '13px' },
  '.cm-scroller': { overflow: 'auto', fontFamily: '"SFMono-Regular", Menlo, monospace', lineHeight: '1.65' },
  '.cm-content': { caretColor: '#e49a58' },
  '.cm-cursor, .cm-dropCursor': { borderLeftColor: '#e49a58' },
  '.cm-gutters': { backgroundColor: '#101411', color: '#647063', border: 'none' },
  '.cm-activeLine, .cm-activeLineGutter': { backgroundColor: '#ffffff06' },
  '&.cm-focused .cm-selectionBackground, .cm-selectionBackground, ::selection': { backgroundColor: '#80562d66' },
  '.cm-panels': { backgroundColor: '#1c221c', color: '#e1d4bb' },
  '.cm-textfield': { backgroundColor: '#111611', border: '1px solid #595442', color: '#e1d4bb' },
}, { dark: true });

export function CodeEditor({ id, active }: { id: number; active: boolean }) {
  const host = useRef<HTMLDivElement>(null);
  const view = useRef<EditorView | null>(null);
  const lineEnding = useRef('\n');
  const lspConfig = useRef(new Compartment());
  const vimConfig = useRef(new Compartment());
  const tab = useSessionStore(s => s.sessions.find(t => t.id === id));
  const enabled = useSessionStore(s => s.vim);
  const focus = useSessionStore(s => s.focusRequest);
  const root = useWorkspaceStore(s => s.status.workspace?.root_path);
  const revision = useLanguageStore(s => s.revision);
  const statuses = useLanguageStore(s => s.statuses);
  const server = tab?.path ? languageFor(tab.path)?.server : undefined;
  const serverState = server ? statuses[server]?.state : undefined;
  const [rename, setRename] = useState<string | null>(null);
  const [renaming, setRenaming] = useState(false);
  const [mode, setMode] = useState('NORMAL');
  const [position, setPosition] = useState('1:1');
  useEffect(() => {
    const initial = useSessionStore.getState().sessions.find(t => t.id === id);
    if (!host.current || !initial?.document) return;
    lineEnding.current = editorText(initial.document.text).separator;
    const language = new Compartment();
    const editor = new EditorView({ parent: host.current, state: EditorState.create({
      doc: Text.of(editorText(initial.document.text).lines),
      extensions: [
        vimConfig.current.of(useSessionStore.getState().vim ? vim() : []), basicSetup, theme, language.of([]), lspConfig.current.of([]),
        EditorState.languageData.of(() => [{ autocomplete: completeAnyWord }]),
        EditorView.contentAttributes.of({ 'aria-label': `Edit ${initial.path}`, spellcheck: 'false' }),
        EditorView.updateListener.of(update => {
          if (update.docChanged) useSessionStore.getState().edit(id, update.state.doc.toString().replace(/\n/g, lineEnding.current));
          if (update.selectionSet || update.docChanged) {
            const head = update.state.selection.main.head;
            const line = update.state.doc.lineAt(head);
            setPosition(`${line.number}:${head - line.from + 1}`);
          }
        }),
        EditorView.domEventHandlers({
          focus: () => { const s = useSessionStore.getState(); if (s.focusRequest?.id !== id) s.select('editor', id); },
          keydown: event => {
            if (event.key === 'F2') { event.preventDefault(); window.dispatchEvent(new CustomEvent('ginger-editor-action', { detail: 'rename' })); return true; }
            if (event.metaKey && ['s', 'k', 'p', 'o', 'q'].includes(event.key.toLowerCase()) || event.metaKey && event.shiftKey && ['e', 'n', 't'].includes(event.key.toLowerCase())) {
              // Let the window-level workbench shortcuts handle these keys.
              event.preventDefault(); return true;
            }
            return false;
          },
        }),
      ],
    }) });
    documents.set(editor, id);
    view.current = editor;
    let disposed = false;
    const support = LanguageDescription.matchFilename(languages, initial.path ?? '');
    void support?.load().then(extension => { if (!disposed) editor.dispatch({ effects: language.reconfigure(extension) }); }).catch(() => {});
    return () => { disposed = true; documents.delete(editor); editor.destroy(); view.current = null; };
  }, [id]);
  useEffect(() => {
    const editor = view.current;
    if (!editor || !root || !tab?.path) return;
    let disposed = false;
    let unregister: (() => void) | undefined;
    editor.dispatch({ effects: lspConfig.current.reconfigure([]) });
    const uri = fileUri(root, tab.path);
    void languageExtension(root, tab.path).then(extension => {
      if (!disposed) { editor.dispatch({ effects: lspConfig.current.reconfigure(extension) }); unregister = registerLanguageView(uri, editor); }
    }).catch(() => { if (!disposed) unregister = registerLanguageView(uri, editor); });
    return () => { disposed = true; unregister?.(); };
  }, [root, tab?.path, revision]);
  useEffect(() => {
    if (serverState === 'Unavailable' && view.current && LSPPlugin.get(view.current)) view.current.dispatch({ effects: lspConfig.current.reconfigure([]) });
  }, [serverState]);
  useEffect(() => {
    const saved = (event: Event) => { const path = (event as CustomEvent<string>).detail; if (path === tab?.path) documentSaved(path); };
    window.addEventListener('ginger-file-saved', saved);
    const action = (event: Event) => {
      if (useSessionStore.getState().active.editor !== id || !view.current) return;
      const editor = view.current;
      const command = (event as CustomEvent<string>).detail;
      editor.focus();
      if (command === 'complete') { startCompletion(editor); return; }
      if (command === 'diagnostics') { openLintPanel(editor); return; }
      const plugin = LSPPlugin.get(editor);
      if (!plugin?.client.serverCapabilities) { useSessionStore.getState().setError('Language server is not ready. Open Language servers from Commands for setup.'); return; }
      const capabilities = plugin.client.serverCapabilities;
      if (command === 'rename' && capabilities.renameProvider) { const word = editor.state.wordAt(editor.state.selection.main.head); setRename(word ? editor.state.sliceDoc(word.from, word.to) : ''); return; }
      const commands = { definition: [capabilities.definitionProvider, jumpToDefinition], references: [capabilities.referencesProvider, findReferences], format: [capabilities.documentFormattingProvider, formatDocument], signature: [capabilities.signatureHelpProvider, showSignatureHelp] } as const;
      const selected = commands[command as keyof typeof commands];
      if (!selected?.[0] || !selected[1](editor)) useSessionStore.getState().setError('This language server does not support that action here.');
    };
    window.addEventListener('ginger-editor-action', action);
    return () => { window.removeEventListener('ginger-editor-action', action); window.removeEventListener('ginger-file-saved', saved); };
  }, [id, tab?.path]);
  useEffect(() => {
    const editor = view.current;
    if (!editor) return;
    editor.dispatch({ effects: vimConfig.current.reconfigure(enabled ? vim() : []) });
    setMode(enabled ? 'NORMAL' : 'EDIT');
    const cm = getCM(editor);
    const changed = (event: { mode: string; subMode?: string }) => setMode((event.subMode ? `${event.mode} ${event.subMode}` : event.mode).toUpperCase());
    cm?.on('vim-mode-change', changed);
    return () => { cm?.off('vim-mode-change', changed); };
  }, [enabled]);
  useEffect(() => {
    const editor = view.current;
    const text = tab?.document?.text;
    if (editor && text !== undefined) {
      const incoming = editorText(text);
      if (/[\r\n]/.test(text)) lineEnding.current = incoming.separator;
      if (incoming.lines.join('\n') !== editor.state.doc.toString()) editor.dispatch({
        changes: { from: 0, to: editor.state.doc.length, insert: Text.of(incoming.lines) },
      });
    }
  }, [tab?.document?.text]);
  useEffect(() => { if (active && focus?.id === id) view.current?.focus(); }, [active, focus, id]);
  useEffect(() => {
    const search = () => { if (useSessionStore.getState().active.editor === id && view.current) openSearchPanel(view.current); };
    window.addEventListener('ginger-editor-search', search);
    return () => window.removeEventListener('ginger-editor-search', search);
  }, [id]);
  return <div className="code-editor-view" style={{ display: active ? 'flex' : 'none' }}>
    {(tab?.document?.conflict || tab?.diskError) && <div className="editor-conflict" role="alert"><span>{tab.diskError ?? 'Changed on disk. Your edits are preserved. Copy them before reloading if you want to keep both versions.'}</span><button onClick={() => void useSessionStore.getState().reload(id)}>Reload from disk</button></div>}
    {rename !== null && <form className="editor-rename" onSubmit={event => {
      event.preventDefault(); if (!view.current || renaming) return; setRenaming(true);
      void renameAcrossFiles(view.current, rename).then(() => setRename(null)).catch(e => useSessionStore.getState().setError(String(e))).finally(() => setRenaming(false));
    }}><label>Rename symbol <input autoFocus value={rename} disabled={renaming} onChange={e => setRename(e.target.value)} onKeyDown={e => { if (e.key === 'Escape') { setRename(null); view.current?.focus(); } }} /></label><button disabled={renaming || !rename.trim()}>Rename</button><button type="button" disabled={renaming} onClick={() => { setRename(null); view.current?.focus(); }}>Cancel</button></form>}
    <div ref={host} className="code-editor-host" />
    <div className="code-editor-status"><span className="accent">{mode}</span><span>{tab?.saving ? 'SAVING…' : tab?.document?.text !== tab?.document?.baseline ? 'UNSAVED' : 'SAVED'}</span><button onClick={() => useLanguageStore.setState({ open: true })}>{tab?.path && languageFor(tab.path) ? `LSP: ${statuses[languageFor(tab.path)!.server]?.state ?? 'Starting'}` : 'Word completion'}</button><span>{position} · UTF-8</span></div>
  </div>;
}
