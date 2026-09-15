import { useRef, useState, type PointerEvent } from 'react';

// Pointer capture works in the native webview without changing its file-drop settings.
export function useReorderDrag(selector: string, onDrop: (source: string, target: string) => void) {
  const gesture = useRef<{ source: string; x: number; y: number; moved: boolean; target: string | null } | null>(null);
  const [target, setTarget] = useState<string | null>(null);
  const [dragging, setDragging] = useState(false);
  const cancel = () => { gesture.current = null; setTarget(null); setDragging(false); };
  const handlers = (source: string) => ({
    onPointerDown: (event: PointerEvent<HTMLElement>) => {
      if (event.button !== 0) return;
      event.preventDefault();
      event.currentTarget.setPointerCapture(event.pointerId);
      gesture.current = { source, x: event.clientX, y: event.clientY, moved: false, target: null };
    },
    onPointerMove: (event: PointerEvent<HTMLElement>) => {
      const current = gesture.current;
      if (!current) return;
      if (!current.moved && Math.hypot(event.clientX - current.x, event.clientY - current.y) < 5) return;
      current.moved = true;
      current.target = document.elementFromPoint(event.clientX, event.clientY)?.closest<HTMLElement>(selector)?.dataset.dragId ?? null;
      setDragging(true);
      setTarget(current.target);
    },
    onPointerUp: () => {
      const current = gesture.current;
      cancel();
      if (current?.moved && current.target !== null) onDrop(current.source, current.target);
    },
    onPointerCancel: cancel,
    onLostPointerCapture: cancel,
  });
  return { handlers, target, dragging };
}
