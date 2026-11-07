# WarRoom collector

The only part of WarRoom that talks to the Helldivers 2 API. It polls the API and writes everything to
Supabase; the app reads from Supabase and never calls the API itself.

- A small Rust program that shares the app's data shapes and war logic (`crates/core`), so what it writes
  is what the app reads.
- Little memory, near-zero CPU, and about 2 API requests a minute.
- Safe to restart: it reloads its state from Supabase. Run only one at a time, though: a second one stores
  every reading and capture again and sends every push twice, so stop the old one before starting a new one.
- Gentle on the API: one request at a time, 2.5 s apart, a fresh connection for each. A 429 pauses the
  whole queue for its Retry-After; timeouts and server errors pause it for 30 s, doubling up to 5 min.
- If no job has succeeded for 10 minutes it exits, and Docker or systemd starts it again clean.

| Job         | Every  | Writes                                                                                   |
| ----------- | ------ | ---------------------------------------------------------------------------------------- |
| planets     | 5 min  | `planets` + `planets_static` snapshots, samples, ownership changes                       |
| campaigns   | 1 min  | `campaigns` snapshot (with planet and region progress rates), samples, ownership changes |
| war         | 2 min  | `war` snapshot, a `war_samples` row                                                      |
| assignments | 5 min  | `assignments` and `order_samples` (the last day of Major Order progress) snapshots       |
| dispatches  | 5 min  | `dispatches` snapshot (newest 50), only when there's a new one                           |
| stations    | 5 min  | `space_stations` snapshot, a `station_jumps` row when a station has moved                |
| episodes    | 15 min | `episodes` (Galactic Campaigns) and `planet_effects` snapshots, from the game's own API  |
| compact     | 1 hour | thins samples and war samples older than 48 h to one per 15 min                          |

`planet_effects` includes arsenal augmentations, the stratagems High Command grants on a planet for a while,
with when each ends.

## Setup with Docker

The collector is published as a ready-built image, `ghcr.io/zearkreast/warroom-collector`, for amd64 and arm64
(a Raspberry Pi or an Arm server runs it too). A server needs only Docker: no clone of the repo and no Rust
build. The image holds no keys; they stay in a `.env` file beside it.

1. Make a folder for it and fetch the config template:

   ```bash
   mkdir warroom-collector && cd warroom-collector
   curl -fsSL https://raw.githubusercontent.com/zearkreast/WarRoom/main/collector/.env.example -o .env
   nano .env
   ```

   Fill in `SUPABASE_URL`, `SUPABASE_SECRET_KEY` (Supabase → Project Settings → API Keys → Secret keys) and
   `SUPER_CONTACT` (an email or link the Helldivers API maintainers can reach you at).

2. Save this as `compose.yml` in the same folder:

   ```yaml
   services:
     collector:
       image: ghcr.io/zearkreast/warroom-collector:${COLLECTOR_TAG:-latest}
       env_file: .env
       restart: unless-stopped
       mem_limit: 128m
       logging:
         driver: json-file
         options:
           max-size: "1m"
           max-file: "3"
   ```

