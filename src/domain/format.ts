/** UI copy is English, so dates and numbers use English formatting whatever the system locale. */
export const LOCALE = "en-GB";

export const fmtInt = (n: number) => Math.round(n).toLocaleString("en-US");

export function fmtCompact(n: number): string {
  return new Intl.NumberFormat("en-US", { notation: "compact", maximumFractionDigits: 1 }).format(n);
}

export const fmtPct = (n: number, digits = 2) => `${n.toFixed(digits)}%`;

export function fmtRate(rate: number | null): string {
  if (rate == null) return "—";
  const sign = rate > 0 ? "+" : "";
  return `${sign}${rate.toFixed(2)}%/h`;
}

/** "2d 4h", "47h 19m", "12m". Negative or zero durations render as "now". */
export function fmtDuration(ms: number): string {
  if (ms <= 0) return "now";
  const m = Math.floor(ms / 60_000);
  const h = Math.floor(m / 60);
  const d = Math.floor(h / 24);
  if (d >= 3) return `${d}d ${h % 24}h`;
  if (h >= 1) return `${h}h ${m % 60}m`;
  return `${m}m`;
}

/** Time left on something that ends, in days and hours once it's a day or more. */
export function fmtTimeLeft(ms: number): string {
  const h = Math.floor(ms / 3_600_000);
  return h >= 24 ? `${Math.floor(h / 24)}d ${h % 24}h` : fmtDuration(ms);
}

export function fmtDate(ms: number): string {
  return new Date(ms).toLocaleDateString(LOCALE, { day: "numeric", month: "short", year: "numeric" });
}

export function fmtTime(ms: number): string {
  return new Date(ms).toLocaleTimeString(LOCALE, { hour: "2-digit", minute: "2-digit" });
}

export function fmtDateTime(ms: number): string {
  return new Date(ms).toLocaleString(LOCALE, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export interface MarkupSpan {
  text: string;
  /** Highlight style id from the game's `<i=N>` tags, or null for plain text. */
  style: number | null;
}

/** Splits the game's dispatch markup (`<i=3>TITLE</i>`) into styled spans. */
export function parseMarkup(message: string): MarkupSpan[] {
  const spans: MarkupSpan[] = [];
  const re = /<i=(\d+)>([\s\S]*?)<\/i>/g;
  let last = 0;
  for (let m = re.exec(message); m; m = re.exec(message)) {
    if (m.index > last) spans.push({ text: message.slice(last, m.index), style: null });
    spans.push({ text: m[2], style: Number(m[1]) });
    last = m.index + m[0].length;
  }
  if (last < message.length) spans.push({ text: message.slice(last), style: null });
  return spans;
}

/** First highlighted span (usually the headline), else the first line. */
export function dispatchTitle(message: string): string {
  const head = parseMarkup(message).find((s) => s.style != null);
  return (head?.text ?? message.split("\n")[0]).trim();
}

/** Super Earth Standard Time: UTC, with the in-game calendar running 160 years ahead. */
export const SEST_YEAR_OFFSET = 160;

export function sestClock(ms: number): string {
  return new Date(ms).toISOString().slice(11, 19);
}

export function sestDate(ms: number): string {
  const d = new Date(ms);
  const dd = String(d.getUTCDate()).padStart(2, "0");
  const mm = String(d.getUTCMonth() + 1).padStart(2, "0");
  return `${dd}/${mm}/${d.getUTCFullYear() + SEST_YEAR_OFFSET}`;
}

/** Day of the Galactic War, counting the start day as day 1. */
export function warDay(startedIso: string, now: number): number {
  const start = Date.parse(startedIso);
  if (!Number.isFinite(start)) return 0;
  return Math.floor((now - start) / 86_400_000) + 1;
}

/** "just now", "12 minutes ago", "3 hours ago", "2 days ago". */
export function fmtAgo(at: number, now: number): string {
  const m = Math.floor((now - at) / 60_000);
  if (m < 1) return "just now";
  if (m < 60) return `${m} minute${m === 1 ? "" : "s"} ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h} hour${h === 1 ? "" : "s"} ago`;
  const d = Math.floor(h / 24);
  return `${d} day${d === 1 ? "" : "s"} ago`;
}

/** Compact age for tight spaces: "now", "12m", "3h", "2d". */
export function fmtAgeShort(at: number, now: number): string {
  const m = Math.floor((now - at) / 60_000);
  if (m < 1) return "now";
  if (m < 60) return `${m}m`;
  const h = Math.floor(m / 60);
  return h < 24 ? `${h}h` : `${Math.floor(h / 24)}d`;
}
