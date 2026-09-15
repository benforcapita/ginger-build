import { useEffect, useRef } from "react";
import { Channel, invoke } from "@tauri-apps/api/core";
import { Terminal as XTerm } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { useSessionStore } from "@/stores/session-store";
import "@xterm/xterm/css/xterm.css";

type TerminalEvent = { type: "output"; data: number[] } | { type: "exit"; code: number | null };
export function TerminalView({ id, active }: { id: number; active: boolean }) {
  const container = useRef<HTMLDivElement>(null);
  const fitRef = useRef<() => void>(() => {});
  const termRef = useRef<XTerm | null>(null);
  useEffect(() => {
    const element = container.current;
    if (!element) return;
    let disposed = false;
    const terminal = new XTerm({
      cursorBlink: true, fontSize: 12, lineHeight: 1.25,
      fontFamily: '"SFMono-Regular", Menlo, Consolas, monospace',
      scrollback: 5000, allowProposedApi: false,
      theme: { background: "#0d0f0f", foreground: "#c9c5b9", cursor: "#df8d4d", selectionBackground: "#df8d4d35", black: "#151818", red: "#d6756b", green: "#a3b977", yellow: "#d5bd78", blue: "#86a6b2", magenta: "#b297bb", cyan: "#81b1a0", white: "#d9d6cb", brightBlack: "#70756e" },
    });
    const fit = new FitAddon();
    terminal.loadAddon(fit);
    terminal.open(element);
    termRef.current = terminal;
    terminal.attachCustomKeyEventHandler((event) => !(event.metaKey && ["p", "o", "k"].includes(event.key.toLowerCase())) && !(event.metaKey && event.key.toLowerCase() === "s"));
    const report = (error: unknown) => { if (!disposed) useSessionStore.getState().setError(String(error)); };
    let inputQueue = Promise.resolve();
    const input = terminal.onData((data) => {
      inputQueue = inputQueue.then(() => invoke<void>("terminal_write", { id, data: Array.from(new TextEncoder().encode(data)) })).catch(report);
    });
    const resize = () => {
      if (disposed || element.clientWidth === 0 || element.clientHeight === 0) return;
      fit.fit();
      void invoke("terminal_resize", { id, rows: terminal.rows, cols: terminal.cols }).catch(report);
    };
    fitRef.current = resize;
    const observer = new ResizeObserver(resize);
    observer.observe(element);
    const channel = new Channel<TerminalEvent>();
    channel.onmessage = (event) => {
      if (disposed) return;
      if (event.type === "output") terminal.write(new Uint8Array(event.data));
      else {
        useSessionStore.getState().markExited(id, event.code);
        terminal.write(`\r\n\x1b[90m[process exited${event.code === null ? "" : ` · code ${event.code}`} ]\x1b[0m\r\n`);
      }
    };
    void invoke("terminal_subscribe", { id, onEvent: channel }).catch(report);
    resize();
    return () => { disposed = true; observer.disconnect(); input.dispose(); terminal.dispose(); termRef.current = null; };
  }, [id]);
  useEffect(() => {
    if (active) {
      const frame = requestAnimationFrame(() => { fitRef.current(); termRef.current?.focus(); });
      return () => cancelAnimationFrame(frame);
    }
  }, [active]);
  return <div ref={container} className="terminal-view" hidden={!active} aria-label={`Terminal session ${id}`} />;
}
