export interface CityFate {
  state: "destroyed" | "rebuilt" | "intact";
  /** The name it was rebuilt under. */
  name?: string;
  note: string;
  /** Longitude and latitude of the real-world city it stands on. */
  at: [number, number];
}

/**
 * What became of Super Earth's seven sector capitals in the Illuminate invasion of 2185, by region id.
 * The API still lists them as they stood before it.
 */
export const SUPER_EARTH_CITIES: Record<number, CityFate> = {
  0: {
    state: "destroyed",
    note: "The first megacity the Great Host attacked, razed on the invasion's second day.",
    at: [33, 30.5],
  },
  1: { state: "destroyed", note: "Razed four days into the invasion, the second megacity to fall.", at: [4.35, 50.85] },
  2: {
    state: "rebuilt",
    name: "GUN",
    note: "Razed four days into the invasion. The first megacity rebuilt, renamed Gun by a Helldiver vote of 72.5%.",
    at: [-58.4, -34.6],
  },
  3: {
    state: "rebuilt",
    name: "NEW YORK SUPREME",
    note: "Razed on the invasion's ninth day. Rebuilt as New York Supreme, the name that won the Helldiver vote over Bernard.",
    at: [-74, 40.7],
  },
  4: { state: "destroyed", note: "Razed on the invasion's ninth day.", at: [103.8, 1.3] },
  5: {
    state: "intact",
    note: "The federal capital held against the Great Host's last assault, in which the President fell.",
    at: [18.07, 59.33],
  },
  6: { state: "intact", note: "Among the first cities attacked, and held for the whole invasion.", at: [121.5, 31.2] },
};
