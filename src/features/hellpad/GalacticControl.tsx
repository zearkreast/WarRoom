import { ChevronDown, Users } from "lucide-react";
import { useMemo, useState, type CSSProperties } from "react";
import type { Faction, Planet } from "../../api/types";
import communityText from "../../data/community-planets.txt?raw";
import { FACTIONS, factionColor, factionLabel } from "../../domain/factions";
import {
  busiestPlanets,
  LIST_SIZE,
  parseCommunityPlanets,
  superEarthPlanets,
  type ListedPlanet,
} from "../../domain/factionPlanets";
import { fmtInt } from "../../domain/format";
import { controlShares } from "../../domain/planetIndex";
import { useSettings } from "../../state";
import { FactionEmblem } from "../../ui/emblems";
import { HoloGlobe } from "../../ui/HoloGlobe";
import { NeonRing } from "../../ui/NeonRing";

const COMMUNITY = parseCommunityPlanets(communityText);
const MAP_WIDTH = 260;

interface PointerMap {
  faction: Faction;
  x: number;
  y: number;
  /** Ring centre, so the map opens on the side away from the readout. */
  cx: number;
  cy: number;
  list: ListedPlanet[];
}

const MAP_HEIGHT = 300;
const OFFSET = 18;

function placeMap(m: PointerMap): { left: number; top: number } {
  const clampX = (v: number) => Math.max(8, Math.min(v, window.innerWidth - MAP_WIDTH - 8));
  const clampY = (v: number) => Math.max(8, Math.min(v, window.innerHeight - MAP_HEIGHT - 8));
  const toLeft = m.x < m.cx;
  const sideX = toLeft ? m.x - MAP_WIDTH - OFFSET : m.x + OFFSET;
  if (sideX >= 8 && sideX + MAP_WIDTH <= window.innerWidth - 8)
    return { left: sideX, top: clampY(m.y - MAP_HEIGHT / 2) };
  // No room beside the ring: open above or below the pointer instead.
  const above = m.y < m.cy;
  return { left: clampX(m.x - MAP_WIDTH / 2), top: clampY(above ? m.y - MAP_HEIGHT - OFFSET : m.y + OFFSET) };
}

function FactionMap({ map, share }: { map: PointerMap; share: { pct: number; planets: number } }) {
  const max = Math.max(1, ...map.list.map((l) => l.planet?.statistics.playerCount ?? 0));
  const { left, top } = placeMap(map);
  return (
    <div className="fmap" style={{ left, top, "--k": factionColor(map.faction) } as CSSProperties} aria-hidden>
      <div className="fmap-head">
        <FactionEmblem faction={map.faction} size={18} />
        <b>{factionLabel(map.faction)}</b>
        <span className="num">
          {Math.round(share.pct)}% · {share.planets}
        </span>
      </div>
      <ol>
        {map.list.map((l) => (
          <li key={l.name}>
            <span className="fmap-name">{l.name}</span>
            <span className="fmap-val">
              {l.lore ?? (
                <>
                  <Users size={10} /> {fmtInt(l.planet!.statistics.playerCount)}
                </>
              )}
            </span>
            <span className="fmap-bar">
              <span style={{ width: `${l.planet ? (l.planet.statistics.playerCount / max) * 100 : 0}%` }} />
            </span>
          </li>
        ))}
      </ol>
      {map.faction !== "Humans" && share.planets > LIST_SIZE && (
        <div className="fmap-foot">+{share.planets - LIST_SIZE} more · busiest first</div>
      )}
    </div>
  );
}

/** Collapsible faction-control ring for the planet index; hovering an arc maps that faction's planets. */
export function GalacticControl({ planets, onFilter }: { planets: Planet[]; onFilter(f: Faction): void }) {
  const open = useSettings((s) => s.planetsControlOpen);
  const setSetting = useSettings((s) => s.set);
  const shares = useMemo(() => controlShares(planets), [planets]);
  const [active, setActive] = useState<Faction | null>(null);
  const [map, setMap] = useState<PointerMap | null>(null);

  const listFor = (f: Faction) =>
    f === "Humans" ? superEarthPlanets(planets, COMMUNITY) : busiestPlanets(planets, f, LIST_SIZE);
  const focus = shares.find((s) => s.faction === active);

  return (
    <section className={`gc ${open ? "open" : ""}`}>
      <button className="gc-toggle" aria-expanded={open} onClick={() => setSetting("planetsControlOpen", !open)}>
        <h2>Galactic control</h2>
        {!open && (
          <span className="gc-peek num">
            {shares.map((s) => (
              <span key={s.faction} style={{ color: factionColor(s.faction) }}>
                {Math.round(s.pct)}%
              </span>
            ))}
          </span>
        )}
        <ChevronDown size={18} className="gc-chevron" />
      </button>
      {open && (
        <div className="gc-body">
          <NeonRing
            label={`Planet control: ${shares.map((s) => `${factionLabel(s.faction)} ${Math.round(s.pct)}%`).join(", ")}`}
            segments={shares.map((s) => ({
              key: s.faction,
              label: factionLabel(s.faction),
              value: s.planets,
              color: factionColor(s.faction),
            }))}
            active={active}
            onActive={(key) => {
              setActive(key as Faction | null);
              if (!key) setMap(null);
            }}
            onPointerMove={(key, e) => {
              const ring = e.currentTarget.ownerSVGElement!.getBoundingClientRect();
              const at = {
                x: e.clientX,
                y: e.clientY,
                cx: ring.left + ring.width / 2,
                cy: ring.top + ring.height / 2,
              };
              setMap((m) =>
                m && m.faction === key
                  ? { ...m, ...at }
                  : { faction: key as Faction, ...at, list: listFor(key as Faction) },
              );
            }}
            onSelect={(key) => onFilter(key as Faction)}
            center={
              focus ? (
                <div key={focus.faction} className="nr-readout" style={{ color: factionColor(focus.faction) }}>
                  <FactionEmblem faction={focus.faction} size={30} />
                  <span className="caps">{factionLabel(focus.faction)}</span>
                  <b className="num">{Math.round(focus.pct)}%</b>
                  <span className="sub num">{focus.planets} planets</span>
                  <span className="pl num">
                    <Users size={11} /> {fmtInt(focus.players)}
                  </span>
                </div>
              ) : (
                <HoloGlobe key="holo" />
              )
            }
          />
          <div className="gc-legend">
            {shares.map((s) => (
              <button
                key={s.faction}
                className={`gc-chip ${active === s.faction ? "on" : ""}`}
                style={{ "--k": factionColor(s.faction) } as CSSProperties}
                onPointerEnter={() => setActive(s.faction)}
                onPointerLeave={() => setActive(null)}
                onFocus={() => setActive(s.faction)}
                onBlur={() => setActive(null)}
                onClick={() => onFilter(s.faction)}
              >
                <FactionEmblem faction={s.faction} size={18} />
                <span className="gc-chip-name">{FACTIONS[s.faction].label}</span>
                <b className="num">{Math.round(s.pct)}%</b>
                <span className="gc-chip-sub num">{s.planets} planets</span>
              </button>
            ))}
            <p className="gc-hint">
              Hover or tap the ring to map a faction's planets; pick a faction to filter the list.
            </p>
          </div>
        </div>
      )}
      {open && map && focus && <FactionMap map={map} share={focus} />}
    </section>
  );
}
