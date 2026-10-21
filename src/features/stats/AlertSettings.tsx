import { notify } from "../../alerts/notify";
import { useSettings, type AlertSettings } from "../../state";
import { Panel } from "../../ui/primitives";

const OPTIONS: { key: keyof AlertSettings; label: string }[] = [
  { key: "orders", label: "New Major Orders" },
  { key: "dispatches", label: "New dispatches" },
  { key: "captures", label: "Planets liberated or lost" },
  { key: "defenses", label: "New defense campaigns" },
  { key: "system", label: "Show as system notifications" },
];

export function AlertSettingsPanel() {
  const { alerts, setAlert } = useSettings();
  return (
    <Panel title="Alerts">
      <div className="panel-body settings">
        {OPTIONS.map((o) => (
          <label key={o.key} className="toggle">
            <input type="checkbox" checked={alerts[o.key]} onChange={(e) => setAlert(o.key, e.target.checked)} />
            <span className="caps">{o.label}</span>
          </label>
        ))}
        <button className="hud-btn" onClick={() => notify("order", "Test alert", "Alerts are working, Helldiver.", alerts.system)}>
          Send test alert
        </button>
      </div>
    </Panel>
  );
}
