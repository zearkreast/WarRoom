export interface ChangelogEntry {
  version: string;
  date: string;
  items: string[];
}

/** Newest first. Shown in the Tracker Dispatch panel. */
export const CHANGELOG: ChangelogEntry[] = [
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
