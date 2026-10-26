# WarRoom
[![Deploy to GitHub Pages](https://github.com/zearkreast/WarRoom/actions/workflows/pages.yml/badge.svg)](https://github.com/zearkreast/WarRoom/actions/workflows/pages.yml) [![Release](https://github.com/zearkreast/WarRoom/actions/workflows/release.yml/badge.svg)](https://github.com/zearkreast/WarRoom/actions/workflows/release.yml)

A fan-made Helldivers 2 Galactic War companion. Not affiliated with Arrowhead Game Studios.

```
collector (VDS) ──► api.helldivers2.dev                 the only API caller
      │        └──► api.live.prod.thehelldiversgame.com  (campaigns, planet effects)
      │
      └──► Supabase ◄── app (desktop / mobile / web)     reads only
```

- **App** (`src/`, `src-tauri/`): Tauri 2 + Leptos, in Rust compiled to WebAssembly, for Windows, macOS, Linux,
  Android and the web.
- **Core** (`crates/core/`): the data shapes and war logic the app and the collector share.
- **Collector** (`collector/`): a Rust service that polls the API and writes to Supabase.
  See [collector/README.md](collector/README.md) for VDS setup.
- **Database** (`supabase/migrations/`): snapshots of the current war, planet progress samples, every
  ownership change, galaxy-wide war samples, each day's busiest and quietest moment, Helldivers on planets
  and in cities outside the fronts, and the space station's FTL jumps. Shared by all users.
  Apply the migrations in order (`0001` to `0005`).

## Download

Installers are on the [Releases](https://github.com/zearkreast/WarRoom/releases) page: `.exe` or `.msi` for
Windows, `.dmg` for macOS (Apple Silicon and Intel), `.AppImage`, `.deb` or `.rpm` for Linux, and `.apk` for
Android. Once installed, the app offers each new release itself: on desktop it downloads, installs and restarts
in one press; on Android it downloads the new APK for the system to install.

The app collects no personal data; see [PRIVACY.md](PRIVACY.md).

## Run the app

It needs a Rust toolchain (https://rustup.rs), [Trunk](https://trunkrs.dev) and, for the desktop and Android
apps, the Tauri CLI:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
cargo install tauri-cli --version "^2" --locked
cargo tauri dev          # desktop app
trunk serve              # browser only, at http://localhost:1420
cargo test --workspace   # unit tests (app, core and collector)
```

The app reads from the Supabase project set in `src/config.rs`. To point a build at another one, set
`WARROOM_SUPABASE_URL` and `WARROOM_SUPABASE_PUBLISHABLE_KEY` in the environment it is built in (see
`.env.example`). The publishable key is read-only by design.

Add `?fixtures` to the browser URL (`http://localhost:1420/?fixtures`) to use the saved snapshots in
`crates/core/tests/fixtures/` instead of Supabase. This only works in debug builds, as `trunk serve` makes.

Android: `cargo tauri android init`, then `cargo tauri android dev`. This needs Android Studio and the SDK.
The Android project (`src-tauri/gen/`) isn't kept in the repo; CI generates it for each build. After `init`,
do what the "Generate the Android project" step of `android.yml` does: copy in the icons, `src-tauri/android/res`
and `MainActivity.kt.template` (over the generated `MainActivity.kt`), run
`cargo run -q -p android-project -- manifest` on `app/src/main/AndroidManifest.xml`, and add Firebase Messaging
to `app/build.gradle.kts`.

## Pushed news on Android

The collector pushes new Strohmann News (Major Orders, dispatches, campaign launches and outcomes) through
Firebase Cloud Messaging, so the Android app shows it even while closed. To set it up:

1. In the [Firebase console](https://console.firebase.google.com), create a project and add an Android app with
   the package `com.warcommand.tracker`. Download its `google-services.json` and paste the whole file into the
   repository secret `GOOGLE_SERVICES_JSON`; APKs built after that subscribe to the news.
2. Under Project settings → Service accounts, generate a private key and give it to the collector as
   `FCM_SERVICE_ACCOUNT` (see [collector/README.md](collector/README.md)).

Without the secret the APK has no pushes, and notifies only while it is open.

## Release a new version

1. Raise the version in all three places together: `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` and
   `[workspace.package]` in `Cargo.toml`. A pre-release such as `0.3.0-beta` also needs a numbers-only
   `bundle.windows.wix.version` in `tauri.conf.json`, since MSI installers can't carry the suffix.
2. Add the version's entry to `crates/core/src/changelog.rs`; Tracker Dispatch shows it.
3. Push a tag such as `v0.3.0-beta`, or run Release from the Actions tab.

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

## Run the collector locally

```bash
cp collector/.env.example collector/.env   # then fill in the secret key and contact
cargo run --profile collector -p warroom-collector             # runs until stopped
cargo run --profile collector -p warroom-collector -- --once   # every job once, then exits
```

## Layout

- `collector/`: API client (one queue, 2.5 s between requests, a fresh connection per request, backs
  off while the API is unreachable or rate-limiting), recorder, polling jobs, Docker and systemd files.
- `crates/core/src/`: pure logic shared by the app and the collector: liberation rates and ETAs, Major Order
  tasks, galaxy sector geometry, planet effects, campaign news, owner-change detection, the snapshot format,
  and the chart and globe maths the app draws with. Covered by tests in `crates/core/tests/`.
- `crates/core/src/data/` and `crates/core/data/`: hand-kept data the APIs don't carry: campaign rewards,
  effect names (from the community list at github.com/helldivers-2/json) and overrides, scorched planets, past
  space stations, and the G.W.W. manuals and corporations.
- `src/assets/`: biome landscapes, faction and sub-faction emblems, holograms, station and point-of-interest
  pictures; the game's own come from the Helldivers Wiki and are credited in About. `build.rs` lists them so
  the app knows which exist.
- `src/api/`: reads snapshots from Supabase and rebuilds them into the planet and campaign shapes the UI uses.
- `src/history/`: reads the shared sample and ownership history.
- `src/features/`: overview (High Command), map, campaigns (Active planets), galactic campaigns, hellpad
  (planets, space stations, G.W.W., past campaigns), planet detail, news, settings. Each keeps its styles in a
  `.css` file beside it; `index.html` loads them in order.
- `src/layout/`: the shell: top bar, status strip with the news ticker, the dock, the About and legend
  dialogs, Tracker Dispatch, and the briefing popup alerts open into.
- `src/update/`: looks for a newer release and installs it (the Tauri updater on desktop; on Android the
  release's APK, downloaded in the app and handed to Android's installer).
- `vendor/tachys/`: Leptos's renderer with a fix for static SVG it otherwise wraps in a stray `<g>`; drop it on
  the move to Leptos 0.9.
- `src-tauri/app-icon.svg`: the source of every app icon; regenerate them with
  `cargo tauri icon src-tauri/app-icon.svg`.
- `src-tauri/android/`: what the generated Android project gets added: a `MainActivity.kt.template` that keeps
  the page between the status and navigation bars, subscribes to pushed news and downloads and installs update
  APKs, and the notification icon.
- `crates/android-project/`: adds to the generated Android project the manifest entries the in-app update
  needs (the install permission and file provider), and turns `google-services.json` into the resources
  Firebase starts from.
- `.github/workflows/`: `release.yml` (installers, APK and update manifest), `android.yml` (the APK, also built
  on pushes that touch `src-tauri/` or `crates/android-project/`) and `pages.yml` (tests, then the web version
  on GitHub Pages).
