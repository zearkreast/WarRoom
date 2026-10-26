use crate::js::{self, num};
use crate::ui::primitives::Countdown;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use std::cell::Cell;
use std::rc::Rc;
use warroom_core::format::{fmt_compact_long, fmt_pct};
use warroom_core::major_order::{DeadlineOutlook, TaskOutlook, TaskView, task_percent};

// Keep in sync with --mo-grow in majorOrder.css.
const GROW_MS: f64 = 1100.0;

/// Counts up to `to` from zero on first paint and glides to each new value.
pub fn use_count_up(to: Signal<f64>) -> ReadSignal<f64> {
    let reduced = js::prefers_reduced_motion();
    let first = if reduced { to.get_untracked() } else { 0.0 };
    let (shown, set_shown) = signal(first);
    let at = Rc::new(Cell::new(first));
    let running: Rc<Cell<Option<Rc<Cell<bool>>>>> = Rc::new(Cell::new(None));
    let r = running.clone();
    Effect::new(move |_| {
        let target = to.get();
        if let Some(stop) = r.take() {
            stop.set(true);
        }
        let from = at.get();
        if reduced || from == target {
            at.set(target);
            set_shown.set(target);
            return;
        }
        let start = js::performance_now();
        let at = at.clone();
        r.set(Some(js::animation_loop(move |now| {
            let k = ((now - start) / GROW_MS).min(1.0);
            let v = if k < 1.0 { from + (target - from) * (1.0 - (1.0 - k).powi(3)) } else { target };
            at.set(v);
            set_shown.set(v);
            k < 1.0
        })));
    });
    let running = SendWrapper::new(running);
    on_cleanup(move || {
        if let Some(stop) = Cell::take(&**running) {
            stop.set(true);
        }
    });
    shown
}

pub struct CountTask {
    pub shown: ReadSignal<f64>,
    /// The bar follows the count-up too, not a CSS transition, so it can't drift from the numbers.
    pub pct: Signal<f64>,
    pub final_pct: Signal<f64>,
    pub value_text: Signal<String>,
}

/// A count task as it counts up: the tally shown, its percent, and the final percent read out to screen readers.
pub fn use_count_task(task: Memo<Option<TaskView>>) -> CountTask {
    let shown = use_count_up(Signal::derive(move || task.with(|t| t.as_ref().map_or(0.0, |t| t.current))));
    let final_pct = Signal::derive(move || task.with(|t| t.as_ref().map_or(0.0, task_percent)));
    let pct = Signal::derive(move || {
        let s = shown.get();
        task.with(|t| t.as_ref().map_or(0.0, |t| task_percent(&TaskView { current: s, done: s >= t.goal, ..t.clone() })))
    });
    let value_text = Signal::derive(move || {
        task.with(|t| {
            t.as_ref().map_or_else(String::new, |t| {
                format!(
                    "{} of {}, {}{}",
                    fmt_compact_long(t.current),
                    fmt_compact_long(t.goal),
                    fmt_pct(task_percent(t), 1),
                    if t.done { ", complete" } else { "" }
                )
            })
        })
    });
    CountTask { shown, pct, final_pct, value_text }
}

/// A Major Order progress bar: cells brightening toward a glowing head over a hatched remainder with
/// quarter marks. It draws `shown`, so a count-up can move it in step with the numbers beside it.
#[component]
pub fn OrderBar(
    #[prop(into)] shown: Signal<f64>,
    /// Where the bar is headed; the value screen readers hear.
    #[prop(into)]
    value: Signal<f64>,
    #[prop(into)] label: Signal<String>,
    #[prop(optional, into)] value_text: Option<Signal<String>>,
    /// Fill colour.
    #[prop(into)]
    tone: Signal<String>,
    /// Remainder colour, usually the enemy holding the planet; the fill colour when left out.
    #[prop(optional, into)]
    rest: Option<Signal<String>>,
    /// Where the bar is projected to stand at the deadline, drawn only when it falls short.
    #[prop(optional, into)]
    marker: Option<Signal<Option<f64>>>,
    #[prop(optional, into)] done: Option<Signal<bool>>,
    #[prop(optional)] slim: bool,
) -> impl IntoView {
    let width = move || shown.get().clamp(0.0, 100.0);
    let class = format!("mo-bar{}{}", if rest.is_some() { " versus" } else { "" }, if slim { " slim" } else { "" });
    let style = move || format!("--k: {}; --rest: {};", tone.get(), rest.map_or_else(|| tone.get(), |r| r.get()));
    let done = move || done.is_some_and(|d| d.get());
    let mark = move || marker.and_then(|m| m.get()).filter(|m| *m > 0.0 && *m < 100.0);
    view! {
        <div
            class=class
            style=style
            role="progressbar"
            aria-label=label
            aria-valuemin="0"
            aria-valuemax="100"
            aria-valuenow=move || num(value.get().floor())
            aria-valuetext=move || value_text.map(|v| v.get())
        >
            <div class="mo-bar-fill" style=move || format!("width: {}%;", num(width()))>
                <i class="mo-bar-sheen"></i>
            </div>
            <Show when=move || !done()>
                <i class="mo-bar-head" style=move || format!("left: {}%;", num(width()))></i>
            </Show>
            <Show when=move || mark().is_some()>
                <i class="mo-bar-mark" style=move || format!("left: {}%;", num(mark().unwrap_or(0.0)))></i>
            </Show>
        </div>
    }
}

/// A task's outlook: when it completes, or where it stands at the deadline, then the verdict.
#[component]
pub fn OutlookTag(o: TaskOutlook) -> impl IntoView {
    view! {
        <span class="mo-outlook">
            {match (o.eta, o.outlook, o.end) {
                (Some(eta), _, _) => Some(view! { <span>"Complete in " <Countdown to=eta /></span> }.into_any()),
                (None, DeadlineOutlook::Behind, Some(end)) => Some(view! { <span>{format!("Ends at {}", fmt_pct(end, 1))}</span> }.into_any()),
                _ => None,
            }}
            <span class="mo-verdict caps" style=format!("--tone: {};", o.outlook.tone())>
                {o.outlook.label()}
            </span>
        </span>
    }
}
