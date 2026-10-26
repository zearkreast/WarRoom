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
import { dispatchTitle } from "../src/domain/format.ts";
import type { Hd2Client } from "./hd2.ts";
import type { Push } from "./push.ts";
import type { Recorder } from "./recorder.ts";
import { unlessMissing, type Supabase } from "./supabase.ts";

export interface Task {
  name: string;
  everyMs: number;
  run(): Promise<string>;
}

/**
 * Polling jobs: each fetches from the API, writes to Supabase, and returns a short status for the log.
 * `game` is the official game API, for data the community API doesn't carry (story campaigns).
 * `push` sends new Strohmann News (Major Orders, dispatches, campaign launches and outcomes) to the app.
 */
export function createTasks(
  hd2: Hd2Client,
  db: Supabase,
  recorder: Recorder,
  now = Date.now,
  game?: Hd2Client,
  push?: Push,
): Task[] {
  let lastStatic = "";
  let lastDispatches = "";
  /**
   * Major Orders and the newest dispatch already announced, or found stored after a restart. With nothing
   * stored yet, whatever is there on the first run counts as announced, so a fresh setup pushes nothing.
   */
  let announcedOrders: Set<number> | null = null;
  let announcedDispatch: number | null = null;
  /** Latest campaigns, so each war sample can split players by front. */
  let lastCampaigns: Campaign[] = [];
  /** Each station's planet as last seen; read back from the stored snapshot after a restart. */
  let stationAt: Map<number, number> | null = null;
  /** The last day of Major Order readings; read back from the stored snapshot after a restart. */
  let orderSamples: OrderSample[] | null = null;

  const snapshot = (kind: SnapshotKind, data: unknown) =>
    db.upsert("snapshots", [{ kind, data, fetched_at: new Date(now()).toISOString() }]);

  const changeNote = (n: number) => (n ? `, ${n} owner change${n > 1 ? "s" : ""}` : "");

  // Keep the wording in step with the app's own alerts (src/sync.ts). A failed push is only logged: the
  // snapshot is already written, so a retry wouldn't see the news as new.
  const announce = async (items: { title: string; body: string }[]): Promise<string> => {
    let sent = 0;
    for (const { title, body } of items) {
      try {
        await push!(title, body);
        sent++;
      } catch (e) {
        console.error(`push failed: ${e instanceof Error ? e.message : String(e)}`);
      }
    }
    return items.length ? `, ${sent}/${items.length} pushed` : "";
  };

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
        const online = war.statistics.playerCount;
        // A reading of nobody online is an API blip, not the day's low.
        const noted =
          online > 0 && (await unlessMissing(db.rpc("note_war_day", { reading_at: new Date(now()).toISOString(), online })));
        return `${online} divers${online > 0 && !noted ? ", no war_days table yet" : ""}`;
      },
    },
    {
      name: "assignments",
      everyMs: 300_000,
      async run() {
        const a = await hd2.get<Assignment[]>("/api/v1/assignments");
        if (push && !announcedOrders) {
          const rows = await db.select<{ data: Assignment[] }>("snapshots", "select=data&kind=eq.assignments");
          announcedOrders = new Set((rows[0]?.data ?? a).map((x) => x.id));
        }
        await snapshot("assignments", a);
        if (!orderSamples) {
          const rows = await db.select<{ data: OrderSample[] }>("snapshots", "select=data&kind=eq.order_samples");
          orderSamples = rows[0]?.data ?? [];
        }
        orderSamples = addOrderSamples(orderSamples, a, now());
        await snapshot("order_samples", orderSamples);
        let pushed = "";
        if (announcedOrders) {
          const fresh = a.filter((x) => !announcedOrders!.has(x.id));
          pushed = await announce(
            fresh.map((x) => ({
              title: "New Major Order",
              body: x.briefing ?? x.title ?? "High Command has issued new orders.",
            })),
          );
          for (const x of fresh) announcedOrders.add(x.id);
        }
        return `${a.length} orders${pushed}`;
      },
    },
    {
      name: "dispatches",
      everyMs: 300_000,
      async run() {
        const d = newestDispatches(await hd2.get<Dispatch[]>("/api/v1/dispatches"));
        if (push && announcedDispatch === null) {
          const rows = await db.select<{ data: Dispatch[] }>("snapshots", "select=data&kind=eq.dispatches");
          announcedDispatch = Math.max(0, ...(rows[0]?.data ?? d).map((x) => x.id));
        }
        // Compared by text, not id: a dispatch stored in the wrong language is replaced once it comes in English.
        const json = JSON.stringify(d);
        if (json === lastDispatches) return "unchanged";
        await snapshot("dispatches", d);
        lastDispatches = json;
        let pushed = "";
        if (announcedDispatch !== null) {
          const since = announcedDispatch;
          // The newest three, like the app, sent oldest first so the newest ends up on top.
          const fresh = d.filter((x) => x.id > since).slice(0, 3).reverse();
          pushed = await announce(fresh.map((x) => ({ title: "New dispatch", body: dispatchTitle(x.message) })));
          announcedDispatch = Math.max(since, ...d.map((x) => x.id));
        }
        return `newest #${d[0]?.id ?? 0}${pushed}`;
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
        let removedActivity = 0;
        await unlessMissing(db.rpc<number>("compact_activity").then((n) => (removedActivity = n)));
        return `${removed} old samples, ${removedWar} war samples and ${removedActivity} activity readings thinned`;
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
        const before = episodes;
        episodes = list;
        // An empty archive is a fresh setup, not a dozen campaigns launching at once.
        const pushed = push && before.length ? await announce(campaignNews(before, list)) : "";
        return `${list.length} campaigns, ${active} active${pushed}`;
      },
    });
  return tasks;
}

/** Campaigns launched or ended between two archives, worded as the app's campaignHeadline() words them. */
export function campaignNews(before: EpisodeLive[], after: EpisodeLive[]): { title: string; body: string }[] {
  const was = new Map(before.map((e) => [e.id, e]));
  return after.flatMap((e) => {
    const old = was.get(e.id);
    if (!old) return [{ title: "New Galactic Campaign", body: `New campaign: ${e.title}` }];
    if (old.status === "active" && e.status !== "active")
      return [{ title: "Galactic Campaign over", body: `${e.title} ${e.status === "won" ? "won" : "failed"}` }];
    return [];
  });
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
