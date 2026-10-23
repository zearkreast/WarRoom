import type { Planet } from "../api/types";
import type { PlanetStatic } from "./snapshot";

/**
 * Where the game has moved things the API still files under the old planet id. Meridia was dragged
 * into the Celeste sector. On 12 Aug 2026 Sangis and Alderidge Cove traded names, places, biomes and
 * links: the API has the new places and links but the old names, sectors and biomes. Values are
 * absolute, so they stay right if the API catches up.
 */
const STATIC_FIXES: Record<number, Partial<Pick<PlanetStatic, "name" | "sector" | "biome">>> = {
  64: { sector: "Celeste" },
  99: {
    name: "ALDERIDGE COVE",
    sector: "Omega",
    biome: {
      name: "Deadlands",
      description:
        "The lifeless grey of this planet is interrupted only by the violet flowers that grow from strange, parasitic outcroppings.",
    },
  },
  273: {
    name: "SANGIS",
    sector: "Guang",
    biome: {
      name: "Deciduous Autumn Forest",
      description:
        "Broadleaf woodlands spread across rolling hills, broken by shallow waters that wind through mossy logs, flowers, and stone. (UNOFFICIAL)",
    },
  },
};

export const fixStatic = <T extends PlanetStatic>(p: T): T =>
  STATIC_FIXES[p.index] ? { ...p, ...STATIC_FIXES[p.index] } : p;

const cache = new WeakMap<Planet[], Map<number, Planet>>();

/** Planet indices are not array positions (the list has gaps), so look them up by index. */
export function byIndex(planets: Planet[]): Map<number, Planet> {
  let m = cache.get(planets);
  if (!m) {
    m = new Map(planets.map((p) => [p.index, p]));
    cache.set(planets, m);
  }
  return m;
}

/** The API stacks unnamed placeholder worlds on Super Earth at the origin; nothing in the war uses them. */
export const isPlaceholder = (p: Pick<Planet, "index" | "position">) =>
  p.index !== 0 && p.position.x === 0 && p.position.y === 0;

/**
 * Who holds and who attacks each planet, as one string. Territory and region shapes depend on nothing
 * else that changes, so they are recomputed only when this does, not on every poll.
 */
export function ownershipKey(planets: Pick<Planet, "currentOwner" | "event">[]): string {
  return planets.map((p) => `${p.currentOwner[0]}${p.event?.faction[0] ?? "-"}`).join("");
}
