import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface EditorStatus {
  alive: boolean;
  runtime_path: string;
  safe_mode: boolean;
}

export function Editor() {
  const [status, setStatus] = useState<EditorStatus | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    invoke<EditorStatus>("editor_status")
      .then(setStatus)
      .catch(() => setStatus(null));
  }, []);

  const startEditor = async () => {
    setLoading(true);
    try {
      await invoke("editor_start");
      const s = await invoke<EditorStatus>("editor_status");
      setStatus(s);
    } catch (e) {
      console.error("Failed to start editor:", e);
    } finally {
      setLoading(false);
    }
  };

  if (!status || !status.alive) {
    return (
      <div className="editor">
        <div className="editor-placeholder">
          <p>Neovim not connected</p>
          <button
            className="editor-start-btn"
            onClick={startEditor}
            disabled={loading}
          >
            {loading ? "Starting..." : "Start Neovim"}
          </button>
          <p className="editor-hint">Or open a folder to begin</p>
        </div>
      </div>
    );
  }

  return (
    <div className="editor">
      <div className="editor-active">
        <p>Neovim connected</p>
        <p className="editor-hint">Runtime: {status.runtime_path}</p>
        {status.safe_mode && (
          <p className="editor-warning">⚠️ Safe Mode — user Lua disabled</p>
        )}
      </div>
    </div>
  );
}