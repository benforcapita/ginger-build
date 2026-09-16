import { Editor } from '@/components/editor/Editor';
import { Terminal } from '@/components/terminal/Terminal';
import { AgentDock } from '@/components/agent-dock/AgentDock';
import { useLayoutStore } from '@/stores/layout-store';
import { type Pane } from '@/workspace-layout';
import { useReorderDrag } from './useReorderDrag';
const panes: Pane[] = ['editor', 'agent', 'shell'];
export const paneNames: Record<Pane, string> = { editor: 'Editor', agent: 'Agents', shell: 'Terminal' };
export const slotNames = ['center', 'upper right', 'lower right'];
export function WorkspacePanes() {
  const layout = useLayoutStore();
  const drag = useReorderDrag('[data-pane]', (source, target) => {
    if (panes.includes(source as Pane) && panes.includes(target as Pane)) layout.movePane(source as Pane, layout.order.indexOf(target as Pane));
  });
  // Stable siblings: moving a pane changes CSS placement, never its React parent.
  return <>{panes.map(pane => {
    const slot = layout.order.indexOf(pane);
    return <div key={pane} data-pane={pane} tabIndex={-1} aria-label={`${paneNames[pane]} pane`} data-drag-id={pane} className={`movable-pane pane-slot-${slot} ${layout.focused === pane ? 'pane-focused' : ''} ${drag.dragging && drag.target === pane ? 'pane-drop-target' : ''}`}>
      <div className="pane-arrangement">
        <button className="drag-handle" {...drag.handlers(pane)} aria-label={`Move ${paneNames[pane]} pane`} title="Drag to another pane to swap. Use left/right arrows to change position." onKeyDown={event => {
          if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') return;
          event.preventDefault();
          layout.movePane(pane, (slot + (event.key === 'ArrowRight' ? 1 : 2)) % 3);
        }}><span className="grip-icon" aria-hidden="true" /><span>Drag {paneNames[pane]}</span></button>
        <span className="pane-position">{slotNames[slot]}</span>
        <button onClick={() => layout.movePane(pane, 0)} disabled={slot === 0} aria-label={`Move ${paneNames[pane]} to center`}>Center</button>
      </div>
      {pane === 'editor' ? <Editor /> : pane === 'agent' ? <AgentDock /> : <Terminal />}
    </div>;
  })}</>;
}
