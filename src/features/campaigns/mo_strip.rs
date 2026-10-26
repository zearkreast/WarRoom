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
use warroom_core::major_order::{DeadlineOutlook, order_verdict};
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
    let tone = Signal::derive(move || faction().map_or("var(--cta)".to_string(), |f| faction_color(Some(f)).to_string()));
    let label = move || task.with(|t| t.as_ref().map(|t| t.label.clone()).unwrap_or_default());
    let goal = move || task.with(|t| t.as_ref().map_or(0.0, |t| t.goal));
    view! {
        <li class=move || format!("ms-count {}", if done.get() { "done" } else { "" }) style=move || format!("--k: {};", tone.get())>
            <div class="ms-count-head caps">
                {move || faction().map(|f| view! { <FactionEmblem faction=f.as_str() size=14.0 /> })}
                <span class="ms-count-name">
                    {label}
                    {move || faction().map(|f| view! { <b>{format!(" {}", faction_label(Some(f)))}</b> })}
                </span>
                <span class="ms-count-pct num">{move || fmt_pct(pct.get(), 1)}</span>
            </div>
            <OrderBar
                shown=pct
                value=final_pct
                label=Signal::derive(move || format!("{}{}", label(), faction().map(|f| format!(" {}", faction_label(Some(f)))).unwrap_or_default()))
                value_text=value_text
                tone=tone
                marker=Signal::derive(move || if done.get() { None } else { outlook.get().and_then(|o| o.end) })
                done=done
            />
            <div class="ms-count-foot caps">
                <span class="num">
                    <b>{move || fmt_compact(js_round(shown.get()))}</b>
                    {move || format!(" / {}", fmt_compact(goal()))}
                </span>
                {move || outlook.get().map(|o| view! { <OutlookTag o=o /> })}
            </div>
        </li>
    }
}

#[component]
fn TargetItem(s: OrderState, i: usize) -> impl IntoView {
    let item = task_at(s, i);
    let planet = Memo::new(move |_| item.with(|x| x.as_ref().and_then(|(t, _)| t.planet.clone())));
    let outlook = Memo::new(move |_| item.with(|x| x.as_ref().map(|(_, o)| *o)));
    let done = Signal::derive(move || outlook.with(|o| o.is_some_and(|o| o.outlook == DeadlineOutlook::Done)));
    let percent = Signal::derive(move || if done.get() { 100.0 } else { planet.with(|p| p.as_ref().map_or(0.0, tracked_percent)) });
    let shown = use_count_up(percent);
    let end = move || outlook.get().and_then(|o| o.end).filter(|_| !done.get());
    let owner = Signal::derive(move || faction_color(planet.with(|p| p.as_ref().map(|p| p.current_owner))).to_string());
    let name = move || planet.with(|p| p.as_ref().map(|p| p.name.clone()).unwrap_or_default());
    view! {
        <li style=move || format!("--k: {};", owner.get())>
            <button
                class="ms-name"
                on:click=move |e| {
                    if let Some(index) = planet.with_untracked(|p| p.as_ref().map(|p| p.index)) {
                        ui().open_options(Some(OptionsAnchor { planet: index, x: e.client_x() as f64, y: e.client_y() as f64 }))
                    }
                }
            >
                {move || planet.with(|p| p.as_ref().map(|p| p.current_owner)).map(|f| view! { <FactionEmblem faction=f.as_str() size=16.0 /> })}
                {name}
            </button>
            <OrderBar
                slim=true
                shown=shown
                value=percent
                label=Signal::derive(name)
                value_text=Signal::derive(move || {
                    format!("{}{}", fmt_pct(percent.get(), 1), end().map(|e| format!(", {} by the deadline", fmt_pct(e, 1))).unwrap_or_default())
                })
                tone="var(--f-humans)".to_string()
                rest=owner
                marker=Signal::derive(end)
                done=done
            />
            <span class="ms-path num">
                {move || fmt_pct(shown.get(), 1)}
                {move || end().map(|e| format!(" → {}", fmt_pct(e, 1)))}
            </span>
            {move || {
                outlook.get().map(|o| {
                    view! { <span class="ms-chip caps" style=format!("--tone: {};", o.outlook.tone())>{o.outlook.label()}</span> }
                })
            }}
        </li>
    }
}

#[component]
fn OrderStrip(id: i64) -> impl IntoView {
    let s = order_state(id);
    let open = Memo::new(move |_| settings().get(|s| s.mo_strip_open));
    let targets = Memo::new(move |_| {
        s.tasks.with(|t| {
            t.iter().enumerate().filter_map(|(i, t)| t.planet.as_ref().filter(|_| t.is_planet).map(|p| (i, p.index))).collect::<Vec<_>>()
        })
    });
    let counts = Memo::new(move |_| {
        s.tasks.with(|t| t.iter().enumerate().filter(|(_, t)| !t.is_planet).map(|(i, _)| i).collect::<Vec<_>>())
    });
    let verdict = Memo::new(move |_| order_verdict(&s.tasks.get(), &s.outlooks.get()));
    let title = move || s.order.with(|a| a.as_ref().and_then(|a| a.title.clone()).unwrap_or_else(|| "Major Order".into()));
    let expires = Signal::derive(move || s.order.with(|a| a.as_ref().map_or(0.0, |a| parse_ms(&a.expiration))));
    let toggle = view! {
        <button
            class="ms-toggle"
            aria-expanded=move || open.get().to_string()
            on:click=move |_| settings().set(|s| s.mo_strip_open = !s.mo_strip_open)
        >
            <Icon icon=icons::CHEVRON_RIGHT size=14.0 class="ms-chevron" />
            {title}
        </button>
    };
    view! {
        <Panel
            class=Signal::derive(move || format!("mo-strip {}", if open.get() { "" } else { "folded" }))
            title=""
            title_extra=toggle.into_any()
            right=view! { <span class="mo-ends">"Ends in " <b><Countdown to=expires /></b></span> }.into_any()
        >
            <Show when=move || open.get()>
                {move || verdict.get().map(|(text, tone)| view! { <p class="ms-verdict" style=format!("color: {tone};")>{text}</p> })}
                <Show when=move || targets.with(|t| !t.is_empty())>
                    <ul class="ms-rows">
                        <For each=move || targets.get() key=|k| *k let:t>
                            <TargetItem s=s i=t.0 />
                        </For>
                    </ul>
                </Show>
                <Show when=move || counts.with(|c| !c.is_empty())>
                    <ul class="ms-counts">
                        <For each=move || counts.get() key=|k| *k let:i>
                            <CountRow s=s i=i />
                        </For>
                    </ul>
                </Show>
            </Show>
        </Panel>
    }
}

/// The Major Order answered in one glance: the deadline, and whether each objective makes it at the current pace.
/// Folded by default down to its header; the choice is remembered.
#[component]
pub fn MoStrip() -> impl IntoView {
    let q = queries();
    let ids = Memo::new(move |_| q.assignments.with(|l| l.iter().flat_map(|l| l.iter()).map(|a| a.id).collect::<Vec<_>>()));
    view! {
        <For each=move || ids.get() key=|id| *id let:id>
            <OrderStrip id=id />
        </For>
    }
}
