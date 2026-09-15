import { CodeEditor } from "@/components/editor/CodeEditor";
import { flushSync } from "react-dom";
import { useSessionStore, type SessionKind } from "@/stores/session-store";
import { useLayoutStore } from '@/stores/layout-store';
import { useReorderDrag } from '@/components/workspace/useReorderDrag';
import { TerminalView } from "./TerminalView";
export function SessionTabs({ kind }: { kind: SessionKind }) {
  const sessions = useSessionStore((s) => s.sessions).filter((s) => s.kind === kind);
  const active = useSessionStore((s) => s.active[kind]);
  const select = useSessionStore((s) => s.select);
  const close = useSessionStore((s) => s.close);
  const move = useSessionStore((s) => s.moveTab);
  const drag = useReorderDrag(`[data-tab-kind="${kind}"]`, (source, target) => move(Number(source), Number(target)));
  return <>
    <div className="session-tabs" role="tablist" aria-label={`${kind} tabs`}>
      {sessions.map((s, index) => <div key={s.id} data-tab-kind={kind} data-drag-id={s.id} className={`session-tab ${s.id === active ? "selected" : ""} ${drag.dragging && drag.target === String(s.id) ? 'tab-drop-target' : ''}`}>
        <span className="tab-grip" {...drag.handlers(String(s.id))} title="Drag to reorder tabs"><span className="grip-icon" aria-hidden="true" /></span>
        <button role="tab" aria-selected={s.id === active} onFocus={event => { if (event.currentTarget.matches(':focus-visible')) useLayoutStore.getState().focusPane(kind); }} onClick={() => select(kind, s.id)} title={`${s.path ?? s.title} · Option+Shift+Arrow to reorder`} onKeyDown={event => {
          if (!['ArrowLeft', 'ArrowRight'].includes(event.key)) return;
          event.preventDefault();
          const next = sessions[index + (event.key === 'ArrowLeft' ? -1 : 1)];
          if (!next) return;
          if (event.altKey && event.shiftKey) {
            const button = event.currentTarget;
            flushSync(() => move(s.id, next.id));
            button.focus();
          }
          else select(kind, next.id);
        }}>
          <span className={s.exited ? "dot exited" : "dot"} />{s.title}{s.document && s.document.text !== s.document.baseline ? " •" : ""}
        </button>
        <button className="tab-close" aria-label={`Close ${s.title}`} onClick={() => void close(s.id)}>×</button>
      </div>)}
    </div>
    <div className="session-views">{[...sessions].sort((a, b) => a.id - b.id).map((s) => s.document ? <CodeEditor key={s.id} id={s.id} active={s.id === active} /> : <TerminalView key={s.id} id={s.id} active={s.id === active} />)}</div>
  </>;
}
