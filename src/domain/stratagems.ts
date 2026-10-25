export type Arrow = "up" | "down" | "left" | "right";

export interface Stratagem {
  name: string;
  code: Arrow[];
}

/** How far a code is keyed in after one more arrow: the longest start of the code the last presses spell. */
export function step(at: number, a: Arrow, code: Arrow[]): number {
  const keyed = [...code.slice(0, at), a];
  for (let n = Math.min(keyed.length, code.length); n > 0; n--) {
    if (keyed.slice(-n).every((k, i) => k === code[i])) return n;
  }
  return 0;
}

const KEYS: Record<string, Arrow> = {
  w: "up",
  a: "left",
  s: "down",
  d: "right",
  arrowup: "up",
  arrowleft: "left",
  arrowdown: "down",
  arrowright: "right",
};

/** The arrow a key dials: W A S D or the arrow keys, as in the game. */
export const arrowOf = (key: string): Arrow | undefined => KEYS[key.toLowerCase()];
