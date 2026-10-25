export type Arrow = "up" | "down" | "left" | "right";

export interface Stratagem {
  name: string;
  code: Arrow[];
}

export type Dialled = { kind: "partial" } | { kind: "match"; stratagem: Stratagem } | { kind: "miss" };

/** What a code dialled so far is: the start of some code, a whole one, or nothing on file. */
export function dial(entered: Arrow[], list: Stratagem[]): Dialled {
  const fits = list.filter((s) => entered.every((a, i) => s.code[i] === a));
  if (fits.length === 0) return { kind: "miss" };
  const whole = fits.find((s) => s.code.length === entered.length);
  return whole ? { kind: "match", stratagem: whole } : { kind: "partial" };
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
