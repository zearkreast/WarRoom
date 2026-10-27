import { isTauri } from "@tauri-apps/api/core";
import { fetch as tauriFetch } from "@tauri-apps/plugin-http";
import { SUPABASE_PUBLISHABLE_KEY, SUPABASE_URL } from "../config";
import {
  mergeEpisodes,
  newestDispatches,
  toCampaignLive,
  toEffectsLive,
  toEpisodes,
  toLive,
  toStatic,
  toStationLive,
  type SnapshotKind,
} from "../domain/snapshot";
import type { Campaign, Dispatch, EpisodeRaw, Planet, SpaceStation, WarStatusRaw } from "./types";

type FetchFn = (url: string, init: RequestInit) => Promise<Response>;

export class SourceError extends Error {
  status: number;
  constructor(status: number, path: string) {
    super(`Supabase ${status} for ${path}`);
    this.status = status;
  }
}

export interface SnapshotRow {
  kind: SnapshotKind;
  data: unknown;
  fetched_at: string;
}

export interface Source {
  /** Snapshots written after `since` (ISO time), or all of them when `since` is empty. */
  snapshots(since: string): Promise<SnapshotRow[]>;
  /** Raw PostgREST select, for history tables. */
  select<T>(table: string, query: string): Promise<T[]>;
  /** A database function's rows, called with its named arguments. */
  rpc<T>(fn: string, args: Record<string, string | number>): Promise<T[]>;
}

const REQUEST_TIMEOUT_MS = 20_000;

export function createSource(opts: { url: string; key: string; fetchFn?: FetchFn }): Source {
  const fetchFn = opts.fetchFn ?? ((u: string, i: RequestInit) => fetch(u, i));
  const base = `${opts.url.replace(/\/$/, "")}/rest/v1`;

  async function select<T>(table: string, query: string): Promise<T[]> {
    const ctrl = new AbortController();
    const timer = setTimeout(() => ctrl.abort(), REQUEST_TIMEOUT_MS);
    try {
      const res = await fetchFn(`${base}/${table}?${query}`, {
        signal: ctrl.signal,
        headers: { apikey: opts.key, Accept: "application/json" },
      });
      if (!res.ok) throw new SourceError(res.status, table);
      return (await res.json()) as T[];
    } finally {
      clearTimeout(timer);
    }
  }

  return {
    snapshots: (since) =>
      select<SnapshotRow>(
        "snapshots",
        `select=kind,data,fetched_at${since ? `&fetched_at=gt.${encodeURIComponent(since)}` : ""}`,
      ),
    select,
    rpc: (fn, args) =>
      select(
        `rpc/${fn}`,
        Object.entries(args)
          .map(([k, v]) => `${k}=${encodeURIComponent(v)}`)
          .join("&"),
      ),
  };
}

/** Dev only: `?fixtures` serves the saved test fixtures, shaped like the collector's snapshots. */
async function fixtureSource(): Promise<Source> {
  const [war, assignments, campaigns, planets, dispatches, stations, episodes, status] = await Promise.all(
    ["war", "assignments", "campaigns", "planets", "dispatches", "space-stations", "episodes", "status-effects"].map((n) =>
      import(`../test/fixtures/${n}.json`).then((m) => m.default),
    ),
  );
  const at = new Date().toISOString();
  const rows: SnapshotRow[] = [
    { kind: "war", data: war, fetched_at: at },
    { kind: "assignments", data: assignments, fetched_at: at },
    { kind: "campaigns", data: (campaigns as Campaign[]).map((c) => toCampaignLive(c, null)), fetched_at: at },
    { kind: "planets", data: (planets as Planet[]).map(toLive), fetched_at: at },
    { kind: "planets_static", data: (planets as Planet[]).map(toStatic), fetched_at: at },
    { kind: "dispatches", data: newestDispatches(dispatches as Dispatch[]), fetched_at: at },
    { kind: "space_stations", data: (stations as SpaceStation[]).map(toStationLive), fetched_at: at },
    { kind: "planet_effects", data: toEffectsLive(status as WarStatusRaw, Date.now()), fetched_at: at },
  ];
  const ep = episodes as { episodes: EpisodeRaw[]; warTime: number; fetchedAt: number };
  rows.push({
    kind: "episodes",
    data: mergeEpisodes([], toEpisodes(ep.episodes, ep.warTime, ep.fetchedAt), 0),
    fetched_at: at,
  });
  return {
    snapshots: async (since) => (since ? [] : rows),
    select: async () => [],
    rpc: async () => [],
  };
}

function defaultSource(): Promise<Source> {
  if (import.meta.env?.DEV && typeof location !== "undefined" && location.search.includes("fixtures")) {
    return fixtureSource();
  }
  // Inside Tauri, requests go through Rust (no CORS, same code on every platform).
  const fetchFn = isTauri() ? (tauriFetch as FetchFn) : undefined;
  return Promise.resolve(createSource({ url: SUPABASE_URL, key: SUPABASE_PUBLISHABLE_KEY, fetchFn }));
}

let source: Promise<Source> | null = null;
export const getSource = () => (source ??= defaultSource());
