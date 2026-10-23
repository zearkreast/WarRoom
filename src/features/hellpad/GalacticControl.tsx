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
import { PointerCard, pointerAt, type PointerAt } from "../../ui/PointerCard";

const COMMUNITY = parseCommunityPlanets(communityText);
interface FactionHover {
  faction: Faction;
  at: PointerAt;
  list: ListedPlanet[];
}

/** Collapsible faction-control ring for the planet index; hovering an arc maps that faction's planets. */
export function GalacticControl({ planets, onFilter }: { planets: Planet[]; onFilter(f: Faction): void }) {
  const open = useSettings((s) => s.planetsControlOpen);
  const setSetting = useSettings((s) => s.set);
  const shares = useMemo(() => controlShares(planets), [planets]);
  const [active, setActive] = useState<Faction | null>(null);
  const [map, setMap] = useState<FactionHover | null>(null);

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
                <FactionEmblem faction={s.faction} size={13} />
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
              const at = pointerAt(e, e.currentTarget.ownerSVGElement!);
              setMap((m) =>
                m && m.faction === key ? { ...m, at } : { faction: key as Faction, at, list: listFor(key as Faction) },
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
      {open && map && focus && (
        <PointerCard
          at={map.at}
          tone={factionColor(map.faction)}
          icon={<FactionEmblem faction={map.faction} size={18} />}
          title={factionLabel(map.faction)}
          badge={`${Math.round(focus.pct)}% · ${focus.planets}`}
          rows={map.list.map((l) => ({
            key: l.name,
            name: l.name,
            value: l.planet ? l.planet.statistics.playerCount : null,
            note: l.lore,
          }))}
          footer={
            map.faction !== "Humans" && focus.planets > LIST_SIZE
              ? `+${focus.planets - LIST_SIZE} more · busiest first`
              : undefined
          }
        />
      )}
    </section>
  );
}
