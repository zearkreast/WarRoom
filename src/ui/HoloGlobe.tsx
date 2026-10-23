/** Idle hologram for a ring's centre: a wireframe globe with turning meridians. */
export function HoloGlobe({ size = 120 }: { size?: number }) {
  return (
    <svg className="holo" width={size} height={size} viewBox="-60 -60 120 120" aria-hidden>
      <defs>
        <radialGradient id="holo-fill" cx="40%" cy="35%" r="70%">
          <stop offset="0%" stopColor="#9fd4ff" stopOpacity={0.35} />
          <stop offset="70%" stopColor="#46a4ff" stopOpacity={0.12} />
          <stop offset="100%" stopColor="#46a4ff" stopOpacity={0.02} />
        </radialGradient>
      </defs>
      <circle r={48} fill="url(#holo-fill)" className="holo-rim" />
      {[-32, -16, 0, 16, 32].map((y) => (
        <ellipse key={y} cy={y} rx={Math.sqrt(48 * 48 - y * y)} ry={4} className="holo-line" />
      ))}
      {[0, 1, 2, 3, 4, 5].map((i) => (
        <ellipse key={i} rx={48} ry={48} className="holo-meridian" style={{ animationDelay: `${-i}s` }} />
      ))}
      <ellipse rx={58} ry={9} className="holo-orbit" />
    </svg>
  );
}
