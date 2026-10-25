import { describe, expect, it } from "vitest";
import { CALL_IN } from "../data/stratagems";
import { arrowOf, step } from "../domain/stratagems";

describe("stratagem codes", () => {
  const code = CALL_IN.code;

  it("steps on with the next arrow and falls back on a wrong one", () => {
    expect(code.reduce((at, a) => step(at, a, code), 0)).toBe(code.length);
    expect(step(3, "right", code)).toBe(0);
    expect(step(3, "up", code)).toBe(1);
  });

  it("keeps the presses that still start the code", () => {
    expect(step(2, "up", code)).toBe(2);
  });

  it("reads W A S D and the arrow keys", () => {
    expect(["w", "A", "s", "d", "ArrowUp", "ArrowLeft"].map(arrowOf)).toEqual(["up", "left", "down", "right", "up", "left"]);
    expect(arrowOf("q")).toBeUndefined();
  });
});
