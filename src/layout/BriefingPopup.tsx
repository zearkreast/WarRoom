import { useEffect, useRef, useState } from "react";
import { useToasts, type Toast } from "../alerts/notify";
import { dispatchKind, type DispatchKind } from "../domain/dispatch";
import { dispatchTitle } from "../domain/format";
import { MarkupText } from "../features/news/NewsCard";
import { FactionEmblem, SealArt, SealGradient } from "../ui/emblems";
import { PLEASE_WAIT_MS, PleaseWait } from "../ui/PleaseWait";
import { KeyButton } from "../ui/primitives";
import "../ui/neonRing.css";


const VERDICT: Record<DispatchKind, { word: string; tone: string }> = {
  victory: { word: "Victory", tone: "var(--good)" },
  defeat: { word: "Failure", tone: "var(--bad)" },
  order: { word: "Major order", tone: "var(--cta)" },
  alert: { word: "Alert", tone: "var(--warn)" },
  advisory: { word: "Advisory", tone: "var(--info)" },
  news: { word: "Dispatch", tone: "var(--info)" },
};

function verdictOf(t: Toast): DispatchKind {
  if (t.kind === "capture") return "victory";
  if (t.kind === "loss") return "defeat";
  if (t.kind === "defense") return "alert";
  if (t.kind === "order") return "order";
  return dispatchKind(t.message ?? t.body);
}

const EMBERS = Array.from({ length: 14 }, (_, i) => ({ x: (i * 47 + 13) % 640, d: 3 + (i % 5), r: 1 + (i % 3) * 0.6 }));
const CLOTH = "M74 14C210 -4 330 40 470 22s130 6 150 14V206c-40-14-90-24-160-8S200 214 74 196Z";

/** A flag on a pole against a smoky sky, drawn for the app, bearing the game's Super Earth emblem. */
function Banner() {
  return (
    <svg className="bp-banner" viewBox="0 0 640 240" preserveAspectRatio="xMidYMid slice" aria-hidden>
      <defs>
        <linearGradient id="bp-sky" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor="#0a0b10" />
          <stop offset="55%" stopColor="#2a1a14" />
          <stop offset="100%" style={{ stopColor: "color-mix(in srgb, var(--bp-tone) 28%, #3a2418)" }} />
        </linearGradient>
        <linearGradient id="bp-cloth" x1="0" y1="0" x2="1" y2="0">
          <stop offset="0%" stopColor="#15224a" />
          <stop offset="22%" stopColor="#1f3264" />
          <stop offset="40%" stopColor="#121c3e" />
          <stop offset="62%" stopColor="#22376b" />
          <stop offset="82%" stopColor="#111a3a" />
          <stop offset="100%" stopColor="#1b2c5a" />
        </linearGradient>
        <SealGradient id="bp-seal" />
        <filter id="bp-smoke" x="-20%" y="-50%" width="140%" height="200%">
          <feGaussianBlur stdDeviation="18" />
        </filter>
      </defs>
      <rect width="640" height="240" fill="url(#bp-sky)" />
      <g filter="url(#bp-smoke)" opacity="0.55">
        <ellipse cx="520" cy="210" rx="160" ry="40" fill="#5a3a2a" />
        <ellipse cx="150" cy="230" rx="180" ry="30" fill="#4a2c22" />
        <ellipse cx="380" cy="60" rx="140" ry="26" fill="#1a1416" />
      </g>
      <rect x="66" y="0" width="8" height="240" fill="#c9c2b4" />
      <g className="bp-flag">
        <path d={CLOTH} fill="url(#bp-cloth)" />
        <g transform="translate(345 113)">
          <SealArt size={158} gradient="bp-seal" />
        </g>
      </g>
      {EMBERS.map((e, i) => (
        <circle
          key={i}
          className="bp-ember"
          cx={e.x}
          cy={240}
          r={e.r}
          style={{ animationDuration: `${e.d}s`, animationDelay: `${-i * 0.7}s` }}
        />
      ))}
    </svg>
  );
}

/** Opens from a toast: a short "please wait democratically" hold, then the full briefing. */
export function BriefingPopup() {
  const { briefing, closeBriefing } = useToasts();
  const ref = useRef<HTMLDialogElement>(null);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    const d = ref.current;
    if (!d) return;
    if (briefing && !d.open) d.showModal();
    if (!briefing && d.open) d.close();
    setReady(false);
    if (!briefing) return;
    const id = setTimeout(() => setReady(true), PLEASE_WAIT_MS);
    return () => clearTimeout(id);
  }, [briefing]);

  const verdict = briefing ? VERDICT[verdictOf(briefing)] : null;
  const isDispatch = briefing?.kind === "dispatch" && briefing.message != null;
  return (
    <dialog
      ref={ref}
      className={`briefing ${ready ? "" : "waiting"}`}
      aria-label={verdict?.word ?? "Briefing"}
      onClose={closeBriefing}
      onClick={(e) => e.target === e.currentTarget && closeBriefing()}
      style={verdict ? ({ "--bp-tone": verdict.tone } as React.CSSProperties) : undefined}
    >
      {briefing && verdict && !ready && (
        <PleaseWait />
      )}
      {briefing && verdict && ready && (
        <>
          <article className="bp-card">
            <Banner />
            <header className="bp-head">
              <FactionEmblem faction="Humans" size={30} color="var(--text-hi)" />
              <h2>{verdict.word}</h2>
            </header>
            <div className="bp-body">
              <h3>{isDispatch ? dispatchTitle(briefing.message!) : briefing.title}</h3>
              <MarkupText message={briefing.message ?? briefing.body} className="bp-text" whole={!isDispatch} />
            </div>
          </article>
          <div className="bp-foot">
            <KeyButton k="ESC" onClick={closeBriefing}>
              Continue
            </KeyButton>
          </div>
        </>
      )}
    </dialog>
  );
}
