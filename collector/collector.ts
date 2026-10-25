import type {
  Assignment,
  Campaign,
  Dispatch,
  EpisodeRaw,
  Planet,
  SpaceStation,
  War,
  WarStatusRaw,
} from "../src/api/types.ts";
import {
  mergeEpisodes,
  newestDispatches,
  toCampaignLive,
  toLive,
  toStatic,
  toEffectsLive,
  toEpisodes,
  toStationLive,
  toWarSample,
  addOrderSamples,
  type EpisodeLive,
  type OrderSample,
  type SnapshotKind,
  type StationLive,
} from "../src/domain/snapshot.ts";
import type { Hd2Client } from "./hd2.ts";
import type { Recorder } from "./recorder.ts";
import type { Supabase } from "./supabase.ts";

export interface Task {
  name: string;
  everyMs: number;
  run(): Promise<string>;
}

/**
 * Polling jobs: each fetches from the API, writes to Supabase, and returns a short status for the log.
 * `game` is the official game API, for data the community API doesn't carry (story campaigns).
 */
export function createTasks(
  hd2: Hd2Client,
  db: Supabase,
  recorder: Recorder,
  now = Date.now,
  game?: Hd2Client,
): Task[] {
  let lastStatic = "";
  let lastDispatches = "";
  /** Latest campaigns, so each war sample can split players by front. */
  let lastCampaigns: Campaign[] = [];
  /** Each station's planet as last seen; read back from the stored snapshot after a restart. */
  let stationAt: Map<number, number> | null = null;
  /** The last day of Major Order readings; read back from the stored snapshot after a restart. */
  let orderSamples: OrderSample[] | null = null;

  const snapshot = (kind: SnapshotKind, data: unknown) =>
    db.upsert("snapshots", [{ kind, data, fetched_at: new Date(now()).toISOString() }]);

  const changeNote = (n: number) => (n ? `, ${n} owner change${n > 1 ? "s" : ""}` : "");

  const tasks: Task[] = [
    {
      // Full list first, so a fresh database learns every planet's owner on the first cycle.
      name: "planets",
      everyMs: 300_000,
      async run() {
        const planets = await hd2.get<Planet[]>("/api/v1/planets");
        const { samples, changes } = await recorder.ingest(planets, now());
        await snapshot("planets", planets.map(toLive));
        const statics = planets.map(toStatic);
        const json = JSON.stringify(statics);
        // Static data barely changes; only rewrite it when it does.
        if (json !== lastStatic) {
          await snapshot("planets_static", statics);
          lastStatic = json;
        }
        return `${planets.length} planets, ${samples} samples${changeNote(changes.length)}`;
      },
    },
    {
      name: "campaigns",
      everyMs: 60_000,
      async run() {
        const campaigns = await hd2.get<Campaign[]>("/api/v1/campaigns");
        lastCampaigns = campaigns;
        const { samples, changes } = await recorder.ingest(
          campaigns.map((c) => c.planet),
          now(),
        );
        await snapshot(
          "campaigns",
          campaigns.map((c) =>
            toCampaignLive(c, recorder.rate(c.planet.index), (region) => recorder.regionRate(c.planet.index, region)),
          ),
        );
        return `${campaigns.length} fronts, ${samples} samples${changeNote(changes.length)}`;
      },
    },
    {
      name: "war",
      everyMs: 120_000,
      async run() {
        const war = await hd2.get<War>("/api/v1/war");
        await snapshot("war", war);
        await db.upsert("war_samples", [toWarSample(war, lastCampaigns, now())], { ignoreDuplicates: true });
        return `${war.statistics.playerCount} divers`;
      },
    },
    {
      name: "assignments",
      everyMs: 300_000,
      async run() {
        const a = await hd2.get<Assignment[]>("/api/v1/assignments");
        await snapshot("assignments", a);
        if (!orderSamples) {
          const rows = await db.select<{ data: OrderSample[] }>("snapshots", "select=data&kind=eq.order_samples");
          orderSamples = rows[0]?.data ?? [];
        }
        orderSamples = addOrderSamples(orderSamples, a, now());
        await snapshot("order_samples", orderSamples);
        return `${a.length} orders`;
      },
    },
    {
      name: "dispatches",
      everyMs: 300_000,
      async run() {
        const d = newestDispatches(await hd2.get<Dispatch[]>("/api/v1/dispatches"));
        // Compared by text, not id: a dispatch stored in the wrong language is replaced once it comes in English.
        const json = JSON.stringify(d);
        if (json === lastDispatches) return "unchanged";
        await snapshot("dispatches", d);
        lastDispatches = json;
        return `newest #${d[0]?.id ?? 0}`;
      },
    },
    {
      name: "stations",
      everyMs: 300_000,
      async run() {
        const stations = await hd2.get<SpaceStation[]>("/api/v2/space-stations");
        if (!stationAt) {
          const rows = await db.select<{ data: StationLive[] }>("snapshots", "select=data&kind=eq.space_stations");
          stationAt = new Map((rows[0]?.data ?? []).map((s) => [s.id32, s.planetIndex]));
        }
        const jumps = stations.flatMap((s) => {
          const from = stationAt!.get(s.id32);
          return from !== undefined && from !== s.planet.index
            ? [{ station: s.id32, at: new Date(now()).toISOString(), from_planet: from, to_planet: s.planet.index }]
            : [];
        });
        if (jumps.length) await db.upsert("station_jumps", jumps, { ignoreDuplicates: true });
        await snapshot("space_stations", stations.map(toStationLive));
        for (const s of stations) stationAt.set(s.id32, s.planet.index);
        const count = `${stations.length} station${stations.length === 1 ? "" : "s"}`;
        return jumps.length ? `${count}, ${jumps.length} FTL jump${jumps.length > 1 ? "s" : ""}` : count;
      },
    },
    {
      name: "compact",
      everyMs: 3_600_000,
      async run() {
        const removed = await db.rpc<number>("compact_samples");
        const removedWar = await db.rpc<number>("compact_war_samples");
        return `${removed} old samples and ${removedWar} war samples thinned`;
      },
    },
  ];
  /** Campaigns as last written; read back once after a restart so the archive and issue dates survive it. */
  let episodes: EpisodeLive[] | null = null;
  let lastEffects = "";
  if (game)
    tasks.push({
      name: "episodes",
      everyMs: 900_000,
      async run() {
        const { id } = await game.get<{ id: number }>("/api/WarSeason/current/WarID");
        const raw = await game.get<{ episodes: EpisodeRaw[] }>(`/api/Episode/${id}`);
        const status = await game.get<WarStatusRaw>(`/api/WarSeason/${id}/Status`);
        const { time } = status;
        // Effects come from the status this job already reads, at no extra request.
        const effects = JSON.stringify(toEffectsLive(status));
        if (effects !== lastEffects) {
          await snapshot("planet_effects", JSON.parse(effects));
          lastEffects = effects;
        }
        if (!episodes) {
          const rows = await db.select<{ data: EpisodeLive[] }>("snapshots", "select=data&kind=eq.episodes");
          episodes = rows[0]?.data ?? [];
        }
        const list = mergeEpisodes(episodes, toEpisodes(raw.episodes, time, now()), now());
        const active = list.filter((e) => e.status === "active").length;
        if (JSON.stringify(list) === JSON.stringify(episodes)) return `unchanged, ${active} active`;
        await snapshot("episodes", list);
        episodes = list;
        return `${list.length} campaigns, ${active} active`;
      },
    });
  return tasks;
}

export interface RunResult {
  name: string;
  ok: boolean;
  message: string;
  /** When set, the task should not run again before this time (after a 429 or while the API is down). */
  retryAt?: number;
}

export async function runTask(t: Task, now = Date.now): Promise<RunResult> {
  try {
    return { name: t.name, ok: true, message: await t.run() };
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e);
    const after = (e as { retryAfterMs?: number }).retryAfterMs;
    const retryAt = after ? now() + after : undefined;
    return { name: t.name, ok: false, message, retryAt };
  }
}
