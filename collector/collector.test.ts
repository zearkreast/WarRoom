import { describe, expect, it } from "vitest";
import type { Campaign, EpisodeRaw, Planet } from "../src/api/types.ts";
import {
  campaignTally,
  mergeEpisodes,
  toCampaignLive,
  toEpisodes,
  toLive,
  toPlanet,
  toStatic,
} from "../src/domain/snapshot.ts";
import episodesJson from "../src/test/fixtures/episodes.json";
import planetsJson from "../src/test/fixtures/planets.json";
import campaignsJson from "../src/test/fixtures/campaigns.json";
import warJson from "../src/test/fixtures/war.json";
import assignmentsJson from "../src/test/fixtures/assignments.json";
import dispatchesJson from "../src/test/fixtures/dispatches.json";
import stationsJson from "../src/test/fixtures/space-stations.json";
import { createTasks, runTask } from "./collector.ts";
import { RateLimitError, type Hd2Client } from "./hd2.ts";
import { Recorder } from "./recorder.ts";
import { createSupabase } from "./supabase.ts";

const planets = planetsJson as unknown as Planet[];
const campaigns = campaignsJson as unknown as Campaign[];

interface Call {
  method: string;
  path: string;
  prefer: string | null;
  body: unknown;
}

/** A Supabase stand-in that records every request and answers selects with `rows`. */
function fakeSupabase(rows: Record<string, unknown[]> = {}) {
  const calls: Call[] = [];
  const db = createSupabase({
    url: "https://x.supabase.co",
    key: "sb_secret_test",
    fetchFn: async (url, init) => {
      const path = url.replace("https://x.supabase.co/rest/v1/", "");
      const headers = new Headers(init.headers);
      expect(headers.get("apikey")).toBe("sb_secret_test");
      calls.push({
        method: init.method!,
        path,
        prefer: headers.get("Prefer"),
        body: init.body ? JSON.parse(String(init.body)) : undefined,
      });
      const table = path.split("?")[0];
      if (init.method === "GET") return new Response(JSON.stringify(rows[table] ?? []), { status: 200 });
      if (table.startsWith("rpc/")) return new Response("3", { status: 200 });
      return new Response(null, { status: 201 });
    },
  });
  return { db, calls };
}

function fakeHd2(data: Record<string, unknown>): Hd2Client {
  return {
    async get<T>(path: string) {
      const name = path.split("/").pop()!;
      if (!(name in data)) throw new Error(`unexpected ${path}`);
      const v = data[name];
      if (v instanceof Error) throw v;
      return structuredClone(v) as T;
    },
  };
}

const api = {
  planets,
  campaigns,
  war: warJson,
  assignments: assignmentsJson,
  dispatches: dispatchesJson,
  "space-stations": stationsJson,
};

describe("snapshot shapes", () => {
  it("rebuilds a full planet from its static and live halves", () => {
    for (const p of planets) {
      const back = toPlanet(toStatic(p), toLive(p));
      expect(back.name).toBe(p.name);
      expect(back.health).toBe(p.health);
      expect(back.currentOwner).toBe(p.currentOwner);
      expect(back.statistics.playerCount).toBe(p.statistics.playerCount);
      expect(back.regions.map((r) => r.id)).toEqual(p.regions.map((r) => r.id));
    }
  });

  it("is much smaller than the raw API response", () => {
    const raw = JSON.stringify(planets).length;
    const live = JSON.stringify(planets.map(toLive)).length;
    expect(live).toBeLessThan(raw / 2);
  });

  it("carries the server-side rate on each campaign", () => {
    expect(toCampaignLive(campaigns[0], 1.5)).toMatchObject({
      id: campaigns[0].id,
      rate: 1.5,
      planet: { index: campaigns[0].planet.index },
    });
  });
});

