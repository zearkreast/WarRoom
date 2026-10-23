import type { Dispatch } from "../api/types";
import { campaignHeadline, type CampaignEvent } from "./campaignNews";
import { dispatchTitle } from "./format";
import type { OwnerChange } from "./history";

export interface TickerItem {
  key: string;
  at: number;
  kind: "dispatch" | "liberated" | "lost" | "campaign";
  text: string;
  /** Faction whose emblem leads the item. */
  faction?: string;
  /** Planet the item opens on the map. */
  planet?: number;
}

const WINDOW_MS = 24 * 3_600_000;
const MIN_ITEMS = 6;
const MAX_ITEMS = 14;

/**
 * What the status strip scrolls: the last day's dispatches, captures, losses and campaign news,
 * newest first. A quiet day is topped up with older items so the strip never runs empty.
 */
export function tickerItems(
  dispatches: Dispatch[],
  changes: OwnerChange[],
  campaign: CampaignEvent[],
  now: number,
): TickerItem[] {
  const all: TickerItem[] = [
    ...dispatches.map((d) => ({
      key: `d${d.id}`,
      at: Date.parse(d.published),
      kind: "dispatch" as const,
      text: dispatchTitle(d.message),
    })),
    ...changes.map((c) => {
      const won = c.to === "Humans";
      return {
        key: `c${c.planet}-${c.at}`,
        at: c.at,
        kind: won ? ("liberated" as const) : ("lost" as const),
        text: won ? `${c.planetName} liberated` : `${c.planetName} lost`,
        faction: won ? "Humans" : c.to,
        planet: c.planet,
      };
    }),
    ...campaign.map((ev) => ({
      key: ev.key,
      at: ev.at,
      kind: "campaign" as const,
      text: campaignHeadline(ev),
      faction: ev.campaign.faction,
    })),
  ].sort((a, b) => b.at - a.at);
  const recent = all.filter((i) => now - i.at <= WINDOW_MS);
  return (recent.length >= MIN_ITEMS ? recent : all.slice(0, MIN_ITEMS)).slice(0, MAX_ITEMS);
}
