import { BookOpen, Info, ScrollText } from "lucide-react";
import { useEffect } from "react";
import { APP_VERSION } from "../config";
import { CHANGELOG } from "../changelog";
import { useSettings, useUi } from "../state";

/** Companion Dispatch: what's new in the app, plus links to the help pages. */
export function DispatchPanel() {
  const { dispatchOpen, toggleDispatch, openModule } = useUi();
  const markChangelog = useSettings((s) => s.markChangelog);
  useEffect(() => {
    if (dispatchOpen) markChangelog(APP_VERSION);
  }, [dispatchOpen, markChangelog]);
  useEffect(() => {
    if (!dispatchOpen) return;
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && toggleDispatch(false);
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [dispatchOpen, toggleDispatch]);
  if (!dispatchOpen) return null;
  const go = (m: "about" | "legend") => {
    toggleDispatch(false);
    openModule(m);
  };
  return (
    <aside className="dispatch-panel panel glass" aria-label="Companion dispatch">
      <div className="dp-links">
        <button className="dp-link" onClick={() => go("legend")}>
          <BookOpen size={16} />
          <span>
            <b>Icons legend</b>
            What every mark on the map and cards means
          </span>
        </button>
        <button className="dp-link" onClick={() => go("about")}>
          <Info size={16} />
          <span>
            <b>About</b>
            Data sources, history and alerts
          </span>
        </button>
      </div>
      <div className="dp-head">
        <ScrollText size={14} />
        <span className="caps">Companion dispatch</span>
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
