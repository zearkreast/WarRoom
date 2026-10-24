import { useEffect, useRef } from "react";
import { useAssignments, useCampaigns, useDispatches } from "./api/queries";
import type { Planet } from "./api/types";
import { notify } from "./alerts/notify";
import { POLL_MS } from "./config";
import { factionLabel } from "./domain/factions";
import { dispatchTitle } from "./domain/format";
import { detectNewDefenses } from "./domain/history";
import { useHistory } from "./history/useHistory";
import { useSettings } from "./state";

/**
 * Watches the shared data for anything worth an alert. Mounted once at the app root.
 */
export function useDataSync() {
  const campaigns = useCampaigns();
  const assignments = useAssignments();
  const dispatches = useDispatches();
  const refresh = useHistory((s) => s.refresh);
  const lastDefenseSnapshot = useRef<Planet[] | undefined>(undefined);

  // Captures and losses, from the collector's ownership log.
  useEffect(() => {
    let live = true;
    async function check() {
      const fresh = await refresh().catch(() => []);
      const { alerts } = useSettings.getState();
      if (!live || !alerts.captures) return;
      for (const c of [...fresh].reverse()) {
        const won = c.to === "Humans";
        notify(
          won ? "capture" : "loss",
          won ? `${c.planetName} liberated` : `${c.planetName} lost`,
          won
            ? `Super Earth took ${c.planetName} from the ${factionLabel(c.from)}.`
            : `The ${factionLabel(c.to)} took ${c.planetName}.`,
          alerts.system,
        );
      }
    }
    void check();
    const id = setInterval(check, POLL_MS);
    return () => {
      live = false;
      clearInterval(id);
    };
  }, [refresh]);

  // New defense campaigns.
  useEffect(() => {
    if (!campaigns.data) return;
    const list = campaigns.data.map((c) => c.planet);
    const fresh = detectNewDefenses(lastDefenseSnapshot.current, list);
    lastDefenseSnapshot.current = list;
    const { alerts } = useSettings.getState();
    if (!alerts.defenses) return;
    for (const p of fresh) {
      notify("defense", `Defend ${p.name}`, `The ${factionLabel(p.event!.faction)} are attacking ${p.name}.`, alerts.system);
    }
  }, [campaigns.data]);

  // New Major Orders and dispatches. The first load only seeds what's been seen.
  useEffect(() => {
    if (!assignments.data || !dispatches.data) return;
    const s = useSettings.getState();
    const firstRun = s.seenAssignments.length === 0 && s.seenDispatch === 0;
    const latestDispatch = Math.max(0, ...dispatches.data.map((d) => d.id));
    if (!firstRun) {
      if (s.alerts.orders) {
        for (const a of assignments.data.filter((a) => !s.seenAssignments.includes(a.id))) {
          const briefing = a.briefing ?? a.title ?? "High Command has issued new orders.";
          notify("order", "New Major Order", briefing, s.alerts.system, briefing);
        }
      }
      if (s.alerts.dispatches) {
        for (const d of dispatches.data.filter((d) => d.id > s.seenDispatch).slice(0, 3)) {
          notify("dispatch", "New dispatch", dispatchTitle(d.message), s.alerts.system, d.message);
        }
      }
    }
    s.markSeen(
      [...new Set([...s.seenAssignments, ...assignments.data.map((a) => a.id)])].slice(-50),
      Math.max(s.seenDispatch, latestDispatch),
    );
  }, [assignments.data, dispatches.data]);
}
