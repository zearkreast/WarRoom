# Changelog

What changed in each release of WarCommand Tracker. Tracker Dispatch in the app shows a shorter version of
each entry, from `crates/core/src/changelog.rs`.

## [Unreleased]

### Upgrading

- Redeploy the collector (`git pull && docker compose up -d --build`) before releasing an APK from this version:
  new APKs follow one push topic per alert setting, and only the updated collector sends to them. There are no
  new database migrations.

### New look

- The app takes a Super Earth Command look: warm near-black ground, Super Earth yellow as the one hot colour,
  cut-cornered panels and cards with 2px frames, and larger type, with condensed figures that stay still as they
  count.
- Navigation: a rail down the left holds all five destinations (keys 1 to 5) with Settings, About, the legend
  and Tracker Dispatch at its foot, and the status strip joins the header. On phones a tab bar runs along the
  foot and the rest sits in a More menu.
- Campaign cards show the type and countdown inside the card, the game's campaign glyphs and a large
  percentage; Major Order targets are framed in yellow. Active planets opens with its fronts, defenses and
  Helldivers in large figures.
- On/off settings are switches; filter chips fill yellow when chosen; the briefing's hold plays once a session
  and not at all with reduced motion; fields no longer make iOS zoom in, and small buttons are easier to tap.
- The map's Minimal quality now drops the hologram and ember glows it promises to.

### Planet effects

- Scavenger Rampage, the game's Perilous Population effect, now shows beside its planets on the map, and with its
  time left in the planet's hover card and on the planet page. Threats are marked in red, and effect descriptions
  on the planet page keep the game's highlights.

### War history

- The shared history reaches back to the war's first days: every planet that changed hands since 8 February
  2024, Helldivers online, the fronts and campaign progress since 2 March 2024, and the DSS's FTL jumps since it
  first deployed in November 2024.
- A planet's war history starts at its oldest change of hands when that is older than its first reading.

### Fixed

- Planet charts and War history read every row of the last day instead of stopping at 1000, so busy fronts
  chart the whole 24 hours and recent or ongoing defenses no longer vanish or read as held. The G.W.W. war log
  says "latest" when older changes aren't loaded and can show older ones; the FTL jump log says when it is cut
  at 50.
- Major Orders: a task with no planet is no longer shown as Super Earth, done from the start; defense targets
  are judged against the defense's own deadline; a defense's bar shows the attacker's colour.
- Defenses: the planet page no longer promises a victory after the deadline, liberation regen is no longer
  counted as enemy regen or added to Helldiver impact, and the status strip and supply lines show the defense's
  progress instead of 0%.
- Mission dive time is shown in the right unit, and accuracy no longer reads over 100%.
- War history names the attacker of a held defense correctly and no longer claims how long the enemy held a
  planet before the record began. Order announcements pair with the right orders when the dispatch list is cut.
- Zero-player readings from the API no longer drag the activity chart and the day's low down.
- Chart tooltips no longer crash when the data shrinks under the pointer; a stalled download no longer hangs the
  app; the cache stops growing; "Check now" can't start a second install.
- Map: "Show on map" flies to the planet instead of being cancelled, a remounted map doesn't replay it, zooming
  out on a small window no longer zooms in, Spheres marks Major Order targets, and the legend describes defended
  planets.
- Esc closes one layer at a time; number keys don't switch pages behind dialogs; the city page's "Planets"
  breadcrumb works; planet links in a briefing open their options in front of it; keyboard-opened planet menus
  appear by their row.
- Alerts: several new dispatches arrive oldest first, Priority popups no longer lose toasts to hidden ones, the
  web build asks for notification permission only from a click, and Galactic Campaign briefings are headed as
  such. On Android, pushed news follows the alert settings.
- Collector: a capture whose write fails is retried instead of lost, campaign planets at 0% or 100% are
  sampled, and empty readings no longer reach the war samples.

## [0.5.0] - 2026-10-27

Everything since v0.3.5-beta, and the first release out of beta.

