export interface ChangelogEntry {
  version: string;
  date: string;
  items: string[];
}

/** Newest first. Shown in the Companion Dispatch panel. */
export const CHANGELOG: ChangelogEntry[] = [
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
