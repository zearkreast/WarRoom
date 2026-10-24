export interface ChangelogEntry {
  version: string;
  date: string;
  items: string[];
}

/** Newest first. Shown in the Tracker Dispatch panel. */
export const CHANGELOG: ChangelogEntry[] = [
  {
    version: "0.2.7",
    date: "2026-10-24",
    items: [
      "Planet effects from the game: sub-factions such as the Incineration Corps, Heavy SEAF Presence, and points of interest like TCS+ towers and the three Maximum Security Cities, as pawns beside planets on the map and under each planet's new Operational parameters, with its cities.",
      "The map draws the Gloom in every overlay, fractured planets as debris, and razed worlds (Widow's Harbor, Pilen V, New Haven, Mars, Merak, Lesath, Chort Bay) scorched.",
      "Sectors overlay looks like the game's war table: only enemy sectors painted, as hatched tiles. The Theater of War shows the same, fills Super Earth's regions, carries its emblem at the centre and can stop turning.",
      "Galactic war activity has its own full-width panel under the Overview board: Helldivers per front, minute by minute, stacked under the online total, with the impact multiplier on its own right-hand axis.",
      "Every biome has its own colours and landscape on planet cards, the map's hover card and the planet page; black holes and fractured planets show what they have become instead.",
      "The campaign panel keeps a slot for each of a campaign's three Major Orders, filled as the game issues them.",
      "Strohmann News tells each Galactic Campaign as one card, with its orders, dispatches and outcome on a timeline and the right dates.",
      "Space stations: a picker that also shows the destroyed Automaton station, active tactical actions with their time left, cooldowns, station effects and an FTL jump history.",
      "G.W.W. is now the Galactic War Web: the war log, training manuals on how the war works, and the Federation's corporations.",
      "Settings, About and the icons legend sit together at the bottom left; the map's Dispatches box lists the latest five by default (Settings changes it).",
      "The status strip no longer jumps, and the collector no longer stalls when the API stops answering.",
    ],
  },
  {
    version: "0.2.6",
    date: "2026-10-24",
    items: [
      "Galactic Campaigns as the game runs them: the Overview shows the current one with its Major Orders, outcome so far and campaign reward; between campaigns, the last one and how it ended.",
      "Hellpad: Past campaigns, every ended campaign with its Major Orders, briefings, result and reward. A campaign moves there as soon as it ends.",
      "Strohmann News: campaign launches, new Major Orders and results, with their own Campaign filter.",
      "Campaigns is now Active planets. Its Major Order strip folds away and remembers how you left it.",
      "Map: sector shapes rebuilt so shared borders line up from ring to ring; hovering anywhere outlines that sector in every overlay and on the Theater of War.",
      "Map: Meridia counts under Celeste, and Sangis and Alderidge Cove sit where the game moved them, so Umlaut, Guang and Omega are drawn whole.",
      "Map: see-through faction emblems instead of names, on each enemy's largest holding in every overlay; a calmer Sectors overlay, and themed tags on active planets. The controls panel starts closed; the control bar and the Fit galaxy button are gone (0 still fits the galaxy).",
      "War activity charts the last 24 hours straight away, or everything since the last gap in the data.",
      "The status strip scrolls the last day's news, captures and losses; it pauses under the pointer and each item opens its source.",
      "About opens over the current page.",
    ],
  },
  {
    version: "0.2.5",
    date: "2026-10-23",
    items: [
      "Overview fills wide screens: the distribution, the Theater of War with 24h activity, and the orders side by side, each scrolling on its own.",
      "Overview: the running story campaign with its phase, briefing and reward, plus how earlier campaigns ended.",
      "Campaigns: a Major Order strip with the deadline and whether each target makes it at the current pace; cards warn when victory lands after the deadline.",
      "Planet detail shows how fresh its data is instead of always saying live; Campaigns notes when data is stale.",
      "Calmer colour: Terminids are orange, yellow is kept for the selection, actions and the Major Order, and only data glows.",
    ],
  },
  {
    version: "0.2.1",
    date: "2026-10-23",
    items: [
      "Web version on GitHub Pages.",
      "Campaigns hide stalled fronts: under 5,000 Helldivers and gaining less than +0.4%/h. Major Order targets and pinned planets always show.",
      "Planets: a folded Galactic control ring with faction icons; hovering an arc lists that faction's planets.",
      "Planets: pinned shelf and Pinned filter; pinned planets are marked on the map.",
      "Overview: Major Order first on phones; Distribution of Democracy split by front, with planet lists on hover.",
      "Overview: Theater of War turns slowly, can be dragged and follows the map's overlay.",
      "Map: Sectors drawn on the game's own sector rings, with held and contested sectors told apart.",
      "Map: rotate and tilt with right-drag, Q/E and R/F; smoother panning and zooming.",
      "Map: Mars, Meridia's wormhole and Penta's black hole; planet style (markers or spheres) is its own setting.",
      "Map: readable controls, dispatches from the last 72 hours and a quick settings box.",
      "Fixes: Super Earth's label is no longer hidden; the Planets link in the planet detail works.",
      "Companion Dispatch is now Tracker Dispatch.",
    ],
  },
  {
    version: "0.2.0",
    date: "2026-10-23",
    items: [
      "New layout: Galaxy map, Campaigns and Hellpad tabs, with News and Overview in the top bar.",
      "New look for every screen: a dark war-table HUD.",
      "Planet detail with liberation forecast, service record and war history.",
      "Hellpad: planet index, Democracy Space Station and galaxy-wide war history.",
      "Map overlays: Regions, Sectors and Galaxy.",
      "Pin planets to keep them at the top of Campaigns.",
    ],
  },
  {
    version: "0.1.0",
    date: "2026-10-21",
    items: ["First release: map, Major Orders, fronts, news, statistics, alerts and ownership history."],
  },
];
