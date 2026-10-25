import { describe, expect, it } from "vitest";
import { isNewer, releaseNumber, versionParts } from "../domain/version";

describe("versions", () => {
  it("reads release tags", () => {
    expect(versionParts("v0.2.8-alpha")).toEqual([0, 2, 8]);
    expect(versionParts("1.10")).toEqual([1, 10]);
  });

  it("drops the pre-release label", () => {
    expect(releaseNumber("0.3.0-beta")).toBe("0.3.0");
    expect(releaseNumber("v0.3.0-beta.2+build.5")).toBe("0.3.0");
    expect(releaseNumber("0.2.7")).toBe("0.2.7");
  });

  it("compares by number, not text", () => {
    expect(isNewer("v0.2.10", "0.2.9")).toBe(true);
    expect(isNewer("v0.3.0", "0.2.9")).toBe(true);
    expect(isNewer("v0.2.6", "0.2.7")).toBe(false);
    expect(isNewer("v0.3.0-beta", "0.2.7")).toBe(true);
  });

  it("puts a full release after its pre-releases", () => {
    expect(isNewer("v0.3.0", "0.3.0-beta")).toBe(true);
    expect(isNewer("v0.3.0-beta", "0.3.0")).toBe(false);
    expect(isNewer("v0.2.7-alpha", "0.2.7")).toBe(false);
    expect(isNewer("v0.3.0-beta", "0.3.0-beta")).toBe(false);
  });

  it("orders pre-releases by label", () => {
    expect(isNewer("v0.3.0-rc", "0.3.0-beta")).toBe(true);
    expect(isNewer("v0.3.0-beta.10", "0.3.0-beta.2")).toBe(true);
    expect(isNewer("v0.3.0-alpha", "0.3.0-beta")).toBe(false);
  });
});
