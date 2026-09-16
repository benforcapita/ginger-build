import { useEffect, useState } from "react";
import { useSessionStore } from "@/stores/session-store";
import { useWorkspaceStore } from "@/stores/workspace-store";
import "./ginger-mascot.css";
import { useWorkbenchStore } from "@/stores/workbench-store";
export function GingerMascot({ onMinimize }: { onMinimize?: () => void }) {
  const sessions = useSessionStore((s) => s.sessions);
  const workspace = useWorkspaceStore((s) => s.status.workspace);
  const error = useSessionStore((s) => s.error);
  const pets = useWorkbenchStore((s) => s.pets);
  const pet = useWorkbenchStore((s) => s.pet);
  const [petted, setPetted] = useState(false);
  const quiet = useWorkbenchStore((s) => s.quiet);
  const agents = sessions.filter((s) => s.kind === "agent" && !s.exited).length;
  const editing = sessions.some((s) => s.kind === "editor" && !s.exited);
  const state = error ? "heads up" : petted ? "feeling appreciated" : agents ? "keeping you company" : editing ? "in the zone" : "standing by";
  useEffect(() => { if (pets) { setPetted(true); const timer = setTimeout(() => setPetted(false), 2400); return () => clearTimeout(timer); } }, [pets]);
  const message = petted ? ["Fine. One more head pat.", "Morale improved. Code still needs saving.", "I accept payment in head pats."][pets % 3] : error ? "Something needs your attention. Check the message above." : agents ? "Your harness is running. I’ll keep you company while you work." : editing ? "You write the code. I’ll look thoughtfully at the semicolons." : workspace ? "Folder’s open. Pick a file and let’s make something." : "A terminal. A good idea. Questionable amounts of coffee.";
  return <div className={`ginger-companion ${petted ? "petted" : ""}`}>
    <div className="ginger-intro"><span className="version-tag">GINGER / 0.1</span><span className="presence-label"><i />{state}</span></div>
    <button className="portrait-button" onClick={pet} aria-label="Pet Ginger" title="Pet Ginger">
      <span className="ginger-portrait" aria-hidden="true"><span className="ginger-sprite" /></span>
      {petted && <span className="pet-heart" aria-hidden="true">♡</span>}
    </button>
    <div className="ginger-name"><h1>ginger<span>_</span></h1><span>your code companion</span></div>
    {!quiet && <div className="ginger-speech" aria-live="polite"><span className="accent">›</span><p>{message}</p></div>}
    <div className="companion-controls"><span>Witty. Focused. Slightly sarcastic.</span>{onMinimize && <button onClick={onMinimize}>minimize</button>}<button onClick={() => useWorkbenchStore.setState({ quiet: !quiet })} aria-pressed={quiet} title="Toggle Ginger commentary">{quiet ? "unmute" : "quiet"}</button></div>
  </div>;
}
