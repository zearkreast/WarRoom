import { isTauri } from "@tauri-apps/api/core";
import { BellRing, ClipboardCopy, Download, PinOff, RefreshCw, RotateCcw, Settings } from "lucide-react";
import { useState, type ReactNode } from "react";
import { useLastUpdate } from "../../api/queries";
import { notify } from "../../alerts/notify";
import { APP_VERSION } from "../../config";
import { MIN_PLAYERS, MIN_RATE } from "../../domain/campaignFilter";
import { fmtInt } from "../../domain/format";
import {
  useSettings,
  type AlertSettings,
  type MapDispatches,
  type MapOverlay,
  type MapPlanets,
  type MapQuality,
  type Popups,
} from "../../state";
import { Panel, Stepper } from "../../ui/primitives";
import { DISPATCH_OPTIONS, OVERLAY_OPTIONS, PLANET_OPTIONS, QUALITY_OPTIONS } from "../map/mapOptions";
import { checkForUpdate, useUpdate } from "../../update/appUpdate";
import "./settings.css";

const ON_OFF = [
  { value: "on", label: "On" },
  { value: "off", label: "Off" },
] as const;

function Row(props: { title: string; text: string; tone?: string; children: ReactNode }) {
  return (
    <div className="set-row">
      <div className="set-text">
        <b style={{ color: props.tone }}>{props.title}</b>
        <span>{props.text}</span>
      </div>
      {props.children}
    </div>
  );
}

const UPDATE_TEXT: Record<string, string> = {
  idle: "The app looks for a new release a few seconds after it starts, then every six hours.",
  checking: "Checking for a new release…",
  current: "You have the latest release.",
  available: "A new release is ready: see the card at the top right.",
  downloading: "Downloading the new release…",
  installing: "Installing; the app restarts when it is done.",
  "handed-off": "The APK is downloading; open it from your notifications to install.",
  error: "The last check failed.",
};

function UpdateRow() {
  const state = useUpdate((s) => s.state);
  if (!isTauri())
    return (
      <Row title={`Version ${APP_VERSION}`} text="The web version is always the latest; there is nothing to install.">
        {null}
      </Row>
    );
  return (
    <Row title={`Version ${APP_VERSION}`} text={UPDATE_TEXT[state.phase]}>
      <button className="hud-btn" onClick={() => void checkForUpdate(true)} disabled={state.phase === "checking"}>
        <RefreshCw size={14} /> Check now
      </button>
    </Row>
  );
}

function OnOff(props: { label: string; value: boolean; onChange(v: boolean): void }) {
  return (
    <Stepper
      label={props.label}
      options={[...ON_OFF]}
      value={props.value ? "on" : "off"}
      onChange={(v) => props.onChange(v === "on")}
    />
  );
}

const ALERTS: { key: keyof AlertSettings; title: string; text: string }[] = [
  { key: "orders", title: "New Major Orders", text: "When High Command issues a new order." },
  { key: "dispatches", title: "New dispatches", text: "Messages from Super Earth." },
  { key: "captures", title: "Planets liberated or lost", text: "Whenever a planet changes hands." },
  { key: "defenses", title: "New defenses", text: "When an enemy attacks a Super Earth planet." },
  { key: "system", title: "System notifications", text: "Also show alerts outside the app, as OS notifications." },
];

