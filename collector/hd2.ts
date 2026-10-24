// Rate-limited client for api.helldivers2.dev and the official game API. The collector is its only caller.

import { request } from "node:https";

export class RateLimitError extends Error {
  retryAfterMs: number;
  constructor(retryAfterMs: number) {
    super(`Rate limited, retry in ${Math.round(retryAfterMs / 1000)}s`);
    this.retryAfterMs = retryAfterMs;
  }
}

export class ApiError extends Error {
  status: number;
  /** Set when the whole queue backs off after this failure; the job should not retry sooner. */
  retryAfterMs?: number;
  constructor(status: number, path: string, retryAfterMs?: number) {
    super(`API ${status} for ${path}`);
    this.status = status;
    this.retryAfterMs = retryAfterMs;
  }
}

type FetchFn = (url: string, init: RequestInit) => Promise<Response>;

export interface ClientOptions {
  client: string;
  contact: string;
  fetchFn?: FetchFn;
  baseUrl?: string;
  /** Minimum gap between consecutive requests. The community API allows 5 requests per 10 s. */
  minGapMs?: number;
  /** First pause after a timeout or network failure; doubles on each further one, up to MAX_BACKOFF_MS. */
  backoffMs?: number;
  now?: () => number;
  sleep?: (ms: number) => Promise<void>;
  timeoutMs?: number;
}

const DEFAULT_RETRY_MS = 30_000;
const REQUEST_TIMEOUT_MS = 25_000;
const MAX_BACKOFF_MS = 300_000;
/** Wait once X-RateLimit-Remaining reaches 0. */
const WINDOW_MS = 10_000;

/**
 * fetch over node:https with a fresh socket per request: Node's pooled fetch kept reusing a dead
 * keep-alive socket, so every call timed out until a restart.
 */
export const freshSocketFetch: FetchFn = (url, init) =>
  new Promise((resolve, reject) => {
    const req = request(
      url,
      { method: "GET", agent: false, headers: init.headers as Record<string, string>, signal: init.signal ?? undefined },
      (res) => {
        const chunks: Buffer[] = [];
        res.on("data", (c: Buffer) => chunks.push(c));
        res.on("end", () => {
          const headers = new Headers();
          for (const [k, v] of Object.entries(res.headers)) if (typeof v === "string") headers.set(k, v);
          resolve(new Response(Buffer.concat(chunks), { status: res.statusCode ?? 0, headers }));
        });
        res.on("error", reject);
      },
    );
    req.on("error", reject);
    req.end();
  });

/**
 * Serializes every API call through one queue, spaces them out, and pauses the whole queue when the
 * server answers 429 or stops answering.
 */
export function createHd2Client(opts: ClientOptions) {
  const fetchFn = opts.fetchFn ?? freshSocketFetch;
  const baseUrl = opts.baseUrl ?? "https://api.helldivers2.dev";
  const minGapMs = opts.minGapMs ?? 2500;
  const firstBackoff = opts.backoffMs ?? DEFAULT_RETRY_MS;
  const now = opts.now ?? Date.now;
  const sleep = opts.sleep ?? ((ms: number) => new Promise<void>((r) => setTimeout(r, ms)));
  const timeoutMs = opts.timeoutMs ?? REQUEST_TIMEOUT_MS;

  let chain: Promise<unknown> = Promise.resolve();
  let nextSlot = 0;
  let pausedUntil = 0;
  let backoff = 0;

  const pause = (ms: number) => {
    pausedUntil = Math.max(pausedUntil, now() + ms);
  };
  /** A server that has gone quiet gets ever longer breaks instead of every job retrying into it. */
  const failed = (status: number, path: string): never => {
    backoff = backoff ? Math.min(backoff * 2, MAX_BACKOFF_MS) : firstBackoff;
    pause(backoff);
    throw new ApiError(status, path, backoff);
  };

  async function run<T>(path: string): Promise<T> {
    const wait = Math.max(nextSlot, pausedUntil) - now();
    if (wait > 0) await sleep(wait);
    nextSlot = now() + minGapMs;

    // Requests are serialized, so a hung one would stall every other endpoint without a timeout.
    const ctrl = new AbortController();
    const timer = setTimeout(() => ctrl.abort(), timeoutMs);
    let res: Response;
    try {
      res = await fetchFn(baseUrl + path, {
        signal: ctrl.signal,
        headers: {
          "X-Super-Client": opts.client,
          "X-Super-Contact": opts.contact,
          Accept: "application/json",
          // The official game API rejects requests without it.
          "Accept-Language": "en-US",
        },
      });
    } catch {
      return failed(0, ctrl.signal.aborted ? `${path} (timed out)` : `${path} (unreachable)`);
    } finally {
      clearTimeout(timer);
    }
    if (res.status === 429) {
      await res.body?.cancel();
      const header = Number(res.headers.get("Retry-After"));
      const retryMs = Number.isFinite(header) && header > 0 ? header * 1000 : DEFAULT_RETRY_MS;
      pause(retryMs);
      throw new RateLimitError(retryMs);
    }
    if (!res.ok) {
      await res.body?.cancel();
      if (res.status >= 500) return failed(res.status, path);
      throw new ApiError(res.status, path);
    }
    backoff = 0;
    if (res.headers.get("X-RateLimit-Remaining") === "0") pause(WINDOW_MS);
    return (await res.json()) as T;
  }

  return {
    get<T>(path: string): Promise<T> {
      const p = chain.then(() => run<T>(path));
      chain = p.catch(() => undefined);
      return p;
    },
  };
}

export type Hd2Client = ReturnType<typeof createHd2Client>;
