import { useEffect } from "react";
import { useUi, type GwwPage, type HellpadModule, type InfoDialog, type View } from "./state";

/** What Back and Forward step through: the screen, the Hellpad module, the G.W.W. page and the open overlay. */
export interface Place {
  view: View;
  module: HellpadModule | null;
  gww: GwwPage[];
  detail: number | null;
  dialog: InfoDialog | null;
  dispatchOpen: boolean;
}

/** A history entry's state: its place, and the place of the entry before it. */
interface Entry {
  warroom: 1;
  place: Place;
  prev: Place | null;
}

export interface HistoryLike {
  readonly state: unknown;
  pushState(data: unknown, unused: string): void;
  replaceState(data: unknown, unused: string): void;
  back(): void;
}

export const placeOf = ({ view, module, gww, detail, dialog, dispatchOpen }: Place): Place => ({
  view,
  module,
  gww,
  detail,
  dialog,
  dispatchOpen,
});

export const samePlace = (a: Place, b: Place): boolean => JSON.stringify(placeOf(a)) === JSON.stringify(placeOf(b));

const isEntry = (s: unknown): s is Entry => {
  const e = s as Entry | null;
  return e?.warroom === 1 && typeof e.place?.view === "string" && Array.isArray(e.place.gww);
};

/**
 * Keeps the history in step with the app. A new place is pushed; returning to the place before is a step
 * back, so closing an overlay or the G.W.W.'s own Back leaves nothing for Back to reopen. A reload resumes
 * the entry's place.
 */
export function syncHistory(h: HistoryLike, get: () => Place, set: (p: Place) => void) {
  let entry: Entry;
  // A step back of ours is in flight; changes made meanwhile are caught up when it lands.
  let stepping = false;
  if (isEntry(h.state)) {
    entry = h.state;
    set(entry.place);
  } else {
    entry = { warroom: 1, place: get(), prev: null };
    h.replaceState(entry, "");
  }
  const onChange = () => {
    if (stepping) return;
    const now = get();
    if (samePlace(now, entry.place)) return;
    if (entry.prev && samePlace(now, entry.prev)) {
      stepping = true;
      h.back();
      return;
    }
    // One planet page opened from another takes its entry, so closing it steps back past both.
    const swapped = now.detail != null && entry.place.detail != null;
    if (swapped && samePlace({ ...now, detail: entry.place.detail }, entry.place)) {
      entry = { ...entry, place: now };
      h.replaceState(entry, "");
      return;
    }
    entry = { warroom: 1, place: now, prev: entry.place };
    h.pushState(entry, "");
  };
  const onPop = (state: unknown) => {
    if (!isEntry(state)) return;
    const ours = stepping;
    stepping = false;
    entry = state;
    if (ours) onChange();
    else set(state.place);
  };
  return { onChange, onPop };
}

/** Puts back the place a reload left; called before the first render so the default screen never shows. */
export function resumePlace() {
  const s = window.history.state;
  if (isEntry(s)) useUi.setState(s.place);
}

/** Makes the browser's and the phone's Back button step back through the app. */
export function useNavHistory() {
  useEffect(() => {
    const sync = syncHistory(
      window.history,
      () => placeOf(useUi.getState()),
      (p) => {
        // An unchanged G.W.W. page list keeps its identity, or the G.W.W. would take it for navigation.
        const { gww } = useUi.getState();
        useUi.setState({ ...p, gww: JSON.stringify(gww) === JSON.stringify(p.gww) ? gww : p.gww, options: null });
      },
    );
    const stop = useUi.subscribe(sync.onChange);
    const onPop = (e: PopStateEvent) => sync.onPop(e.state);
    window.addEventListener("popstate", onPop);
    return () => {
      stop();
      window.removeEventListener("popstate", onPop);
    };
  }, []);
}
