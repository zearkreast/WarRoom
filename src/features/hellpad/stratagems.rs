//! The stratagem collection: every stratagem by permit and type, with its code, timings and unlock.

use crate::features::hellpad::gww::drill::open_drill;
use crate::features::map::stratagem_art::{code_view, icon_of, permit_color};
use crate::state::settings;
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::Empty;
use leptos::prelude::*;
use warroom_core::drill::{Mode, best};
use warroom_core::format::fmt_int;
use warroom_core::stratagems::{KINDS, Kind, PERMITS, Permit, STRATAGEMS, Stratagem, Unlock, Uses};

fn seconds(s: f32) -> String {
    let n = (s * 100.0).round() / 100.0;
    format!("{n} s")
}

fn unlock_text(s: &Stratagem) -> String {
    match s.unlock {
        Unlock::Requisition { module, level, slips: 0 } => format!("Free from level {level} · {module}"),
        Unlock::Requisition { module, level, slips } => {
            format!("{} Requisition Slips · level {level} · {module}", fmt_int(slips as f64))
        }
        Unlock::Warbond { warbond, page, medals } => format!("{warbond} warbond · page {page} · {medals} Medals"),
        Unlock::Reward(what) => format!("Reward: {what}"),
        Unlock::Issued if s.kind == Kind::Ship => "Standard issue on every mission".into(),
        Unlock::Issued => "Issued for mission objectives".into(),
        Unlock::Unreleased => "Not yet released".into(),
    }
}

fn unlock_icon(u: Unlock) -> icons::IconData {
    match u {
        Unlock::Requisition { .. } => icons::LOCK,
        Unlock::Warbond { .. } => icons::MEDAL,
        Unlock::Reward(_) => icons::GIFT,
        Unlock::Issued => icons::FLAG,
        Unlock::Unreleased => icons::CALENDAR_CLOCK,
    }
}

fn matches(s: &Stratagem, q: &str) -> bool {
    if q.is_empty() {
        return true;
    }
    let mut hay = vec![
        s.name.to_string(),
        s.kind.label().into(),
        s.permit().label().into(),
        s.permit().colour().into(),
        unlock_text(s),
    ];
    hay.extend(s.traits.iter().map(|t| t.to_string()));
    hay.iter().any(|h| h.to_lowercase().contains(q))
}

fn stat(label: &'static str, value: Option<String>) -> impl IntoView {
    value.map(|v| {
        view! {
            <div>
                <dt>{label}</dt>
                <dd>{v}</dd>
            </div>
        }
    })
}

fn card(s: &'static Stratagem, i: usize) -> impl IntoView {
    let picked = Memo::new(move |_| settings().get(|d| d.drill_picks.iter().any(|p| p == s.id)));
    let cooldown = s.cooldown.map(|c| if c > 0.0 { seconds(c) } else { "None".into() });
    let uses = s.uses.map(|u| match u {
        Uses::Unlimited => "Unlimited".to_string(),
        Uses::Times(n) => n.to_string(),
    });
    view! {
        <li class="sg-card" style=format!("--p: {}; --i: {i};", permit_color(s.permit()))>
            <span class="sg-icon">{icon_of(s).map(|src| view! { <img src=src alt="" width="56" height="56" loading="lazy" /> })}</span>
            <div class="sg-body">
                <h3 class="sg-title">
                    <b class="sg-name">{s.short_name()}</b>
                    {s.designation().map(|d| view! { <span class="sg-desig num">{d}</span> })}
                </h3>
                {code_view(s.code)}
                <dl class="sg-stats num">
                    {stat("Cooldown", cooldown)}
                    {stat("Call-in", s.call_in.map(seconds))}
                    {stat("Uses", uses)}
                </dl>
                <p class="sg-unlock">
                    <Icon icon=unlock_icon(s.unlock) size=13.0 />
                    {unlock_text(s)}
                </p>
                {(!s.traits.is_empty()).then(|| view! {
                    <ul class="sg-traits" aria-label="Traits">
                        {s.traits.iter().map(|t| view! { <li class="caps">{*t}</li> }).collect_view()}
                    </ul>
                })}
            </div>
            <button
                class=move || format!("sg-pick {}", if picked.get() { "on" } else { "" })
                aria-pressed=move || picked.get().to_string()
                aria-label=format!("{} in your drill loadout", s.short_name())
                title=move || if picked.get() { "In your drill loadout" } else { "Add to your drill loadout" }
                on:click=move |_| settings().toggle_drill_pick(s.id)
            >
                <Icon icon=icons::CROSSHAIR size=16.0 />
            </button>
        </li>
    }
}

