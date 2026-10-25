import { describe, expect, it } from "vitest";
import { CHAPTERS, MANUALS, TIPS } from "../data/gww";
import { ADS, COMPANIES, GOV, MINISTRIES, type SiteSection } from "../data/gwwSites";
import { PAST_STATIONS, STAR_OF_PEACE } from "../data/stations";
import { linksIn, nextYearly, parseRich, plainText } from "../domain/gwwText";
import { adUrl, iconUrl, logoUrl } from "../features/hellpad/gwwArt";
import { APP_PAGES, pageOf } from "../features/hellpad/gwwNav";
import planetsJson from "./fixtures/planets.json";

const allText = (): string[] => {
  const sections: SiteSection[] = [
    ...MANUALS.flatMap((m) => m.sections),
    ...MINISTRIES.flatMap((m) => m.sections),
    ...COMPANIES.flatMap((c) => c.sections),
    ...GOV.sections,
  ];
  return [
    ...sections.flatMap((s) => [
      s.text ?? "",
      ...(s.points ?? []),
      ...(s.table?.rows ?? []).flatMap((r) => r.cells),
      ...(s.kinds ?? []).map((k) => k.text),
      ...(s.faq ?? []).map((f) => f.a),
      ...(s.timeline ?? []).map((d) => d.text),
    ]),
    ...MANUALS.flatMap((m) => m.tips),
  ];
};

const targets = (): string[] => [
  ...allText().flatMap(linksIn),
  ...MANUALS.flatMap((m) => m.related),
  ...MINISTRIES.flatMap((m) => [...(m.related ?? []), ...(m.ties ?? []).map((t) => t.to)]),
  ...COMPANIES.flatMap((c) => [...(c.related ?? []), ...(c.ties ?? []).map((t) => t.to)]),
  ...GOV.related,
  ...ADS.flatMap((a) => ("by" in a ? [a.by] : [])),
];

describe("galactic wide web", () => {
  it("finds every picture its pages name", () => {
    const icons = [
      ...MANUALS.flatMap((m) => [
        ...(m.icon ? [m.icon] : []),
        ...m.sections.flatMap((s) => [...(s.kinds ?? []).map((k) => k.icon), ...(s.table?.rows ?? []).flatMap((r) => (r.icon ? [r.icon] : []))]),
      ]),
      ...MINISTRIES.map((m) => m.icon),
      GOV.icon,
    ];
    expect(icons.filter((i) => !iconUrl(i))).toEqual([]);
    expect(COMPANIES.flatMap((c) => [c.logo, c.showcase?.image].filter((l): l is string => !!l && !logoUrl(l)))).toEqual([]);
    const placed = [...MINISTRIES.flatMap((m) => m.ads ?? []), ...COMPANIES.flatMap((c) => c.ads ?? [])];
    expect([...ADS.map((a) => a.file), ...placed].filter((a) => !adUrl(a))).toEqual([]);
    expect(placed.filter((f) => !ADS.some((a) => a.file === f))).toEqual([]);
    // A site's own advert shows on its site, so it has to link there.
    const site = (f: string) => [...MINISTRIES, ...COMPANIES].find((s) => s.ads?.includes(f));
    expect(ADS.filter((a) => site(a.file) && !("by" in a && a.by.endsWith(`:${site(a.file)!.id}`))).map((a) => a.file)).toEqual([]);
  });

  it("gives every site its own address and id", () => {
    const addresses = [...MANUALS, ...MINISTRIES, ...COMPANIES, GOV].map((s) => s.address);
    expect(new Set(addresses).size).toBe(addresses.length);
    for (const list of [MANUALS, MINISTRIES, COMPANIES]) expect(new Set(list.map((x) => x.id)).size).toBe(list.length);
  });

  it("links only to pages, tracker views and planets that exist", () => {
    const planets = new Set((planetsJson as { name: string }[]).map((p) => p.name.toLowerCase()));
    const broken = targets().filter((t) => {
      const id = t.slice(t.indexOf(":") + 1);
      if (t.startsWith("planet:")) return !planets.has(id.toLowerCase());
      if (t.startsWith("app:")) return !(id in APP_PAGES);
      return !pageOf(t);
    });
    expect(broken).toEqual([]);
  });

  it("names only planets that exist in the station histories", () => {
    const planets = new Set((planetsJson as { name: string }[]).map((p) => p.name.toLowerCase()));
    const text = [
      STAR_OF_PEACE.summary,
      ...STAR_OF_PEACE.points,
      ...STAR_OF_PEACE.timeline.map((e) => e.text),
      ...PAST_STATIONS.flatMap((s) => [s.note, ...s.sections.map((x) => x.text), ...s.timeline.map((e) => e.text)]),
    ];
    const links = text.flatMap(linksIn);
    expect(links.filter((t) => !t.startsWith("planet:") || !planets.has(t.slice(7).toLowerCase()))).toEqual([]);
  });

  it("puts every manual in a chapter, and the chapters in reading order", () => {
    const order = CHAPTERS.map((c) => c.id);
    const seq = MANUALS.map((m) => order.indexOf(m.chapter));
    expect(seq.every((c) => c >= 0)).toBe(true);
    expect([...seq].sort((a, b) => a - b)).toEqual(seq);
    expect(TIPS.length).toBeGreaterThan(0);
  });
});

describe("g.w.w. text", () => {
  it("splits links out of text", () => {
    expect(parseRich("See [[manual:defense|gambits]] and [[app:map|the map]].")).toEqual([
      { text: "See " },
      { kind: "manual", target: "defense", label: "gambits" },
      { text: " and " },
      { kind: "app", target: "map", label: "the map" },
      { text: "." },
    ]);
    expect(parseRich("[[nope:x|plain]] stays")).toEqual([{ text: "plain" }, { text: " stays" }]);
    expect(plainText("On [[planet:Tien Kwan|Tien Kwan]].")).toBe("On Tien Kwan.");
  });

  it("counts down to the next yearly day", () => {
    const days = [
      { name: "Liberty Day", month: 10, day: 26 },
      { name: "Memorial", month: 4, day: 3 },
    ];
    expect(nextYearly(days, new Date(2026, 9, 25, 23, 0))).toMatchObject({ name: "Liberty Day", in: 1 });
    expect(nextYearly(days, new Date(2026, 9, 26, 8, 0))).toMatchObject({ name: "Liberty Day", in: 0 });
    expect(nextYearly(days, new Date(2026, 9, 27))).toMatchObject({ name: "Memorial", in: 158 });
  });
});
