import { describe, expect, it } from "vitest";
import { syncHistory, type HistoryLike, type Place } from "../navHistory";

const at = (over: Partial<Place> = {}): Place => ({
  view: "campaigns",
  module: null,
  gww: [{ kind: "home" }],
  detail: null,
  dialog: null,
  dispatchOpen: false,
  ...over,
});

interface FakeHistory extends HistoryLike {
  entries: unknown[];
  index(): number;
  land(): void;
  browserBack(): void;
  listen(f: (s: unknown) => void): void;
}

/** A session history whose back() lands only when `land()` is called, as popstate arrives later in a browser. */
function fakeHistory(start: unknown = null) {
  const entries: unknown[] = [start];
  let i = 0;
  let pending = 0;
  let onPop: (s: unknown) => void = () => {};
  const h: FakeHistory = {
    get state() {
      return entries[i];
    },
    pushState(data) {
      entries.splice(i + 1, entries.length, data);
      i++;
    },
    replaceState(data) {
      entries[i] = data;
    },
    back() {
      pending++;
    },
    entries,
    index: () => i,
    land() {
      while (pending > 0) {
        pending--;
        i--;
        onPop(entries[i]);
      }
    },
    browserBack() {
      i--;
      onPop(entries[i]);
    },
    listen(f) {
      onPop = f;
    },
  };
  return h;
}

function app(h: FakeHistory, first = at()) {
  let place = first;
  const sync = syncHistory(
    h,
    () => place,
    (p) => (place = p),
  );
  h.listen(sync.onPop);
  return {
    go(p: Place) {
      place = p;
      sync.onChange();
    },
    get place() {
      return place;
    },
  };
}

describe("nav history", () => {
  it("pushes each new place and restores it on Back", () => {
    const h = fakeHistory();
    const a = app(h);
    a.go(at({ view: "map" }));
    a.go(at({ view: "map", detail: 7 }));
    expect(h.index()).toBe(2);
    h.browserBack();
    expect(a.place).toEqual(at({ view: "map" }));
    h.browserBack();
    expect(a.place).toEqual(at());
  });

  it("turns a return to the place before into a step back, not a new entry", () => {
    const h = fakeHistory();
    const a = app(h);
    a.go(at({ view: "hellpad", module: "gww" }));
    a.go(at({ view: "hellpad", module: "gww", gww: [{ kind: "home" }, { kind: "manual", id: "basics" }] }));
    a.go(at({ view: "hellpad", module: "gww" }));
    h.land();
    expect(h.index()).toBe(1);
    expect(h.entries).toHaveLength(3);
    expect(a.place).toEqual(at({ view: "hellpad", module: "gww" }));
  });

  it("catches up on a change made while its step back is in flight", () => {
    const h = fakeHistory();
    const a = app(h);
    a.go(at({ dialog: "about" }));
    a.go(at());
    a.go(at({ view: "news" }));
    h.land();
    expect(h.index()).toBe(1);
    expect(a.place).toEqual(at({ view: "news" }));
    h.browserBack();
    expect(a.place).toEqual(at());
  });

  it("resumes the entry's place after a reload", () => {
    const h = fakeHistory();
    const a = app(h);
    a.go(at({ view: "overview" }));
    const again = app(h);
    expect(again.place).toEqual(at({ view: "overview" }));
    again.go(at());
    h.land();
    expect(h.index()).toBe(0);
  });
});
