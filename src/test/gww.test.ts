import { describe, expect, it } from "vitest";
import { ADS, COMPANIES, MANUALS, MINISTRIES, TIPS } from "../data/gww";
import { adUrl, iconUrl, logoUrl } from "../features/hellpad/gwwArt";

describe("galactic wide web", () => {
  it("finds every picture its pages name", () => {
    const icons = [
      ...MANUALS.flatMap((m) => [
        ...(m.icon ? [m.icon] : []),
        ...m.sections.flatMap((s) => [...(s.kinds ?? []).map((k) => k.icon), ...(s.table?.rows ?? []).flatMap((r) => (r.icon ? [r.icon] : []))]),
      ]),
      ...MINISTRIES.map((m) => m.icon),
      "super-earth",
    ];
    expect(icons.filter((i) => !iconUrl(i))).toEqual([]);
    expect(COMPANIES.flatMap((c) => (c.logo && !logoUrl(c.logo) ? [c.logo] : []))).toEqual([]);
    const ads = [...ADS.map((a) => a.file), ...MINISTRIES.flatMap((m) => m.ads ?? []), ...COMPANIES.flatMap((c) => c.ads ?? [])];
    expect(ads.filter((a) => !adUrl(a))).toEqual([]);
  });

  it("gives every site its own address", () => {
    const addresses = [...MANUALS, ...MINISTRIES, ...COMPANIES].map((s) => s.address);
    expect(new Set(addresses).size).toBe(addresses.length);
    expect(new Set(MANUALS.map((m) => m.id)).size).toBe(MANUALS.length);
  });

  it("links ministries only to corporations it lists", () => {
    const names = new Set(COMPANIES.map((c) => c.name));
    expect(MINISTRIES.flatMap((m) => (m.partners ?? []).filter((p) => !names.has(p)))).toEqual([]);
    expect(TIPS.length).toBeGreaterThan(0);
  });
});
