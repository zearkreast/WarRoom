import { describe, expect, it } from "vitest";
import { bodyEchoesTitle, dayLabel, groupByDay, matchesQuery, plainText } from "../domain/newsFeed";

const at = (iso: string) => Date.parse(iso);

describe("news feed", () => {
  it("spots an alert whose body only repeats its headline", () => {
    const alert =
      "<i=3>TERREK IS UNDER ATTACK BY THE TERMINIDS! ALL HELLDIVERS MUST DEPLOY IMMEDIATELY TO DEFEND IT!</i>\nTerrek is under attack by the Terminids! All Helldivers must deploy immediately to defend it!";
    expect(bodyEchoesTitle(alert)).toBe(true);
    expect(bodyEchoesTitle("<i=3>ARSENAL BOOST</i>\nHigh Command has authorized high-volume use.")).toBe(false);
    expect(bodyEchoesTitle("<i=3>BRIEF</i>")).toBe(true);
  });

  it("strips the game's markup", () => {
    expect(plainText("Defend <i=1>Malevelon Creek</i> now")).toBe("Defend Malevelon Creek now");
  });

  it("names days on Super Earth Standard Time", () => {
    const now = at("2026-10-25T18:00:00Z");
    expect(dayLabel(at("2026-10-25T00:10:00Z"), now)).toBe("Today");
    expect(dayLabel(at("2026-10-24T23:59:00Z"), now)).toBe("Yesterday");
    expect(dayLabel(at("2026-10-22T12:00:00Z"), now)).toBe("22/10/2186");
  });

  it("groups items by day and keeps their order", () => {
    const now = at("2026-10-25T18:00:00Z");
    const items = [
      { at: at("2026-10-25T09:00:00Z"), id: "a" },
      { at: at("2026-10-25T01:00:00Z"), id: "b" },
      { at: at("2026-10-23T20:00:00Z"), id: "c" },
    ];
    const groups = groupByDay(items, now);
    expect(groups.map((g) => g.label)).toEqual(["Today", "23/10/2186"]);
    expect(groups[0].items.map((i) => i.id)).toEqual(["a", "b"]);
  });

  it("matches every word of a search, in any order", () => {
    expect(matchesQuery("The Automaton Incineration Corps on Vindemitarix Prime", "corps vindemitarix")).toBe(true);
    expect(matchesQuery("Arsenal boost", "arsenal terminid")).toBe(false);
    expect(matchesQuery("anything", "  ")).toBe(true);
  });
});
