use crate::api::history::{self, PlayerReading, history};
use crate::api::queries::queries;
use crate::js;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use warroom_core::owner_changes::OwnerChange;
use warroom_core::progress::Sample;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlanetRecord {
    pub loading: bool,
    /// The 24 hours of samples up to the newest data (not the clock, so a stalled collector still shows its last day).
    pub samples: Arc<Vec<Sample>>,
    /// Helldivers on the planet over the same 24 hours while it had no campaign.
    pub idle: Arc<Vec<PlayerReading>>,
    pub changes: Arc<Vec<OwnerChange>>,
    pub defending_at: Arc<Vec<f64>>,
    /// When the collector first recorded this planet (or anything, if never this one).
    pub record_start: Option<f64>,
}

const DAY_MS: f64 = 86_400_000.0;
const REFRESH_MS: f64 = 5.0 * 60_000.0;

/// Everything the history store knows about one planet, re-read every 5 minutes.
pub fn use_planet_record(planet: i64) -> ReadSignal<PlanetRecord> {
    let (rec, set_rec) = signal(PlanetRecord { loading: true, ..Default::default() });
    let tick = js::use_now(REFRESH_MS);
    let last_update = queries().last_update;
    let latest = history().changes;
    let generation = Arc::new(AtomicU64::new(0));
    let alive = generation.clone();
    on_cleanup(move || {
        alive.fetch_add(1, Ordering::SeqCst);
    });
    Effect::new(move |_| {
        tick.track();
        latest.track();
        let since = last_update.get().unwrap_or_else(js::now) - DAY_MS;
        let run = generation.fetch_add(1, Ordering::SeqCst) + 1;
        let generation = generation.clone();
        set_rec.update(|r| r.loading = true);
        spawn_local(async move {
            let start = async {
                match history::earliest(Some(planet)).await? {
                    Some(t) => Ok(Some(t)),
                    None => history::earliest(None).await,
                }
            };
            let result = futures::try_join!(
                history::samples(planet, since),
                history::idle_players(planet, since),
                history::changes(Some(planet), 200),
                history::defending_times(planet),
                start
            );
            if generation.load(Ordering::SeqCst) != run {
                return;
            }
            match result {
                Ok((samples, idle, changes, defending_at, record_start)) => set_rec.set(PlanetRecord {
                    loading: false,
                    samples: Arc::new(samples),
                    idle: Arc::new(idle),
                    changes: Arc::new(changes.into_iter().map(|c| c.change).collect()),
                    defending_at: Arc::new(defending_at),
                    record_start,
                }),
                Err(_) => set_rec.update(|r| r.loading = false),
            }
        });
    });
    rec
}