export function SettingsPage() {
  const s = useSettings();
  const lastUpdate = useLastUpdate().data;
  const [copied, setCopied] = useState(false);

  async function copyDiagnostics() {
    const info = {
      version: APP_VERSION,
      lastData: lastUpdate ? new Date(lastUpdate).toISOString() : null,
      userAgent: navigator.userAgent,
      viewport: `${innerWidth}x${innerHeight}@${devicePixelRatio}`,
      settings: {
        popups: s.popups,
        mapOverlay: s.mapOverlay,
        mapQuality: s.mapQuality,
        mapPlanets: s.mapPlanets,
        showInactive: s.showInactive,
        pinned: s.pinned.length,
      },
    };
    await navigator.clipboard?.writeText(JSON.stringify(info, null, 2)).catch(() => undefined);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  return (
    <div className="page scroll starfield">
      <div className="page-inner settings-page">
        <h1 className="page-title">Settings</h1>
        <Panel title="Interface" icon={Settings}>
          <Row
            title="Stalled planets"
            text={`Fronts under ${fmtInt(MIN_PLAYERS)} Helldivers that gain less than +${MIN_RATE}%/h. Major Order targets and pinned planets always show.`}
          >
            <Stepper
              label="Stalled planets"
              options={[
                { value: "hidden", label: "Hidden" },
                { value: "shown", label: "Shown" },
              ]}
              value={s.showInactive ? "shown" : "hidden"}
              onChange={(v) => s.set("showInactive", v === "shown")}
            />
          </Row>
        </Panel>
        <Panel title="Galaxy map" icon={Settings}>
          <Row
            title="Map overlay"
            text="Regions draws merged enemy fronts; Sectors shows the sector grid with enemy territory hatched; Galaxy shows supply lines only."
          >
            <Stepper<MapOverlay>
              label="Map overlay"
              options={OVERLAY_OPTIONS}
              value={s.mapOverlay}
              onChange={(v) => s.set("mapOverlay", v)}
            />
          </Row>
          <Row title="Planets" text="Glowing markers, or shaded spheres tinted by biome like the in-game map.">
            <Stepper<MapPlanets>
              label="Planet style"
              options={PLANET_OPTIONS}
              value={s.mapPlanets}
              onChange={(v) => s.set("mapPlanets", v)}
            />
          </Row>
          <Row
            title="Render quality"
            text="Minimal keeps the look but stops the pulsing markers and draws fewer, softer stars. Maximum draws the most stars at full sharpness."
          >
            <Stepper<MapQuality>
              label="Render quality"
              options={QUALITY_OPTIONS}
              value={s.mapQuality}
              onChange={(v) => s.set("mapQuality", v)}
            />
          </Row>
          <Row
            title="Map dispatches"
            text="How many dispatches the map's Dispatches box lists: the latest few whatever their age, or every one from the last 72 hours."
          >
            <Stepper<MapDispatches>
              label="Map dispatches"
              options={DISPATCH_OPTIONS}
              value={s.mapDispatches}
              onChange={(v) => s.set("mapDispatches", v)}
            />
          </Row>
        </Panel>
        <Panel title="Notifications" icon={BellRing}>
          <Row title="Popups" text="Which in-app popups appear. Priority hides dispatches.">
            <Stepper<Popups>
              label="Popups"
              options={[
                { value: "all", label: "All" },
                { value: "priority", label: "Priority" },
                { value: "none", label: "None" },
              ]}
              value={s.popups}
              onChange={(v) => s.set("popups", v)}
            />
          </Row>
          {ALERTS.map((a) => (
            <Row key={a.key} title={a.title} text={a.text}>
              <OnOff label={a.title} value={s.alerts[a.key]} onChange={(v) => s.setAlert(a.key, v)} />
            </Row>
          ))}
          <div className="set-actions">
            <button
              className="hud-btn"
              onClick={() => notify("order", "Test alert", "Alerts are working, Helldiver.", s.alerts.system)}
            >
              <BellRing size={14} /> Send test alert
            </button>
          </div>
        </Panel>
        <Panel title="Updates" icon={Download}>
          <UpdateRow />
        </Panel>
        <div className="set-footer">
          <button className="hud-btn" onClick={copyDiagnostics}>
            <ClipboardCopy size={14} /> {copied ? "Copied" : "Copy diagnostics"}
          </button>
          <button className="hud-btn set-cta" onClick={s.resetUnread}>
            <RotateCcw size={14} /> Reset hints and unread
          </button>
          <button className="hud-btn" onClick={s.clearPins} disabled={s.pinned.length === 0}>
            <PinOff size={14} /> Clear pinned planets{s.pinned.length ? ` (${s.pinned.length})` : ""}
          </button>
        </div>
        <p className="set-version caps num">WarCommand Tracker v{APP_VERSION} · changes apply immediately</p>
      </div>
    </div>
  );
}