3. Apply the database schema (see [Supabase](#supabase)), then start it:

   ```bash
   docker compose up -d
   docker compose logs -f collector
   ```

   It logs `started (… planets known, pushes off)`, then a line of job results each minute; `FAIL` marks a job
   that didn't succeed.

The container is capped at 128 MB and its logs at 3 MB.

### Updating

`latest` is the newest release. Each release also has its own tag, such as `0.7.4`, and `0.7` follows the
newest of that line.

```bash
docker compose pull && docker compose up -d
```

Read the release's notes first: when it adds a migration, apply it in Supabase before updating.

- Run a given release, for instance to go back one: `COLLECTOR_TAG=0.7.3 docker compose up -d`. It stays on
  that release until started again without `COLLECTOR_TAG`.
- Stop: `docker compose down`

### Building it yourself

For a change of your own or a branch, compile the image from a clone. The repo's `docker-compose.yml` is the
file above with a `build:` section, reading `collector/.env`:

```bash
git clone https://github.com/zearkreast/WarRoom.git && cd WarRoom
cp collector/.env.example collector/.env && nano collector/.env
docker compose up -d --build
```

The first build compiles the collector inside Docker and takes a few minutes. Update with
`git pull && docker compose up -d --build`; a `docker compose pull` there swaps back to the published image.

## Pushed news (optional)

With `FCM_SERVICE_ACCOUNT` set, the collector also pushes new Strohmann News to the Android app through Firebase
Cloud Messaging: each new Major Order, the newest three new dispatches, and campaign launches and outcomes. Set
up the Firebase project first (see "Pushed news on Android" in [docs/DEVELOPMENT.md](../docs/DEVELOPMENT.md)), then:

1. Firebase console → Project settings → Service accounts → Generate new private key.
2. `base64 -w0 key.json` and put the output in the `.env` file as `FCM_SERVICE_ACCOUNT=…`, then restart the
   collector. Its start-up line says `pushes on`.

What is already stored counts as announced, so a restart, or turning pushes on, sends nothing old. A push that
fails is logged and not retried.

Orders and campaign news go to one topic and dispatches to another, so the app's alert settings decide what a
phone receives; older APKs on the single `news` topic still get everything. Update the collector before
releasing an APK built from the same version, or new installs receive nothing until it is updated.

## Setup without Docker

Needs a Rust toolchain (https://rustup.rs). From the repo root:

```bash
cargo run --profile collector -p warroom-collector             # runs until stopped
cargo run --profile collector -p warroom-collector -- --once   # every job once, then exits; non-zero exit on failure
```

To keep it running with systemd, build it with `cargo build --profile collector -p warroom-collector`, edit
the paths and user in `collector/warroom-collector.service`, then:

```bash
sudo cp collector/warroom-collector.service /etc/systemd/system/
sudo systemctl enable --now warroom-collector
journalctl -u warroom-collector -f
```

## Supabase

The schema is in [`supabase/migrations/`](../supabase/migrations), applied in order in the Supabase SQL Editor. The secret key bypasses row-level security, so it
only ever lives in the collector's `.env` file on the server. The app uses the publishable key, which can only read.

Region rates are measured from readings kept in memory, so after a restart they read as measuring for about
10 minutes. Major Order readings live in the `order_samples` snapshot and carry over a restart.

The `episodes` snapshot keeps every campaign it has seen, so ended campaigns stay even if the game API stops
listing them. A Major Order is dated when the collector first sees it on a running campaign; older ones take
the date of their announcement.

After pulling a version that adds a migration, apply it in Supabase before restarting the collector; a missing
table makes that one job fail, the others keep running. The tables of `0005_activity.sql` (each day's peak and
low, Helldivers on planets without a campaign, city readings) are the exception: until it is applied those
writes are skipped and the jobs carry on.

On the free plan, the history stays under the 500 MB limit: about 140 MB a year, plus about 125 MB for the history
imported from before the collector first ran. Free projects pause after a week without activity; a running
collector keeps it active.

## Source

The collector shares its data shapes and war logic with the app through `crates/core`; what is here is the
polling, storing and pushing.

- `src/main.rs`: config, the loop that runs each job when it is due, and the exit when nothing succeeds.
- `src/tasks.rs`: the jobs, one method each, and what each writes.
- `src/hd2.rs`: the API client: one queue, 2.5 s between requests, back-off on errors and 429s.
- `src/http.rs`: the HTTP transport, behind a trait the tests answer in place of the network.
- `src/language.rs`: spots game API answers cached for a player in another language.
- `src/recorder.rs`: planet samples and ownership changes, reloaded at start and written each poll.
- `src/supabase.rs`: the few Supabase REST calls the collector makes.
- `src/push.rs`: Firebase Cloud Messaging.
- `src/tests.rs`: the jobs end to end, against a stand-in for Supabase and the saved API answers in
  `crates/core/tests/fixtures/`.
