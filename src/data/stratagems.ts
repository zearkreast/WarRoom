import type { Stratagem } from "../domain/stratagems";

/** Not a stratagem the game has: the Hellpad's own, which opens the G.W.W.'s Super Destroyer manual. */
export const UPLINK: Stratagem = { name: "Super Destroyer Uplink", code: ["up", "right", "down", "down", "left", "up"] };

/** Codes as the Helldivers Wiki gives them; no code may be the start of another. */
export const STRATAGEMS: Stratagem[] = [
  UPLINK,
  { name: "Reinforce", code: ["up", "down", "right", "left", "up"] },
  { name: "Resupply", code: ["down", "down", "up", "right"] },
  { name: "NUX-223 Hellbomb", code: ["down", "up", "left", "down", "up", "right", "down", "up"] },
  { name: "Eagle 500kg Bomb", code: ["up", "right", "down", "down", "down"] },
  { name: "Eagle Airstrike", code: ["up", "right", "down", "right"] },
  { name: "Super Earth Flag", code: ["down", "up", "down", "up"] },
  { name: "Seismic Probe", code: ["up", "up", "left", "right", "down", "down"] },
  { name: "Upload Data", code: ["left", "right", "up", "up", "up"] },
];
