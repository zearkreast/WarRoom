import { create } from "zustand";
import { supabaseHistory, type ChangeRecord, type HistoryStore } from "./store";

const KEEP = 100;

interface HistoryState {
  ready: boolean;
  store: HistoryStore;
  /** Most recent ownership changes, newest first. */
  changes: ChangeRecord[];
  /**
   * Loads anything newer than what's already held. Returns only the new rows, and none
   * on the first call, so alerts don't fire for history the user hasn't missed.
   */
  refresh(): Promise<ChangeRecord[]>;
}

export const useHistory = create<HistoryState>((set, get) => ({
  ready: false,
  store: supabaseHistory,
  changes: [],

  async refresh() {
    const { ready, changes, store } = get();
    if (!ready) {
      set({ changes: await store.changes({ limit: KEEP }), ready: true });
      return [];
    }
    const fresh = await store.changesAfter(changes[0]?.id ?? 0, KEEP);
    if (fresh.length) set({ changes: [...fresh, ...changes].slice(0, KEEP) });
    return fresh;
  },
}));
