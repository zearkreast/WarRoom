import { factionColor } from "../domain/factions";

// Simple authored marks for each faction (not the game's artwork).
const PATHS: Record<string, string> = {
  // A globe inside an orbit ring.
  Humans:
    "M12 6.5a5.5 5.5 0 1 1 0 11 5.5 5.5 0 0 1 0-11Zm0-4.5c6.6 0 11 3.4 11 6.6 0 1.7-1.2 3.2-3.2 4.3a7 7 0 0 0 .2-1.6c0-.3 0-.6-.1-.9 1-.6 1.5-1.2 1.5-1.8 0-2-3.9-4.6-9.4-4.6S2.6 6.6 2.6 8.6c0 .6.5 1.2 1.5 1.8l-.1.9c0 .6.1 1.1.2 1.6C2.2 11.8 1 10.3 1 8.6 1 5.4 5.4 2 12 2Z",
  // Three swept claws.
  Terminids:
    "M4 20c3.5-1 7.2-4.6 8.2-10.8C13 4.8 11.6 2.6 10 2c4.6 1.3 7.4 5.5 6.6 10.4C15.8 17.8 10.6 21 4 20Zm9.6.6c3.4-1.6 5.6-5 5.7-8.8 0-1.8-.3-3.2-.9-4.4 2.6 1.8 4 4.6 3.4 7.8-.6 3.4-4 5.6-8.2 5.4ZM2.2 15.2c2.2-.4 4.6-2.4 5.4-6.2.3-1.6.1-3-.5-4 2.4 1.2 3.4 3.8 2.6 6.6-.9 3.2-4 4.6-7.5 3.6Z",
  // A broken five-point star.
  Automaton:
    "M12 1.5 14.7 8.6 22.5 8.9 16.4 13.7 18.6 21.2 12 16.9 5.4 21.2 7.6 13.7 1.5 8.9 9.3 8.6Zm0 6.2-1.2 3.3H7.6l2.6 2-1 3.3 2.8-1.9 2.8 1.9-1-3.3 2.6-2h-3.2Z",
  // A crescent cradling an eye.
  Illuminate:
    "M12 9.2a3 3 0 1 1 0 6 3 3 0 0 1 0-6ZM3 9.5c.6 5.4 4.3 9 9 9s8.4-3.6 9-9c1 1.3 1.5 2.9 1.5 4.6C22.5 19.6 17.8 23 12 23S1.5 19.6 1.5 14.1c0-1.7.5-3.3 1.5-4.6ZM12 1c1.8 0 3.3 1.3 3.6 3a5.9 5.9 0 0 0-7.2 0C8.7 2.3 10.2 1 12 1Z",
};

export function FactionEmblem({ faction, size = 16, color }: { faction: string; size?: number; color?: string }) {
  const d = PATHS[faction];
  if (!d) return null;
  return (
    <svg className="emblem" width={size} height={size} viewBox="0 0 24 24" aria-hidden>
      <path d={d} fill={color ?? factionColor(faction)} fillRule="evenodd" />
    </svg>
  );
}

/** App mark: a shield with a star, drawn in the CTA colour. */
export function AppMark({ size = 26 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 32 32" aria-hidden>
      <path d="M16 2 29 8v8c0 7-5.5 12-13 14C8.5 28 3 23 3 16V8z" fill="none" stroke="var(--cta)" strokeWidth="2" />
      <path d="m16 9 2.2 4.6 5 .6-3.7 3.4 1 5-4.5-2.5-4.5 2.5 1-5-3.7-3.4 5-.6z" fill="var(--cta)" />
    </svg>
  );
}
