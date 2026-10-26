import type { Dispatch } from "../api/types";
import { dispatchTitle } from "./format";
import { issuedAt, phaseName, type EpisodeLive } from "./snapshot";

/** A moment in a Galactic Campaign worth a news item: its launch, a new Major Order, its end. */
export interface CampaignEvent {
  key: string;
  at: number;
  type: "launched" | "order" | "ended";
  campaign: EpisodeLive;
  /** Index of the Major Order a "launched" or "order" event announces. */
  phase: number;
}

/**
 * One line of a campaign's story: one of its events, a dispatch the game sent about it, or both when
 * the dispatch is the announcement of that event.
 */
export type ThreadEntry = { at: number; key: string } & (
  | { event: CampaignEvent; dispatch?: Dispatch }
  | { dispatch: Dispatch; event?: never }
);

/** Everything about one campaign, oldest first; it sorts into the news by its latest entry. */
export interface CampaignThread {
  key: string;
  at: number;
  campaign: EpisodeLive;
  entries: ThreadEntry[];
}

const HOUR = 3_600_000;
/** Headlines the game uses for its Major Orders, which since campaigns began are always campaign phases. */
const ORDER_HEADLINE = /major order|control center|campaign/i;
const NEW_ORDER = /new major order/i;

const published = (d: Dispatch) => Date.parse(d.published);

/** Whole words only: order names are plain words ("DEPLOYMENT"), and "deployments" isn't a mention. */
function mentions(text: string, name: string): boolean {
  return new RegExp(`(^|[^A-Z0-9])${name.toUpperCase().replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}($|[^A-Z0-9])`).test(text);
}

/**
 * The campaign a dispatch belongs to: one running when it went out that it names, else the one running
 * when an order headline went out. Names alone aren't enough: campaigns reuse them (two have a
 * "CONTAINMENT"), and an unnamed order's "MAJOR ORDER" is in every order headline.
 */
export function campaignOf(d: Dispatch, list: EpisodeLive[]): EpisodeLive | null {
  const text = d.message.toUpperCase();
  const at = published(d);
  const running = list.filter((e) => at >= e.start - 2 * HOUR && at <= (e.end ?? Infinity) + 12 * HOUR);
  const named = running.find(
    (e) =>
      mentions(text, e.title) ||
      e.phases.some((p) => {
        const name = phaseName(p);
        return name != null && mentions(text, name);
      }),
  );
  if (named) return named;
  if (!ORDER_HEADLINE.test(dispatchTitle(d.message))) return null;
  return running.sort((a, b) => b.start - a.start)[0] ?? null;
}

/**
 * The campaign's events, each with the dispatch that announced it when there is one. The n-th "new
 * Major Order" dispatch announces order n; it also dates the order when the collector didn't see it go
 * out, so orders that predate the collector still get a date.
 */
function eventsOf(e: EpisodeLive, own: Dispatch[]): { event: CampaignEvent; dispatch?: Dispatch }[] {
  const announced = own.filter((d) => NEW_ORDER.test(dispatchTitle(d.message)));
  const out: { event: CampaignEvent; dispatch?: Dispatch }[] = [];
  e.phases.forEach((p, i) => {
    const dispatch = announced[i];
    const at = i === 0 ? e.start : (issuedAt(e, p.issued) ?? (dispatch ? published(dispatch) : null));
    if (at == null) return;
    const type = i === 0 ? "launched" : "order";
    out.push({ event: { key: `${e.id}-${type}-${i}`, at, type, campaign: e, phase: i }, dispatch });
  });
  if (e.status !== "active" && e.end != null)
    out.push({ event: { key: `${e.id}-ended`, at: e.end, type: "ended", campaign: e, phase: e.phases.length - 1 } });
  return out;
}

/** Campaign threads, newest activity first, and the ids of the dispatches they took in. */
export function campaignThreads(
  list: EpisodeLive[],
  dispatches: Dispatch[],
): { threads: CampaignThread[]; claimed: Set<number> } {
  const own = new Map<number, Dispatch[]>();
  const claimed = new Set<number>();
  for (const d of dispatches) {
    const e = campaignOf(d, list);
    if (!e) continue;
    own.set(e.id, [...(own.get(e.id) ?? []), d]);
    claimed.add(d.id);
  }
  const threads = list.map((e) => {
    const mine = (own.get(e.id) ?? []).sort((a, b) => published(a) - published(b));
    const events = eventsOf(e, mine);
    const told = new Set(events.flatMap((x) => (x.dispatch ? [x.dispatch.id] : [])));
    const entries: ThreadEntry[] = [
      ...events.map((x) => ({ at: x.event.at, key: x.event.key, ...x })),
      ...mine.filter((d) => !told.has(d.id)).map((dispatch) => ({ at: published(dispatch), key: `d${dispatch.id}`, dispatch })),
    ].sort((a, b) => a.at - b.at);
    return { key: `c${e.id}`, at: entries[entries.length - 1].at, campaign: e, entries };
  });
  return { threads: threads.sort((a, b) => b.at - a.at), claimed };
}

/** Every dated campaign event, newest first. Orders with no known or announced date are left out. */
export function campaignEvents(list: EpisodeLive[], dispatches: Dispatch[] = []): CampaignEvent[] {
  return campaignThreads(list, dispatches)
    .threads.flatMap((t) => t.entries.flatMap((x) => (x.event ? [x.event] : [])))
    .sort((a, b) => b.at - a.at);
}

/**
 * Campaign launches and outcomes not alerted on yet, oldest first, and the keys to remember as seen. New
 * orders are left to the Major Order alert, which comes sooner. With nothing seen yet, nothing is fresh.
 */
export function campaignAlerts(list: EpisodeLive[], seen: string[]): { fresh: CampaignEvent[]; keys: string[] } {
  const events = campaignEvents(list).filter((ev) => ev.type !== "order");
  const fresh = seen.length === 0 ? [] : events.filter((ev) => !seen.includes(ev.key)).reverse();
  return { fresh, keys: events.map((ev) => ev.key) };
}

/** One-line headline for a campaign event, in the game's words. */
export function campaignHeadline(ev: CampaignEvent): string {
  const { campaign: e, phase } = ev;
  if (ev.type === "launched") return `New campaign: ${e.title}`;
  if (ev.type === "order") {
    const name = phaseName(e.phases[phase]);
    return `${e.title}: Major Order ${phase + 1}${name ? `, ${name}` : ""}`;
  }
  return `${e.title} ${e.status === "won" ? "won" : "failed"}`;
}
