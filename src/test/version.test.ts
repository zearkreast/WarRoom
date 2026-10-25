import { describe, expect, it } from "vitest";
import { isNewer, versionParts } from "../domain/version";

describe("versions", () => {
  it("reads release tags", () => {
    expect(versionParts("v0.2.8-alpha")).toEqual([0, 2, 8]);
    expect(versionParts("1.10")).toEqual([1, 10]);
  });

  it("compares by number, not text", () => {
    expect(isNewer("v0.2.10", "0.2.9")).toBe(true);
    expect(isNewer("v0.3.0", "0.2.9")).toBe(true);
    expect(isNewer("v0.2.7-alpha", "0.2.7")).toBe(false);
    expect(isNewer("v0.2.6", "0.2.7")).toBe(false);
  });
});
