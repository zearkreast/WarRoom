import type { EpisodeLive } from "./snapshot";

/** A moment in a Galactic Campaign worth a news item: its launch, a new Major Order, its end. */
export interface CampaignEvent {
  key: string;
  at: number;
  type: "launched" | "order" | "ended";
  campaign: EpisodeLive;
  /** Index of the Major Order a "launched" or "order" event announces. */
  phase: number;
}

/** Every dated campaign event, newest first. Orders with no known issue date are left out. */
export function campaignEvents(list: EpisodeLive[]): CampaignEvent[] {
  const out: CampaignEvent[] = [];
  for (const e of list) {
    out.push({ key: `${e.id}-launched`, at: e.start, type: "launched", campaign: e, phase: 0 });
    e.phases.forEach((p, i) => {
      if (i > 0 && p.issued != null)
        out.push({ key: `${e.id}-order-${i}`, at: p.issued, type: "order", campaign: e, phase: i });
    });
    if (e.status !== "active" && e.end != null)
      out.push({ key: `${e.id}-ended`, at: e.end, type: "ended", campaign: e, phase: e.phases.length - 1 });
  }
  return out.sort((a, b) => b.at - a.at);
}

/** One-line headline for a campaign event, in the game's words. */
export function campaignHeadline(ev: CampaignEvent): string {
  const { campaign: e, phase } = ev;
  if (ev.type === "launched") return `New campaign: ${e.title}`;
  if (ev.type === "order") return `${e.title}: Major Order ${phase + 1}, ${e.phases[phase].title}`;
  return `${e.title} ${e.status === "won" ? "won" : "failed"}`;
}
