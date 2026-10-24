# WarRoom

A fan-made Helldivers 2 Galactic War companion. Not affiliated with Arrowhead Game Studios.

```
collector (VDS) ──► api.helldivers2.dev                 the only API caller
      │        └──► api.live.prod.thehelldiversgame.com  (campaigns, planet effects)
      │
      └──► Supabase ◄── app (desktop / mobile / web)     reads only
```

- **App** (`src/`, `src-tauri/`): Tauri 2 + React + TypeScript, for Windows/macOS/Linux and Android/iOS.
- **Collector** (`collector/`): a dependency-free Node service that polls the API and writes to Supabase.
  See [collector/README.md](collector/README.md) for VDS setup.
- **Database** (`supabase/migrations/`): snapshots of the current war, planet progress samples, every
  ownership change, galaxy-wide war samples and the space station's FTL jumps. Shared by all users.
  Apply the migrations in order (`0001` to `0004`).

## Run the app

```bash
npm install
npm run tauri dev        # desktop app
npm run dev              # browser only, at http://localhost:1420
npm test                 # unit tests (app + collector)
```

The app reads from the Supabase project set in `src/config.ts` (override with `VITE_SUPABASE_URL` and
`VITE_SUPABASE_PUBLISHABLE_KEY`; see `.env.example`). The publishable key is read-only by design.

Add `?fixtures` to the browser URL (`http://localhost:1420/?fixtures`) to use the saved snapshots in
`src/test/fixtures/` instead of Supabase. This only works in dev mode.

Android: `npm run tauri android init`, then `npm run tauri android dev`. This needs Android Studio and the SDK.
iOS builds need a Mac.

## Run the collector locally

```bash
cp collector/.env.example collector/.env   # then fill in the secret key and contact
npm run collect          # runs until stopped
npm run collect:once     # every job once, then exits
```

## Layout

- `collector/`: API client (one queue, 2.5 s between requests, a fresh connection per request, backs
  off while the API is unreachable or rate-limiting), recorder, polling jobs, Docker and systemd files.
- `src/domain/`: pure logic shared by the app and the collector: liberation rates and ETAs, Major Order
  tasks, galaxy sector geometry, planet effects, campaign news, owner-change detection, and the snapshot
  format. Covered by tests.
- `src/data/`: hand-kept data the APIs don't carry: campaign rewards, effect names (from the community
  list at github.com/helldivers-2/json) and overrides, scorched planets, past space stations, and the
  G.W.W. manuals and corporations.
- `src/assets/biomes/`: one landscape banner per biome, from the Helldivers Wiki.
- `src/api/`: reads snapshots from Supabase and rebuilds them into the planet and campaign shapes the UI uses.
- `src/history/`: reads the shared sample and ownership history.
- `src/features/`: overview (High Command), map, campaigns (Active planets), galactic-campaigns, hellpad
  (planets, space stations, G.W.W., past campaigns), planet detail, news, settings.
- `src/layout/`: the shell: top bar, status strip with the news ticker, the dock, the About and legend
  dialogs, Tracker Dispatch.

## Notes

- Planet `index` is not the array position: the list has gaps. Always look planets up with `byIndex()`.
- Several planets share position (0,0), so sector geometry dedupes sites.
- Files in `src/domain/` must only use type imports from outside the folder: the collector runs them
  directly under Node.
- Free Supabase plan: about 140 MB of history a year against a 500 MB limit; download traffic is the
  tighter limit with many users. Pro ($25/month) is the upgrade path.
