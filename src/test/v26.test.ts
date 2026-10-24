import { describe, expect, it } from "vitest";
import type { Dispatch, EpisodeRaw, Planet } from "../api/types";
import { rewardLabel } from "../data/campaignRewards";
import { campaignEvents, campaignHeadline, campaignOf, campaignThreads } from "../domain/campaignNews";
import { fmtAgeShort } from "../domain/format";
import { BUCKET_MS, frontBuckets, latestRun, mergeActivity, type FrontRow } from "../domain/frontActivity";
import { factionMarks } from "../domain/factionMarks";
import { buildPolarSectors, GALAXY_SCALE, sectorAt, sectorTerritory, type PolarSectors } from "../domain/galaxy";
import { fixStatic, isOffMap, isPlaceholder, ownershipKey } from "../domain/planets";
import { issuedAt, mergeEpisodes, toEpisodes } from "../domain/snapshot";
import { tickerItems } from "../domain/ticker";
import episodesJson from "./fixtures/episodes.json";
import planetsJson from "./fixtures/planets.json";

const planets = (planetsJson as unknown as Planet[]).filter((p) => !isPlaceholder(p)).map(fixStatic);
const fixture = episodesJson as { episodes: EpisodeRaw[]; warTime: number; fetchedAt: number };
const campaigns = mergeEpisodes([], toEpisodes(fixture.episodes, fixture.warTime, fixture.fetchedAt), 0);

/** Sectors whose cells fall apart into more than one piece. */
function splitSectors(geo: PolarSectors): string[] {
  const TAU = Math.PI * 2;
  const cells = geo.rings.flatMap((ring, i) => ring.runs.map((r, k) => ({ i, k, ...r })));
  const overlap = (a: (typeof cells)[number], b: (typeof cells)[number]) =>
    [-TAU, 0, TAU].some((s) => Math.min(a.a1, b.a1 + s) - Math.max(a.a0, b.a0 + s) > 1e-6);
  return [...new Set(cells.map((c) => c.sector))].filter((sector) => {
    const own = cells.filter((c) => c.sector === sector);
    const seen = new Set([own[0]]);
    for (const c of seen)
      for (const d of own)
        if ((Math.abs(d.i - c.i) === 1 && overlap(c, d)) || (d.i === c.i && Math.abs(d.k - c.k) === 1)) seen.add(d);
    return seen.size < own.length;
  });
}

