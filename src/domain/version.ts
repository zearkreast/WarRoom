/** The numeric parts of a version or release tag: "v0.2.8-alpha" → [0, 2, 8]. */
export function versionParts(v: string): number[] {
  return (v.replace(/^v/i, "").split(/[-+]/)[0] ?? "").split(".").map((n) => Number.parseInt(n, 10) || 0);
}

/** Whether `candidate` is a later version than `current`, ignoring pre-release suffixes. */
export function isNewer(candidate: string, current: string): boolean {
  const a = versionParts(candidate);
  const b = versionParts(current);
  for (let i = 0; i < Math.max(a.length, b.length); i++) {
    const d = (a[i] ?? 0) - (b[i] ?? 0);
    if (d !== 0) return d > 0;
  }
  return false;
}