/// The way to the G.W.W.'s stratagem drill, with the best classic score and the loadout picked here.
#[component]
fn DrillBanner() -> impl IntoView {
    let top = Memo::new(move |_| settings().get(|d| best(&d.drill_scores, Mode::Classic).map(|s| s.score)));
    let picks = Memo::new(move |_| settings().get(|d| d.drill_picks.len()));
    view! {
        <button class="sg-drill" on:click=move |_| open_drill()>
            <span class="sg-drill-keys" aria-hidden="true">
                <i class="hp-key on up"></i>
                <i class="hp-key on right"></i>
                <i class="hp-key on down"></i>
                <i class="hp-key on left"></i>
            </span>
            <span class="sg-drill-text">
                <b class="caps">"Stratagem drill"</b>
                <span>
                    "Key these codes against the fuse on the G.W.W. "
                    {move || match picks.get() {
                        0 => "Mark stratagems with the crosshair to drill your own loadout.".to_string(),
                        1 => "1 stratagem in your loadout.".to_string(),
                        n => format!("{n} stratagems in your loadout."),
                    }}
                </span>
            </span>
            {move || top.get().map(|s| view! {
                <span class="sg-drill-best num">
                    <Icon icon=icons::TROPHY size=14.0 />
                    {format!(" {}", fmt_int(s as f64))}
                </span>
            })}
            <Icon icon=icons::CHEVRON_RIGHT size=16.0 />
        </button>
    }
}

#[component]
pub fn StratagemsModule() -> impl IntoView {
    let query = RwSignal::new(String::new());
    let permit = RwSignal::new(None::<Permit>);
    let q = Memo::new(move |_| query.with(|s| s.trim().to_lowercase()));
    let shown = Memo::new(move |_| {
        let p = permit.get();
        q.with(|q| STRATAGEMS.iter().filter(|s| p.is_none_or(|p| s.permit() == p) && matches(s, q)).collect::<Vec<_>>())
    });
    let count = move |p: Option<Permit>| {
        q.with(|q| STRATAGEMS.iter().filter(|s| p.is_none_or(|p| s.permit() == p) && matches(s, q)).count())
    };
    let chip = move |p: Option<Permit>| {
        let label = p.map_or("All", |p| p.label());
        view! {
            <button
                class=move || format!("filter-chip {}", if permit.get() == p { "on" } else { "" })
                aria-pressed=move || (permit.get() == p).to_string()
                on:click=move |_| permit.set(p)
            >
                {p.map(|p| view! { <i class="sg-swatch" style=format!("--p: {};", permit_color(p))></i> })}
                {label}
                " "
                <span class="count">{move || count(p)}</span>
            </button>
        }
    };
    let group = move |k: Kind| {
        let list: Vec<&'static Stratagem> = shown.with(|s| s.iter().copied().filter(|s| s.kind == k).collect());
        (!list.is_empty()).then(|| {
            let n = list.len();
            view! {
                <section class="sg-group" style=format!("--p: {};", permit_color(k.permit()))>
                    <header class="sg-group-head">
                        <h2>{k.label()}</h2>
                        <span class="sg-group-rule"></span>
                        <span class="sg-group-meta caps num">{format!("{} · {n}", k.permit().label())}</span>
                    </header>
                    <ul class="sg-grid">{list.into_iter().enumerate().map(|(i, s)| card(s, i)).collect_view()}</ul>
                </section>
            }
        })
    };
    view! {
        <div class="sg">
            <div class="page-head">
                <h1 class="page-title">
                    "Stratagems "
                    <span class="pi-count num">{STRATAGEMS.len()}</span>
                </h1>
            </div>
            <DrillBanner />
            <div class="pi-controls">
                <label class="pi-search">
                    <Icon icon=icons::SEARCH size=16.0 />
                    <input
                        type="search"
                        prop:value=move || query.get()
                        on:input=move |e| query.set(event_target_value(&e))
                        placeholder="Search by name, trait, warbond or ship module"
                        aria-label="Search stratagems"
                    />
                </label>
            </div>
            <div class="filter-row" role="toolbar" aria-label="Filter by permit">
                {chip(None)}
                {PERMITS.map(|p| chip(Some(p))).collect_view()}
            </div>
            <Show when=move || shown.with(Vec::is_empty)>
                <Empty>"No stratagem matches “"{move || query.get()}"”."</Empty>
            </Show>
            {move || KINDS.map(group).collect_view()}
        </div>
    }
}
