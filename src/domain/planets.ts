import type { Planet } from "../api/types";

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
