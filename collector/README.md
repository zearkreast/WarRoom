# WarRoom collector

The only part of WarRoom that talks to the Helldivers 2 API. It polls the API and writes everything to
Supabase; the app reads from Supabase and never calls the API itself.

- No npm dependencies and no build step: Node runs the TypeScript sources directly.
- About 40–60 MB of RAM, near-zero CPU, and about 2 API requests a minute.
- Safe to restart: it reloads its state from Supabase. Running two at once is harmless too (duplicate
  readings are ignored).
- Gentle on the API: one request at a time, 2.5 s apart, a fresh connection for each. A 429 pauses the
  whole queue for its Retry-After; timeouts and server errors pause it for 30 s, doubling up to 5 min.
- If no job has succeeded for 10 minutes it exits, and Docker or systemd starts it again clean.

| Job         | Every  | Writes                                                                                     |
| ----------- | ------ | ------------------------------------------------------------------------------------------ |
| planets     | 5 min  | `planets` + `planets_static` snapshots, samples, ownership changes                         |
| campaigns   | 1 min  | `campaigns` snapshot (with planet and region progress rates), samples, ownership changes   |
| war         | 2 min  | `war` snapshot, a `war_samples` row                                                        |
| assignments | 5 min  | `assignments` and `order_samples` (the last day of Major Order progress) snapshots         |
| dispatches  | 5 min  | `dispatches` snapshot (newest 50), only when there's a new one                             |
| stations    | 5 min  | `space_stations` snapshot, a `station_jumps` row when a station has moved                  |
| episodes    | 15 min | `episodes` (Galactic Campaigns) and `planet_effects` snapshots, from the official game API |
| compact     | 1 hour | thins samples and war samples older than 48 h to one per 15 min                            |

## Setup on a VDS (Docker)

```bash
git clone https://github.com/zearkreast/WarRoom.git && cd WarRoom
cp collector/.env.example collector/.env && nano collector/.env
docker compose up -d --build
```

Fill in `SUPABASE_SECRET_KEY` (Supabase → Project Settings → API Keys → Secret keys) and `SUPER_CONTACT`
(an email or link the Helldivers API maintainers can reach you at).

- Logs: `docker compose logs -f collector`
- Update: `git pull && docker compose up -d --build`
- Stop: `docker compose down`

The container is capped at 128 MB and its logs at 3 MB.

## Setup without Docker

Needs Node 22.18+ or 24.

```bash
node collector/collect.ts           # runs until stopped
node collector/collect.ts --once    # every job once, then exits; non-zero exit on failure
```

To keep it running with systemd, edit the paths and user in `collector/warroom-collector.service`, then:

```bash
sudo cp collector/warroom-collector.service /etc/systemd/system/
sudo systemctl enable --now warroom-collector
journalctl -u warroom-collector -f
```

## Supabase

The schema is in `supabase/migrations/`, applied in order. The secret key bypasses row-level security, so it
only ever lives in `collector/.env` on the server. The app uses the publishable key, which can only read.

Region rates are measured from readings kept in memory, so after a restart they read as measuring for about
10 minutes. Major Order readings live in the `order_samples` snapshot and carry over a restart.

The `episodes` snapshot keeps every campaign it has seen, so ended campaigns stay even if the game API stops
listing them. A Major Order is dated when the collector first sees it on a running campaign; older ones take
the date of their announcement.

After pulling a version that adds a migration, apply it in Supabase before restarting the collector; a missing
table makes that one job fail, the others keep running.

On the free plan, the history stays well under the 500 MB limit (about 140 MB a year). Free projects pause
after a week without activity; a running collector keeps it active.
