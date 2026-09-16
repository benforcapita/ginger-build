import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useLanguageStore, type ServerInfo } from '@/stores/language-store';
import { serverNames } from '@/language-support';
import { restartLanguageServers } from '@/editor/language-client';
import { useSessionStore } from '@/stores/session-store';
export function LanguageServers() {
  const [servers, setServers] = useState<ServerInfo[]>([]);
  const [error, setError] = useState('');
  const [installing, setInstalling] = useState(false);
  const busy = useSessionStore(s => s.busy);
  const statuses = useLanguageStore(s => s.statuses);
  const dialog = useRef<HTMLDivElement>(null);
  const close = () => useLanguageStore.setState({ open: false });
  const refresh = () => { void invoke<ServerInfo[]>('language_server_status').then(setServers).catch(e => setError(String(e))); };
  useEffect(() => { const previous = document.activeElement as HTMLElement; dialog.current?.querySelector<HTMLButtonElement>('button')?.focus(); refresh(); return () => { if (previous?.isConnected) previous.focus(); }; }, []);
  const install = async (server: ServerInfo) => {
    setInstalling(true);
    try {
      await useSessionStore.getState().start('shell', { program: server.install_program, args: server.install_args, title: `Install ${serverNames[server.id]}` });
      if (useSessionStore.getState().error) setError(useSessionStore.getState().error!);
      else close();
    } finally { setInstalling(false); }
  };
  return <div className="palette-overlay" onClick={close}><div ref={dialog} role="dialog" aria-modal="true" aria-label="Language servers" className="language-servers" onClick={e => e.stopPropagation()} onKeyDown={e => {
    if (e.key === 'Escape') { e.preventDefault(); close(); }
    if (e.key === 'Tab') {
      const buttons = [...dialog.current!.querySelectorAll<HTMLButtonElement>('button:not(:disabled)')];
      const index = buttons.indexOf(document.activeElement as HTMLButtonElement);
      e.preventDefault(); buttons[(index + (e.shiftKey ? buttons.length - 1 : 1)) % buttons.length]?.focus();
    }
  }}>
    <header><h2>Language servers</h2><button onClick={close} aria-label="Close language servers">×</button></header>
    <p>Local code intelligence for your project. Highlighting and word suggestions work without a server.</p>
    {error && <p role="alert">{error}</p>}
    {servers.map(server => <section key={server.id}><div><strong>{serverNames[server.id]}</strong><span>{statuses[server.id]?.state ?? (server.available ? 'Installed · starts when a file opens' : 'Not installed')}</span></div>
      <small>{statuses[server.id]?.detail || server.path || `Requires ${server.executable}`}</small>
      <button disabled={installing || busy} onClick={() => void install(server)}>{server.available ? 'Update / repair' : 'Install'} in terminal</button>
    </section>)}
    <p>Node servers require Node.js/npm and install under ~/.ginger/language-servers. Rust requires rustup. After installation finishes, restart the servers.</p>
    <footer><button onClick={refresh}>Refresh installed tools</button><button onClick={() => void restartLanguageServers().then(refresh).catch(e => setError(String(e)))}>Restart language servers</button></footer>
  </div></div>;
}
