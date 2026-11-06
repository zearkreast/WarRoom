use crate::api::queries::queries;
use crate::features::major_order::order_bar::{CountTask, OrderBar, OutlookTag, use_count_task, use_count_up};
use crate::features::major_order::panel::{OrderState, order_state, task_at};
use crate::state::{OptionsAnchor, settings, ui};
use crate::ui::emblems::FactionEmblem;
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::{Countdown, Panel};
use leptos::prelude::*;
use warroom_core::factions::{faction_color, faction_label};
use warroom_core::format::{fmt_compact, fmt_pct, js_round};
use warroom_core::major_order::{DeadlineOutlook, order_outlook, order_verdict};
use warroom_core::progress::tracked_percent;
use warroom_core::time::parse_ms;

#[component]
fn CountRow(s: OrderState, i: usize) -> impl IntoView {
    let item = task_at(s, i);
    let task = Memo::new(move |_| item.with(|x| x.as_ref().map(|(t, _)| t.clone())));
    let outlook = Memo::new(move |_| item.with(|x| x.as_ref().map(|(_, o)| *o)));
    let CountTask { shown, pct, final_pct, value_text } = use_count_task(task);
    let done = Signal::derive(move || task.with(|t| t.as_ref().is_some_and(|t| t.done)));
    let faction = move || task.with(|t| t.as_ref().and_then(|t| t.faction));
    let tone =
        Signal::derive(move || faction().map_or("var(--cta)".to_string(), |f| faction_color(Some(f)).to_string()));
    let label = move || task.with(|t| t.as_ref().map(|t| t.label.clone()).unwrap_or_default());
    let goal = move || task.with(|t| t.as_ref().map_or(0.0, |t| t.goal));
    view! {
        <li class=move || format!("ms-obj {}", if done.get() { "done" } else { "" }) style=move || format!("--k: {};", tone.get())>
            <span class="ms-name">
                {move || faction().map(|f| view! { <FactionEmblem faction=f.as_str() size=16.0 /> })}
                <span class="ms-what">
                    <span class="ms-verb caps">{label}</span>
                    {move || faction().map(|f| view! { <b>{faction_label(Some(f))}</b> })}
                </span>
            </span>
            <OrderBar
                shown=pct
                value=final_pct
                label=Signal::derive(move || format!("{}{}", label(), faction().map(|f| format!(" {}", faction_label(Some(f)))).unwrap_or_default()))
                value_text=value_text
                tone=tone
                marker=Signal::derive(move || if done.get() { None } else { outlook.get().and_then(|o| o.end) })
                done=done
            />
            <span class="ms-value num">
                <span class="ms-tally">{move || format!("{} / {}", fmt_compact(js_round(shown.get())), fmt_compact(goal()))}</span>
                <b>{move || fmt_pct(pct.get(), 1)}</b>
            </span>
            {move || outlook.get().map(|o| view! { <OutlookTag o=o /> })}
        </li>
    }
}

#[component]
fn TargetItem(s: OrderState, i: usize) -> impl IntoView {
    let item = task_at(s, i);
    let planet = Memo::new(move |_| item.with(|x| x.as_ref().and_then(|(t, _)| t.planet.clone())));
    let verb = Memo::new(move |_| item.with(|x| x.as_ref().map(|(t, _)| t.label.clone()).unwrap_or_default()));
    let outlook = Memo::new(move |_| item.with(|x| x.as_ref().map(|(_, o)| *o)));
    let done = Signal::derive(move || outlook.with(|o| o.is_some_and(|o| o.outlook == DeadlineOutlook::Done)));
    let percent =
        Signal::derive(
            move || if done.get() { 100.0 } else { planet.with(|p| p.as_ref().map_or(0.0, tracked_percent)) },
        );
    let shown = use_count_up(percent);
    let end = move || outlook.get().and_then(|o| o.end).filter(|_| !done.get());
    let owner = Signal::derive(move || faction_color(planet.with(|p| p.as_ref().map(|p| p.current_owner))).to_string());
    let enemy = Signal::derive(move || {
        faction_color(planet.with(|p| p.as_ref().map(|p| p.event.as_ref().map_or(p.current_owner, |e| e.faction))))
            .to_string()
    });
    let name = move || planet.with(|p| p.as_ref().map(|p| p.name.clone()).unwrap_or_default());
    view! {
        <li class=move || format!("ms-obj {}", if done.get() { "done" } else { "" }) style=move || format!("--k: {};", owner.get())>
            <button
                class="ms-name"
                on:click=move |e| {
                    if let Some(index) = planet.with_untracked(|p| p.as_ref().map(|p| p.index)) {
                        ui().open_options(Some(OptionsAnchor::at(index, &e)))
                    }
                }
            >
                {move || planet.with(|p| p.as_ref().map(|p| p.current_owner)).map(|f| view! { <FactionEmblem faction=f.as_str() size=16.0 /> })}
                <span class="ms-what">
                    <span class="ms-verb caps">{move || verb.get()}</span>
                    <b>{name}</b>
                </span>
            </button>
            <OrderBar
                shown=shown
                value=percent
                label=Signal::derive(move || format!("{} {}", verb.get(), name()))
                value_text=Signal::derive(move || {
                    format!("{}{}", fmt_pct(percent.get(), 1), end().map(|e| format!(", {} by the deadline", fmt_pct(e, 1))).unwrap_or_default())
                })
                tone="var(--f-humans)".to_string()
                rest=enemy
                marker=Signal::derive(end)
                done=done
            />
            <span class="ms-value num">
                <b>{move || fmt_pct(shown.get(), 1)}</b>
            </span>
            {move || outlook.get().map(|o| view! { <OutlookTag o=o /> })}
        </li>
    }
}