describe("collector cycle", () => {
  it("writes snapshots, samples and owners on a first run", async () => {
    const { db, calls } = fakeSupabase();
    const recorder = new Recorder(db);
    await recorder.load(0);
    const tasks = createTasks(fakeHd2(api), db, recorder, () => 1_000_000);
    for (const t of tasks) expect((await runTask(t)).ok).toBe(true);

    const snaps = calls.filter((c) => c.path === "snapshots").map((c) => (c.body as { kind: string }[])[0].kind);
    expect(snaps).toEqual([
      "planets",
      "planets_static",
      "campaigns",
      "war",
      "assignments",
      "dispatches",
      "space_stations",
    ]);

    const war = calls.find((c) => c.path === "war_samples")!.body as Record<string, number>[];
    expect(war).toHaveLength(1);
    expect(war[0].players).toBe(warJson.statistics.playerCount);
    expect(war[0].terminids + war[0].automatons + war[0].illuminate).toBeGreaterThan(0);
    for (const c of calls.filter((c) => c.path === "snapshots")) expect(c.prefer).toContain("merge-duplicates");

    const samples = calls.filter((c) => c.path === "samples");
    expect(samples.length).toBeGreaterThan(0);
    expect(samples[0].prefer).toContain("ignore-duplicates");
    expect((samples[0].body as { owner: number }[]).every((r) => r.owner >= 1 && r.owner <= 4)).toBe(true);

    const owners = calls.find((c) => c.path === "planet_owners")!.body as unknown[];
    expect(owners).toHaveLength(planets.length);
    expect(calls.some((c) => c.path === "owner_changes")).toBe(false);
    expect(calls.some((c) => c.path === "rpc/compact_samples")).toBe(true);
    expect(calls.some((c) => c.path === "rpc/compact_war_samples")).toBe(true);
  });

  it("records a capture and skips unchanged static data and dispatches", async () => {
    const { db, calls } = fakeSupabase();
    const recorder = new Recorder(db);
    const tasks = createTasks(fakeHd2(api), db, recorder, () => 1_000_000);
    const [planetsTask] = tasks;
    const dispatchTask = tasks.find((t) => t.name === "dispatches")!;
    await runTask(planetsTask);
    await runTask(dispatchTask);
    calls.length = 0;

    const captured = planets.map((p) =>
      p.index === 261 ? { ...p, currentOwner: "Humans" as const, health: p.maxHealth } : p,
    );
    const next = createTasks(fakeHd2({ ...api, planets: captured }), db, recorder, () => 2_000_000);
    const r = await runTask(next[0]);
    expect(r.message).toContain("1 owner change");
    const change = calls.find((c) => c.path === "owner_changes")!.body as Record<string, unknown>[];
    expect(change).toEqual([
      { planet: 261, planet_name: "FURY", from_owner: 3, to_owner: 1, at: new Date(2_000_000).toISOString() },
    ]);
    expect(calls.find((c) => c.path === "planet_owners")!.body).toEqual([{ planet: 261, owner: 1 }]);

    calls.length = 0;
    expect((await runTask(dispatchTask)).message).toBe("unchanged");
    expect(calls).toHaveLength(0);
  });

  it("restores owners after a restart so it doesn't invent captures", async () => {
    const owners = planets.map((p) => ({
      planet: p.index,
      owner: { Humans: 1, Terminids: 2, Automaton: 3, Illuminate: 4 }[p.currentOwner],
    }));
    const { db, calls } = fakeSupabase({ planet_owners: owners });
    const recorder = new Recorder(db);
    await recorder.load(0);
    expect(recorder.knownPlanets).toBe(planets.length);
    await runTask(createTasks(fakeHd2(api), db, recorder)[0]);
    expect(calls.some((c) => c.path === "owner_changes")).toBe(false);
    expect(calls.some((c) => c.path === "planet_owners")).toBe(false);
  });

  it("reports a rate limit with a retry time", async () => {
    const { db } = fakeSupabase();
    const tasks = createTasks(fakeHd2({ ...api, war: new RateLimitError(20_000) }), db, new Recorder(db), () => 0);
    const r = await runTask(tasks.find((t) => t.name === "war")!, () => 0);
    expect(r).toMatchObject({ ok: false, retryAt: 20_000 });
  });
});

