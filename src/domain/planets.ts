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
