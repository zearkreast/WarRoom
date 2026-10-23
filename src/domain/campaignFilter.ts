/** Fronts with at least this many Helldivers always show, moving or not. */
export const MIN_PLAYERS = 5000;

/** Net %/h after enemy regen; below it a front is holding (+6%/h gross can meet -5.6%/h regen). */
export const MIN_RATE = 0.4;

export interface FrontActivity {
  players: number;
  /** Net progress in %/h, null while the collector is still measuring. */
  rate: number | null;
  inMajorOrder: boolean;
  pinned: boolean;
}

/** A front too small to matter that isn't gaining ground. */
export function isStalled(f: FrontActivity): boolean {
  if (f.inMajorOrder || f.pinned) return false;
  if (f.players >= MIN_PLAYERS) return false;
  return f.rate == null || f.rate < MIN_RATE;
}
