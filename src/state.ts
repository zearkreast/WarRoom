import { create } from "zustand";
import { persist } from "zustand/middleware";

export type View = "map" | "fronts" | "orders" | "news" | "stats" | "history";

interface UiState {
  view: View;
  selectedPlanet: number | null;
  setView(v: View): void;
  select(planet: number | null): void;
}

export const useUi = create<UiState>((set) => ({
  view: "map",
  selectedPlanet: null,
  setView: (view) => set({ view }),
  select: (selectedPlanet) => set({ selectedPlanet }),
}));

export interface AlertSettings {
  system: boolean;
  orders: boolean;
  dispatches: boolean;
  captures: boolean;
  defenses: boolean;
}

interface SettingsState {
  alerts: AlertSettings;
  /** Ids already alerted on, so a restart doesn't repeat them. */
  seenAssignments: number[];
  seenDispatch: number;
  setAlert(k: keyof AlertSettings, v: boolean): void;
  markSeen(assignments: number[], dispatch: number): void;
}

export const useSettings = create<SettingsState>()(
  persist(
    (set) => ({
      alerts: { system: true, orders: true, dispatches: true, captures: true, defenses: true },
      seenAssignments: [],
      seenDispatch: 0,
      setAlert: (k, v) => set((s) => ({ alerts: { ...s.alerts, [k]: v } })),
      markSeen: (seenAssignments, seenDispatch) => set({ seenAssignments, seenDispatch }),
    }),
    { name: "warcommand-settings" },
  ),
);
