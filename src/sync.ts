import { isTauri } from "@tauri-apps/api/core";
import { useEffect, useRef } from "react";
import { useAssignments, useCampaigns, useDispatches, useEpisodes } from "./api/queries";
import type { Planet } from "./api/types";
import { notify, requestNotifications } from "./alerts/notify";
import { POLL_MS } from "./config";
import { campaignAlerts, campaignHeadline } from "./domain/campaignNews";
import { factionLabel } from "./domain/factions";
import { dispatchTitle } from "./domain/format";
import { detectNewDefenses } from "./domain/history";
import { useHistory } from "./history/useHistory";
import { useSettings } from "./state";

/** Android builds with Firebase get Strohmann News pushed by the collector, so here it is only toasted. */
const NEWS_PUSHED = import.meta.env.VITE_NEWS_PUSH === "1";

/**
 * Watches the shared data for anything worth an alert. Mounted once at the app root.
 */
export function useDataSync() {
  const campaigns = useCampaigns();
  const assignments = useAssignments();
  const dispatches = useDispatches();
  const episodes = useEpisodes();
  const refresh = useHistory((s) => s.refresh);
  const lastDefenseSnapshot = useRef<Planet[] | undefined>(undefined);

  // Not on the web, where browsers block or bury a permission prompt nobody clicked for.
  useEffect(() => {
    if (isTauri() && useSettings.getState().alerts.system) requestNotifications();
  }, []);

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
  // Strohmann News alerts: keep their wording in step with the collector's pushes (collector/collector.ts).
  useEffect(() => {
    if (!assignments.data || !dispatches.data) return;
    const s = useSettings.getState();
    const system = s.alerts.system && !NEWS_PUSHED;
    const firstRun = s.seenAssignments.length === 0 && s.seenDispatch === 0;
    const latestDispatch = Math.max(0, ...dispatches.data.map((d) => d.id));
    if (!firstRun) {
      if (s.alerts.orders) {
        for (const a of assignments.data.filter((a) => !s.seenAssignments.includes(a.id))) {
          const briefing = a.briefing ?? a.title ?? "High Command has issued new orders.";
          notify("order", "New Major Order", briefing, system, briefing);
        }
      }
      if (s.alerts.dispatches) {
        for (const d of dispatches.data.filter((d) => d.id > s.seenDispatch).slice(0, 3)) {
          notify("dispatch", "New dispatch", dispatchTitle(d.message), system, d.message);
        }
      }
    }
    s.markSeen(
      [...new Set([...s.seenAssignments, ...assignments.data.map((a) => a.id)])].slice(-50),
      Math.max(s.seenDispatch, latestDispatch),
    );
  }, [assignments.data, dispatches.data]);

  // Galactic Campaign launches and outcomes, as Strohmann News tells them.
  useEffect(() => {
    if (!episodes.data) return;
    const s = useSettings.getState();
    const { fresh, keys } = campaignAlerts(episodes.data, s.seenCampaignEvents);
    if (s.alerts.orders) {
      for (const ev of fresh) {
        const launched = ev.type === "launched";
        notify(
          "order",
          launched ? "New Galactic Campaign" : "Galactic Campaign over",
          campaignHeadline(ev),
          s.alerts.system && !NEWS_PUSHED,
          launched ? ev.campaign.description : undefined,
        );
      }
    }
    s.markCampaignSeen(keys);
  }, [episodes.data]);
}
