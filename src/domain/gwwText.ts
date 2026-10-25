/**
 * G.W.W. text links. Content writes `[[kind:target|label]]` inline:
 * - `manual:`, `ministry:`, `company:`, `brand:` an id; `site:log` or `site:gov`; `dir:` a directory id;
 * - `planet:` a planet's name, opened like a planet name anywhere else in the app;
 * - `app:` a page outside the G.W.W.: news, overview, map, campaigns, stations, past or planets.
 */

export type LinkKind = "manual" | "ministry" | "company" | "brand" | "site" | "dir" | "planet" | "app";

export type Segment = { text: string } | { kind: LinkKind; target: string; label: string };

const KINDS = new Set<string>(["manual", "ministry", "company", "brand", "site", "dir", "planet", "app"]);
const LINK = /\[\[([a-z]+):([^|\]]+)\|([^\]]+)\]\]/g;

/** Splits text into plain runs and links; markup with an unknown kind stays as its label. */
export function parseRich(text: string): Segment[] {
  const out: Segment[] = [];
  let at = 0;
  for (const m of text.matchAll(LINK)) {
    const i = m.index ?? 0;
    if (i > at) out.push({ text: text.slice(at, i) });
    out.push(KINDS.has(m[1]) ? { kind: m[1] as LinkKind, target: m[2], label: m[3] } : { text: m[3] });
    at = i + m[0].length;
  }
  if (at < text.length) out.push({ text: text.slice(at) });
  return out;
}

/** The text as read, links reduced to their labels: for search and screen-reader labels. */
export const plainText = (text: string): string => text.replace(LINK, "$3");

/** Every link in the text, as `kind:target`. */
export const linksIn = (text: string): string[] => [...text.matchAll(LINK)].map((m) => `${m[1]}:${m[2]}`);

/** The next of some yearly days, counted in whole days from `now`'s date; 0 means today. */
export function nextYearly<T extends { month: number; day: number }>(days: T[], now: Date): (T & { in: number }) | undefined {
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  return days
    .map((d) => {
      let at = new Date(today.getFullYear(), d.month - 1, d.day);
      if (at < today) at = new Date(today.getFullYear() + 1, d.month - 1, d.day);
      return { ...d, in: Math.round((at.getTime() - today.getTime()) / 86_400_000) };
    })
    .sort((a, b) => a.in - b.in)[0];
}
