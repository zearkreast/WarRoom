import type { Faction } from "../api/types";

export interface FactionInfo {
  label: string;
  /** CSS variable holding this faction's color. */
  color: string;
}

export const FACTIONS: Record<Faction, FactionInfo> = {
  Humans: { label: "Super Earth", color: "var(--f-humans)" },
  Terminids: { label: "Terminids", color: "var(--f-terminids)" },
  Automaton: { label: "Automatons", color: "var(--f-automaton)" },
  Illuminate: { label: "Illuminate", color: "var(--f-illuminate)" },
};

/** Faction ids as used in assignment task values (valueType 1). */
export const FACTION_BY_ID: Record<number, Faction> = {
  1: "Humans",
  2: "Terminids",
  3: "Automaton",
  4: "Illuminate",
};

/** Faction → id, for the smallint owner columns in Supabase. */
export const FACTION_ID: Record<Faction, number> = { Humans: 1, Terminids: 2, Automaton: 3, Illuminate: 4 };

export const factionColor = (f: string | undefined) => FACTIONS[f as Faction]?.color ?? "var(--text-dim)";
export const factionLabel = (f: string | undefined) => FACTIONS[f as Faction]?.label ?? "Unknown";
