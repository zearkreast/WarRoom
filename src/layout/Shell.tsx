import {
  BookOpen,
  Flag,
  FlagOff,
  Info,
  LayoutGrid,
  Newspaper,
  Orbit,
  Radio,
  Settings,
  ShieldAlert,
  ShieldHalf,
  TabletSmartphone,
  Target,
  X,
  type LucideIcon,
} from "lucide-react";
import { useEffect, useMemo, useState, type ReactNode } from "react";
import { useCampaigns, usePlanets, useWar } from "../api/queries";
import { useToasts, type ToastKind } from "../alerts/notify";
import { APP_VERSION } from "../config";
import { humanControl } from "../domain/galaxy";
import { fmtDuration, fmtInt, fmtPct, sestClock, sestDate, warDay } from "../domain/format";
import { liberationPercent } from "../domain/progress";
import { NewsTicker } from "../features/news/NewsTicker";
import { PlanetDetail } from "../features/planet/PlanetDetail";
import { PlanetOptions } from "../features/planet/PlanetOptions";
import { AboutDialog } from "./AboutDialog";
import { DispatchPanel } from "./DispatchPanel";
import { useSettings, useUi, type View } from "../state";
import { AppMark } from "../ui/emblems";
import { ErrorBoundary } from "../ui/ErrorBoundary";
import { useFeedStatus } from "../ui/feed";
import { Keycap, StatusChip } from "../ui/primitives";
import "./shell.css";

export function Logo() {
  return (
    <div className="logo">
      <AppMark />
      <div>
        <div className="logo-name">WarCommand</div>
        <div className="logo-sub">Galactic War Tracker</div>
      </div>
    </div>
  );
}

function TopButton(props: { view: View; icon: LucideIcon; small: string; big: string }) {
  const { view, setView } = useUi();
  const on = view === props.view;
  return (
    <button
      className={`top-btn ${on ? "on" : ""}`}
      onClick={() => setView(on ? "campaigns" : props.view)}
      aria-pressed={on}
    >
      <span className="top-btn-icon">
        <props.icon size={18} strokeWidth={1.75} />
      </span>
      <span className="top-btn-text">
        <span className="top-btn-small">{props.small}</span>
        <span className="top-btn-big">{props.big}</span>
      </span>
    </button>
  );
}

/** Rotating one-line war status. */
function useStatusLines(): ReactNode[] {
  const planets = usePlanets().data;
  const campaigns = useCampaigns().data;
  const war = useWar().data;
  return useMemo(() => {
    const lines: ReactNode[] = [];
    if (planets)
      lines.push(
        <>
          Super Earth holds <b className="hi">{fmtPct(humanControl(planets), 1)}</b> of the galaxy
        </>,
      );
    if (campaigns) {
      lines.push(
        <>
          <b className="hi">{campaigns.length}</b> active planets on the front
        </>,
      );
      const top = [...campaigns].sort((a, b) => b.planet.statistics.playerCount - a.planet.statistics.playerCount)[0];
      if (top)
        lines.push(
          <>
            {top.planet.event ? "Defense" : "Liberation"} at <b className="hi">{top.planet.name}</b>{" "}
            <b className="hi">{fmtPct(liberationPercent(top.planet), 1)}</b>
          </>,
        );
    }
    if (war)
      lines.push(
        <>
          <b className="hi">{fmtInt(war.statistics.playerCount)}</b> Helldivers deployed
        </>,
      );
    return lines;
  }, [planets, campaigns, war]);
}

function StatusStrip() {
  const war = useWar();
  const { state, lastUpdate, now } = useFeedStatus();
  const offline = state === "offline";
  const stale = state === "stale";
  const lines = useStatusLines();
  const [i, setI] = useState(0);
  useEffect(() => {
    const id = setInterval(() => setI((n) => n + 1), 8000);
    return () => clearInterval(id);
  }, []);
  return (
    <div className="strip">
      <div className="strip-status">
        {offline ? (
          <StatusChip tone="var(--bad)">Link lost</StatusChip>
        ) : stale ? (
          <StatusChip tone="var(--warn)">Stale</StatusChip>
        ) : (
          <StatusChip>Front</StatusChip>
        )}
        <span className="strip-line caps" key={stale ? "stale" : i % Math.max(1, lines.length)}>
          {offline
            ? "Comms link down, showing last known data"
            : stale
              ? `No new war data for ${fmtDuration(now - lastUpdate!)}; the collector may be down`
              : lines[i % Math.max(1, lines.length)]}
        </span>
      </div>
      <NewsTicker />
      <div className="strip-clock num" title="Super Earth Standard Time (UTC)">
        <span className="caps">SEST</span> {sestClock(now)} <span className="strip-date">{sestDate(now)}</span>
      </div>
      {war.data && <div className="strip-day caps num">Day {warDay(war.data.started, now)}</div>}
    </div>
  );
}

