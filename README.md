# WarRoom
[![Deploy to GitHub Pages](https://github.com/zearkreast/WarRoom/actions/workflows/pages.yml/badge.svg)](https://github.com/zearkreast/WarRoom/actions/workflows/pages.yml) [![Release](https://github.com/zearkreast/WarRoom/actions/workflows/release.yml/badge.svg)](https://github.com/zearkreast/WarRoom/actions/workflows/release.yml)

A fan-made Helldivers 2 Galactic War companion. Not affiliated with Arrowhead Game Studios.

WarRoom is the project; the app it ships is **WarCommand Tracker** (the name on installers and windows, and
the Android package `com.warcommand.tracker`). It runs on Windows, macOS, Linux, Android and the web.

## How it fits together

```
collector (VDS) ──► api.helldivers2.dev                 the only API caller
      │        └──► api.live.prod.thehelldiversgame.com  (campaigns, planet effects)
      │
      └──► Supabase ◄── app (desktop / mobile / web)     reads only
```

- **App** (`src/`, `src-tauri/`): Leptos, written in Rust and compiled to WebAssembly, inside a Tauri 2 shell
  on desktop and Android; the web version is the same page without the shell.
- **Core** (`crates/core/`): the data shapes and war logic the app and the collector share, so what the
  collector writes is what the app reads.
- **Collector** (`collector/`): a small Rust service that polls the APIs and writes snapshots and history to
  Supabase. See [collector/README.md](collector/README.md) for setting it up on a server.
- **Database** (`supabase/migrations/`): snapshots of the current war, planet progress samples, every
  ownership change, galaxy-wide war samples, each day's busiest and quietest moment, Helldivers on planets
  and in cities outside the fronts, and the space stations' FTL jumps. Shared by all users. Apply the
  migrations in order (`0001` to `0005`).

## Download

Installers are on the [Releases](https://github.com/zearkreast/WarRoom/releases) page: `.exe` or `.msi` for
Windows, `.dmg` for macOS (Apple Silicon and Intel), `.AppImage`, `.deb` or `.rpm` for Linux, and `.apk` for
Android. Once installed, the app offers each new release itself: on desktop it downloads, installs and restarts
in one press; on Android it downloads the new APK for the system to install.

The app collects no personal data; see [PRIVACY.md](PRIVACY.md).

## Repository map

```
src/                     the app
  main.rs                start-up: shared state, polling and alerts, then mounts the UI
  app.rs                 the shell around whichever page is open
  state.rs               what is on screen (Back steps through it) and the settings saved on the device
  api/                   everything read from Supabase: snapshots, history, fronts, station jumps
  alerts/                toasts and OS notifications, and the watcher that raises them
  features/              one folder per page or shared panel (see "Pages" below)
  layout/                the frame: header with the status and ticker, navigation rail (a tab bar on phones), About and legend, briefing popup
  ui/                    shared components: panels, meters, charts, icons, emblems
  update/                finds and installs new releases
  assets/                pictures, fonts and map data, served next to the page; the game's pictures come
                         from the Helldivers Wiki and are credited in About
  js.rs, tauri.rs        bridges to the browser and to the Tauri shell
  nav_history.rs         makes Back and Forward step through the app
src-tauri/               the Tauri shell for desktop and Android (built by the Tauri CLI, own Cargo.lock)
  android/               what the generated Android project gets added (MainActivity, notification icon)
  app-icon.svg           the source of every app icon: cargo tauri icon src-tauri/app-icon.svg
crates/core/             shared data shapes and war logic
  src/data/, data/       game knowledge the APIs don't carry: rewards, effect names, G.W.W. pages, station history
  tests/                 one test file per module, and fixtures/ (saved API answers)
crates/android-project/  patches the generated Android project: manifest entries and Firebase values
collector/               the polling service, with its Dockerfile and systemd unit
supabase/migrations/     the database schema
vendor/tachys/           Leptos's renderer with a fix for static SVG; drop it on the move to Leptos 0.9
index.html               Trunk's entry: the app, every stylesheet, and the assets
build.rs                 lists src/assets, so the app knows which pictures exist
.github/workflows/       release.yml (installers, APK, update manifest), android.yml (the APK),
                         pages.yml (tests, then the web version on GitHub Pages)
```

Where to start reading: `src/main.rs`, then `src/app.rs`, then `src/api/queries.rs` (polling and the data
every page derives from), then a page under `src/features/`. For the war logic, `crates/core/src/lib.rs`
lists the modules. Each file opens with a one-line summary of what it holds, or its main item carries one.

### Pages

| On screen               | `View`      | Code                                                                                |
| ----------------------- | ----------- | ----------------------------------------------------------------------------------- |
| Galaxy map              | `Map`       | `features/map/`, the map itself in `galaxy/`                                        |
| Active planets          | `Campaigns` | `features/campaigns/`                                                               |
| Hellpad                 | `Hellpad`   | `features/hellpad/`: `planets`, `stations/`, `gww/`, `past_campaigns`, `stratagems` |
| Overview (High Command) | `Overview`  | `features/overview/`                                                                |
| Strohmann News          | `News`      | `features/news/`                                                                    |
| Settings                | `Settings`  | `features/settings/`                                                                |

Panels several pages share: `features/major_order/`, `features/galactic_campaigns/`, and `features/planet/`
(the planet detail overlay, cities, charts and the planet options menu).

Each folder keeps its stylesheet beside its code. Every stylesheet sits in a cascade layer, declared in order in
`ui/fonts.css` (`tokens`, `base`, `primitives`, `features`, `shell`), and `index.html` loads them; within a layer,
later files win. `ui/theme.css` holds the tokens: colours, type scale, spacing, the `--cut` chamfer, z-index
levels and the shell's sizes. Class names carry a
short prefix for their area: `cc-` campaign cards and `ms-` the Major Order strip (`campaigns.css`), `mo-`
Major Orders, `hc-` and `map-` the map and its hover card, `pd-` planet detail and `po-` planet options
(`planet.css`), `city-` and `cd-` cities, `hp-` Hellpad, `pi-` its planet index, `st-` space stations,
`pc-` past campaigns, `sg-` the stratagem collection and `cmp-` the Galactic Campaign panel, `gww-` the G.W.W.,
`dr-` its stratagem drill, `ni-` and `news-` Strohmann News, `ov-` Overview, `tw-` the Theater of War, `wa-` War
Activity, `nr-` the neon ring chart, `chart-` charts, and `top-`, `strip-`, `nav-` and `bp-` (briefing popup)
the shell.

### Words used in the code

- **Front**: a planet being fought over right now, a liberation or a defense. The API calls these campaigns,
  hence `View::Campaigns` for the Active planets page.
- **Galactic Campaign**: the game's story arc of three Major Orders; the official API calls it an episode
  (`EpisodeLive`).
- **Major Order**: High Command's current objectives; the API calls them assignments.
- **Dispatch**: a message from Super Earth. Strohmann News is where the app collects them.
- **Hellpad**: the app's field terminal, a hub for the planet index, space stations, the G.W.W., past
  campaigns and the stratagem collection.
- **G.W.W.** (Galactic War Web): an in-universe net of training manuals, ministries, corporations, holidays
  and logs, adapted from the Helldivers Wiki. Pages are drawn in `features/hellpad/gww/`; their text is in
  `crates/core/src/data/`.
- **DSS**: the Democracy Space Station. **SEAF**: Super Earth Armed Forces.
- **Circumvallation**: enemy planets a liberation would leave surrounded by Super Earth (`sieges.rs`).
- **Snapshot**: the collector's latest copy of an API answer, stored in Supabase (`snapshot.rs`).

## Develop

It needs a Rust toolchain (https://rustup.rs), [Trunk](https://trunkrs.dev) and, for the desktop and Android
apps, the Tauri CLI:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
cargo install tauri-cli --version "^2" --locked
trunk serve              # browser only, at http://localhost:1420
cargo tauri dev          # desktop app
cargo test --workspace   # tests: core (crates/core/tests), collector (collector/src/tests.rs) and the app's few
```

The app reads from the Supabase project set in `src/config.rs`. To point a build at another one, set
`WARROOM_SUPABASE_URL` and `WARROOM_SUPABASE_PUBLISHABLE_KEY` in the environment it is built in (see
`.env.example`). The publishable key is read-only by design.

Add `?fixtures` to the browser URL (`http://localhost:1420/?fixtures`) to use the saved API answers in
`crates/core/tests/fixtures/` instead of Supabase. This only works in debug builds, as `trunk serve` makes.
The core and collector tests read the same fixtures.

Android: `cargo tauri android init`, then `cargo tauri android dev`. This needs Android Studio and the SDK.
The Android project (`src-tauri/gen/`) isn't kept in the repo; CI generates it for each build. After `init`,
do what the "Generate the Android project" step of `android.yml` does: copy in the icons, `src-tauri/android/res`
and `MainActivity.kt.template` (over the generated `MainActivity.kt`), run
`cargo run -q -p android-project -- manifest` on `app/src/main/AndroidManifest.xml`, and add Firebase Messaging
to `app/build.gradle.kts`.

To run the collector locally:

```bash
cp collector/.env.example collector/.env   # then fill in the secret key and contact
cargo run --profile collector -p warroom-collector             # runs until stopped
cargo run --profile collector -p warroom-collector -- --once   # every job once, then exits
```

## Pushed news on Android

The collector pushes new Strohmann News (Major Orders, dispatches, campaign launches and outcomes) through
Firebase Cloud Messaging, so the Android app shows it even while closed. To set it up:

1. In the [Firebase console](https://console.firebase.google.com), create a project and add an Android app with
   the package `com.warcommand.tracker`. Download its `google-services.json` and paste the whole file into the
   repository secret `GOOGLE_SERVICES_JSON`; APKs built after that subscribe to the news.
2. Under Project settings → Service accounts, generate a private key and give it to the collector as
   `FCM_SERVICE_ACCOUNT` (see [collector/README.md](collector/README.md)).

Without the secret the APK has no pushes, and notifies only while it is open.

Update the collector before releasing an APK built from this version: its APKs leave the old `news` topic for
one topic per alert setting, and only the updated collector sends to those.

## Release a new version

1. Raise the version in all three places together: `[workspace.package]` in `Cargo.toml`,
   `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json` (`version` and `bundle.windows.wix.version`, which
   must stay numbers only, as MSI installers can't carry a label such as `-beta`).
2. Add the version's entry to `crates/core/src/changelog.rs` (Tracker Dispatch shows it in the app) and to
   `CHANGELOG.md`.
3. Push a tag such as `v0.5.0`, or run Release from the Actions tab.

The Release workflow builds every desktop installer, then the APK (`android.yml`), and publishes them as one
GitHub Release with `latest.json`, which installed apps read to find the update. Keep the release a full one,
not a GitHub pre-release: installed apps only look at the latest full release. It needs these repository secrets:

| Secret                      | What it is                                                     |
| --------------------------- | -------------------------------------------------------------- |
| `TAURI_SIGNING_PRIVATE_KEY` | Signs desktop updates; its public half is the updater `pubkey` |
| `ANDROID_KEYSTORE`          | The APK signing keystore, base64-encoded                       |
| `ANDROID_KEYSTORE_PASSWORD` | The keystore's password                                        |
| `ANDROID_KEY_ALIAS`         | The key's alias in the keystore                                |
| `ANDROID_KEY_PASSWORD`      | The key's password                                             |
| `GOOGLE_SERVICES_JSON`      | Firebase's `google-services.json`, for pushed news (optional)  |

Without the Android secrets each APK is signed with a throwaway key, and a newer APK won't install over an older
one. Losing either signing key means installed apps can no longer take updates from new releases.
