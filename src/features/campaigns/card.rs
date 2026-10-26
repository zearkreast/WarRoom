use crate::features::planet::tint::planet_tint;
use crate::features::planets::fronts::{Front, FrontKind, fronts};
use crate::state::{OptionsAnchor, ui};
use crate::ui::emblems::{DssIcon, FactionEmblem};
use crate::ui::hazard_icons::hazard_icon;
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::{Countdown, Meter, MeterSize};
use leptos::prelude::*;
use warroom_core::api::{Faction, Named};
use warroom_core::biomes::regen_pct_per_hour;
use warroom_core::factions::{faction_color, faction_label};
use warroom_core::format::{fmt_int, fmt_pct, to_fixed};

/// What a card shows that rarely changes; the card is rebuilt only when it does.
#[derive(Clone, PartialEq)]
struct Look {
    index: i64,
    name: String,
    sector: String,
    biome: String,
    hazards: Vec<Named>,
    kind: FrontKind,
    enemy: Faction,
    style: String,
}

#[derive(Clone, Copy, PartialEq)]
struct EtaView {
    good: bool,
    label: &'static str,
    to: f64,
    late: bool,
}

/// "Victory in …" / "Falls in …" above the card, if the rate says anything.
fn eta(f: &Front, deadline: Option<f64>) -> Option<EtaView> {
    f.rate?;
    if f.kind == FrontKind::Defense {
        let wins = matches!((f.victory_at, f.fail_at), (Some(v), Some(d)) if v < d);
        let to = if wins { f.victory_at } else { f.fail_at };
        return Some(EtaView {
            good: wins,
            label: if wins { "Defended in" } else { "Falls in" },
            to: to.unwrap_or(0.0),
            late: false,
        });
    }
    if let Some(v) = f.victory_at.filter(|v| *v != 0.0 && !v.is_nan()) {
        return Some(EtaView { good: true, label: "Victory in", to: v, late: deadline.is_some_and(|d| v > d) });
    }
    if let Some(d) = f.fail_at.filter(|d| *d != 0.0 && !d.is_nan()) {
        return Some(EtaView { good: false, label: "Withdrawn in", to: d, late: false });
    }
    None
}

#[derive(Clone, Copy, PartialEq)]
struct OutlookView {
    holding: bool,
    percent: f64,
    defense: bool,
    rate: Option<f64>,
}

fn outlook_view(o: OutlookView) -> AnyView {
    if o.holding {
        return view! {
            <div class="cc-outlook holding caps">
                <Icon icon=icons::USERS size=14.0 />
                " Holding for reinforcement"
            </div>
        }
        .into_any();
    }
    view! {
        <div class="cc-outlook">
            <span class="num cc-pct">
                {fmt_pct(o.percent, 4)} " " <span class="caps">{if o.defense { "defended" } else { "liberated" }}</span>
            </span>
            {match o.rate {
                None => view! { <span class="muted caps">"Measuring rate…"</span> }.into_any(),
                Some(r) => {
                    view! {
                        <span class=format!("num cc-rate {}", if r > 0.0 { "good" } else { "bad" })>
                            <Icon icon=if r < 0.0 { icons::TRENDING_DOWN } else { icons::TRENDING_UP } size=14.0 />
                            {format!(" {}{}% / h", if r > 0.0 { "+" } else { "" }, to_fixed(r, 3))}
                        </span>
                    }
                    .into_any()
                }
            }}
        </div>
    }
    .into_any()
}