const TABS: { view: View; label: string; icon: LucideIcon; key: string }[] = [
  { view: "map", label: "Galaxy map", icon: Orbit, key: "1" },
  { view: "campaigns", label: "Active planets", icon: LayoutGrid, key: "2" },
  { view: "hellpad", label: "Hellpad", icon: TabletSmartphone, key: "3" },
];

function Dock() {
  const { view, setView, showDialog, toggleDispatch, dispatchOpen } = useUi();
  const seen = useSettings((s) => s.seenChangelog);
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.ctrlKey || e.metaKey || e.altKey || (e.target as HTMLElement)?.closest("input, textarea, select")) return;
      const tab = TABS.find((t) => t.key === e.key);
      if (tab) setView(tab.view);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [setView]);
  return (
    <nav className="dock" aria-label="Main">
      <div className="dock-side">
        <button
          className={`hud-btn icon ${view === "settings" ? "on" : ""}`}
          aria-label="Settings"
          title="Settings"
          onClick={() => setView(view === "settings" ? "campaigns" : "settings")}
        >
          <Settings size={18} strokeWidth={1.75} />
        </button>
        <button className="hud-btn icon" aria-label="About" title="About" onClick={() => showDialog("about")}>
          <Info size={18} strokeWidth={1.75} />
        </button>
        <button
          className="hud-btn icon"
          aria-label="Icons legend"
          title="Icons legend"
          onClick={() => showDialog("legend")}
        >
          <BookOpen size={18} strokeWidth={1.75} />
        </button>
      </div>
      <div className="dock-tabs" role="tablist">
        {TABS.map((t) => (
          <button
            key={t.view}
            role="tab"
            aria-selected={view === t.view}
            className={`dock-tab ${view === t.view ? "on" : ""}`}
            onClick={() => setView(t.view)}
            title={`${t.label} (${t.key})`}
          >
            <t.icon size={22} strokeWidth={1.6} />
            <span className="dock-label">{t.label}</span>
            <span className="dock-key">
              <Keycap k={t.key} />
            </span>
          </button>
        ))}
      </div>
      <div className="dock-side right">
        <button
          className={`dock-dispatch ${dispatchOpen ? "on" : ""}`}
          onClick={() => toggleDispatch()}
          aria-expanded={dispatchOpen}
        >
          <span>Tracker</span>
          <b>Dispatch</b>
          {seen !== APP_VERSION && <span className="unread" aria-label="Unread" />}
        </button>
      </div>
    </nav>
  );
}

const TOAST_LOOK: Record<ToastKind, { tone: string; icon: LucideIcon }> = {
  order: { tone: "var(--cta)", icon: Target },
  dispatch: { tone: "var(--info)", icon: Radio },
  capture: { tone: "var(--good)", icon: Flag },
  loss: { tone: "var(--bad)", icon: FlagOff },
  defense: { tone: "var(--warn)", icon: ShieldAlert },
};

function Toasts() {
  const { toasts, dismiss } = useToasts();
  const popups = useSettings((s) => s.popups);
  const shown = popups === "none" ? [] : popups === "priority" ? toasts.filter((t) => t.kind !== "dispatch") : toasts;
  return (
    <div className="toasts" aria-live="polite">
      {shown.map((t) => {
        const look = TOAST_LOOK[t.kind];
        return (
          <div key={t.id} className="toast" style={{ "--tone": look.tone } as React.CSSProperties}>
            <span className="toast-icon">
              <look.icon size={16} />
            </span>
            <div className="toast-body">
              <b>{t.title}</b> {t.body}
            </div>
            <button className="toast-x" aria-label="Dismiss" onClick={() => dismiss(t.id)}>
              <X size={16} />
            </button>
          </div>
        );
      })}
    </div>
  );
}

export function Shell({ children }: { children: ReactNode }) {
  const view = useUi((s) => s.view);
  const detail = useUi((s) => s.detail);
  return (
    <div className="shell">
      <header className="topbar">
        <Logo />
        <div className="top-actions">
          <TopButton view="news" icon={Newspaper} small="Strohmann" big="News" />
          <TopButton view="overview" icon={ShieldHalf} small="High Command" big="Overview" />
        </div>
      </header>
      <StatusStrip />
      <main className="main">
        <ErrorBoundary key={view} label="This screen">
          {children}
        </ErrorBoundary>
      </main>
      <Dock />
      <DispatchPanel />
      <AboutDialog />
      <PlanetOptions />
      <ErrorBoundary key={detail ?? -1} label="Planet details">
        <PlanetDetail />
      </ErrorBoundary>
      <Toasts />
    </div>
  );
}