describe("galactic campaigns", () => {
  const raw = [
    {
      id32: 1,
      title: "OLD",
      description: "",
      race: 4,
      status: 3,
      startWarTime: 1_000,
      endWarTime: 2_000,
      phases: [{ introTitle: "A", introMessage: "", outroMessage: "lost", status: 3, rewards: [] }],
      rewards: [{ mixId: 7, amount: 1 }],
    },
    {
      id32: 2,
      title: "NOW",
      description: "d",
      race: 1,
      status: 0,
      startWarTime: 9_000,
      phases: [{ introTitle: "B", introMessage: "brief", status: 0, rewards: [{ mixId: 897894480, amount: 40 }] }],
      rewards: [],
    },
  ];
  const episodesTask = (db: ReturnType<typeof fakeSupabase>["db"], list: unknown, at = 50_000_000) =>
    createTasks(
      fakeHd2(api),
      db,
      new Recorder(db),
      () => at,
      fakeHd2({ WarID: { id: 801 }, "801": { episodes: list }, Status: { time: 10_000 } }),
    ).find((x) => x.name === "episodes")!;

  it("dates war-second timestamps against the war clock and maps outcomes, faction and reward", () => {
    const [old, now] = toEpisodes(raw, 10_000, 50_000_000);
    expect(old).toMatchObject({
      status: "lost",
      faction: "Illuminate",
      reward: 7,
      start: 50_000_000 - 9_000_000,
      end: 50_000_000 - 8_000_000,
      phases: [{ outro: "lost", issued: 50_000_000 - 9_000_000 }],
    });
    expect(now).toMatchObject({
      status: "active",
      faction: "Humans",
      end: null,
      reward: null,
      phases: [{ title: "B", status: "active", medals: 40 }],
    });
  });

  it("maps the live API's campaigns: won by majority, lost by majority", () => {
    const { episodes, warTime, fetchedAt } = episodesJson as {
      episodes: EpisodeRaw[];
      warTime: number;
      fetchedAt: number;
    };
    const list = toEpisodes(episodes, warTime, fetchedAt);
    for (const e of list.filter((x) => x.status !== "active")) {
      const t = campaignTally(e);
      expect(e.status).toBe(t.won > t.lost ? "won" : "lost");
    }
    expect(list.find((e) => e.title === "BLAZING ELECTORATE")?.status).toBe("lost");
    expect(list.every((e) => e.reward != null)).toBe(true);
  });

  it("keeps campaigns the game stops listing and stamps new Major Orders when they appear", () => {
    const stored = toEpisodes(raw, 10_000, 50_000_000);
    const next = toEpisodes(
      [
        {
          ...raw[1],
          phases: [...raw[1].phases, { introTitle: "C", introMessage: "", status: 0, rewards: [] }],
        },
      ],
      20_000,
      60_000_123,
    );
    const merged = mergeEpisodes(stored, next, 60_000_123);
    expect(merged.map((e) => e.id)).toEqual([1, 2]);
    expect(merged[1].start).toBe(stored[1].start);
    expect(merged[1].phases.map((p) => p.issued)).toEqual([stored[1].start, 60_000_123]);
    // Nothing stored yet: orders after the first predate the collector, so their date is unknown.
    expect(mergeEpisodes([], next, 1)[0].phases[1].issued).toBeNull();
  });

  it("reads the stored archive once, then writes only when something changed", async () => {
    const stored = toEpisodes(raw, 10_000, 50_000_000);
    const { db, calls } = fakeSupabase({ snapshots: [{ data: stored }] });
    const t = episodesTask(db, raw);
    expect((await runTask(t)).message).toBe("unchanged, 1 active");
    expect((await runTask(t)).message).toBe("unchanged, 1 active");
    expect(calls.filter((c) => c.method === "GET")).toHaveLength(1);
    const kinds = calls.filter((c) => c.method === "POST").map((c) => (c.body as { kind: string }[])[0].kind);
    expect(kinds).toEqual(["planet_effects"]);
  });

  it("keeps the planet effects from the status it already reads, without another request", async () => {
    const { db, calls } = fakeSupabase();
    const t = createTasks(
      fakeHd2(api),
      db,
      new Recorder(db),
      () => 50_000_000,
      fakeHd2({
        WarID: { id: 801 },
        "801": { episodes: raw },
        Status: {
          time: 10_000,
          planetActiveEffects: [
            { index: 5, galacticEffectId: 1249 },
            { index: 5, galacticEffectId: 1248 },
            { index: 2, galacticEffectId: 1395 },
          ],
          spaceStations: [{ id32: 9, planetIndex: 5, activeEffectIds: [1238] }],
        },
      }),
    ).find((x) => x.name === "episodes")!;
    await runTask(t);
    const written = calls.find((c) => c.method === "POST" && (c.body as { kind: string }[])[0].kind === "planet_effects");
    expect((written!.body as { data: unknown }[])[0].data).toEqual({
      planets: [
        { index: 2, ids: [1395] },
        { index: 5, ids: [1248, 1249] },
      ],
      stations: [{ id: 9, ids: [1238] }],
    });
  });

  it("dates a new order only on a running campaign it already knew, and drops impossible dates", () => {
    const stored = toEpisodes(raw, 10_000, 50_000_000);
    const ended = { ...raw[0], phases: [...raw[0].phases, { introTitle: "Late", introMessage: "", status: 3, rewards: [] }] };
    const merged = mergeEpisodes(stored, toEpisodes([ended, raw[1]], 20_000, 60_000_000), 60_000_000);
    expect(merged[0].phases[1].issued).toBeNull();
    const broken = [{ ...stored[0], phases: [stored[0].phases[0], { ...stored[0].phases[0], issued: 60_000_000 }] }];
    expect(mergeEpisodes(broken, toEpisodes([ended], 20_000, 60_000_000), 70_000_000)[0].phases[1].issued).toBeNull();
  });

  it("writes an episodes snapshot only when the game API client is given", async () => {
    const { db, calls } = fakeSupabase();
    expect((await runTask(episodesTask(db, raw))).message).toBe("2 campaigns, 1 active");
    expect(calls.some((c) => c.path === "snapshots" && (c.body as { kind: string }[])[0].kind === "episodes")).toBe(
      true,
    );
    expect(createTasks(fakeHd2(api), db, new Recorder(db)).some((x) => x.name === "episodes")).toBe(false);
  });
});
