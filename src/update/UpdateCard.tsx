import { Download, RotateCw, X } from "lucide-react";
import { APP_VERSION } from "../config";
import { CtaButton, Meter } from "../ui/primitives";
import { checkForUpdate, installUpdate, isAndroid, useUpdate } from "./appUpdate";
import "./update.css";

/** The first few lines of the release notes, without headings or Markdown marks. */
const summary = (notes: string) =>
  notes
    .split("\n")
    .filter((l) => !/^\s*#/.test(l))
    .map((l) =>
      l
        .replace(/^[>*\-\s]+/, "")
        .replace(/\*\*/g, "")
        .trim(),
    )
    .filter(Boolean)
    .slice(0, 3)
    .join(" · ");

/** A newer release, offered at the top of the toast column until it is installed or put off. */
export function UpdateCard() {
  const { state, dismissed, dismiss } = useUpdate();
  if (dismissed) return null;
  if (!["available", "downloading", "installing", "handed-off", "error"].includes(state.phase)) return null;
  const version = "version" in state ? state.version : undefined;
  const busy = state.phase === "downloading" || state.phase === "installing";
  return (
    <section className="update-card" aria-live="polite" aria-label="App update">
      <div className="uc-head">
        <span className="uc-icon">
          <Download size={16} />
        </span>
        <div className="uc-title">
          <b className="caps">{state.phase === "error" ? "Update failed" : "Update available"}</b>
          <span className="num caps">
            {version ? `v${version} · ` : ""}you have v{APP_VERSION}
          </span>
        </div>
        {!busy && (
          <button className="toast-x" aria-label="Remind me next launch" title="Later" onClick={dismiss}>
            <X size={16} />
          </button>
        )}
      </div>
      {state.phase === "available" && state.notes && <p className="uc-notes">{summary(state.notes)}</p>}
      {state.phase === "available" && (
        <div className="uc-actions">
          <CtaButton icon={Download} onClick={() => void installUpdate()}>
            {isAndroid() ? "Download APK" : "Download and install"}
          </CtaButton>
        </div>
      )}
      {state.phase === "downloading" && (
        <div className="uc-progress">
          <Meter value={(state.progress ?? 0) * 100} color="var(--cta)" size="m" />
          <span className="num caps">
            {state.progress == null ? "Downloading…" : `Downloading ${Math.round(state.progress * 100)}%`}
          </span>
        </div>
      )}
      {state.phase === "installing" && <p className="uc-notes">Installing. The app restarts when it is done.</p>}
      {state.phase === "handed-off" && (
        <p className="uc-notes">
          The APK is downloading. Open it from your notifications to install; the first time, Android asks you to allow
          installs from that app.
        </p>
      )}
      {state.phase === "error" && (
        <>
          <p className="uc-notes">{state.message}</p>
          <div className="uc-actions">
            <CtaButton icon={RotateCw} onClick={() => void (state.version ? installUpdate() : checkForUpdate(true))}>
              Try again
            </CtaButton>
          </div>
        </>
      )}
    </section>
  );
}
