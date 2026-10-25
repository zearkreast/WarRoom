import { describe, expect, it } from "vitest";
import { STRATAGEMS, UPLINK } from "../data/stratagems";
import { arrowOf, dial } from "../domain/stratagems";

describe("stratagem terminal", () => {
  it("knows a code by its start, its whole, or not at all", () => {
    expect(dial(["up", "down"], STRATAGEMS)).toEqual({ kind: "partial" });
    expect(dial(["up", "down", "right", "left", "up"], STRATAGEMS)).toMatchObject({ kind: "match", stratagem: { name: "Reinforce" } });
    expect(dial([...UPLINK.code], STRATAGEMS)).toEqual({ kind: "match", stratagem: UPLINK });
    expect(dial(["left", "left"], STRATAGEMS)).toEqual({ kind: "miss" });
  });

  it("has no code that is the start of another, so every code can be dialled", () => {
    const starts = STRATAGEMS.flatMap((a) =>
      STRATAGEMS.filter((b) => a !== b && a.code.every((k, i) => b.code[i] === k)).map((b) => `${a.name} → ${b.name}`),
    );
    expect(starts).toEqual([]);
  });

  it("reads W A S D and the arrow keys", () => {
    expect(["w", "A", "s", "d", "ArrowUp", "ArrowLeft"].map(arrowOf)).toEqual(["up", "left", "down", "right", "up", "left"]);
    expect(arrowOf("q")).toBeUndefined();
  });
});
