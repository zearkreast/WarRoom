import { describe, expect, it } from "vitest";
import { createHd2Client, RateLimitError, type ClientOptions } from "./hd2.ts";

const createClient = (o: Omit<ClientOptions, "client" | "contact">) =>
  createHd2Client({ client: "warcommand-tracker", contact: "test", ...o });

function fakeClock() {
  let t = 0;
  return {
    now: () => t,
    sleep: async (ms: number) => {
      t += ms;
    },
    advance: (ms: number) => {
      t += ms;
    },
  };
}

const json = (body: unknown, status = 200, headers: Record<string, string> = {}) =>
  new Response(JSON.stringify(body), { status, headers });

describe("api client", () => {
  it("sends the required headers", async () => {
    let seen: Headers | undefined;
    const c = createClient({
      baseUrl: "https://x",
      fetchFn: async (_u, init) => {
        seen = new Headers(init.headers);
        return json({ ok: 1 });
      },
    });
    await c.get("/api/v1/war");
    expect(seen?.get("X-Super-Client")).toBe("warcommand-tracker");
    expect(seen?.get("X-Super-Contact")).toBeTruthy();
  });

  it("spaces requests out", async () => {
    const clock = fakeClock();
    const times: number[] = [];
    const c = createClient({
      baseUrl: "",
      minGapMs: 1500,
      ...clock,
      fetchFn: async () => {
        times.push(clock.now());
        return json([]);
      },
    });
    await Promise.all([c.get("/a"), c.get("/b"), c.get("/c")]);
    expect(times).toEqual([0, 1500, 3000]);
  });

  it("times out a hung request so the queue keeps moving", async () => {
    let calls = 0;
    const c = createClient({
      baseUrl: "",
      minGapMs: 0,
      timeoutMs: 20,
      fetchFn: (_u, init) =>
        calls++ === 0
          ? new Promise((_, reject) => init.signal!.addEventListener("abort", () => reject(new Error("aborted"))))
          : Promise.resolve(json({ ok: 1 })),
    });
    await expect(c.get("/hang")).rejects.toThrow(/timed out/);
    await expect(c.get("/next")).resolves.toEqual({ ok: 1 });
  });

  it("pauses the whole queue after a 429 using Retry-After", async () => {
    const clock = fakeClock();
    const times: number[] = [];
    let calls = 0;
    const c = createClient({
      baseUrl: "",
      minGapMs: 0,
      ...clock,
      fetchFn: async () => {
        times.push(clock.now());
        return calls++ === 0 ? json(null, 429, { "Retry-After": "20" }) : json({ ok: 1 });
      },
    });
    await expect(c.get("/a")).rejects.toBeInstanceOf(RateLimitError);
    await c.get("/b");
    expect(times).toEqual([0, 20_000]);
  });
});
