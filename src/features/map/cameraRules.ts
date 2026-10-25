/**
 * The counter-transforms that keep planet markers and labels upright, round and slowly growing however
 * the camera turns, tilts and zooms. They are a handful of rules rewritten each frame: an inherited
 * CSS variable would restyle every node in the map on each change (11-18 ms on desktop).
 */
const SELECTORS = {
  upright: ".galaxy .upright",
  body: ".galaxy .pbody",
  label: ".galaxy .planet-label",
  tag: ".galaxy .planet-tag",
  sphereLabel: ".galaxy.pl-spheres .planet-label",
  sphereTag: ".galaxy.pl-spheres .planet-tag",
  poiLabel: ".galaxy .planet.poi .planet-label",
  poiSphereLabel: ".galaxy.pl-spheres .planet.poi .planet-label",
  holoFloor: ".galaxy .holo-floor",
  holoBeam: ".galaxy .holo-beam, .galaxy .holo-edge",
  holoLift: ".galaxy .holo-lift",
} as const;

type Name = keyof typeof SELECTORS;

let rules: Record<Name, CSSStyleRule> | null = null;
const written = new Map<string, string>();

function sheetRules(): Record<Name, CSSStyleRule> {
  if (rules) return rules;
  const sheet = new CSSStyleSheet();
  const names = Object.keys(SELECTORS) as Name[];
  names.forEach((n, i) => sheet.insertRule(`${SELECTORS[n]} {}`, i));
  document.adoptedStyleSheets = [...document.adoptedStyleSheets, sheet];
  return (rules = Object.fromEntries(names.map((n, i) => [n, sheet.cssRules[i] as CSSStyleRule])) as Record<
    Name,
    CSSStyleRule
  >);
}

function set(name: Name, prop: string, value: string) {
  const key = `${name}:${prop}`;
  if (written.get(key) === value) return;
  written.set(key, value);
  sheetRules()[name].style.setProperty(prop, value);
}

const px = (v: number) => `${v.toFixed(4)}px`;

/** Writes the counter-transforms for zoom `k`, a turn of `rot` radians and a tilt of `tilt` radians. */
export function writeCamera(k: number, rot: number, tilt: number) {
  set("upright", "transform", `rotate(${((-rot * 180) / Math.PI).toFixed(3)}deg) scale(1, ${(1 / Math.cos(tilt)).toFixed(4)})`);
  // Markers grow slower than the map when zooming in.
  const marker = 1.4 / Math.pow(k, 0.55);
  set("body", "transform", `scale(${marker.toFixed(4)})`);
  const below = px(8 * marker + 9 / k);
  set("label", "font-size", px(10 / k));
  set("label", "stroke-width", px(3 / k));
  set("label", "transform", `translateY(${px(-8 * marker - 3 / k)})`);
  set("tag", "font-size", px(8 / k));
  set("tag", "stroke-width", px(3 / k));
  set("tag", "transform", `translateY(${below})`);
  // Spheres carry the active-planet tag on top, so their names go underneath, as on the in-game map.
  set("sphereLabel", "transform", `translateY(${below})`);
  set("sphereTag", "transform", `translateY(${px(8 * marker + 19 / k)})`);
  // Over a point of interest's logo, which sits where the name would.
  set("poiLabel", "transform", `translateY(${px(-16.5 * marker - 3 / k)})`);
  set("poiSphereLabel", "transform", `translateY(${below})`);
  // Hologram projectors lie flat on the table, flatter as the map tilts; units float higher as it tilts away.
  const lift = 4 + 6 * Math.sin(tilt);
  set("holoFloor", "transform", `scale(1, ${Math.min(0.5, Math.cos(tilt)).toFixed(4)})`);
  set("holoBeam", "transform", `scale(1, ${(lift + 6).toFixed(4)})`);
  set("holoLift", "transform", `translateY(${px(-lift)})`);
}
