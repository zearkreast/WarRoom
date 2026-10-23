import { create } from "zustand";
import { persist } from "zustand/middleware";

export type View = "campaigns" | "map" | "hellpad" | "overview" | "news" | "settings";
export type HellpadModule = "planets" | "stations" | "gww" | "about" | "legend";

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
  /** Planet whose detail overlay is open. */
  detail: number | null;
  options: OptionsAnchor | null;
  dispatchOpen: boolean;
  setView(v: View): void;
  openModule(m: HellpadModule | null): void;
  select(planet: number | null): void;
  showOnMap(planet: number): void;
  openDetail(planet: number | null): void;
  openOptions(o: OptionsAnchor | null): void;
  toggleDispatch(open?: boolean): void;
}

export const useUi = create<UiState>((set) => ({
  view: "campaigns",
  module: null,
  selectedPlanet: null,
  detail: null,
  options: null,
  dispatchOpen: false,
  setView: (view) => set({ view, options: null, module: null }),
  openModule: (module) => set({ view: "hellpad", module, options: null }),
  select: (selectedPlanet) => set({ selectedPlanet }),
  showOnMap: (planet) => set({ view: "map", selectedPlanet: planet, detail: null, options: null }),
  openDetail: (detail) => set({ detail, options: null }),
  openOptions: (options) => set({ options }),
  toggleDispatch: (open) => set((s) => ({ dispatchOpen: open ?? !s.dispatchOpen })),
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
export type Popups = "all" | "priority" | "none";

interface SettingsState {
  alerts: AlertSettings;
  popups: Popups;
  mapOverlay: MapOverlay;
  mapQuality: MapQuality;
  showInactive: boolean;
  pinned: number[];
  /** Views whose first-run hint has been dismissed. */
  hintsSeen: string[];
  /** Ids already alerted on, so a restart doesn't repeat them. */
  seenAssignments: number[];
  seenDispatch: number;
  /** App version whose changelog the user has opened. */
  seenChangelog: string;
  setAlert(k: keyof AlertSettings, v: boolean): void;
  set<K extends "popups" | "mapOverlay" | "mapQuality" | "showInactive">(k: K, v: SettingsState[K]): void;
  togglePin(planet: number): void;
  clearPins(): void;
  dismissHint(id: string): void;
  resetUnread(): void;
  markSeen(assignments: number[], dispatch: number): void;
  markChangelog(version: string): void;
}

export const useSettings = create<SettingsState>()(
  persist(
    (set) => ({
      alerts: { system: true, orders: true, dispatches: true, captures: true, defenses: true },
      popups: "all",
      mapOverlay: "regions",
      mapQuality: "normal",
      showInactive: false,
      pinned: [],
      hintsSeen: [],
      seenAssignments: [],
      seenDispatch: 0,
      seenChangelog: "",
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
    }),
    { name: "warcommand-settings" },
  ),
);
