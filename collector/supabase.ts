// Minimal Supabase REST (PostgREST) client: just the calls the collector makes.

type FetchFn = (url: string, init: RequestInit) => Promise<Response>;

export interface SupabaseOptions {
  url: string;
  /** The project's secret key (sb_secret_…). It bypasses RLS, so it stays on the server. */
  key: string;
  fetchFn?: FetchFn;
}

export function createSupabase(opts: SupabaseOptions) {
  const fetchFn = opts.fetchFn ?? ((u: string, i: RequestInit) => fetch(u, i));
  const base = `${opts.url.replace(/\/$/, "")}/rest/v1`;

  async function call(method: string, path: string, body?: unknown, prefer?: string): Promise<unknown> {
    const headers: Record<string, string> = { apikey: opts.key, "Content-Type": "application/json" };
    if (prefer) headers.Prefer = prefer;
    const res = await fetchFn(`${base}/${path}`, {
      method,
      headers,
      body: body === undefined ? undefined : JSON.stringify(body),
    });
    if (!res.ok) throw new Error(`Supabase ${method} ${path.split("?")[0]} → ${res.status}: ${await res.text()}`);
    const text = await res.text();
    return text ? JSON.parse(text) : null;
  }

  return {
    select<T>(table: string, query = ""): Promise<T[]> {
      return call("GET", `${table}?${query}`) as Promise<T[]>;
    },
    insert(table: string, rows: unknown[]) {
      return rows.length ? call("POST", table, rows, "return=minimal") : Promise.resolve(null);
    },
    /** Insert; on a primary-key (or `onConflict`) clash, update the row, or skip it with `ignoreDuplicates`. */
    upsert(table: string, rows: unknown[], opt: { onConflict?: string; ignoreDuplicates?: boolean } = {}) {
      if (!rows.length) return Promise.resolve(null);
      const q = opt.onConflict ? `?on_conflict=${opt.onConflict}` : "";
      const resolution = opt.ignoreDuplicates ? "ignore-duplicates" : "merge-duplicates";
      return call("POST", `${table}${q}`, rows, `resolution=${resolution},return=minimal`);
    },
    rpc<T>(fn: string, args: Record<string, unknown> = {}): Promise<T> {
      return call("POST", `rpc/${fn}`, args) as Promise<T>;
    },
  };
}

export type Supabase = ReturnType<typeof createSupabase>;

/** A table or function the database doesn't have yet: its migration hasn't been applied. */
export const isMissing = (e: unknown): boolean => e instanceof Error && / → 404:/.test(e.message);

/** Runs a write that needs a newer migration; until it's applied, the write is skipped instead of failing its task. */
export async function unlessMissing(write: Promise<unknown>): Promise<boolean> {
  try {
    await write;
    return true;
  } catch (e) {
    if (isMissing(e)) return false;
    throw e;
  }
}
