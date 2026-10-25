import { ScrollText } from "lucide-react";
import { useEffect } from "react";
import { APP_VERSION } from "../config";
import { CHANGELOG } from "../changelog";
import { useSettings, useUi } from "../state";
import { useEscape } from "../ui/primitives";

/** Tracker Dispatch: what's new in the app. */
export function DispatchPanel() {
  const { dispatchOpen, toggleDispatch } = useUi();
  const markChangelog = useSettings((s) => s.markChangelog);
  useEffect(() => {
    if (dispatchOpen) markChangelog(APP_VERSION);
  }, [dispatchOpen, markChangelog]);
  useEscape(() => toggleDispatch(false), dispatchOpen);
  if (!dispatchOpen) return null;
  return (
    <aside className="dispatch-panel panel glass" aria-label="Tracker dispatch">
      <div className="dp-head">
        <ScrollText size={14} />
        <span className="caps">Tracker dispatch</span>
        <span className="dp-ver num">v{APP_VERSION}</span>
      </div>
      <div className="dp-log scroll">
        {CHANGELOG.map((c) => (
          <section key={c.version} className="dp-entry">
            <header>
              <b className="caps">Update log</b>
              <span className="num">
                v{c.version} · {c.date}
              </span>
            </header>
            <ul>
              {c.items.map((i) => (
                <li key={i}>{i}</li>
              ))}
            </ul>
          </section>
        ))}
      </div>
    </aside>
  );
}
