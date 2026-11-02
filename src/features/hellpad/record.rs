//! The Hellpad's Service record: the player's own dives and how each campaign they joined turned out.

use crate::api::fronts::fronts;
use crate::api::history::changes_on;
use crate::js::fmt_date;
use crate::state::{OptionsAnchor, settings, ui};
use crate::ui::emblems::FactionEmblem;
use crate::ui::icons;
use crate::ui::primitives::{Empty, Loading, Panel, Tile};
use leptos::prelude::*;
use leptos::task::spawn_local;
use warroom_core::dive_log::{DiveGroup, DiveOutcome, Fight, dive_outcome, group_dives};
use warroom_core::factions::{faction_color, faction_label};
use warroom_core::owner_changes::OwnerChange;

fn outcome_tone(o: DiveOutcome) -> &'static str {
    match o {
        DiveOutcome::Liberated(_) | DiveOutcome::Held => "var(--good)",
        DiveOutcome::Lost(_) => "var(--bad)",
        DiveOutcome::Withdrawn => "var(--warn)",
        DiveOutcome::Ongoing => "var(--cta)",
    }
}

fn span(g: &DiveGroup) -> String {
    let (first, last) = (fmt_date(g.first), fmt_date(g.last));
    if first == last { first } else { format!("{first} – {last}") }
}

fn group_row(g: DiveGroup) -> impl IntoView {
    let planet = g.planet;
    let outcome = match g.outcome {
        DiveOutcome::Liberated(at) | DiveOutcome::Lost(at) => format!("{} {}", g.outcome.label(), fmt_date(at)),
        o => o.label().to_string(),
    };
    view! {
        <li class="sr-row" style=format!("--k: {};", faction_color(Some(g.enemy)))>
            <FactionEmblem faction=g.enemy.as_str() size=18.0 />
            <div class="sr-what">
                <button class="link sr-name" on:click=move |e| ui().open_options(Some(OptionsAnchor::at(planet, &e)))>
                    {g.name.clone()}
                </button>
                <span class="sr-sub caps">
                    {format!(
                        "{} against the {} · dived {}× · {}",
                        if g.defense { "Defense" } else { "Liberation" },
                        faction_label(Some(g.enemy)),
                        g.count,
                        span(&g)
                    )}
                    {g.major_order.then_some(" · MO target")}
                </span>
            </div>
            <span class="sr-outcome caps" style=format!("--tone: {};", outcome_tone(g.outcome))>{outcome}</span>
        </li>
    }
}

#[component]
pub fn ServiceRecordModule() -> impl IntoView {
    let st = settings();
    let dives = Memo::new(move |_| st.get(|s| s.dive_log.clone()));
    let changes = RwSignal::new(None::<Result<Vec<OwnerChange>, String>>);
    Effect::new(move |_| {
        let list = dives.get();
        let mut planets: Vec<i64> = list.iter().map(|d| d.planet).collect();
        planets.sort_unstable();
        planets.dedup();
        let Some(since) = list.iter().map(|d| d.at).min_by(f64::total_cmp) else {
            changes.set(Some(Ok(Vec::new())));
            return;
        };
        spawn_local(async move {
            let got = changes_on(&planets, since).await.map(|l| l.into_iter().map(|c| c.change).collect());
            changes.set(Some(got));
        });
    });
    let all = fronts();
    let groups = Memo::new(move |_| {
        let changes = changes.get()?.ok()?;
        let fight = |planet: i64| -> Fight {
            all.with(|l| l.iter().find(|f| f.planet.index == planet).map(|f| f.planet.event.as_ref().map(|e| e.id)))
        };
        let judged: Vec<_> =
            dives.with(|l| l.iter().map(|d| (d.clone(), dive_outcome(d, &changes, fight(d.planet)))).collect());
        Some(group_dives(&judged))
    });
    let confirm = RwSignal::new(false);
    let right = move || {
        let n = dives.with(Vec::len);
        (n > 0).then(|| {
            view! {
                <button
                    class=move || format!("hud-btn sr-clear {}", if confirm.get() { "on" } else { "" })
                    on:click=move |_| {
                        if confirm.get_untracked() {
                            st.set(|s| s.dive_log.clear());
                            confirm.set(false);
                        } else {
                            confirm.set(true);
                        }
                    }
                >
                    {move || if confirm.get() { format!("Clear all {n} dives?") } else { "Clear the record".to_string() }}
                </button>
            }
        })
    };
    view! {
        <Panel title="Service record" icon=icons::MEDAL right=view! { {right} }.into_any()>
            {move || {
                if dives.with(Vec::is_empty) {
                    return view! {
                        <Empty>"No dives logged yet. Open a planet with a campaign and press Log a dive after each mission there."</Empty>
                    }
                    .into_any();
                }
                match changes.get() {
                    None => return view! { <Loading what="your service record" rows=3 /> }.into_any(),
                    Some(Err(_)) => {
                        return view! { <Empty>"Could not reach the war archive, so the outcomes can't be judged yet."</Empty> }.into_any();
                    }
                    Some(Ok(_)) => {}
                }
                let list = groups.get().unwrap_or_default();
                let count = |f: fn(&DiveOutcome) -> bool| list.iter().filter(|g| f(&g.outcome)).count();
                let liberated = count(|o| matches!(o, DiveOutcome::Liberated(_)));
                let held = count(|o| *o == DiveOutcome::Held);
                let lost = count(|o| matches!(o, DiveOutcome::Lost(_)));
                let ongoing = count(|o| *o == DiveOutcome::Ongoing);
                view! {
                    <div class="sr">
                        <div class="sr-tiles">
                            <Tile icon=icons::FLAG label="Dives" value=dives.with(Vec::len).into_any() sub=format!("{} campaigns, {ongoing} going on", list.len()).into_any() rule="var(--cta)" />
                            <Tile icon=icons::TROPHY label="Liberated" value=liberated.into_any() color="var(--good)" rule="var(--good)" />
                            <Tile icon=icons::SHIELD_CHECK label="Defenses held" value=held.into_any() color="var(--good)" rule="var(--good)" />
                            <Tile icon=icons::SKULL label="Lost" value=lost.into_any() color="var(--bad)" rule="var(--bad)" />
                        </div>
                        <ol class="sr-list">{list.into_iter().map(group_row).collect_view()}</ol>
                        <p class="sr-note caps">"Kept on this device only. Outcomes come from each planet's changes of hands since your dive."</p>
                    </div>
                }
                .into_any()
            }}
        </Panel>
    }
}