#[component]
pub fn CampaignCard(
    id: i64,
    #[prop(into)] total_players: Signal<f64>,
    #[prop(into)] deadline: Signal<Option<f64>>,
) -> impl IntoView {
    let all = fronts();
    let front = Memo::new(move |_| all.with(|l| l.iter().find(|f| f.campaign.id == id).cloned()));
    let look = Memo::new(move |_| {
        front.with(|f| {
            let f = f.as_ref()?;
            let p = &f.planet;
            let tint = planet_tint(p, &[]);
            let style = format!(
                "--enemy: {}; --sky: {}; --ground: {};{}",
                faction_color(Some(f.enemy)),
                tint.sky,
                tint.ground,
                tint.art.map(|a| format!(" --art: {a};")).unwrap_or_default()
            );
            Some(Look {
                index: p.index,
                name: p.name.clone(),
                sector: p.sector.clone(),
                biome: p.biome.name.clone(),
                hazards: p.hazards.iter().take(4).cloned().collect(),
                kind: f.kind,
                enemy: f.enemy,
                style,
            })
        })
    });
    let get = move |pick: fn(&Front) -> f64| move || front.with(|f| f.as_ref().map_or(0.0, pick));
    let percent = Signal::derive(get(|f| f.percent));
    let players = Signal::derive(get(|f| f.players));
    let regen = Signal::derive(get(|f| regen_pct_per_hour(f.planet.regen_per_second, f.planet.max_health)));
    let projected = Signal::derive(move || front.with(|f| f.as_ref().and_then(|f| f.projected)));
    let rate = Memo::new(move |_| front.with(|f| f.as_ref().and_then(|f| f.rate)));
    let flag = move |pick: fn(&Front) -> bool| Memo::new(move |_| front.with(|f| f.as_ref().is_some_and(pick)));
    let (in_mo, pinned, station) = (flag(|f| f.in_major_order), flag(|f| f.pinned), flag(|f| f.station));
    let eta = Memo::new(move |_| front.with(|f| f.as_ref().and_then(|f| eta(f, deadline.get()))));
    let outlook = Memo::new(move |_| {
        front.with(|f| {
            f.as_ref().map(|f| OutlookView {
                holding: f.rate.is_none() && f.kind == FrontKind::Liberation && f.percent == 0.0,
                percent: f.percent,
                defense: f.kind == FrontKind::Defense,
                rate: f.rate,
            })
        })
    });
    let share = Signal::derive(move || {
        let total = total_players.get();
        if total != 0.0 && !total.is_nan() { players.get() / total * 100.0 } else { 0.0 }
    });
    move || {
        let l = look.get()?;
        let enemy = faction_color(Some(l.enemy)).to_string();
        let kind = l.kind.as_str();
        let index = l.index;
        let aria = format!("{}, {kind} against {}. Open options", l.name, faction_label(Some(l.enemy)));
        Some(view! {
            <article class="cc" style=l.style.clone()>
                <div class="cc-above">
                    <span class="cc-type caps">
                        <Icon icon=if l.kind == FrontKind::Defense { icons::SHIELD_ALERT } else { icons::CROSSHAIR } size=13.0 />
                        {format!(" {kind}")}
                    </span>
                    {move || {
                        eta.get().map(|e| {
                            view! {
                                <span class=format!("cc-eta {}", if e.good { "good" } else { "bad" })>
                                    {format!("{} ", e.label)}
                                    <b><Countdown to=e.to /></b>
                                    {e.late.then(|| view! { <span class="cc-late caps">"After MO deadline"</span> })}
                                </span>
                            }
                        })
                    }}
                </div>
                <button
                    class=move || format!("cc-card {}", if in_mo.get() { "mo" } else { "" })
                    on:click=move |e| {
                        ui().open_options(Some(OptionsAnchor { planet: index, x: e.client_x() as f64, y: e.client_y() as f64 }))
                    }
                    aria-label=aria
                >
                    <header class="cc-head">
                        <FactionEmblem faction=l.enemy.as_str() size=24.0 />
                        <div class="cc-title">
                            <h3 class="cc-name">{l.name.clone()}</h3>
                            <div class="cc-sector caps">{format!("{} sector", l.sector)}</div>
                        </div>
                        <Show when=move || pinned.get()>
                            <Icon icon=icons::PIN size=14.0 class="cc-pin" label="Pinned" />
                        </Show>
                    </header>
                    <div class="cc-art">
                        <div class="cc-badges">
                            <Show when=move || in_mo.get()>
                                <span class="cc-badge mo" title="Major Order target">
                                    <Icon icon=icons::TARGET size=15.0 />
                                </span>
                            </Show>
                            <Show when=move || station.get()>
                                <span class="cc-badge dss" title="Democracy Space Station in orbit">
                                    <DssIcon size=15.0 />
                                </span>
                            </Show>
                        </div>
                        <span class="cc-biome caps">{l.biome.clone()}</span>
                        <div class="cc-hazards">
                            {l
                                .hazards
                                .iter()
                                .map(|h| {
                                    view! {
                                        <span class="cc-hazard" title=format!("{}: {}", h.name, h.description)>
                                            <Icon icon=hazard_icon(&h.name) size=13.0 />
                                        </span>
                                    }
                                })
                                .collect_view()}
                        </div>
                    </div>
                    <div class="cc-meter">
                        <Meter
                            value=percent
                            color="var(--f-humans)".to_string()
                            rest=enemy.clone()
                            marker=projected
                            size=MeterSize::L
                            chevron=true
                            label=format!("{} progress", l.name)
                        />
                    </div>
                    {move || outlook.get().map(outlook_view)}
                    <footer class="cc-foot num">
                        <span class="cc-share" title="Share of all Helldivers online">
                            <Meter value=share color="var(--info)".to_string() size=MeterSize::S />
                            {move || format!("{}%", to_fixed(share.get(), 0))}
                        </span>
                        <span title="Helldivers deployed">
                            <Icon icon=icons::USERS size=12.0 />
                            {move || format!(" {}", fmt_int(players.get()))}
                        </span>
                        <span title="Super Earth progress per hour, before enemy regeneration" style="color: var(--f-humans);">
                            <FactionEmblem faction="Humans" size=12.0 />
                            {move || format!(" {}", rate.get().map_or("—".to_string(), |r| format!("{}%", to_fixed(r + regen.get(), 2))))}
                        </span>
                        <span title="Enemy regeneration per hour" style=format!("color: {enemy};")>
                            <FactionEmblem faction=l.enemy.as_str() size=12.0 />
                            {move || format!(" {}%", to_fixed(regen.get(), 2))}
                        </span>
                    </footer>
                </button>
            </article>
        })
    }
}
