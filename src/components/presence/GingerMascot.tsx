import { useEffect, useState } from "react";
import { usePresenceStore } from "@/stores/presence-store";
import "./ginger-mascot.css";

const ASCII_GINGER = `
     🫚
    /|\\
   / | \\
  /  |  \\
 /___|___\\
 |  🫚  |
 |_____|
`;

export function GingerMascot() {
  const state = usePresenceStore((s) => s.state);
  const message = usePresenceStore((s) => s.message);
  const refreshMessage = usePresenceStore((s) => s.refreshMessage);
  const [showMessage, setShowMessage] = useState(false);

  useEffect(() => {
    refreshMessage();
  }, [state, refreshMessage]);

  useEffect(() => {
    if (message) {
      setShowMessage(true);
      const timer = setTimeout(() => setShowMessage(false), 5000);
      return () => clearTimeout(timer);
    }
  }, [message]);

  const stateClass = `ginger-mascot ginger-state-${state}`;

  return (
    <div className={stateClass}>
      <pre className="ginger-ascii">{ASCII_GINGER}</pre>
      {showMessage && message && (
        <div className={`ginger-bubble ginger-bubble-${message.tier}`}>
          {message.text}
        </div>
      )}
    </div>
  );
}