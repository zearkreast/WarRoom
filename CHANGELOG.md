# Changelog

What changed in each release of WarCommand Tracker. Tracker Dispatch in the app shows a shorter version of
each entry, from `crates/core/src/changelog.rs`.

## [0.7.2] - 2026-11-06

Everything since v0.7.1.

### Upgrading

- On a database of your own, apply `supabase/migrations/0013_campaign_art.sql`, which adds the public
  `campaign-art` storage bucket, and redeploy the collector (`git pull && docker compose up -d --build`) so the
  campaign archive keeps the game's picture ids. The game's API names each picture only by id, so the pictures
  are uploaded to the bucket by hand as `<id>.webp`; until a picture is there, its place stays as before.

### Galactic Campaigns

- The running campaign on High Command opens with its picture, and each of its Major Orders shows its own when
  unfolded.
- In Hellpad › Past campaigns, each campaign's picture fades in across its row, and each Major Order's shows when it
  is unfolded. The Major Order boxes elsewhere stay as they were.

### Strohmann News

- A campaign story carries the campaign's picture.
- A story pictures the first planet it names, also when one highlight names several ("hold FRONTERIA, CRIMSICA and
  …") or the headline names it without a highlight, stamped with the stratagem it brings.
- A story without a planet shows the stratagem it is about, the hologram of the sub-faction or the DSS it names, or
  its enemy's emblem.

- Strategic Opportunities, Imperatives and Directives read as orders: "New Strategic Directive" is an order,
  "Strategic Opportunity Seized" or "Imperative Achieved" a victory, "Lost" a defeat. Order dispatches are labelled
  Order instead of Major order, as the campaigns now hold the Major Orders.

### Fixed

- The Major Orders desk of the G.W.W.'s Strohmann News was empty. Since every Major Order runs inside a Galactic
  Campaign, its dispatches sit on the campaign's story; the desk now holds the campaigns and High Command's other
  orders.
- The Front lines desk took only dispatches whose headline said victory or defeat, which the game's front news
  rarely does ("HEETH LOST", "MINOR TERMINID OUTBREAK"). It now holds every dispatch that names a planet and isn't
  about an order, so it no longer repeats the Major Orders desk.
- On phones, the pictures of Strohmann News briefs stretched to the height of their text.

## [0.7.1] - 2026-11-06

Everything since v0.7.0.

### Upgrading

- No new migrations, and the collector is unchanged, so it needs no redeploy.

### Hellpad

- Every module opens under one head: back, the module's address, its icon and title, what it is and what it holds.
- The address follows the module's state, such as `hp://planets/automatons?q=creek`, `hp://stations/css` or
  `hp://stratagems/supply`. Past campaigns is `hp://campaigns`.
- Modules not built yet are smaller tiles marked Planned, and the Ship library's reads Awaits the game, in place of a
  progress bar showing progress that isn't there.
- G.W.W.: the Super Earth and corporation directories move to `se://gww/super-earth` and `se://gww/corporations`.
  The old addresses still open them.

### Look and start-up

- Pages and Hellpad modules rise in, top down, as they open; what a search, sort or refresh redraws later just
  appears. Reduced motion turns it off.
- A start-up screen shows while the app downloads.
- The web app has a manifest with icons and shortcuts to the Galaxy map, Active planets, High Command and Strohmann News, so
  it installs as an app, plus an icon for iOS home screens and a favicon in the current palette.
- The header ticker fades out at its edges, and on phones rows that scroll sideways fade at the edge with more to
  show.

### Planets

- A planet description the planet list marks unofficial shows without the mark, with a small "Unofficial
  description" note under it, on the map's card and panel and in the planet page's Operational parameters.
- The corrected biomes of Sangis and Alderidge Cove take the current text for that biome where a planet has it.

### Fixed

- Leaving Strohmann News moments after typing a search, or closing a planet while its 7 or 30 day front line loaded,
  stopped the app.
- Folding one order, such as the Strategic Threat, folded the Major Order with it, on Active planets and on High
  Command. Each order now folds on its own and is remembered by itself; folds kept from earlier versions start over.
- A long planet name widened its campaign card past the page. Names over 13 letters are now drawn condensed, and the
  longest shrink until they fit the card.
- Phones: the Major Order strip's countdown ran past its frame, and the map's buttons showed through the planet
  sheet above them.
- Space stations: the tactical action record said "Not active in the last 30 days" over a log only hours old; it now
  says when the log began.

## [0.7.0] - 2026-11-04

Everything since v0.6.3.

### Upgrading

- On a database of your own, apply `supabase/migrations/0008_planet_history.sql` to `0012_dispatches.sql` and
  redeploy the collector (`git pull && docker compose up -d --build`). 0008 adds the read behind the planet page's 7
  and 30 day ranges; 0009 to 0012 add the tables the collector now fills: each Major Order and its progress, each
  change in the space stations' tactical actions, the hourly mission, kill and death counters, and every dispatch.
  These records start from the update. Until the migrations are in, the features that read them stay hidden or fall
  back to 24 h, and the collector skips those writes; its assignments, dispatches and stations jobs log which table
  is missing.

### Active planets

- Sort the fronts by Priority, Most needed, Most Helldivers or Soonest victory from a select beside the filters; the
  choice is remembered. Most needed puts Major Order targets and fronts losing ground or missing their deadline
  first, then those where a Helldiver adds the most, and badges on the cards say why: Losing ground, Misses its
  deadline or Each Helldiver counts.
- Pinning is now Watch everywhere, and watched planets make up your watchlist, which is also the filter's name. They
  lead every sort, those with no campaign get a row of their own, and a toast says what watching does.
- Defenses, rare campaigns, and gambits or circumvallations worth playing always show, whatever their Helldiver
  count. Cards gain a Circumvallation badge, and Gambit, Circumvallation and Special join the filters. A tactic not
  worth playing (holding the defense or liberating the targets directly is quicker, or it can't land in time)
  follows the usual Helldiver and progress thresholds, and its badge is greyed like its advisory. Double and
  multiple circumvallations are named and marked as such, and a defense card with a gambit open gives each
  attacker's resistance.
- Siege liberation: an enemy planet whose every supply-line neighbour Super Earth holds gets a Siege liberation
  badge, and in place of Regen its card shows a SEAF figure: its resistance, then, once that turns negative, the pace
  it liberates itself at. Other cards' Regen gains a helmet mark.
- A Major Order liberation target shows on its card the pace the order needs.
- A calmer card: a liberation at 0% that isn't gaining says "Holding for reinforcement" instead of a red 0.00% per
  hour, a flat rate is grey and only a falling one is red, and the other figures are plain apart from Needed and MO
  needs, which still judge the pace. Badges are quiet chips, a projected withdrawal or fall is amber, rare campaigns
  read "Special", and on phones the filters are one scrolling row.
- The Major Order strip gives its verdict as a small tag beside the title, so it reads even while folded, and every
  objective shares one row: what to do, the bar, the figures, when it completes and whether it makes the deadline.

### Major Orders and alerts

- On the Major Order card, each unfinished objective shows the pace it needs to finish by the deadline beside its
  measured pace.
- The Major Order card can chart each objective's progress over the run on record, and Hellpad › Past campaigns
  lists the ended Major Orders on record with where each objective stood at the end. Both start from the collector
  update.
- New alerts, both on by default: Watched planets (a watched front is an hour from victory, stops gaining ground, or
  a city opens on it) and Major Order progress (the order turns to losing, has 6 hours or 1 hour left, or completes
  an objective). Nothing fires for what was already so when the app starts. The app works these out itself, so
  they come while it runs (on desktop, from the tray too); Android's pushed news doesn't carry them.
- Coming back after three hours or more, High Command shows "While you were away" above the Major Order: planets
  liberated and lost, how far each Major Order objective moved, Galactic Campaigns begun or ended, new dispatches and
  the peak of Helldivers online. It can be dismissed. The privacy policy notes the timestamp this keeps on the
  device.

### Planet page

- The page is laid out in reading order: progress, activity, intel, points of interest, service record, war history
  and supply lines, with a bar under the header that links to each section and lights the one in view. The war log
  starts folded, with its entry count and latest date on the fold.
- The page assembles as it opens: the header scans in, the name types out, the sphere's rings draw and its meridians
  turn, and each section rises in as it is first reached. Reduced motion skips all of it.
- Front line activity charts the last 24 h, 7 days or 30 days.
- A gambit that holding beats, or that can't land before the deadline, opens its advisory with a Notice, as a
  circumvallation not worth closing does, here and on the map's planet card.
- The Service record of a planet with a campaign, and the statistics in High Command's Distribution of Democracy
  panel, add the last 24 hours: missions won and lost, the success rate, kills of each enemy and Helldivers killed
  in action.
- A Major Order liberation target gets the needs box defenses have: the pace needed, where it stands at the order's
  deadline and roughly how many Helldivers short.
- The Martian Armory wears its own logo instead of the generic industry mark, and the points of interest of a razed
  world, Mars among them, flicker red on its card and page. Super Earth and Cyberstan list themselves as Humanity's
  Home and the Automaton Homeworld among their points of interest.

### Galaxy map

- The defense beacon is now the Priority beacon: it also calls out a Major Order target being fought over, in the
  order's yellow, and planets with a beacon draw over their neighbours. Your saved choice carries over.
- Holograms stand round their planet on the table and turn with the map, each projector carrying its unit's emblem.
  A projector flattens only as far as the map tilts, so its emblem stays readable; at Minimal quality projectors
  keep one angle. The DSS stands on Super Earth's emblem.
- Regions outlines, here and in High Command's Theater of War, are smoothed and cut to the galaxy's rim, and the
  hatch on a planet under attack follows them and shows over the Gloom. A planet under siege is ground of its own, in
  Super Earth blue, and its tag reads Siege.
- Standing TCS+ towers spread a blue Termicide haze; at Maximum quality it is drawn the Gloom's way.
- The Noxious and Caustic strains show on their planets, and they, the Hive Lords and the Dragonroaches wear their
  own emblems. Force Evacuation shows among the war events of the planets that carry it.
- The legend explains the yellow beacon, siege liberation, TCS+ towers and the projectors' emblems.
- Lighter: the news ticker scrolls on the compositor instead of from script every frame, a drag no longer opens the
  planet cards it passes, and the Gloom image is kept for the session. At Minimal quality, and on touch screens below
  Maximum, the Gloom and the haze are drawn at half resolution.

### Strohmann News and the Hellpad

- Load older news carries on past the newest 50 dispatches, and a search of three letters or more also searches
  every dispatch on record, bringing back its newest 100 matches.
- The G.W.W.'s Strohmann News is a news site of its own: a lead story with its planet's picture, two more beside it,
  the rest in brief, and desks for Major Orders, front lines, alerts and campaigns. Coming back to the G.W.W. from the
  Hellpad opens its home page, and Home on the home page scrolls back to the top. Read next on a training manual
  leads with the next unread manual of the course and ticks the ones already read.
- Galactic logs gives each log a tab on a wider page. The war log opens with the last seven days' tally and puts each
  change on one line, under a heading per day with its count, and Helldivers online opens with a chart of each day's
  peak and low over the last 90 days. Both logs add days a few at a time, and each tab keeps its filters.
- Space stations: each tactical action says when it was last active and how often in the last 30 days, and a new
  switch on the page, on by default, alerts when an action is charged or goes active.

### Calmer look

- Colour is kept for what judges the war, and fewer things loop: live dots no longer pulse, Galactic war activity's
  live point pings three times, Major Order bars sweep once after counting up, the Major Order strip and panel lose
  their hazard stripes, and High Command's tiles their yellow rules.

### Fixed

- An Active planets card no longer shows a victory more than seven days off, such as "Victory in 23395d".
- Galactic war activity's Galactic impact mod no longer runs into Peak Helldivers.

### Project

- The README leads with the app, its screenshots and its downloads; the developer guide moves to
  `docs/DEVELOPMENT.md`.
- About credits the Noxious and Caustic Strain emblems as traced for the app from the game's planet card.

## [0.6.3] - 2026-11-02

Everything since v0.6.2.

### Upgrading

- On a database of your own, apply `supabase/migrations/0006_compact_recent.sql`, which keeps the hourly thinning
  within the collector's time limit. The collector is unchanged, so it needs no redeploy.

### Galactic logs

- Search the war log by planet or sector, and filter it by enemy and by sector. Any of these reads the whole log, so
  the results reach back to the war's first days instead of only the latest 100.

### Fixed

- On the first visit after new news, the newest item was pulled out of the page and squeezed against the right edge.

### Security and legal

- The desktop updater moves to `zip` 9.2.1, which clears one critical, one high and one medium security alert.
- `SECURITY.md` says how to report a vulnerability privately and what counts as one.
- The privacy policy lists every Android permission and the update requests to GitHub, and says pushed news is off in
  current releases.
- About credits the holograms, points of interest and DSS action icons, and the README and About say what the GPL
  doesn't cover and how to ask for credit or removal.

## [0.6.2] - 2026-11-01

Everything since v0.6.0.

### Upgrading

- On a database of your own, apply `supabase/migrations/0007_samples_defending.sql`. The collector is unchanged, so
  it needs no redeploy.

### Galactic logs

- Helldivers online opens on the newest day. Press Peak online or Lowest online to sort the days from most to
  fewest, and again for fewest to most. SEST day goes back to newest first, and again to oldest first.
- Helldivers online reaches every logged day back to 2 March 2024, not only the last year.

### Fixed

- A planet's page could lose its record and show "Could not reach the war archive" when the read of its defenses
  took too long. An index on those readings makes that read quick.

## [0.6.0] - 2026-10-31

Everything since v0.5.0.

### Upgrading

- Redeploy the collector (`git pull && docker compose up -d --build`) before releasing an APK from this version:
  new APKs follow one push topic per alert setting, and only the updated collector sends to them. There are no
  new database migrations.
- The war's earlier history lives in the project's shared database. Migrations don't bring it to a database of
  your own, which starts from its collector's first run.

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

### City liberations

- Front line activity puts the city's mark, with its stars for size, on the progress line where freeing it jumped
  the front forward, and a faint line down both charts at that moment. Point at the mark for the city's name and
  the boost it gave. Same-sized cities are told apart by the collector's city readings; when those can't settle
  it, the box names each city it could be.
- A city's boost on a defended planet counts against the defense's own health, so it now shows what freeing the
  city really adds, and says so.

### Mars

- Mars has a planet page with what's known of it and its points of interest, opens the map's planet panel like any
  other planet, and appears in Planets in the Sol sector. Galactic control's shares still count only the war
  data's planets.

### Stratagems

- The Hellpad's stratagem collection opens: all 118 stratagems by permit and type, each with its icon, code,
  cooldown, call-in time, uses, traits and where it is unlocked, with search and permit colour filters.
- The G.W.W. gains the stratagem drill: key stratagem codes against a burning fuse, round after round, with a
  streak multiplier and round, time and flawless bonuses. Play every stratagem, your own loadout (picked in the
  drill or with the crosshair on the collection's cards), or the permit colours you choose. Its march and sound
  effects are synthesised in the app; the best ten scores of each mode stay on this device.

### Defenses and planet status

- A defense shows both sides as the game does: Super Earth's defense over the enemy's advance on the deadline. Both
  bars are on the map's hover card and planet panel and on the campaign card, the planet page's bar and the ring
  round its sphere show both, and Front line activity charts the enemy's advance up to 100% at the deadline.
- Defenses say where the attack comes from and how long it has run. The planet page works out what holding needs:
  the pace to hold by the deadline, the projection at the deadline and roughly how many more Helldivers it takes,
  as estimates at today's pace. The campaign card shows the needed pace.
- Gambit: when every planet attacking a defense is a liberation front, the defended planet and its attackers carry
  an advisory saying whether liberating them ends the defense before the deadline, and whether holding outright is
  quicker.
- Map: in the Regions overlay a planet under attack joins its attacker's territory, hatched, and every planet under
  attack turns a dashed ring in the attacker's colour. A defense beacon picks it out from afar: Ping, Glow, Beacon
  or Off, in the map settings. Chevrons run along the attack line from each attacking world, and the dashed lines
  from Super Earth's planets to liberation fronts are gone.

### War history

- The shared history reaches back to the war's first days: every planet that changed hands since the first
  capture on 10 February 2024, Helldivers online, the fronts and campaign progress since 2 March 2024, and the
  DSS's FTL jumps since it first deployed in November 2024.
- A planet's war history starts at its oldest change of hands when that is older than its first reading.

### Fixed

- Opening the map's Controls no longer shoves the Regions, Sectors and Galaxy switch aside or off the screen.
- New Haven is no longer drawn scorched.
- Cities: the liberation bar's marks stand exactly where each city opens, stacking when close instead of shifting
  aside, and the planet page's bar carries them too. A closed city says where it opens instead of a bare Locked,
  special sites no longer read "Opens at 100%", a defended planet's cities stay Super Earth's, and city lists
  follow the bar's order.
- Switching the map's overlay or planet style no longer hides labels and holograms until the camera moves.
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
