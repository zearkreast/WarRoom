import { useCampaigns, useLastUpdate, usePlanets, useWar } from "../api/queries";
import { fmtAgo, fmtDateTime } from "../domain/format";
import { LiveDot, StatusChip, useNow } from "./primitives";

/** The collector writes campaigns every minute; quiet for this long means it has stopped. */
export const STALE_MS = 10 * 60_000;

export type FeedState = "live" | "stale" | "offline";

/** How fresh the war data is: shared by the status strip, the planet detail and page notes. */
export function useFeedStatus(tickMs = 1000) {
  const planets = usePlanets();
  const war = useWar();
  const campaigns = useCampaigns();
  const lastUpdate = useLastUpdate().data;
  const now = useNow(tickMs);
  const offline = [planets, war, campaigns].some((q) => q.isError);
  const state: FeedState = offline ? "offline" : lastUpdate != null && now - lastUpdate > STALE_MS ? "stale" : "live";
  return { state, lastUpdate, now };
}

/** Live dot with the data's age, or the STALE / LINK LOST chip; never claims "live" for old data. */
export function FeedTag() {
  const { state, lastUpdate, now } = useFeedStatus(15_000);
  if (state === "offline") return <StatusChip tone="var(--bad)">Link lost</StatusChip>;
  if (state === "stale")
    return (
      <>
        <StatusChip tone="var(--warn)">Stale</StatusChip>
        <span className="num">Data from {fmtAgo(lastUpdate!, now)}</span>
      </>
    );
  return (
    <>
      <LiveDot /> Live
      {lastUpdate != null && <span className="num">· updated {fmtAgo(lastUpdate, now)}</span>}
    </>
  );
}

/** One line above a page's data when it is stale or offline; nothing while live. */
export function FeedNote() {
  const { state, lastUpdate, now } = useFeedStatus(60_000);
  if (state === "live") return null;
  return (
    <p className={`feed-note caps ${state}`} role="status">
      {state === "offline"
        ? "Comms link down: showing the last data received."
        : `Showing data from ${fmtDateTime(lastUpdate!)} (${fmtAgo(lastUpdate!, now)}): projections may be off.`}
    </p>
  );
}
