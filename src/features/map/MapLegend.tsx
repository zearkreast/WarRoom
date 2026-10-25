import { BookOpen, X } from "lucide-react";
import { useEffect, useRef, type ReactNode } from "react";
import { EmblemArt } from "../../ui/emblems";

/** A small picture of each mark, drawn the way the map draws it, on a 24-unit grid centred on (0, 0). */
const Swatch = ({ children }: { children: ReactNode }) => (
  <svg className="ml-swatch" viewBox="-12 -12 24 24" width={26} height={26} aria-hidden>
    {children}
  </svg>
);

const ENTRIES: { mark: ReactNode; name: string; text: string }[] = [
  {
    mark: (
      <Swatch>
        <circle cx={-5} r={2.4} fill="var(--f-humans)" />
        <circle cx={1} cy={-4} r={2} fill="var(--f-terminids)" />
        <circle cx={1} cy={4} r={2} fill="var(--f-automaton)" />
        <circle cx={7} r={2} fill="var(--f-illuminate)" />
      </Swatch>
    ),
    name: "Planets",
    text: "Blue for Super Earth, otherwise the colour of the faction holding it.",
  },
  {
    mark: (
      <Swatch>
        <circle r={7} fill="none" stroke="var(--f-automaton)" strokeWidth={1.4} />
        <circle r={2.6} fill="var(--f-automaton)" />
      </Swatch>
    ),
    name: "Active front",
    text: "A pulsing ring: Helldivers are fighting for the planet now.",
  },
  {
    mark: (
      <Swatch>
        <circle r={6} fill="none" stroke="var(--cta)" strokeWidth={1.4} strokeDasharray="2.4 2" />
        <circle r={2} fill="var(--f-terminids)" />
      </Swatch>
    ),
    name: "Major Order target",
    text: "A dashed yellow ring round an objective of the current order.",
  },
  {
    mark: (
      <Swatch>
        <line x1={-9} y1={6} x2={9} y2={-6} stroke="var(--f-illuminate)" strokeWidth={1.5} strokeDasharray="3 3" />
      </Swatch>
    ),
    name: "Attack line",
    text: "From an enemy world to the planet it is attacking.",
  },
  {
    mark: (
      <Swatch>
        <path d="M-9 5 -2 -3 4 2 9 -6" fill="none" stroke="rgba(210, 228, 255, 0.5)" strokeWidth={1} />
        <circle cx={-2} cy={-3} r={1.4} fill="var(--f-humans)" />
        <circle cx={4} cy={2} r={1.4} fill="var(--f-humans)" />
      </Swatch>
    ),
    name: "Supply lane",
    text: "Planets can only be attacked or liberated along these lines.",
  },
  {
    mark: (
      <Swatch>
        <path
          d="M-9 -7 7 -9 9 6 -6 8Z"
          fill="color-mix(in srgb, var(--f-automaton) 14%, transparent)"
          stroke="var(--f-automaton)"
          strokeWidth={1.2}
        />
      </Swatch>
    ),
    name: "Enemy territory",
    text: "Regions outline each enemy's ground; Sectors hatch the sectors it holds.",
  },
  {
    mark: (
      <Swatch>
        <circle r={9} fill="var(--f-terminids)" opacity={0.25} />
        <circle cx={-3} cy={2} r={5} fill="var(--f-terminids)" opacity={0.3} />
      </Swatch>
    ),
    name: "The Gloom",
    text: "An amber cloud over the planets in or at the edge of the Gloom.",
  },
  {
    mark: (
      <Swatch>
        <rect x={-8} y={-8} width={16} height={16} fill="var(--surface-0)" stroke="var(--f-automaton)" />
        <EmblemArt art="incineration-corps" size={12} color="var(--f-automaton)" />
      </Swatch>
    ),
    name: "Sub-faction",
    text: "An enemy unit on the planet, such as the Incineration Corps, in its emblem. Zoom in to see it.",
  },
  {
    mark: (
      <Swatch>
        <ellipse cy={6} rx={7} ry={2.4} fill="none" stroke="var(--f-humans)" />
        <path d="M-4 5 -2 -7 2 -7 4 5Z" fill="var(--f-humans)" opacity={0.35} />
      </Swatch>
    ),
    name: "Holograms",
    text: "SEAF forces, enemy units and the Space Station stand beside a planet once zoomed in.",
  },
];

/** What the marks on the map mean; opens beside the rail like the map settings. */
export function MapLegend({ onClose }: { onClose(): void }) {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const onDown = (e: PointerEvent) => {
      const t = e.target as Element;
      // The rail's own button toggles the panel; let its click do that.
      if (!ref.current?.contains(t) && !t.closest(".map-rail")) onClose();
    };
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    document.addEventListener("pointerdown", onDown);
    window.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("pointerdown", onDown);
      window.removeEventListener("keydown", onKey);
    };
  }, [onClose]);
  return (
    <div ref={ref} className="map-settings map-legend panel glass" role="dialog" aria-label="Map legend">
      <div className="ms-head caps">
        <BookOpen size={14} /> Legend
        <button className="ml-close" onClick={onClose} aria-label="Close legend">
          <X size={14} />
        </button>
      </div>
      <dl className="ml-list">
        {ENTRIES.map((e) => (
          <div key={e.name}>
            <dt>{e.mark}</dt>
            <dd>
              <b className="caps">{e.name}</b>
              <span>{e.text}</span>
            </dd>
          </div>
        ))}
      </dl>
    </div>
  );
}
