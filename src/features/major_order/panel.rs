use crate::api::queries::queries;
use crate::features::major_order::order_bar::{CountTask, OrderBar, OutlookTag, use_count_task, use_count_up};
use crate::features::major_order::outlooks::outlooks;
use crate::js;
use crate::state::{OptionsAnchor, ui};
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::{Countdown, Empty, Loading, Panel};
use leptos::prelude::*;
use std::sync::Arc;
use warroom_core::api::Assignment;
use warroom_core::factions::{faction_color, faction_label};
use warroom_core::format::{fmt_compact, fmt_pct, js_round};
use warroom_core::major_order::{
    DeadlineOutlook, TaskOutlook, TaskView, describe_assignment, order_name, order_verdict, reward_label,
};
use warroom_core::progress::tracked_percent;
use warroom_core::time::parse_ms;

pub fn signed(n: f64, text: String) -> String {
    format!("{}{text}", if n >= 0.0 { "+" } else { "−" })
}

/// An order's tasks and their outlooks, kept current as snapshots arrive and the clock moves.
#[derive(Clone, Copy)]
pub struct OrderState {
    pub order: Memo<Option<Assignment>>,
    pub tasks: Memo<Arc<Vec<TaskView>>>,
    pub outlooks: Memo<Arc<Vec<TaskOutlook>>>,
}

pub fn order_state(id: i64) -> OrderState {
    let q = queries();
    let order = Memo::new(move |_| q.assignments.with(|l| l.as_ref()?.iter().find(|a| a.id == id).cloned()));
    let tasks = Memo::new(move |_| {
        let planets = q.fresh_planets.get();
        Arc::new(order.with(|a| {
            a.as_ref().map(|a| describe_assignment(a, planets.as_deref().map(|p| p.as_slice()))).unwrap_or_default()
        }))
    });
    let now = js::use_now(60_000.0);
    let outlooks = Memo::new(move |_| {
        let tasks = tasks.get();
        Arc::new(order.with(|a| a.as_ref().map(|a| outlooks(a, &tasks, now.get())).unwrap_or_default()))
    });
    OrderState { order, tasks, outlooks }
}

pub fn task_at(s: OrderState, i: usize) -> Signal<Option<(TaskView, TaskOutlook)>> {
    Signal::derive(move || {
        let t = s.tasks.with(|t| t.get(i).cloned())?;
        let o = s.outlooks.with(|o| o.get(i).copied())?;
        Some((t, o))
    })
}

#[component]
fn PlanetTask(s: OrderState, i: usize) -> impl IntoView {
    let item = task_at(s, i);
    let done =
        Signal::derive(move || item.with(|x| x.as_ref().is_some_and(|(_, o)| o.outlook == DeadlineOutlook::Done)));
    let pct = Signal::derive(move || {
        if done.get() {
            return 100.0;
        }
        item.with(|x| x.as_ref().and_then(|(t, _)| t.planet.as_ref()).map_or(0.0, tracked_percent))
    });
    let shown = use_count_up(pct);
    let planet = Memo::new(move |_| item.with(|x| x.as_ref().and_then(|(t, _)| t.planet.clone())));
    let label = Memo::new(move |_| item.with(|x| x.as_ref().map(|(t, _)| t.label.clone()).unwrap_or_default()));
    let owner = Signal::derive(move || faction_color(planet.with(|p| p.as_ref().map(|p| p.current_owner))).to_string());
    let name = move || planet.with(|p| p.as_ref().map(|p| p.name.clone()).unwrap_or_default());
    let outlook = Memo::new(move |_| item.with(|x| x.as_ref().map(|(_, o)| *o)));
    view! {
        <li class=move || format!("mo-task {}", if done.get() { "done" } else { "" }) style="--k: var(--f-humans);">
            <div class="mo-task-head">
                {move || view! { <Icon icon=if done.get() { icons::CHECK_SQUARE } else { icons::SQUARE } size=16.0 class="mo-check" /> }}
                <span>
                    {move || format!("{} ", label.get())}
                    <button
                        class="link"
                        style=move || format!("color: {};", owner.get())
                        on:click=move |e| {
                            if let Some(index) = planet.with_untracked(|p| p.as_ref().map(|p| p.index)) {
                                ui().open_options(Some(OptionsAnchor { planet: index, x: e.client_x() as f64, y: e.client_y() as f64 }))
                            }
                        }
                    >
                        {name}
                    </button>
                </span>
                <span class="num mo-pct">{move || fmt_pct(shown.get(), 1)}</span>
            </div>
            <OrderBar
                shown=shown
                value=pct
                label=Signal::derive(move || format!("{} {}", label.get(), name()))
                value_text=Signal::derive(move || format!("{}{}", fmt_pct(pct.get(), 1), if done.get() { ", complete" } else { "" }))
                tone="var(--f-humans)".to_string()
                rest=owner
                marker=Signal::derive(move || if done.get() { None } else { outlook.get().and_then(|o| o.end) })
                done=done
            />
            <div class="mo-task-foot num">
                <span>
                    {move || {
                        let o = outlook.get()?;
                        let rate = o.rate.filter(|_| !done.get()).map(|r| signed(r, format!("{}/h", fmt_pct(r.abs(), 2))));
                        let end = o.end.filter(|_| !done.get());
                        Some(view! {
                            {rate.map(|r| view! { <b>{r}</b> {end.map(|_| " · ")} })}
                            {end.map(|e| format!("End result {}", fmt_pct(e, 2)))}
                        })
                    }}
                </span>
                {move || outlook.get().map(|o| view! { <OutlookTag o=o /> })}
            </div>
        </li>
    }
}

