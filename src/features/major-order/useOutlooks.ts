import { useCampaigns, useOrderSamples } from "../../api/queries";
import type { Assignment } from "../../api/types";
import { orderTaskRates, taskOutlook, taskPercent, type TaskOutlook, type TaskView } from "../../domain/majorOrder";
import { trackedPercent } from "../../domain/progress";
import { useNow } from "../../ui/primitives";

/** Each task's outlook at the current pace: counts from the order's own readings, planets from their campaign. */
export function useOutlooks(a: Assignment, tasks: TaskView[]): TaskOutlook[] {
  const samples = useOrderSamples().data;
  const campaigns = useCampaigns().data;
  const now = useNow(60_000);
  const deadline = Date.parse(a.expiration);
  const counts = orderTaskRates(samples, a.id, a.tasks.length);
  return tasks.map((t, i): TaskOutlook => {
    if (t.kind === "count") {
      const rate = counts[i] != null && t.goal ? (counts[i]! / t.goal) * 100 : null;
      return taskOutlook(taskPercent(t), rate, now, deadline);
    }
    const p = t.planet!;
    if (t.done || (p.currentOwner === "Humans" && !p.event)) return { outlook: "done", end: 100, eta: null, rate: null };
    const front = campaigns?.find((c) => c.planet.index === p.index);
    if (campaigns && !front) return { outlook: "no-front", end: null, eta: null, rate: null };
    return taskOutlook(trackedPercent(p), front?.rate ?? null, now, deadline);
  });
}
