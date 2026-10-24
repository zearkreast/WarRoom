/**
 * Names for galactic effect ids the community list (planetEffects.json) doesn't carry yet, and fixes
 * to the ones it gets wrong. Unnamed ids are left off the map and the planet page until added here.
 * Ids seen without a name on 24 Oct 2026: 1272 (Mastia), 1315–1317 (Esker, Erson Sands, Socorro III),
 * 1318–1320 (Zagon Prime, Omicron), 1327 (Grafmere), 1333 (Oshaune), 1363 (Aurora Bay),
 * 1365 (Chort Bay), 1366 (Turing), 1418 (Calypso).
 */
export const EFFECT_OVERRIDES: Record<number, { name: string; description: string }> = {};
