import { create } from "zustand";
import { persist } from "zustand/middleware";

export type View = "campaigns" | "map" | "hellpad" | "overview" | "news" | "settings";
export type HellpadModule = "planets" | "stations" | "gww" | "campaigns";
export type InfoDialog = "about" | "legend";

/** A page of the Galactic Wide Web. */
export type GwwPage =
  | { kind: "home" }
  | { kind: "manual"; id: string }
  | { kind: "ministry"; id: string }
  | { kind: "company"; id: string }
  | { kind: "brand"; id: string }
  | { kind: "site"; id: "log" | "gov" }
  | { kind: "dir"; id: "manuals" | "gov" | "corps" | "market" };

/** Where the planet options popover is anchored, in viewport pixels. */
export interface OptionsAnchor {
  planet: number;
  x: number;
  y: number;
}

interface UiState {
  view: View;
  module: HellpadModule | null;
  /** Planet highlighted on the map. */
  selectedPlanet: number | null;
  /** Bumped on every request to centre the map on the selected planet, so repeating one still flies. */
  focusSeq: number;
  /** Planet whose detail overlay is open. */
  detail: number | null;
  options: OptionsAnchor | null;
  dispatchOpen: boolean;
  /** The About or Icons legend dialog, when one is open. */
  dialog: InfoDialog | null;
  /** The G.W.W.'s page history, kept while the app runs so leaving the module and coming back resumes it. */
  gww: GwwPage[];
  setView(v: View): void;
  openModule(m: HellpadModule | null): void;
  select(planet: number | null): void;
  showOnMap(planet: number): void;
  focus(planet: number): void;
  openDetail(planet: number | null): void;
  openOptions(o: OptionsAnchor | null): void;
  toggleDispatch(open?: boolean): void;
  showDialog(d: InfoDialog | null): void;
  setGww(stack: GwwPage[]): void;
}

export const useUi = create<UiState>((set) => ({
  view: "campaigns",
  module: null,
  selectedPlanet: null,
  focusSeq: 0,
  detail: null,
  options: null,
  dispatchOpen: false,
  dialog: null,
  gww: [{ kind: "home" }],
  setView: (view) => set({ view, options: null, module: null }),
  openModule: (module) => set({ view: "hellpad", module, options: null, detail: null }),
  select: (selectedPlanet) => set({ selectedPlanet }),
  showOnMap: (planet) =>
    set((s) => ({ view: "map", selectedPlanet: planet, focusSeq: s.focusSeq + 1, detail: null, options: null })),
  focus: (planet) => set((s) => ({ selectedPlanet: planet, focusSeq: s.focusSeq + 1, options: null })),
  openDetail: (detail) => set({ detail, options: null }),
  openOptions: (options) => set({ options }),
  toggleDispatch: (open) => set((s) => ({ dispatchOpen: open ?? !s.dispatchOpen })),
  showDialog: (dialog) => set({ dialog }),
  setGww: (gww) => set({ gww }),
}));

export interface AlertSettings {
  system: boolean;
  orders: boolean;
  dispatches: boolean;
  captures: boolean;
  defenses: boolean;
}

export type MapOverlay = "regions" | "sectors" | "galaxy";
export type MapQuality = "max" | "normal" | "min";
export type MapPlanets = "markers" | "spheres";
export type Popups = "all" | "priority" | "none";

export type MapDispatches = "3" | "5" | "10" | "72h";

interface SettingsState {
  alerts: AlertSettings;
  popups: Popups;
  mapOverlay: MapOverlay;
  mapQuality: MapQuality;
  mapPlanets: MapPlanets;
  showInactive: boolean;
  /** Whether the Galactic control ring is unfolded on the planet index. */
  planetsControlOpen: boolean;
  /** Whether the Major Order strip on Active planets is unfolded. */
  moStripOpen: boolean;
  /** Whether the Theater of War disc turns by itself. */
  twSpin: boolean;
  /** How many dispatches the map's Dispatches box lists, or "72h" for all from the last 72 hours. */
  mapDispatches: MapDispatches;
  pinned: number[];
  /** Views whose first-run hint has been dismissed. */
  hintsSeen: string[];
  /** Ids already alerted on, so a restart doesn't repeat them. */
  seenAssignments: number[];
  seenDispatch: number;
  /** App version whose changelog the user has opened. */
  seenChangelog: string;
  /** G.W.W. training manuals read to the end, by id. */
  gwwRead: string[];
  setAlert(k: keyof AlertSettings, v: boolean): void;
  set<
    K extends
      | "popups"
      | "mapOverlay"
      | "mapQuality"
      | "mapPlanets"
      | "showInactive"
      | "planetsControlOpen"
      | "moStripOpen"
      | "twSpin"
      | "mapDispatches",
  >(
    k: K,
    v: SettingsState[K],
  ): void;
  togglePin(planet: number): void;
  clearPins(): void;
  dismissHint(id: string): void;
  resetUnread(): void;
  markSeen(assignments: number[], dispatch: number): void;
  markChangelog(version: string): void;
  markRead(manual: string): void;
  clearRead(): void;
}

export const useSettings = create<SettingsState>()(
  persist(
    (set) => ({
      alerts: { system: true, orders: true, dispatches: true, captures: true, defenses: true },
      popups: "all",
      mapOverlay: "regions",
      mapQuality: "normal",
      mapPlanets: "markers",
      showInactive: false,
      planetsControlOpen: true,
      moStripOpen: false,
      twSpin: typeof window === "undefined" || !window.matchMedia?.("(prefers-reduced-motion: reduce)").matches,
      mapDispatches: "5",
      pinned: [],
      hintsSeen: [],
      seenAssignments: [],
      seenDispatch: 0,
      seenChangelog: "",
      gwwRead: [],
      setAlert: (k, v) => set((s) => ({ alerts: { ...s.alerts, [k]: v } })),
      set: (k, v) => set({ [k]: v } as Partial<SettingsState>),
      togglePin: (planet) =>
        set((s) => ({
          pinned: s.pinned.includes(planet) ? s.pinned.filter((p) => p !== planet) : [...s.pinned, planet],
        })),
      clearPins: () => set({ pinned: [] }),
      dismissHint: (id) => set((s) => ({ hintsSeen: [...new Set([...s.hintsSeen, id])] })),
      resetUnread: () => set({ seenChangelog: "", hintsSeen: [] }),
      markSeen: (seenAssignments, seenDispatch) => set({ seenAssignments, seenDispatch }),
      markChangelog: (seenChangelog) => set({ seenChangelog }),
      markRead: (manual) => set((s) => (s.gwwRead.includes(manual) ? s : { gwwRead: [...s.gwwRead, manual] })),
      clearRead: () => set({ gwwRead: [] }),
    }),
    { name: "warcommand-settings" },
  ),
);
