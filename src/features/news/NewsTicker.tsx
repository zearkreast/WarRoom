import { Flag, Radio } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { useDispatches, useEpisodes } from "../../api/queries";
import { campaignEvents } from "../../domain/campaignNews";
import { fmtAgeShort } from "../../domain/format";
import { tickerItems, type TickerItem } from "../../domain/ticker";
import { useHistory } from "../../history/useHistory";
import { useUi } from "../../state";
import { FactionEmblem } from "../../ui/emblems";
import { useNow } from "../../ui/primitives";

/** Scroll speed, so a long strip doesn't race and a short one doesn't crawl. */
const PX_PER_S = 45;
const STEP_MS = 8000;

function Glyph({ item }: { item: TickerItem }) {
  if (item.kind === "dispatch") return <Radio size={12} aria-hidden />;
  if (item.kind === "campaign" && !item.faction) return <Flag size={12} aria-hidden />;
  return <FactionEmblem faction={item.faction ?? "Humans"} size={12} />;
}

/** The last day's war news scrolling in the status strip; it pauses under the pointer and each item opens its source. */
export function NewsTicker() {
  const dispatches = useDispatches().data;
  const episodes = useEpisodes().data;
  const changes = useHistory((s) => s.changes);
  const { setView, showOnMap } = useUi();
  const now = useNow(60_000);
  const items = useMemo(
    () => tickerItems(dispatches ?? [], changes, campaignEvents(episodes ?? [], dispatches ?? []), now),
    [dispatches, changes, episodes, now],
  );
  const track = useRef<HTMLDivElement>(null);
  const held = useRef(false);
  const reduced = useMemo(() => window.matchMedia("(prefers-reduced-motion: reduce)").matches, []);
  const [step, setStep] = useState(0);

  // Scrolled by offset rather than a CSS animation: the strip's width changes every minute as ages
  // tick over, and re-timing an animation mid-run made it jump.
  useEffect(() => {
    if (reduced) return;
    let offset = 0;
    let last = performance.now();
    let raf = requestAnimationFrame(function tick(t) {
      const dt = Math.min((t - last) / 1000, 0.1);
      last = t;
      const el = track.current;
      if (el && !held.current && !document.hidden) {
        const loop = el.scrollWidth / 2;
        offset = loop > 0 ? (offset + PX_PER_S * dt) % loop : 0;
        el.style.transform = `translateX(${-offset.toFixed(1)}px)`;
      }
      raf = requestAnimationFrame(tick);
    });
    return () => cancelAnimationFrame(raf);
  }, [reduced]);
  useEffect(() => {
    if (!reduced) return;
    const id = setInterval(() => setStep((n) => n + 1), STEP_MS);
    return () => clearInterval(id);
  }, [reduced]);

  if (!items.length) return <div className="ticker" />;
  const open = (i: TickerItem) => {
    if (i.planet != null) showOnMap(i.planet);
    else setView(i.kind === "campaign" ? "overview" : "news");
  };
  const item = (i: TickerItem, copy: boolean) => (
    <button
      key={`${copy ? "b" : "a"}-${i.key}`}
      className={`ticker-item ${i.kind}`}
      onClick={() => open(i)}
      tabIndex={copy ? -1 : undefined}
      aria-hidden={copy || undefined}
    >
      <Glyph item={i} />
      <span className="ticker-text">{i.text}</span>
      <span className="ticker-age num">{fmtAgeShort(i.at, now)}</span>
    </button>
  );

  if (reduced)
    return (
      <div className="ticker still" aria-live="polite">
        {item(items[step % items.length], false)}
      </div>
    );
  return (
    <div
      className="ticker"
      aria-label="Latest war news"
      onPointerEnter={() => (held.current = true)}
      onPointerLeave={() => (held.current = false)}
      onFocus={() => (held.current = true)}
      onBlur={() => (held.current = false)}
    >
      <div ref={track} className="ticker-track caps">
        {items.map((i) => item(i, false))}
        {items.map((i) => item(i, true))}
      </div>
    </div>
  );
}