describe("planet corrections", () => {
  it("files Meridia under Celeste and puts Sangis and Alderidge Cove back where the game shows them", () => {
    const byName = new Map(planets.map((p) => [p.name, p]));
    expect(byName.get("MERIDIA")?.sector).toBe("Celeste");
    expect(byName.get("SANGIS")).toMatchObject({ index: 273, sector: "Guang" });
    expect(byName.get("ALDERIDGE COVE")).toMatchObject({ index: 99, sector: "Omega" });
  });

  it("leaves no sector in pieces once corrected", () => {
    const raw = (planetsJson as unknown as Planet[]).filter((p) => !isPlaceholder(p));
    expect(splitSectors(buildPolarSectors(raw))).toEqual(expect.arrayContaining(["Umlaut", "Guang", "Omega"]));
    expect(splitSectors(buildPolarSectors(planets))).toEqual([]);
  });
});

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
  it("puts each enemy's emblem inside its own territory in the Sectors overlay", () => {
    const geo = buildPolarSectors(planets);
    const territory = sectorTerritory(planets);
    const marks = factionMarks(planets, "sectors", geo);
    expect(marks.length).toBeGreaterThan(0);
    for (const m of marks) {
      const sector = sectorAt(geo, m.x / GALAXY_SCALE, -m.y / GALAXY_SCALE);
      expect(territory.get(sector ?? "")?.faction).toBe(m.faction);
    }
  });

  it("puts each enemy's emblem on its largest group of planets in the other overlays", () => {
    for (const overlay of ["regions", "galaxy"] as const) {
      const marks = factionMarks(planets, overlay, undefined);
      const held = new Set(planets.filter((p) => p.currentOwner !== "Humans").map((p) => p.currentOwner));
      expect(marks.map((m) => m.faction).sort()).toEqual([...held].sort());
      for (const m of marks) {
        const nearest = planets.reduce((a, b) =>
          Math.hypot(b.position.x * 500 - m.x, -b.position.y * 500 - m.y) <
          Math.hypot(a.position.x * 500 - m.x, -a.position.y * 500 - m.y)
            ? b
            : a,
        );
        expect(nearest.currentOwner).toBe(m.faction);
      }
    }
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

  it("gathers a campaign's orders and dispatches on one thread, and leaves other dispatches alone", () => {
    const e = campaigns.find((c) => c.status === "active") ?? campaigns[campaigns.length - 1];
    const at = (h: number) => new Date(e.start + h * 3_600_000).toISOString();
    const d = (id: number, message: string, h: number): Dispatch => ({ id, published: at(h), type: 0, message });
    const list = [
      d(1, "<i=3>NEW MAJOR ORDER</i> First", 0.5),
      d(2, "<i=3>MAJOR ORDER FAILED</i> Too few samples", 30),
      d(3, "<i=3>STRATEGIC ALERT</i> Incineration Corps sighted", 31),
    ];
    const { threads, claimed } = campaignThreads(campaigns, list);
    expect([...claimed].sort()).toEqual([1, 2]);
    const mine = threads.find((t) => t.campaign.id === e.id)!;
    expect(mine.entries.filter((x) => x.dispatch).map((x) => x.dispatch!.id)).toEqual([1, 2]);
    // The first order's announcement is told on the launch, not as a second line.
    expect(mine.entries[0].event?.type).toBe("launched");
    expect(mine.entries[0].dispatch?.id).toBe(1);
    expect(mine.entries.every((x, i) => i === 0 || mine.entries[i - 1].at <= x.at)).toBe(true);
  });

  it("doesn't let an order the game hasn't named yet claim every Major Order headline", () => {
    const [older, newer] = [...campaigns].sort((a, b) => a.start - b.start).slice(-2);
    const unnamed = { ...newer, phases: [...newer.phases, { ...newer.phases[0], title: "MAJOR ORDER", status: "active" as const }] };
    const old: Dispatch = { id: 9, published: new Date(older.start + 3_600_000).toISOString(), type: 0, message: "<i=3>MAJOR ORDER WON</i> Done" };
    const { threads } = campaignThreads([older, unnamed], [old]);
    expect(threads.find((t) => t.campaign.id === unnamed.id)!.entries.some((x) => x.dispatch?.id === 9)).toBe(false);
  });

  it("matches order names as whole words, and only while their campaign runs", () => {
    const e = { ...campaigns[0], title: "CELESTIAL FENCE", start: 0, end: 10 * 3_600_000, phases: [{ ...campaigns[0].phases[0], title: "DEPLOYMENT" }] };
    const d = (h: number, message: string): Dispatch => ({ id: h, published: new Date(h * 3_600_000).toISOString(), type: 0, message });
    expect(campaignOf(d(5, "<i=3>WARFRONT UPDATE</i> DEPLOYMENT is under way"), [e])).toBe(e);
    expect(campaignOf(d(5, "<i=3>ARSENAL AUGMENTATION</i> for all Helldiver deployments"), [e])).toBeNull();
    expect(campaignOf(d(500, "<i=3>WARFRONT UPDATE</i> DEPLOYMENT is under way"), [e])).toBeNull();
  });

  it("never dates an order after its campaign ended or before it began", () => {
    const ended = campaigns.find((c) => c.status !== "active")!;
    expect(issuedAt(ended, ended.end! + 1)).toBeNull();
    expect(issuedAt(ended, ended.start - 1)).toBeNull();
    expect(issuedAt(ended, ended.start + 1)).toBe(ended.start + 1);
  });

  it("names known rewards and falls back to the type, then a generic label", () => {
    expect(rewardLabel(campaigns[0].reward)).toBe("M-103 Supply FRV");
    expect(rewardLabel(2659326411)).toBe("Stratagem");
    expect(rewardLabel(null)).toBe("Campaign reward");
  });
});

describe("activity chart rows", () => {
  const t0 = 1_000 * BUCKET_MS;
  const bucket = (at: number, n: number) => ({ at, terminids: n, automatons: 0, illuminate: 0, players: n });
  const reading = (at: number, split: number) => ({
    at,
    impact: 0.01,
    players: 5000,
    terminids: split,
    automatons: 0,
    illuminate: 0,
  });

  it("uses the buckets until the first reading, then each reading's own split by front", () => {
    const rows = mergeActivity(
      [bucket(t0, 100), bucket(t0 + BUCKET_MS, 200), bucket(t0 + 2 * BUCKET_MS, 999)],
      [reading(t0 + BUCKET_MS / 2, 4000)],
      t0 + 2 * BUCKET_MS + 60_000,
    );
    expect(rows.map((r) => [r.at - t0, r.terminids, r.online])).toEqual([
      [0, 100, null],
      [BUCKET_MS / 2, 4000, 5000],
    ]);
  });

  it("draws from the readings alone once they cover an hour", () => {
    const rows = mergeActivity(
      [bucket(t0, 100), bucket(t0 + 8 * BUCKET_MS, 200)],
      [reading(t0 + 4 * BUCKET_MS, 4000), reading(t0 + 8 * BUCKET_MS, 4100)],
      t0 + 10 * BUCKET_MS,
    );
    expect(rows.map((r) => r.terminids)).toEqual([4000, 4100]);
  });

  it("falls back to the buckets for a reading saved without a split", () => {
    const rows = mergeActivity([bucket(t0, 100), bucket(t0 + BUCKET_MS, 200)], [reading(t0 + BUCKET_MS / 2, 0)], t0 + 3 * BUCKET_MS);
    expect(rows[rows.length - 1].terminids).toBe(150);
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

describe("off-map planets", () => {
  it("drops Calypso, which the game removed but the API still lists", () => {
    const calypso = (planetsJson as unknown as Planet[]).find((p) => p.name === "CALYPSO")!;
    expect(isOffMap(calypso)).toBe(true);
    expect(isOffMap(planets[1])).toBe(false);
  });
});
