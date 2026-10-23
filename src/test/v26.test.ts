import { describe, expect, it } from "vitest";
import type { Dispatch, EpisodeRaw, Planet } from "../api/types";
import { rewardLabel } from "../data/campaignRewards";
import { campaignEvents, campaignHeadline } from "../domain/campaignNews";
import { fmtAgeShort } from "../domain/format";
import { BUCKET_MS, frontBuckets, latestRun, type FrontRow } from "../domain/frontActivity";
import { buildPolarSectors, factionAnchors } from "../domain/galaxy";
import { isPlaceholder, ownershipKey } from "../domain/planets";
import { mergeEpisodes, toEpisodes } from "../domain/snapshot";
import { tickerItems } from "../domain/ticker";
import episodesJson from "./fixtures/episodes.json";
import planetsJson from "./fixtures/planets.json";

const planets = (planetsJson as unknown as Planet[]).filter((p) => !isPlaceholder(p));
const fixture = episodesJson as { episodes: EpisodeRaw[]; warTime: number; fetchedAt: number };
const campaigns = mergeEpisodes([], toEpisodes(fixture.episodes, fixture.warTime, fixture.fetchedAt), 0);

describe("sector boundaries", () => {
  it("gives two sectors one boundary angle in every ring where their planets allow it", () => {
    const geo = buildPolarSectors(planets);
    const angles = new Map<string, Set<string>>();
    for (const ring of geo.rings)
      ring.runs.forEach((run, k) => {
        const prev = ring.runs[(k - 1 + ring.runs.length) % ring.runs.length];
        if (ring.runs.length < 2) return;
        const key = `${prev.sector}|${run.sector}`;
        const a = (((run.a0 % (2 * Math.PI)) + 2 * Math.PI) % (2 * Math.PI)).toFixed(4);
        angles.set(key, (angles.get(key) ?? new Set()).add(a));
      });
    const shared = [...angles.values()].filter((s) => s.size === 1).length;
    expect(shared / angles.size).toBeGreaterThan(0.9);
  });
});

describe("faction emblems", () => {
  it("places one emblem per enemy just outside its planets, near the rim", () => {
    const anchors = factionAnchors(planets);
    const held = new Set(planets.filter((p) => p.currentOwner !== "Humans").map((p) => p.currentOwner));
    expect(anchors.map((a) => a.faction).sort()).toEqual([...held].sort());
    for (const a of anchors) expect(Math.hypot(a.x, a.y) / 500).toBeLessThanOrEqual(1.05 + 1e-9);
  });

  it("only changes the ownership key when an owner or an attack changes", () => {
    const key = ownershipKey(planets);
    expect(ownershipKey(planets.map((p) => ({ ...p, health: 1 })))).toBe(key);
    const flipped = planets.map((p, i) => (i === 3 ? { ...p, currentOwner: "Automaton" as const } : p));
    expect(ownershipKey(flipped)).not.toBe(key);
  });
});

describe("campaign news", () => {
  it("dates a launch and an end for every finished campaign, newest first", () => {
    const events = campaignEvents(campaigns);
    const ended = campaigns.filter((e) => e.status !== "active");
    expect(events.filter((e) => e.type === "ended")).toHaveLength(ended.length);
    expect(events.filter((e) => e.type === "launched")).toHaveLength(campaigns.length);
    expect(events.every((e, i) => i === 0 || events[i - 1].at >= e.at)).toBe(true);
    const lost = events.find((e) => e.type === "ended" && e.campaign.title === "BLAZING ELECTORATE")!;
    expect(campaignHeadline(lost)).toBe("BLAZING ELECTORATE failed");
  });

  it("names known rewards and falls back to the type, then a generic label", () => {
    expect(rewardLabel(campaigns[0].reward)).toBe("M-103 Supply FRV");
    expect(rewardLabel(2659326411)).toBe("Stratagem");
    expect(rewardLabel(null)).toBe("Campaign reward");
  });
});

describe("ticker", () => {
  const now = Date.parse("2026-10-23T12:00:00Z");
  const dispatch = (id: number, hoursAgo: number): Dispatch => ({
    id,
    published: new Date(now - hoursAgo * 3_600_000).toISOString(),
    type: 0,
    message: `<i=3>NEWS ${id}</i>\n\nBody`,
  });

  it("shows the last day newest first, with captures and losses alongside dispatches", () => {
    const items = tickerItems(
      [1, 2, 3, 4, 5, 6].map((i) => dispatch(i, i)),
      [{ planet: 7, planetName: "Fury", from: "Automaton", to: "Humans", at: now - 1_800_000 }],
      [],
      now,
    );
    expect(items[0]).toMatchObject({ kind: "liberated", text: "Fury liberated", planet: 7 });
    expect(items).toHaveLength(7);
  });

  it("tops a quiet day up with older news", () => {
    const items = tickerItems([dispatch(1, 30), dispatch(2, 50), dispatch(3, 2)], [], [], now);
    expect(items.map((i) => i.text)).toEqual(["NEWS 3", "NEWS 1", "NEWS 2"]);
    expect(fmtAgeShort(now - 90 * 60_000, now)).toBe("1h");
  });
});

describe("front activity", () => {
  const at = (q: number) => new Date(Date.parse("2026-10-23T12:00:00Z") + q * BUCKET_MS).toISOString();
  const rows: FrontRow[] = [
    { bucket: at(0), owner: 3, planet: null, players: 100 },
    { bucket: at(4), owner: 3, planet: null, players: 200 },
    { bucket: at(4), owner: 1, planet: 9, players: 50 },
    { bucket: at(5), owner: 2, planet: null, players: 300 },
    { bucket: at(5), owner: 1, planet: 8, players: 70 },
  ];

  it("credits defenses to their attacker and drops ones it can't place", () => {
    const buckets = frontBuckets(rows, (p) => (p === 9 ? "Illuminate" : undefined));
    expect(buckets[1]).toMatchObject({ automatons: 200, illuminate: 50, players: 250 });
    expect(buckets[2]).toMatchObject({ terminids: 300, players: 300 });
  });

  it("starts the chart after the latest gap in the data", () => {
    const run = latestRun(frontBuckets(rows, () => undefined));
    expect(run.map((b) => b.at)).toEqual([Date.parse(at(4)), Date.parse(at(5))]);
  });
});
