// Rate-limited client for api.helldivers2.dev. The collector is its only caller.

export class RateLimitError extends Error {
  retryAfterMs: number;
  constructor(retryAfterMs: number) {
    super(`Rate limited, retry in ${Math.round(retryAfterMs / 1000)}s`);
    this.retryAfterMs = retryAfterMs;
  }
}

export class ApiError extends Error {
  status: number;
  constructor(status: number, path: string) {
    super(`API ${status} for ${path}`);
    this.status = status;
  }
}

type FetchFn = (url: string, init: RequestInit) => Promise<Response>;

export interface ClientOptions {
  client: string;
  contact: string;
  fetchFn?: FetchFn;
  baseUrl?: string;
  /** Minimum gap between consecutive requests. The API 429s on bursts of ~5. */
  minGapMs?: number;
  now?: () => number;
  sleep?: (ms: number) => Promise<void>;
  timeoutMs?: number;
}

const DEFAULT_RETRY_MS = 30_000;
const REQUEST_TIMEOUT_MS = 25_000;

/**
 * Serializes every API call through one queue, spaces them out, and pauses the whole
 * queue when the server answers 429.
 */
export function createHd2Client(opts: ClientOptions) {
  const fetchFn = opts.fetchFn ?? ((u: string, i: RequestInit) => fetch(u, i));
  const baseUrl = opts.baseUrl ?? "https://api.helldivers2.dev";
  const minGapMs = opts.minGapMs ?? 1500;
  const now = opts.now ?? Date.now;
  const sleep = opts.sleep ?? ((ms: number) => new Promise<void>((r) => setTimeout(r, ms)));
  const timeoutMs = opts.timeoutMs ?? REQUEST_TIMEOUT_MS;

  let chain: Promise<unknown> = Promise.resolve();
  let nextSlot = 0;
  let pausedUntil = 0;

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
        },
      });
    } catch (e) {
      if (ctrl.signal.aborted) throw new ApiError(0, `${path} (timed out)`);
      throw e;
    } finally {
      clearTimeout(timer);
    }
    if (res.status === 429) {
      const header = Number(res.headers.get("Retry-After"));
      const retryMs = Number.isFinite(header) && header > 0 ? header * 1000 : DEFAULT_RETRY_MS;
      pausedUntil = now() + retryMs;
      throw new RateLimitError(retryMs);
    }
    if (!res.ok) throw new ApiError(res.status, path);
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
