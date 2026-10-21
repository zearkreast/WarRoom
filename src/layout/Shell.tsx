import { useCampaigns, usePlanets, useWar } from "../api/queries";
import { useToasts } from "../alerts/notify";
import { humanControl } from "../domain/galaxy";
import { fmtInt, fmtPct } from "../domain/format";
import { NewsTicker } from "../features/news/NewsTicker";
import { useUi, type View } from "../state";
import "./shell.css";

const NAV: { view: View; label: string; icon: string }[] = [
  { view: "map", label: "War Map", icon: "◎" },
  { view: "fronts", label: "Fronts", icon: "⚔" },
  { view: "orders", label: "Orders", icon: "★" },
  { view: "news", label: "News", icon: "✉" },
  { view: "stats", label: "Stats", icon: "▤" },
  { view: "history", label: "History", icon: "⟲" },
];

export function Logo() {
  return (
    <div className="logo">
      <svg viewBox="0 0 32 32" width="26" height="26" aria-hidden>
        <path d="M16 2 29 8v8c0 7-5.5 12-13 14C8.5 28 3 23 3 16V8z" fill="none" stroke="var(--accent)" strokeWidth="2" />
        <path d="m16 9 2.2 4.6 5 .6-3.7 3.4 1 5-4.5-2.5-4.5 2.5 1-5-3.7-3.4 5-.6z" fill="var(--accent)" />
      </svg>
      <div>
        <div className="logo-name">WARCOMMAND</div>
        <div className="logo-sub">Galactic War Tracker</div>
      </div>
    </div>
  );
}

function FrontBar() {
  const planets = usePlanets();
  const war = useWar();
  const campaigns = useCampaigns();
  const control = planets.data ? humanControl(planets.data) : null;
  const offline = [planets, war, campaigns].some((q) => q.isError);
  return (
    <div className="frontbar">
      {offline ? (
        <span className="chip" style={{ color: "var(--bad)" }} title="The API is unreachable. Showing the last data received.">
          Link lost · retrying
        </span>
      ) : (
        <span className="chip" style={{ color: "var(--good)" }}>Front</span>
      )}
      <span className="caps frontbar-text">
        Super Earth holds <b className="num">{control == null ? "—" : fmtPct(control, 1)}</b> of the galaxy
      </span>
      <span className="frontbar-meta caps muted">
        {campaigns.data && <span className="num">{campaigns.data.length} active fronts</span>}
        {war.data && <span className="num">{fmtInt(war.data.statistics.playerCount)} divers</span>}
      </span>
      <NewsTicker />
    </div>
  );
}

function Toasts() {
  const { toasts, dismiss } = useToasts();
  const color: Record<string, string> = {
    order: "var(--accent)",
    dispatch: "var(--f-illuminate)",
    capture: "var(--good)",
    loss: "var(--bad)",
    defense: "var(--f-terminids)",
  };
  return (
    <div className="toasts">
      {toasts.map((t) => (
        <div key={t.id} className="panel toast" style={{ borderLeftColor: color[t.kind] }}>
          <div>
            <div className="t-title" style={{ color: color[t.kind] }}>
              {t.title}
            </div>
            <div className="t-body">{t.body}</div>
          </div>
          <button aria-label="Dismiss" onClick={() => dismiss(t.id)}>
            ×
          </button>
        </div>
      ))}
    </div>
  );
}

export function Shell({ children }: { children: React.ReactNode }) {
  const { view, setView } = useUi();
  const nav = (cls: string) => (
    <nav className={cls}>
      {NAV.map((n) => (
        <button key={n.view} className={view === n.view ? "active" : ""} onClick={() => setView(n.view)}>
          <span className="nav-icon">{n.icon}</span>
          <span className="nav-label">{n.label}</span>
        </button>
      ))}
    </nav>
  );
  return (
    <div className="shell">
      <header className="topbar">
        <Logo />
        {nav("topnav")}
        <div className="topbar-right muted caps">Fan-made · not affiliated with Arrowhead</div>
      </header>
      <FrontBar />
      <main className="main">{children}</main>
      {nav("bottomnav")}
      <Toasts />
    </div>
  );
}
