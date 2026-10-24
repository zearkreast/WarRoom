// WarRoom collector: polls the Helldivers 2 API and writes to Supabase.
//
//   node collector/collect.ts          run forever
//   node collector/collect.ts --once   run every job once and exit (non-zero on failure)
//
// Config comes from the environment, or from collector/.env when present.

import { existsSync } from "node:fs";
import { join } from "node:path";
import { createTasks, runTask, type Task } from "./collector.ts";
import { createHd2Client } from "./hd2.ts";
import { Recorder } from "./recorder.ts";
import { createSupabase } from "./supabase.ts";

const TICK_MS = 5_000;
/** With no job succeeding for this long, exit so Docker or systemd starts a clean process. */
const STALL_MS = 10 * 60_000;

const stamp = () => new Date().toISOString().slice(11, 19);
const log = (msg: string) => console.log(`[${stamp()}] ${msg}`);

async function main(): Promise<number> {
  const envFile = join(import.meta.dirname, ".env");
  if (existsSync(envFile)) process.loadEnvFile(envFile);

  const env = (name: string) => process.env[name]?.trim() ?? "";
  const missing = ["SUPABASE_URL", "SUPABASE_SECRET_KEY", "SUPER_CONTACT"].filter((n) => !env(n));
  if (missing.length) {
    console.error(`Missing ${missing.join(", ")}. Copy collector/.env.example to collector/.env and fill it in.`);
    return 2;
  }

  const once = process.argv.includes("--once");
  const db = createSupabase({ url: env("SUPABASE_URL"), key: env("SUPABASE_SECRET_KEY") });
  const hd2 = createHd2Client({ client: env("SUPER_CLIENT") || "warcommand-tracker", contact: env("SUPER_CONTACT") });
  const game = createHd2Client({
    client: env("SUPER_CLIENT") || "warcommand-tracker",
    contact: env("SUPER_CONTACT"),
    baseUrl: "https://api.live.prod.thehelldiversgame.com",
  });
  const recorder = new Recorder(db);

  await recorder.load();
  log(`started (${recorder.knownPlanets} planets known)${once ? ", single run" : ""}`);
  const tasks = createTasks(hd2, db, recorder, Date.now, game);

  if (once) {
    let failed = false;
    for (const t of tasks) {
      const r = await runTask(t);
      log(`${r.ok ? "ok  " : "FAIL"} ${r.name}: ${r.message}`);
      failed ||= !r.ok;
    }
    return failed ? 1 : 0;
  }

  // Everything is due at start; each job then waits its own interval.
  const due = new Map<Task, number>(tasks.map((t) => [t, 0]));
  let stopping = false;
  let wake = () => {};
  for (const sig of ["SIGINT", "SIGTERM"] as const) {
    process.on(sig, () => {
      stopping = true;
      log(`${sig} received, stopping after the current job`);
      wake();
    });
  }

  let lastOk = Date.now();
  while (!stopping) {
    const parts: string[] = [];
    for (const t of tasks) {
      if (stopping || Date.now() < due.get(t)!) continue;
      const r = await runTask(t);
      if (r.ok) lastOk = Date.now();
      due.set(t, r.retryAt ?? Date.now() + t.everyMs);
      parts.push(`${r.ok ? "" : "FAIL "}${r.name}: ${r.message}`);
    }
    if (parts.length) log(parts.join(" | "));
    if (Date.now() - lastOk > STALL_MS) {
      log(`no job has succeeded for ${STALL_MS / 60_000} minutes, exiting for a clean restart`);
      return 1;
    }
    await new Promise<void>((r) => {
      const timer = setTimeout(r, TICK_MS);
      wake = () => {
        clearTimeout(timer);
        r();
      };
    });
  }
  return 0;
}

// Set the exit code instead of calling process.exit(), so pending output is flushed.
process.exitCode = await main().catch((e) => {
  console.error(`[${stamp()}] fatal:`, e instanceof Error ? e.message : e);
  return 1;
});
