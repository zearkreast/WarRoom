import type { Dispatch } from "../api/types";
import { dispatchTitle, parseMarkup, type MarkupSpan } from "./format";

export type DispatchKind = "order" | "victory" | "defeat" | "alert" | "advisory" | "news";

const RULES: [RegExp, DispatchKind][] = [
  [/major order (won|complete)|victory|liberated|success/i, "victory"],
  [/major order (failed|lost)|defeat|has fallen|failed/i, "defeat"],
  [/major order|new orders?|high command/i, "order"],
  [/alert|attack|invasion|warning|emergency/i, "alert"],
  [/advisory|notice|update|briefing/i, "advisory"],
];

/** Category of a dispatch, from its headline (or first line). */
export function dispatchKind(message: string): DispatchKind {
  const title = dispatchTitle(message);
  return RULES.find(([re]) => re.test(title))?.[1] ?? "news";
}

/** The dispatch body's spans without the leading headline, so cards don't repeat the title. */
export function dispatchBody(message: string): MarkupSpan[] {
  const spans = parseMarkup(message.trim());
  const rest = spans[0]?.style != null ? spans.slice(1) : spans;
  if (rest[0]) rest[0] = { ...rest[0], text: rest[0].text.replace(/^\s+/, "") };
  return rest;
}

export const RECENT_DISPATCH_MS = 72 * 3_600_000;

/** Dispatches published in the last 72 hours, newest first. */
export function recentDispatches(list: Dispatch[], now: number): Dispatch[] {
  return list.filter((d) => now - Date.parse(d.published) <= RECENT_DISPATCH_MS).sort((a, b) => b.id - a.id);
}

/** The newest `count` dispatches whatever their age, or the last 72 hours' for "72h". */
export function latestDispatches(list: Dispatch[], count: string, now: number): Dispatch[] {
  if (count === "72h") return recentDispatches(list, now);
  return [...list].sort((a, b) => b.id - a.id).slice(0, Number(count));
}