### Upgrading

- Redeploy the collector: `git pull && docker compose up -d --build` now builds the Rust collector, and the same
  `collector/.env` works. Apply `supabase/migrations/0005_activity.sql` first if it isn't yet.

### Rebuilt in Rust

- The app is rebuilt in Rust with Leptos, in the same Tauri shell, for Windows, macOS, Linux, Android and the
  web. Every screen keeps its look and behaviour, and settings saved by earlier versions carry over.
- The collector is rebuilt in Rust and writes the same snapshots and rows, so older apps keep reading it.
- Building needs no Node.js: Trunk builds the web app and the Tauri CLI comes from cargo.

## [0.3.5-beta] - 2026-10-27

Everything since v0.3.2-beta.

### Upgrading

- Apply `supabase/migrations/0005_activity.sql` and redeploy the collector. Each SEST day's Helldiver peak and
  low, every planet's Helldiver count and city readings need the migration; until it is applied the collector
  skips those writes. Arsenal augmentations need the redeployed collector.

### Planet cards

- The map's planet card and panel list the planet's arsenal augmentations, with each stratagem's icon and the
  time left; the planet page shows the time left too.
- Circumvallation: when liberating a front would leave enemy planets with only Super Earth planets around them,
  the card, the panel and the planet's supply lines say so, as a SEAF siege could follow. Blue when the ring is
  worth closing, bright for a double one, grey with a notice when liberating the surrounded planet directly is
  quicker, judged on health and resistance.
- Heavy SEAF Presence wears the SEAF emblem.
- The liberation bar marks where each city opens, those open from the start included, with its rank emblem.

### Cities and activity

- Each city opens its own page: its status, what freeing it is worth, when it opens and its activity over the
  last day. A city takes its holder's colour, blue once liberated, and shows the liberation it opens at instead
  of Locked.
- Every planet keeps its Helldiver count through the day, fought over or not, and charts it on its page.

### Space stations

- Every tactical action has its own icon, coloured while it cools down, and a charge bar that glows green when
  it is ready or running. The live list keeps a fixed size for each action; the destination vote box is gone.
- The FTL jumps move to the G.W.W.'s Galactic logs.

### G.W.W.

- Galactic logs (formerly the war log) folds the war log, each SEST day's busiest and quietest Helldiver counts
  and the stations' FTL jumps into boxes.
- Super Earth's holidays have their own pages, reached from Managed Democracy with the Citizenship
  Classification Structure; the front page's holiday chip opens them.
- The training manuals sit behind one tile. Martian Armory joins the corporations with a logo drawn for the app;
  AmpuDyne and Ulterior, which have no logo yet, are left out for now.
- Every page reads as the Federation's own net: no players, studio, crossovers or real-world notes. The wiki
  pages they adapt are credited in About.

### High Command overview

- The Major Order box folds.
- The Galactic war activity chart fits phones and toggles its series without stutter; the Distribution of
  Democracy chart answers taps.

### Fixed

- The news ticker no longer shakes as its status line and clock change.

## [0.3.0-beta] - 2026-10-26

Everything since v0.2.7-alpha, which is the same code as v0.2.7 (commit `c5ca962` on main).

### Upgrading

- 0.2.7 can't update itself. Install 0.3.0-beta from the [Releases](https://github.com/zearkreast/WarRoom/releases)
  page. After that, the app offers each new release itself.
- Redeploy the collector. Major Order outlooks and city rates need its new data; until then Major Order tasks
  read as Measuring. There are no new database migrations.

### In-app updates

- Desktop (Windows, macOS, Linux): a few seconds after start-up, and every six hours, the app looks for a newer
  release. A card at the top of the toast column offers it with a few lines of its notes. Download and install
  fetches the signed bundle, shows its progress, installs it (the Windows installer runs passively) and restarts
  the app.
- Android: the app looks for a newer release with an APK and opens its download; the system then asks to install
  it. Versions are ordered as semver orders them, so 0.3.0 counts as newer than 0.3.0-beta, and rc as newer than
  beta.
