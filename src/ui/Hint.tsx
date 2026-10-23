import { Lightbulb, X } from "lucide-react";
import type { ReactNode } from "react";
import { useSettings } from "../state";

/** One-time hint for a screen; dismissed hints stay hidden (Settings › Reset unread shows them again). */
export function Hint({ id, children }: { id: string; children: ReactNode }) {
  const seen = useSettings((s) => s.hintsSeen.includes(id));
  const dismiss = useSettings((s) => s.dismissHint);
  if (seen) return null;
  return (
    <div className="hint" role="note">
      <Lightbulb size={15} />
      <div className="hint-text">{children}</div>
      <button className="toast-x" aria-label="Dismiss hint" onClick={() => dismiss(id)}>
        <X size={15} />
      </button>
    </div>
  );
}
