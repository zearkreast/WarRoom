/**
 * Names for galactic effect ids the community list (planetEffects.json) doesn't carry yet, and fixes
 * to the ones it gets wrong. Unnamed ids are left off the map and the planet page until added here.
 * Ids seen without a name on 24 Oct 2026: 1272 (Mastia), 1315–1317 (Esker, Erson Sands, Socorro III),
 * 1318–1320 (Zagon Prime, Omicron), 1327 (Grafmere), 1363 (Aurora Bay),
 * 1365 (Chort Bay), 1366 (Turing), 1418 (Calypso).
 */
export const EFFECT_OVERRIDES: Record<number, { name: string; description: string }> = {
  // Named from the game's own planet tooltip, 25 Oct 2026.
  1333: {
    name: "E-711 EXTRACTION FACILITY",
    description:
      "Extensive mining and drilling operations are taking place to comb this former Hive World for all traces of the highly valuable substance known as E-711.",
  },
};
