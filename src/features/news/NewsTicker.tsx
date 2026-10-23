import { Flag, Radio } from "lucide-react";
import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
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
    () => tickerItems(dispatches ?? [], changes, campaignEvents(episodes ?? []), now),
    [dispatches, changes, episodes, now],
  );
  const track = useRef<HTMLDivElement>(null);
  const [duration, setDuration] = useState(60);
  const reduced = useMemo(() => window.matchMedia("(prefers-reduced-motion: reduce)").matches, []);
  const [step, setStep] = useState(0);

  useLayoutEffect(() => {
    const el = track.current;
    if (el) setDuration(Math.max(20, el.scrollWidth / 2 / PX_PER_S));
  }, [items]);
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
    <div className="ticker" aria-label="Latest war news">
      <div ref={track} className="ticker-track caps" style={{ animationDuration: `${duration}s` }}>
        {items.map((i) => item(i, false))}
        {items.map((i) => item(i, true))}
      </div>
    </div>
  );
}
