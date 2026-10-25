import type { Planet } from "../api/types";
import { SUPER_EARTH_CITIES } from "../data/cities";

/** The game marks a region's size class with one to four stars over its icon. */
const STARS: Record<string, number> = { Settlement: 1, Town: 2, City: 3, MegaCity: 4 };

export type CityState = "liberating" | "locked" | "held" | "secured" | "destroyed" | "rebuilt" | "intact";

export interface City {
  id: number;
  name: string;
  /** Its name before it was rebuilt. */
  formerly?: string;
  size: string;
  stars: number;
  description: string;
  note?: string;
  state: CityState;
  /** Liberated share, while the enemy's hold on it is being counted down. */
  percent: number | null;
  players: number;
}

export const cityStars = (size: string) => STARS[size] ?? 0;

/** "MegaCity" → "Mega City". */
export const sizeLabel = (size: string) => size.replace(/([a-z])([A-Z])/g, "$1 $2");

function stateOf(p: Planet, health: number | null, open: boolean): CityState {
  if (open) return "liberating";
  if (p.currentOwner === "Humans" && !p.event) return "secured";
  return health == null ? "held" : "locked";
}

/** A planet's cities as the player sees them, with what is known of each one's fate. */
export function citiesOf(p: Planet): City[] {
  return p.regions.map((r) => {
    const fate = p.index === 0 ? SUPER_EARTH_CITIES[r.id] : undefined;
    const percent = r.health == null || !r.maxHealth ? null : Math.max(0, 100 - (r.health / r.maxHealth) * 100);
    const live = stateOf(p, r.health, r.isAvailable);
    return {
      id: r.id,
      name: fate?.name ?? r.name,
      formerly: fate?.name ? r.name : undefined,
      size: r.size,
      stars: cityStars(r.size),
      // The API sends the string "null" for regions without one.
      description: r.description === "null" ? "" : (r.description ?? ""),
      note: fate?.note,
      state: live === "liberating" ? live : (fate?.state ?? live),
      percent,
      players: r.players,
    };
  });
}
