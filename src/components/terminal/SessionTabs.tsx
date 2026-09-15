import { useSessionStore, type SessionKind } from "@/stores/session-store";
import { TerminalView } from "./TerminalView";
export function SessionTabs({ kind }: { kind: SessionKind }) {
  const sessions = useSessionStore((s) => s.sessions).filter((s) => s.kind === kind);
  const active = useSessionStore((s) => s.active[kind]);
  const select = useSessionStore((s) => s.select);
  const close = useSessionStore((s) => s.close);
  return <>
    <div className="session-tabs" role="tablist" aria-label={`${kind} tabs`}>
      {sessions.map((s) => <div key={s.id} className={`session-tab ${s.id === active ? "selected" : ""}`}>
        <button role="tab" aria-selected={s.id === active} onClick={() => select(kind, s.id)} title={s.path ?? s.title}>
          <span className={s.exited ? "dot exited" : "dot"} />{s.title}
        </button>
        <button className="tab-close" aria-label={`Close ${s.title}`} onClick={() => void close(s.id)}>×</button>
      </div>)}
    </div>
    <div className="session-views">{sessions.map((s) => <TerminalView key={s.id} id={s.id} active={s.id === active} />)}</div>
  </>;
}
