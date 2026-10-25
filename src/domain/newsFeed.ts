import { dispatchBody } from "./dispatch";
import { dispatchTitle, parseMarkup, sestDate } from "./format";

/** So texts that differ only in case and punctuation compare equal. */
const bare = (s: string) => s.toLowerCase().replace(/[^a-z0-9]+/g, "");

/** A dispatch's text without the game's markup tags. */
export const plainText = (message: string) =>
  parseMarkup(message)
    .map((s) => s.text)
    .join("");

/** True when a dispatch's body only repeats its headline, as the game's attack alerts do. */
export function bodyEchoesTitle(message: string): boolean {
  const body = bare(
    dispatchBody(message)
      .map((s) => s.text)
      .join(""),
  );
  return body === "" || bare(dispatchTitle(message)).includes(body);
}

const DAY = 86_400_000;
/** Days run on Super Earth Standard Time, which is UTC. */
const dayStart = (ms: number) => Math.floor(ms / DAY) * DAY;

export function dayLabel(at: number, now: number): string {
  const days = (dayStart(now) - dayStart(at)) / DAY;
  if (days === 0) return "Today";
  if (days === 1) return "Yesterday";
  // No weekday: the in-game calendar runs 160 years ahead, and today's weekday would be wrong for it.
  return sestDate(at);
}

/** Runs of items on the same day, in the order given; sort by time first or a day can appear twice. */
export function groupByDay<T extends { at: number }>(items: T[], now: number) {
  const groups: { key: number; label: string; items: T[] }[] = [];
  for (const item of items) {
    const key = dayStart(item.at);
    const last = groups[groups.length - 1];
    if (last?.key === key) last.items.push(item);
    else groups.push({ key, label: dayLabel(item.at, now), items: [item] });
  }
  return groups;
}

/** Whether every word of the query appears somewhere in the text, in any order and case. */
export function matchesQuery(text: string, query: string): boolean {
  const hay = text.toLowerCase();
  return query
    .toLowerCase()
    .split(/\s+/)
    .filter(Boolean)
    .every((w) => hay.includes(w));
}