- Settings has an Updates panel with the current version, the last check's result and Check now. The web version
  never checks.

### Android, app icon and privacy

- An Android app (arm64 and armv7) joins the Windows, macOS and Linux installers on each release.
- Every platform's app icon, Android's adaptive icons included, is the top bar's yellow shield and star on the
  dark war-table blue.
- Privacy policy ([PRIVACY.md](PRIVACY.md)): the app collects no personal data. The Windows installers show it on
  their agreement page, and About sums it up for Android, which has no installer.

### Strohmann News

- One column, newest first, grouped by day (Today, Yesterday, then the SEST date), with the time on a rail.
- The newest item is set apart as Latest. Items since your last visit are marked New, with a count on the page
  and on the News button.
- Search across dispatches and campaign stories, beside the kind filters.
- A "Last message from Super Earth" strip on top reopens the newest dispatch in the briefing popup.
- Compact items: a small tag in the kind's colour replaces the big outlined banner, and a body that only repeats
  its headline (attack alerts) is left out.

### Alerts

- Clicking a toast shows a "Please wait democratically" hold, then a game-style briefing under Super Earth's
  flag: a banner, the verdict (Victory, Failure, Major order, Alert, Dispatch) and the full dispatch or Major
  Order text. The toast's X still only dismisses it; Esc, Continue or a click outside closes the briefing.

### Galaxy map

- Click a planet to dock its panel beside the map (a sheet on phones): picture, status, sub-factions, cities,
  progress, resistance, Helldivers, owner, and Pin, Focus, Details and Station.
- The rail's buttons are labelled, Regions / Sectors / Galaxy is a one-click switch, and a Legend panel explains
  the marks.
- Holograms: from zoom 2 up, sub-factions, SEAF presence and the DSS stand beside their planet as holographic
  units in their faction's colour. The DSS is yellow with icons of its deployed tactical actions, red while one
  cools down, grey when none is available and blue otherwise. Planets Super Earth holds show no enemy units.