#[component]
fn OrderStrip(id: i64) -> impl IntoView {
    let s = order_state(id);
    let open = Memo::new(move |_| settings().get(|s| s.mo_strip_unfolded.contains(&id)));
    // Planet objectives first, then counts, each keeping the order's own order.
    let rows = Memo::new(move |_| {
        s.tasks.with(|t| {
            let planets = t.iter().enumerate().filter(|(_, t)| t.is_planet && t.planet.is_some());
            let counts = t.iter().enumerate().filter(|(_, t)| !t.is_planet);
            planets.chain(counts).map(|(i, t)| (i, t.is_planet)).collect::<Vec<_>>()
        })
    });
    let state = Memo::new(move |_| {
        let sentence = order_verdict(&s.tasks.get(), &s.outlooks.get())?.0;
        s.outlooks.with(|o| order_outlook(o)).map(|o| (o, sentence))
    });
    let title =
        move || s.order.with(|a| a.as_ref().and_then(|a| a.title.clone()).unwrap_or_else(|| "Major Order".into()));
    let expires = Signal::derive(move || s.order.with(|a| a.as_ref().map_or(0.0, |a| parse_ms(&a.expiration))));
    let toggle = view! {
        <button
            class="ms-toggle"
            aria-expanded=move || open.get().to_string()
            on:click=move |_| settings().flip_strip_fold(id)
        >
            <Icon icon=icons::CHEVRON_RIGHT size=14.0 class="ms-chevron" />
            {title}
        </button>
        {move || {
            state
                .get()
                .map(|(o, sentence)| {
                    view! {
                        <span class="ms-state caps" style=format!("--tone: {};", o.tone()) title=sentence>
                            {o.label()}
                        </span>
                    }
                })
        }}
    };
    view! {
        <Panel
            class=Signal::derive(move || format!("mo-strip {}", if open.get() { "" } else { "folded" }))
            title=""
            title_extra=toggle.into_any()
            right=view! {
                <span class="mo-ends">
                    <span class="mo-ends-label">"Ends in "</span>
                    <b><Countdown to=expires /></b>
                </span>
            }
                .into_any()
        >
            <Show when=move || open.get()>
                <ul class="ms-rows">
                    <For each=move || rows.get() key=|k| *k let:row>
                        {if row.1 { view! { <TargetItem s=s i=row.0 /> }.into_any() } else { view! { <CountRow s=s i=row.0 /> }.into_any() }}
                    </For>
                </ul>
            </Show>
        </Panel>
    }
}

/// The Major Order answered in one glance: the deadline, and whether each objective makes it at the current pace.
/// Each order starts folded down to its header; the choice is remembered per order.
#[component]
pub fn MoStrip() -> impl IntoView {
    let q = queries();
    let ids =
        Memo::new(move |_| q.assignments.with(|l| l.iter().flat_map(|l| l.iter()).map(|a| a.id).collect::<Vec<_>>()));
    view! {
        <For each=move || ids.get() key=|id| *id let:id>
            <OrderStrip id=id />
        </For>
    }
}
