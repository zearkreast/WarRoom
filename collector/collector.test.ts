import { describe, expect, it } from "vitest";
import type { Campaign, Planet } from "../src/api/types.ts";
import { toCampaignLive, toLive, toPlanet, toStatic } from "../src/domain/snapshot.ts";
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
