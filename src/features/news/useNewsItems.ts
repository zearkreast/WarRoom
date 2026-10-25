import { useMemo } from "react";
import { useDispatches, useEpisodes } from "../../api/queries";
import type { Dispatch } from "../../api/types";
import { campaignThreads, type CampaignThread } from "../../domain/campaignNews";
import { dispatchKind, type DispatchKind } from "../../domain/dispatch";
import { plainText } from "../../domain/newsFeed";
import { phaseName } from "../../domain/snapshot";
import { useSettings } from "../../state";

export type NewsKind = DispatchKind | "campaign";

export type NewsItem = { kind: NewsKind; at: number; key: string; text: string } & (
  { d: Dispatch; t?: never } | { t: CampaignThread; d?: never }
);

function threadText(t: CampaignThread): string {
  const e = t.campaign;
  return [
    e.title,
    e.description ?? "",
    ...e.phases.flatMap((p) => [phaseName(p) ?? "", p.briefing ?? ""]),
    ...t.entries.map((x) => (x.dispatch ? plainText(x.dispatch.message) : "")),
  ].join(" ");
}

/** Dispatches and Galactic Campaign threads, newest first. */
export function useNewsItems() {
  const { data, isLoading } = useDispatches();
  const episodes = useEpisodes().data;
  const items = useMemo((): NewsItem[] => {
    // Dispatches about a campaign live on its thread, so its orders and outcomes read as one story.
    const { threads, claimed } = campaignThreads(episodes ?? [], data ?? []);
    const dispatches = (data ?? [])
      .filter((d) => !claimed.has(d.id))
      .map((d): NewsItem => ({
        kind: dispatchKind(d.message),
        at: Date.parse(d.published),
        key: `d${d.id}`,
        text: plainText(d.message),
        d,
      }));
    const campaign = threads.map((t): NewsItem => ({ kind: "campaign", at: t.at, key: t.key, text: threadText(t), t }));
    return [...dispatches, ...campaign].sort((a, b) => b.at - a.at);
  }, [data, episodes]);
  return { items, isLoading, dispatches: data };
}

/** News items newer than the last visit to Strohmann News; none before the first visit. */
export function useUnreadNews(): number {
  const { items } = useNewsItems();
  const seenAt = useSettings((s) => s.newsSeenAt);
  return seenAt ? items.filter((i) => i.at > seenAt).length : 0;
}
