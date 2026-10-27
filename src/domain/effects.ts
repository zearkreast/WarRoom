import { EFFECT_OVERRIDES } from "../data/effectOverrides";
import LIST from "../data/planetEffects.json";

/**
 * What an effect is to the player, which decides where it shows:
 * subfaction, threat, seaf and poi get a map pawn; look and gloom change how the planet is drawn;
 * the rest only appear in the planet's operational parameters.
 */
export type EffectKind =
  | "subfaction"
  | "threat"
  | "seaf"
  | "poi"
  | "dss"
  | "arsenal"
  | "condition"
  | "event"
  | "look"
  | "gloom"
  | "hidden";

export interface PlanetEffect {
  id: number;
  kind: EffectKind;
  name: string;
  description: string;
  /** When a timed effect, such as an arsenal augmentation, ends: epoch ms. */
  until?: number;
}

const NAMES = { ...(LIST as Record<string, { name: string; description: string }>) };

// Each enemy unit comes as a pair, one id per side of the fight; both name the same thing.
const KINDS: [EffectKind, number[]][] = [
  [
    "subfaction",
    [1202, 1203, 1243, 1244, 1245, 1248, 1249, 1269, 1303, 1306, 1307, 1308, 1309, 1310, 1360, 1361, 1377, 1378, 1379, 1380, 1386, 1402, 1403, 1413, 1414],
  ],
  ["threat", [1283, 1285, 1288, 1293, 1355, 1357, 1359]],
  ["seaf", [1400, 1401]],
  ["dss", [1209, 1210, 1211, 1214, 1215, 1216, 1217, 1238, 1261, 1262]],
  ["condition", [1373, 1374, 1375]],
  ["event", [1204, 1240, 1242, 1329, 1407, 1412]],
  ["look", [1177, 1228, 1229, 1230, 1241, 1252, 1311, 1352, 1376]],
  ["gloom", [1186, 1187, 1188, 1190, 1193]],
  ["hidden", [1212, 1213, 1222, 1246, 1406]],
];
const KIND_OF = new Map(KINDS.flatMap(([kind, ids]) => ids.map((id) => [id, kind] as const)));
const ORDER: EffectKind[] = ["subfaction", "threat", "seaf", "poi", "dss", "arsenal", "condition", "event", "look", "gloom"];

/** Planets inside or on the edge of the Gloom; the map draws its cloud over them. */
export const GLOOM_IDS = new Set([1186, 1187, 1188, 1190, 1193]);

function describe(id: number): PlanetEffect | null {
  const named = EFFECT_OVERRIDES[id] ?? NAMES[String(id)];
  if (!named) return null;
  const kind = KIND_OF.get(id) ?? (/^ARSENAL AUGMENTATION/i.test(named.name) ? "arsenal" : "poi");
  // "(Enemies)" marks the enemy-side twin of an id; the player knows them by one name.
  const name = named.name.replace(/\s*\((enemies|marker|old map marker)\)\s*$/i, "");
  return { id, kind, name, description: named.description.replace(/<i=\d+>|<\/i>/g, "") };
}

/** A planet's effects as the player sees them: named, one per name, ordered as the game lists them. */
export function planetEffects(ids: number[]): PlanetEffect[] {
  const seen = new Set<string>();
  const out: PlanetEffect[] = [];
  for (const id of ids) {
    const e = describe(id);
    if (!e || e.kind === "hidden" || seen.has(e.name.toLowerCase())) continue;
    seen.add(e.name.toLowerCase());
    out.push(e);
  }
  return out.sort((a, b) => ORDER.indexOf(a.kind) - ORDER.indexOf(b.kind));
}

/** Effects that earn a pawn beside the planet on the map. */
export const hasPawn = (e: PlanetEffect) => e.kind === "subfaction" || e.kind === "threat" || e.kind === "seaf" || e.kind === "poi";
