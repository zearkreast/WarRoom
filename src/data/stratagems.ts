import type { Stratagem } from "../domain/stratagems";

/** Keyed in on the Hellpad masthead, it opens the G.W.W.'s Super Destroyer manual. */
export const CALL_IN: Stratagem = {
  name: "Call In Super Destroyer",
  code: ["up", "up", "down", "down", "left", "right", "left", "right"],
};
