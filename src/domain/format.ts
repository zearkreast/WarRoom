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

export function fmtDateTime(ms: number): string {
  return new Date(ms).toLocaleString(undefined, {
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
