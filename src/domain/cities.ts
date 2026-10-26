import type { Planet } from "../api/types";
import { liberationPercent } from "./progress";
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
  /** Planet liberation, in points, that freeing it deals: 1.5 times its health. */
  boost: number | null;
  /** Enemy regeneration in %/h of the city's health. */
  resistance: number | null;
  /** Net progress in %/h, measured by the collector. */
  rate: number | null;
  /** The planet's liberation at which it opens, for a city that doesn't open from the start. */
  opensAt: number | null;
  /** Locked, and the planet hasn't reached `opensAt` yet. */
  waiting: boolean;
}

/** Hours until a city is liberated at its current net rate, or null if it isn't gaining. */
export function cityHoursLeft(c: Pick<City, "percent" | "rate">): number | null {
  if (c.percent == null || c.rate == null || c.rate <= 0) return null;
  return (100 - c.percent) / c.rate;
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
    const f = r.availabilityFactor;
    const opensAt = f != null && f > 0 && f < 1 ? (1 - f) * 100 : null;
    const state = live === "liberating" ? live : (fate?.state ?? live);
    return {
      id: r.id,
      name: fate?.name ?? r.name,
      formerly: fate?.name ? r.name : undefined,
      size: r.size,
      stars: cityStars(r.size),
      // The API sends the string "null" for regions without one.
      description: r.description === "null" ? "" : (r.description ?? ""),
      note: fate?.note,
      state,
      percent,
      players: r.players,
      boost: p.maxHealth ? ((1.5 * r.maxHealth) / p.maxHealth) * 100 : null,
      resistance: r.regenPerSecond != null && r.maxHealth ? ((r.regenPerSecond * 3600) / r.maxHealth) * 100 : null,
      rate: r.rate ?? null,
      opensAt,
      waiting: state === "locked" && opensAt != null && !p.event && liberationPercent(p) < opensAt,
    };
  });
}
