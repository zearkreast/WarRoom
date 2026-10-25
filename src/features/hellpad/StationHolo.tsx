import { useId, type CSSProperties, type ReactNode } from "react";
import dssHolo from "../../assets/stations/dss-holo.webp";
import { StationEmblem } from "../../ui/emblems";
import { DSS_TONE, type DssState } from "../map/pawnArt";


function Holo(props: { tone: string; title: ReactNode; stamp?: string; children: ReactNode; readout: [string, ReactNode][] }) {
  return (
    <aside className="st-holo" style={{ "--holo": props.tone } as CSSProperties} aria-label="Station hologram">
      <header className="st-holo-head caps">{props.title}</header>
      <div className="st-holo-stage">
        {props.children}
        <i className="st-holo-sweep" aria-hidden />
        {props.stamp && <span className="st-holo-stamp caps">{props.stamp}</span>}
      </div>
      <i className="st-holo-base" aria-hidden />
      <dl className="st-holo-read">
        {props.readout.map(([k, v]) => (
          <div key={k}>
            <dt className="caps">{k}</dt>
            <dd className="num">{v}</dd>
          </div>
        ))}
      </dl>
    </aside>
  );
}

/** The Democracy Space Station side-on: the game's own summary hologram, lit by the station's state. */
export function DssHolo({ state, readout }: { state: DssState; readout: [string, ReactNode][] }) {
  return (
    <Holo
      tone={DSS_TONE[state]}
      title={
        <>
          <StationEmblem station="dss" size={14} /> DSS · side view
        </>
      }
      readout={readout}
    >
      <i className="st-holo-art" style={{ "--art": `url("${dssHolo}")` } as CSSProperties} role="img" aria-label="The Democracy Space Station" />
    </Holo>
  );
}

const CX = 100;
const CY = 150;
const R = 70;

/** The visible half of a latitude ring, and its hidden half, `t` from -1 (south pole) to 1. */
function latitude(t: number): [string, string] {
  const y = CY - t * R;
  const rx = R * Math.sqrt(1 - t * t);
  const ry = rx * 0.16;
  return [`M${CX - rx} ${y}A${rx} ${ry} 0 0 0 ${CX + rx} ${y}`, `M${CX - rx} ${y}A${rx} ${ry} 0 0 1 ${CX + rx} ${y}`];
}

/** A meridian's visible half, at longitude `deg` from the line of sight. */
function meridian(deg: number): string {
  const rx = Math.abs(R * Math.sin((deg * Math.PI) / 180));
  return `M${CX} ${CY - R}A${rx} ${R} 0 0 ${deg > 0 ? 1 : 0} ${CX} ${CY + R}`;
}

const SEAMS = [-62, -38, -14, 12, 36, 60];
// Red lights along the seams, as (longitude, latitude) in degrees.
const LIGHTS: [number, number][] = SEAMS.flatMap((lon, i) =>
  [-48, -26, -8, 14, 32, 52]
    .filter((_, j) => (i + j) % 3 !== 0)
    .map((lat, j): [number, number] => [lon, lat + ((i * 7 + j * 5) % 9) - 4]),
);
const CRACK = "175,108 152,136 136,124 114,160 94,148 72,190 52,178 24,212";

function SphereWire() {
  return (
    <g className="ash-wire">
      <circle cx={CX} cy={CY} r={R} className="ash-rim" />
      {[-0.78, -0.5, -0.2, 0.12, 0.42, 0.72].map((t) => {
        const [front, back] = latitude(t);
        return (
          <g key={t}>
            <path d={back} className="ash-back" />
            <path d={front} />
          </g>
        );
      })}
      {SEAMS.map((d) => (
        <path key={d} d={meridian(d)} className="ash-seam" />
      ))}
      {LIGHTS.map(([lon, lat], i) => {
        const x = CX + R * Math.cos((lat * Math.PI) / 180) * Math.sin((lon * Math.PI) / 180);
        const y = CY - R * Math.sin((lat * Math.PI) / 180);
        return <rect key={i} x={x - 0.8} y={y - 2.5} width={1.6} height={5} className="ash-light" />;
      })}
      <path d={`M${CX - 24} ${CY - R + 7}A24 6 0 0 0 ${CX + 24} ${CY - R + 7}`} />
      <path d={`M${CX - 24} ${CY - R + 7}L${CX - 19} ${CY - R - 5}M${CX + 24} ${CY - R + 7}L${CX + 19} ${CY - R - 5}`} />
      <ellipse cx={CX} cy={CY - R - 5} rx={19} ry={4.5} />
      <path d={`M${CX - 8} ${CY - R - 9}V${CY - R - 24}M${CX + 10} ${CY - R - 9}V${CY - R - 18}`} />
      <circle cx={CX - 8} cy={CY - R - 26} r={1.8} className="ash-light" />
      <circle cx={CX + 10} cy={CY - R - 20} r={1.5} className="ash-light" />
      <path d={`M${CX + 67} ${CY - 18}H${CX + 94}V${CY - 9}H${CX + 67}M${CX + 80} ${CY - 18}V${CY - 9}`} />
      <path d={`M${CX - 50} ${CY + 49}L${CX - 66} ${CY + 62}L${CX - 58} ${CY + 42}`} />
    </g>
  );
}

/** The Automaton station as drawn from its in-game render, broken in two the way it ended. */
export function CssHolo({ readout }: { readout: [string, ReactNode][] }) {
  const id = useId();
  return (
    <Holo
      tone="var(--f-automaton)"
      title={
        <>
          <StationEmblem station="css" size={14} /> CSS · last scan
        </>
      }
      stamp="Destroyed"
      readout={readout}
    >
      <svg className="st-holo-svg glitch" viewBox="0 0 200 400" role="img" aria-label="The Automaton station, broken apart">
        <defs>
          <clipPath id={`${id}-upper`}>
            <polygon points={`0,0 200,0 200,108 ${CRACK} 0,212`} />
          </clipPath>
          <clipPath id={`${id}-lower`}>
            <polygon points={`200,108 ${CRACK} 0,212 0,400 200,400`} />
          </clipPath>
        </defs>
        <g clipPath={`url(#${id}-upper)`} transform="translate(-6 -6) rotate(-4 100 150)">
          <SphereWire />
        </g>
        <g clipPath={`url(#${id}-lower)`} transform="translate(6 8) rotate(5 100 150)">
          <SphereWire />
        </g>
        <g className="ash-debris">
          {[
            [70, 262, 6],
            [118, 280, 4],
            [92, 318, 5],
            [140, 330, 3],
            [58, 350, 3],
            [106, 372, 4],
          ].map(([x, y, s], i) => (
            <g key={i} transform={`translate(${x} ${y})`}>
              <path d={`M0 ${-s}L${s} ${s * 0.6}L${-s * 0.8} ${s * 0.4}Z`} transform={`rotate(${i * 47})`} />
              <path d={`M0 ${-s - 2}V${-s - 14 - i * 3}`} className="ash-trail" />
            </g>
          ))}
        </g>
      </svg>
    </Holo>
  );
}
