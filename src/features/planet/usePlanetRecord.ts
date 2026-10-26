import { useEffect, useState } from "react";
import { useLastUpdate } from "../../api/queries";
import type { OwnerChange } from "../../domain/history";
import type { Sample } from "../../domain/progress";
import type { PlayerReading } from "../../history/store";
import { useHistory } from "../../history/useHistory";
import { useNow } from "../../ui/primitives";

export interface PlanetRecord {
  loading: boolean;
  /** The 24 hours of samples up to the newest data (not the clock, so a stalled collector still shows its last day). */
  samples: Sample[];
  /** Helldivers on the planet over the same 24 hours while it had no campaign. */
  idle: PlayerReading[];
  changes: OwnerChange[];
  defendingAt: number[];
  /** When the collector first recorded this planet (or anything, if never this one). */
  recordStart: number | null;
}

const DAY_MS = 86_400_000;
const REFRESH_MS = 5 * 60_000;

/** Everything the history store knows about one planet, re-read every 5 minutes. */
export function usePlanetRecord(planet: number | null): PlanetRecord {
  const store = useHistory((s) => s.store);
  const latestChanges = useHistory((s) => s.changes);
  const tick = useNow(REFRESH_MS);
  const lastUpdate = useLastUpdate().data;
  const [rec, setRec] = useState<PlanetRecord>({
    loading: true,
    samples: [],
    idle: [],
    changes: [],
    defendingAt: [],
    recordStart: null,
  });

  useEffect(() => {
    if (planet == null) return;
    let live = true;
    setRec((r) => ({ ...r, loading: true }));
    const since = (lastUpdate ?? Date.now()) - DAY_MS;
    void Promise.all([
      store.samples(planet, since),
      store.idlePlayers(planet, since),
      store.changes({ planet, limit: 200 }),
      store.defendingTimes(planet),
      store.earliest(planet).then((t) => t ?? store.earliest()),
    ])
      .then(([samples, idle, changes, defendingAt, recordStart]) => {
        if (live) setRec({ loading: false, samples, idle, changes, defendingAt, recordStart });
      })
      .catch(() => live && setRec((r) => ({ ...r, loading: false })));
    return () => {
      live = false;
    };
  }, [store, planet, tick, latestChanges, lastUpdate]);

  return rec;
}
