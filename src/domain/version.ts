/** A version or release tag without its pre-release or build label: "v0.3.0-beta" → "0.3.0". */
export const releaseNumber = (v: string) => v.replace(/^v/i, "").split(/[-+]/)[0] ?? "";

/** The numeric parts of a version or release tag: "v0.2.8-alpha" → [0, 2, 8]. */
export function versionParts(v: string): number[] {
  return releaseNumber(v).split(".").map((n) => Number.parseInt(n, 10) || 0);
}

/** The pre-release label after the dash ("beta.2" in "0.3.0-beta.2"), or "" for a full release. */
const preRelease = (v: string) => v.replace(/^v/i, "").split("+")[0].split("-").slice(1).join("-");

/**
 * Whether `candidate` is a later version than `current`, as semver orders them: by number first, then a full
 * release after its own pre-releases, and pre-releases by label ("alpha" < "beta" < "rc", "beta.2" < "beta.10").
 */
export function isNewer(candidate: string, current: string): boolean {
  const a = versionParts(candidate);
  const b = versionParts(current);
  for (let i = 0; i < Math.max(a.length, b.length); i++) {
    const d = (a[i] ?? 0) - (b[i] ?? 0);
    if (d !== 0) return d > 0;
  }
  const [pa, pb] = [preRelease(candidate), preRelease(current)];
  if (pa === pb) return false;
  if (!pa || !pb) return !pa;
  return pa.localeCompare(pb, "en", { numeric: true }) > 0;
}
