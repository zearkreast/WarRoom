//! Campaign rewards added in the database's `campaign_rewards` table, for items the built-in list lacks.

use crate::api::source;
use crate::js;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::sync::{Arc, OnceLock};
use warroom_core::data::ListedReward;

const REFRESH_MS: f64 = 60.0 * 60_000.0;

static LISTED: OnceLock<RwSignal<Arc<Vec<ListedReward>>>> = OnceLock::new();

/// The table's rows; empty until the first read, and while the table doesn't exist.
pub fn listed_rewards() -> RwSignal<Arc<Vec<ListedReward>>> {
    *LISTED.get().expect("rewards initialised")
}

/// Reads the table now and every hour for as long as the app runs; called once, before anything renders.
pub fn start() {
    let rows = *LISTED.get_or_init(|| RwSignal::new(Arc::new(Vec::new())));
    spawn_local(async move {
        loop {
            // A failed read keeps the rows already shown.
            if let Ok(list) = source::select("campaign_rewards", "select=mix_id,kind,name").await
                && let Ok(list) = warroom_core::api::from_value::<Vec<ListedReward>>(serde_json::Value::Array(list))
            {
                rows.set(Arc::new(list));
            }
            js::sleep(REFRESH_MS).await;
        }
    });
}
