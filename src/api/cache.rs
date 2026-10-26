//! Data that isn't a snapshot (history tables, database functions), fetched by key and kept for the session:
//! returning to a page shows what was already loaded while a stale entry refreshes.

use crate::js;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::rc::Rc;

const RETRIES: i32 = 3;

struct Entry {
    at: f64,
    value: Rc<dyn Any>,
}

thread_local! {
    static CACHE: RefCell<HashMap<String, Entry>> = RefCell::new(HashMap::new());
    static IN_FLIGHT: RefCell<HashMap<String, Vec<Box<dyn FnOnce()>>>> = RefCell::new(HashMap::new());
}

fn cached<T: Clone + 'static>(key: &str) -> Option<(f64, T)> {
    CACHE.with_borrow(|c| c.get(key).and_then(|e| Some((e.at, e.value.downcast_ref::<T>()?.clone()))))
}

pub struct Query<T: Send + Sync + 'static> {
    pub data: ReadSignal<Option<T>>,
    /// Nothing to show yet and a fetch under way.
    pub loading: ReadSignal<bool>,
}

impl<T: Send + Sync + 'static> Clone for Query<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: Send + Sync + 'static> Copy for Query<T> {}

/// Fetches `fetch(key)` whenever `key()` changes to Some, unless a cached result is younger than `stale_ms`.
/// Failures are retried, then keep whatever was shown.
pub fn use_query<T, F, Fut>(key: impl Fn() -> Option<String> + 'static, stale_ms: f64, fetch: F) -> Query<T>
where
    T: Clone + Send + Sync + 'static,
    F: Fn(String) -> Fut + Clone + 'static,
    Fut: Future<Output = Result<T, String>> + 'static,
{
    let (data, set_data) = signal::<Option<T>>(None);
    let (loading, set_loading) = signal(false);
    let current = Rc::new(RefCell::new(None::<String>));
    Effect::new(move |_| {
        let Some(k) = key() else { return };
        *current.borrow_mut() = Some(k.clone());
        let hit = cached::<T>(&k);
        if let Some((_, v)) = &hit {
            set_data.set(Some(v.clone()));
        }
        if hit.as_ref().is_some_and(|(at, _)| js::now() - at < stale_ms) {
            set_loading.set(false);
            return;
        }
        set_loading.set(hit.is_none());
        let current = current.clone();
        let done = {
            let k = k.clone();
            move || {
                if current.borrow().as_deref() == Some(k.as_str()) {
                    if let Some((_, v)) = cached::<T>(&k) {
                        set_data.set(Some(v));
                    }
                    set_loading.set(false);
                }
            }
        };
        let first = IN_FLIGHT.with_borrow_mut(|f| match f.get_mut(&k) {
            Some(waiting) => {
                waiting.push(Box::new(done));
                false
            }
            None => {
                f.insert(k.clone(), vec![Box::new(done)]);
                true
            }
        });
        if !first {
            return;
        }
        let fetch = fetch.clone();
        spawn_local(async move {
            // Three retries, 1, 2 and 4 s apart, still loading meanwhile, so a blip doesn't read as an outage.
            for attempt in 0..=RETRIES {
                match fetch(k.clone()).await {
                    Ok(v) => {
                        CACHE.with_borrow_mut(|c| c.insert(k.clone(), Entry { at: js::now(), value: Rc::new(v) }));
                        break;
                    }
                    Err(_) if attempt < RETRIES => js::sleep(1000.0 * 2f64.powi(attempt)).await,
                    Err(_) => {}
                }
            }
            let waiting = IN_FLIGHT.with_borrow_mut(|f| f.remove(&k)).unwrap_or_default();
            for w in waiting {
                w();
            }
        });
    });
    Query { data, loading }
}