- Points of interest wear the game's logos over the planet, coloured by who built them. The hover card shows
  their picture (or a black hole's) and a row for each point of interest or anomaly. The E-711 Extraction
  Facility is named, and Mars lists the Helldiver Training Facility and the Martian Armory.
- Penta is drawn as a black hole with an accretion disk and photon ring.
- Touch: twist two fingers to turn the map, slide them up or down together to tilt it, pinch to zoom about the
  fingers. Turning starts only past about 11 degrees, so a pinch doesn't wobble the map. A one-time hint lists
  the gestures, and a drag that ends on a planet no longer opens it.
- Camera: Q/E, R/F, North and Tilt ease the short way round, and repeated presses add up. A zoom step or fly-to
  no longer fights a touch, a right-drag or a turn, and resizing the window keeps the zoom and the centre instead
  of refitting.
- Smoother zoom, pan and turn, above all on phones: planets no longer all restyle on every frame, supply lanes
  are one path, the Gloom is one pre-rendered image (shared with the Theater of War), and the starfield's pixel
  density is capped below Maximum quality. Minimal quality stops the pulsing markers and draws fewer stars.

### Faction emblems

- The game's Super Earth, Terminid, Automaton and Illuminate emblems, from the Helldivers Wiki, replace the app's
  drawn marks everywhere a faction shows, in the app's faction colours.
- Sub-faction tags show the unit's own emblem: Predator, Spore Burst and Rupture Strain, Jet Brigade,
  Incineration Corps, Cyborgs, Appropriators, Vote Snatchers, Mindless Masses and Invasion Fleet (also used for
  The Great Host). Hive Lords and Dragonroaches keep the Terminid emblem.
- The DSS wears the game's DSS glyph everywhere it had a satellite icon; the Automaton station wears the Automaton
  cog and star.

### Major Orders

- Every task shows its pace, when it completes or where it ends at the deadline, and Winning, Losing, Measuring
  or No front. Count tasks (kills, samples, operations) have a pace now too. The order card and the Major Order
  strip say whether the whole order is won at this pace.
- One animated progress bar for every Major Order bar, on the Overview, the map and the Active planets strip.
  Bars and their tallies count up together; liberation bars fill in Super Earth blue over the enemy's hatched
  share, and a glowing tick marks where a task falls short. A task reads 100% only once it is done.
- Screen readers hear each bar's numbers with its share, and high-contrast themes still see the fill.

### Planets and cities

- Cities show their size the way the game marks it, one to four stars (Settlement, Town, City, Mega City), and
  their standing: liberating, locked, enemy held or secured. A city being fought over shows how far it is
  liberated and how many Helldivers are in it.
- On the planet page each contested city also shows its resistance, net progress, time to liberation and the
  boost freeing it gives the planet (1.5 times its health, over the planet's).
- A Points of interest panel lists the planet's structures and cities; they no longer count among its
  operational parameters.
- Super Earth's capitals show what the 2185 Illuminate invasion left: Eagleopolis, Administrative Center 02 and
  Port Mercy destroyed; Remembrance rebuilt as Gun and York Supreme as New York Supreme; Prosperity City and
  Equality-on-Sea intact. Super Earth's planet page shows a turning globe of the real world map with the capitals
  where they stand.

### War activity

- Switching a series off folds it away instead of dropping it from the chart, and switching it back on brings it
  back: a front's band thins to nothing, the online line sinks onto the fronts and the impact line flatlines. The
  chips sweep in their colour when on and flicker to grey when off.

### Hellpad

- The hub is restyled after the G.W.W.: a masthead with the Hellpad's mark, uplink status, open fronts and
  Helldivers deployed; a station report on where the DSS is and when it jumps; and the modules as sites in their
  own colours with live readouts. An open module has a strip with Back, its breadcrumb and its `hp://` address.
- Key in the game's Call In Super Destroyer code (up up down down left right left right) on the masthead to open
  the G.W.W. on the new Super Destroyers manual. On touch screens the Hellpad mark's four arrows are the keys.
- Modules to come: Enemy encyclopedia, Weaponry, Stratagem collection and Ship library join Warbonds and Boosters.
- The dock's Hellpad tab carries the Hellpad's mark.
- Space stations: on screens wider than 900px, a hologram beside each station's page, the DSS lit by its state as
  on the map, the Automaton station as a wireframe broken in two. The DSS page gains a Star of Peace panel, from
  the weapon and its fuel to Operation Free Space, the stolen blueprints and the failed re-arming. The Automaton
  station's page gets a dated Operation Lightning Intercept timeline and a dossier. Both histories are checked
  against the Helldivers Wiki, and their planet names open the planet's options.
- Past campaigns: a service record (campaigns, victories, Major Orders won, win streak) and a strip with one
  block per campaign in its enemy's colour, failed ones hatched. Cards carry a faction edge and emblem, a pip per
  Major Order, the campaign's length and its reward, struck through when lost.

### Galactic Wide Web (G.W.W.)

- Opens as a browser of Federation sites: an address bar with Back and Home, a front page with search and a
  Training Manual Tip, Super Earth's sites and its seven ministries, the training manuals, the corporations and
  the game's city adverts.
- Type any site's address into the address bar, with or without `se://`, and pick from suggestions as you type.
  An address no site answers opens an error page with the directories to try.
- The training manuals are rewritten from the Helldivers Wiki's Second Galactic War Mechanics page as a
  recruit's course of five chapters, with tables, diagrams, the game's icons and tips, a live chart of the impact
  modifier against Helldivers online, and "how this tracker shows it" sections. Each manual has a live panel from
  the tracker's data, such as the busiest and stalling fronts or each defense with the planet behind it.
- New manual: Super Destroyers, with the six ship modules and their upgrades.
- Ministries and mega-corporations have sites with the game's logos, slogans, products and dated histories. A
  Managed Democracy site covers the vote, the Council, the Wall of Martyrs and every holiday the wiki knows; a
  Citizenship Classification Structure site covers the classes from Super Citizens to Class F.
- Pages link to each other, to the tracker's own pages and to planets (in their owner's colour). Every page has
  a breadcrumb, an "On this page" list, read-next and previous/next links, and every advert opens its
  advertiser's site.
- The G.W.W. remembers where you were: going back restores the scroll, and read manuals are kept, with a
  "continue with" button on the front page and the Hellpad tile.
- The front page counts down to the next Super Earth holiday, and the war log can be filtered and is grouped by
  day.
- Its name is Galactic Wide Web, not Galactic War Web as 0.2.7 had it.
- About gains a Credits section for the wiki's text (CC BY-NC-SA 4.0) and the game's images.

### Navigation

- The browser's Back and Forward, the mouse's back button and the phone's Back gesture step through the app's own
  places: the screen, the Hellpad module, the G.W.W. page, and an open planet page, dialog or Tracker Dispatch.
  On Android, Back walks these before it closes the app.
- Closing something counts as a step back, so Back never reopens what was just closed. Escape closes only the
  layer opened last.
- A reload resumes the place it was on, without flashing the default screen first.

### Phones and accessibility

- Narrow screens get a Settings button in the top bar; before, Settings was only reachable from the map.
- On touch screens, chips, stepper arrows, the toast's X and key buttons are 40px, text links and the Major Order
  toggle have larger hit areas, and Esc keycaps are hidden.
- A long press no longer starts a text selection on buttons, Hellpad modules or the Overview. Dispatch, briefing
  and campaign text stays selectable.
- Campaign outcome chips drop under the dates on phones, so long names wrap at whole words.
- Focus moves to a Hellpad module's Back button when it opens and back to its tile when it closes, and returns to
  what opened the planet menu when the menu closes.
- Station holograms have a button to hold them still. Past campaign cards honour reduced motion and forced
  colours, and locked Hellpad tiles meet contrast.

### Fixes

- The legend and About dialogs open scrolled to the top.
- The API's literal "null" region descriptions are no longer shown.
- Dispatches and briefings no longer turn up in another language when the game's API serves a copy made for
  another player.

### Collector

- The `assignments` job also writes an `order_samples` snapshot: the last day of Major Order progress readings,
  which carries over a restart.
- Live snapshots carry each region's regeneration, and campaign planets their regions' progress rates. Region
  readings are kept in memory, so region rates read as measuring for about 10 minutes after a restart.
- The app still loads older snapshots without these fields.
- API answers that aren't in English are rejected, and the job tries again a minute later. Dispatches are
  compared by text, so one stored in the wrong language is replaced once it comes in English.

### Build and release

- A new Android workflow (`android.yml`) builds the signed APK. Release runs it after the desktop builds and
  attaches the APK to the release; pushes that touch `src-tauri/` on other branches build it as a run artifact.
- Release signs the update bundles and uploads `latest.json`, which installed apps read. Keep releases full ones,
  not GitHub pre-releases: installed apps only look at the latest full release.
- New repository secrets: `TAURI_SIGNING_PRIVATE_KEY` (with `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` if the key has
  one), `ANDROID_KEYSTORE`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS` and `ANDROID_KEY_PASSWORD`. Without
  the Android secrets each APK is signed with a throwaway key, and a newer APK won't install over an older one.
- MSI installers take numbers only, so `bundle.windows.wix.version` gives them 0.3.0.
- App icons are generated from `src-tauri/app-icon.svg` with `npx tauri icon src-tauri/app-icon.svg`.
- New dependencies: `@tauri-apps/plugin-updater` and `@tauri-apps/plugin-process` (desktop only), and `d3-geo`,
  `topojson-client` and `world-atlas` for the Super Earth globe.
- The README covers downloads, in-app updates, how to release a version and the secrets it needs.

[0.3.0-beta]: https://github.com/zearkreast/WarRoom/compare/c5ca962...v0.3.0-beta
