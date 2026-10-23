import type { Assignment, Campaign, Dispatch, EpisodeRaw, Planet, SpaceStation, War } from "../src/api/types.ts";
import {
  newestDispatches,
  toCampaignLive,
  toLive,
  toStatic,
  toEpisodes,
  toStationLive,
  toWarSample,
  type SnapshotKind,
} from "../src/domain/snapshot.ts";
import { RateLimitError, type Hd2Client } from "./hd2.ts";
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
  let lastDispatch = 0;
  /** Latest campaigns, so each war sample can split players by front. */
  let lastCampaigns: Campaign[] = [];

  const snapshot = (kind: SnapshotKind, data: unknown) =>
    db.upsert("snapshots", [{ kind, data, fetched_at: new Date(now()).toISOString() }]);

  const changeNote = (n: number) => (n ? `, ${n} owner change${n > 1 ? "s" : ""}` : "");

  const tasks: Task[] = [
    {
      // Full list first, so a fresh database learns every planet's owner on the first cycle.
      name: "planets",
      everyMs: 180_000,
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
          campaigns.map((c) => toCampaignLive(c, recorder.rate(c.planet.index))),
        );
        return `${campaigns.length} fronts, ${samples} samples${changeNote(changes.length)}`;
      },
    },
    {
      name: "war",
      everyMs: 60_000,
      async run() {
        const war = await hd2.get<War>("/api/v1/war");
        await snapshot("war", war);
        await db.upsert("war_samples", [toWarSample(war, lastCampaigns, now())], { ignoreDuplicates: true });
        return `${war.statistics.playerCount} divers`;
      },
    },
    {
      name: "assignments",
      everyMs: 60_000,
      async run() {
        const a = await hd2.get<Assignment[]>("/api/v1/assignments");
        await snapshot("assignments", a);
        return `${a.length} orders`;
      },
    },
    {
      name: "dispatches",
      everyMs: 300_000,
      async run() {
        const d = newestDispatches(await hd2.get<Dispatch[]>("/api/v1/dispatches"));
        const top = d[0]?.id ?? 0;
        if (top === lastDispatch) return "unchanged";
        await snapshot("dispatches", d);
        lastDispatch = top;
        return `newest #${top}`;
      },
    },
    {
      name: "stations",
      everyMs: 120_000,
      async run() {
        const stations = await hd2.get<SpaceStation[]>("/api/v2/space-stations");
        await snapshot("space_stations", stations.map(toStationLive));
        return `${stations.length} station${stations.length === 1 ? "" : "s"}`;
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
  if (game)
    tasks.push({
      name: "episodes",
      everyMs: 900_000,
      async run() {
        const { id } = await game.get<{ id: number }>("/api/WarSeason/current/WarID");
        const { episodes } = await game.get<{ episodes: EpisodeRaw[] }>(`/api/Episode/${id}`);
        const { time } = await game.get<{ time: number }>(`/api/WarSeason/${id}/Status`);
        const list = toEpisodes(episodes, time, now());
        await snapshot("episodes", list);
        return `${list.length} campaigns, ${list.filter((e) => e.status === "active").length} active`;
      },
    });
  return tasks;
}

export interface RunResult {
  name: string;
  ok: boolean;
  message: string;
  /** When set, the task should not run again before this time (e.g. after a 429). */
  retryAt?: number;
}

export async function runTask(t: Task, now = Date.now): Promise<RunResult> {
  try {
    return { name: t.name, ok: true, message: await t.run() };
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e);
    const retryAt = e instanceof RateLimitError ? now() + e.retryAfterMs : undefined;
    return { name: t.name, ok: false, message, retryAt };
  }
}
