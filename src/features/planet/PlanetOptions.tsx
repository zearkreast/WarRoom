import { Crosshair, FileText, MapPin, Pin, PinOff, X } from "lucide-react";
import { useEffect, useRef } from "react";
import { usePlanet } from "../../api/queries";
import { useSettings, useUi } from "../../state";
import { DssIcon } from "../../ui/emblems";
import { useEscape } from "../../ui/primitives";
import { useStationPlanets } from "../planets/useFronts";
import "./planet.css";

const SIZE = 236;

/** Radial options for a planet: pin, show on map, station, details. */
export function PlanetOptions() {
  const { view, options, openOptions, openDetail, showOnMap, focus, openModule } = useUi();
  const planet = usePlanet(options?.planet);
  const pinned = useSettings((s) => (options ? s.pinned.includes(options.planet) : false));
  const togglePin = useSettings((s) => s.togglePin);
  const stations = useStationPlanets();
  const first = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!options) return;
    const from = document.activeElement as HTMLElement | null;
    first.current?.focus();
    // Back to what opened the menu, unless the choice replaced it or covered it with the planet page.
    return () => {
      requestAnimationFrame(() => {
        if (from?.isConnected && !document.querySelector(".overlay")) from.focus({ preventScroll: true });
      });
    };
  }, [options]);
  useEscape(() => openOptions(null), !!options);

  if (!options || !planet) return null;
  const close = () => openOptions(null);
  const x = Math.min(Math.max(options.x, SIZE / 2 + 8), window.innerWidth - SIZE / 2 - 8);
  const y = Math.min(Math.max(options.y, SIZE / 2 + 8), window.innerHeight - SIZE / 2 - 8);
  const hasStation = stations.has(planet.index);

  const items = [
    {
      pos: "n",
      label: pinned ? "Unpin" : "Pin to top",
      icon: pinned ? PinOff : Pin,
      run: () => {
        togglePin(planet.index);
        close();
      },
    },
    { pos: "e", label: "Details", icon: FileText, run: () => openDetail(planet.index) },
    // The map is already open there, so its slot centres the planet instead.
    view === "map"
      ? { pos: "s", label: "Focus", icon: Crosshair, run: () => focus(planet.index) }
      : { pos: "s", label: "Show on map", icon: MapPin, run: () => showOnMap(planet.index) },
    { pos: "w", label: "Space station", icon: DssIcon, run: () => openModule("stations"), disabled: !hasStation },
  ];

  return (
    <div className="po-scrim" onClick={close}>
      <div
        className="po"
        style={{ left: x - SIZE / 2, top: y - SIZE / 2, width: SIZE, height: SIZE }}
        role="menu"
        aria-label={`${planet.name} options`}
        onClick={(e) => e.stopPropagation()}
      >
        {items.map((it, i) => (
          <button
            key={it.pos}
            ref={i === 0 ? first : undefined}
            role="menuitem"
            className={`po-item po-${it.pos}`}
            onClick={it.run}
            disabled={it.disabled}
            title={it.disabled ? "No space station at this planet" : it.label}
          >
            <it.icon size={22} strokeWidth={1.75} />
            <span>{it.label}</span>
          </button>
        ))}
        <button className="po-center" onClick={close} aria-label="Close options">
          <span className="po-planet">{planet.name}</span>
          <span className="po-close caps">
            <X size={12} /> Close
          </span>
        </button>
      </div>
    </div>
  );
}
