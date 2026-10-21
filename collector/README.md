# WarRoom collector

The only part of WarRoom that talks to the Helldivers 2 API. It polls the API and writes everything to
Supabase; the app reads from Supabase and never calls the API itself.

- No npm dependencies and no build step: Node runs the TypeScript sources directly.
- About 40–60 MB of RAM, near-zero CPU, and about 5 API requests a minute.
- Safe to restart: it reloads its state from Supabase. Running two at once is harmless too (duplicate
  readings are ignored).

| Job | Every | Writes |
|---|---|---|
| planets | 3 min | `planets` + `planets_static` snapshots, samples, ownership changes |
| campaigns | 1 min | `campaigns` snapshot (with progress rates), samples, ownership changes |
| war, assignments | 1 min | `war`, `assignments` snapshots |
| dispatches | 5 min | `dispatches` snapshot (newest 50), only when there's a new one |
| compact | 1 hour | thins samples older than 48 h to one per 15 min |

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

The schema is in `supabase/migrations/0001_init.sql`. The secret key bypasses row-level security, so it
only ever lives in `collector/.env` on the server. The app uses the publishable key, which can only read.

On the free plan, the history stays well under the 500 MB limit (about 140 MB a year). Free projects pause
after a week without activity; a running collector keeps it active.
