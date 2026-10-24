/**
 * Space stations the API doesn't list: retired, destroyed or not yet built. The page shows them next
 * to the live ones; a station the API starts listing again should be taken out of here.
 */
export interface StationRecord {
  id: string;
  /** Short name on the selector. */
  short: string;
  name: string;
  status: "Destroyed" | "Offline" | "Retired";
  faction: "Automaton" | "Humans" | "Terminids" | "Illuminate";
  /** What happened to it, in the game's dates (SEST). */
  note: string;
}

export const PAST_STATIONS: StationRecord[] = [
  {
    id: "css",
    short: "CSS",
    name: "Automaton Orbital Superweapon",
    status: "Destroyed",
    faction: "Automaton",
    note:
      "The Cyborg-built orbital platform first fired on Varylia 5 on 14 July 2186. The DSS crippled it over Mort on 26 July and destroyed it in Mort's orbit on 27 July 2186, after the Helldivers took its facilities on X-45.",
  },
];