#[component]
fn CountTaskRow(s: OrderState, i: usize) -> impl IntoView {
    let item = task_at(s, i);
    let task = Memo::new(move |_| item.with(|x| x.as_ref().map(|(t, _)| t.clone())));
    let outlook = Memo::new(move |_| item.with(|x| x.as_ref().map(|(_, o)| *o)));
    let CountTask { shown, pct, final_pct, value_text } = use_count_task(task);
    let done = Signal::derive(move || task.with(|t| t.as_ref().is_some_and(|t| t.done)));
    let goal = move || task.with(|t| t.as_ref().map_or(0.0, |t| t.goal));
    let faction = move || task.with(|t| t.as_ref().and_then(|t| t.faction));
    let tone =
        Signal::derive(move || faction().map_or("var(--cta)".to_string(), |f| faction_color(Some(f)).to_string()));
    let label = move || task.with(|t| t.as_ref().map(|t| t.label.clone()).unwrap_or_default());
    view! {
        <li class=move || format!("mo-task {}", if done.get() { "done" } else { "" }) style=move || format!("--k: {};", tone.get())>
            <div class="mo-task-head">
                {move || view! { <Icon icon=if done.get() { icons::CHECK_SQUARE } else { icons::SQUARE } size=16.0 class="mo-check" /> }}
                <span>
                    {label}
                    {move || faction().map(|f| view! { <span style=format!("color: {};", tone.get())>{format!(" {}", faction_label(Some(f)))}</span> })}
                </span>
                <span class="num mo-pct">{move || fmt_pct(pct.get(), 1)}</span>
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
            <div class="mo-task-foot num">
                <span>
                    <b>{move || fmt_compact(js_round(shown.get()))}</b>
                    {move || format!(" / {}", fmt_compact(goal()))}
                    {move || {
                        let per_hour = outlook.get().and_then(|o| o.rate).map(|r| r / 100.0 * goal()).filter(|_| !done.get());
                        per_hour.map(|p| format!(" · {}", signed(p, format!("{}/h", fmt_compact(p.abs())))))
                    }}
                </span>
                {move || outlook.get().map(|o| view! { <OutlookTag o=o /> })}
            </div>
        </li>
    }
}

#[component]
pub fn OrderCard(
    id: i64,
    #[prop(into)] name: Signal<Option<String>>,
    #[prop(optional)] compact: bool,
) -> impl IntoView {
    let s = order_state(id);
    let slots =
        Memo::new(move |_| s.tasks.with(|t| t.iter().enumerate().map(|(i, t)| (i, t.is_planet)).collect::<Vec<_>>()));
    let title =
        move || s.order.with(|a| a.as_ref().and_then(|a| a.title.clone()).unwrap_or_else(|| "Major Order".into()));
    let expires = Signal::derive(move || s.order.with(|a| a.as_ref().map_or(0.0, |a| parse_ms(&a.expiration))));
    view! {
        <Panel
            class="mo"
            icon=icons::TARGET
            title=Signal::derive(title)
            title_extra=view! { {move || name.get().map(|n| view! { <span class="mo-name">{n}</span> })} }.into_any()
            right=view! { <span class="mo-ends">"Ends in " <b><Countdown to=expires /></b></span> }.into_any()
        >
            {move || {
                s.order.with(|a| {
                    let a = a.as_ref()?;
                    let verdict = order_verdict(&s.tasks.get(), &s.outlooks.get());
                    Some(view! {
                        {a.briefing.clone().map(|b| view! { <p class="mo-briefing">{b}</p> })}
                        {a.description.clone().filter(|_| !compact).map(|d| view! { <p class="mo-desc">{d}</p> })}
                        {verdict.map(|(text, tone)| view! { <p class="mo-verdict-line" style=format!("color: {tone};")>{text}</p> })}
                    })
                })
            }}
            <div class="mo-sub caps">"Order overview"</div>
            <ul class="mo-tasks">
                <For each=move || slots.get() key=|k| *k let:slot>
                    {if slot.1 { view! { <PlanetTask s=s i=slot.0 /> }.into_any() } else { view! { <CountTaskRow s=s i=slot.0 /> }.into_any() }}
                </For>
            </ul>
            {move || {
                s.order.with(|a| {
                    let a = a.as_ref()?;
                    let reward = a.rewards.first().cloned().or_else(|| a.reward.clone())?;
                    Some(view! {
                        <div class="mo-reward">
                            <Icon icon=icons::AWARD size=20.0 />
                            <span class="caps">"Reward " <b>{reward_label(reward.kind)}</b></span>
                            <b class="num mo-amount">{crate::js::num(reward.amount)}</b>
                        </div>
                    })
                })
            }}
        </Panel>
    }
}

#[component]
pub fn MajorOrderList(#[prop(optional)] compact: bool) -> impl IntoView {
    let q = queries();
    let ids =
        Memo::new(move |_| q.assignments.with(|l| l.as_ref().map(|l| l.iter().map(|a| a.id).collect::<Vec<_>>())));
    let any = Memo::new(move |_| ids.with(|l| l.as_ref().is_some_and(|l| !l.is_empty())));
    let waiting = Memo::new(move |_| q.raw.loading.get() && ids.with(Option::is_none));
    let single = Signal::derive(move || ids.with(|l| l.as_ref().is_some_and(|l| l.len() == 1)));
    let name =
        Signal::derive(
            move || if single.get() { order_name(q.episodes.get().as_deref().map(|e| e.as_slice())) } else { None },
        );
    move || {
        if waiting.get() {
            return view! { <Loading what="orders" rows=2 /> }.into_any();
        }
        if !any.get() {
            return view! { <Empty>"No active Major Order. Await further instructions."</Empty> }.into_any();
        }
        view! {
            <For each=move || ids.get().unwrap_or_default() key=|id| *id let:id>
                <OrderCard id=id name=name compact=compact />
            </For>
        }
        .into_any()
    }
}
