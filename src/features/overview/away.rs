//! "While you were away": the war's changes since the player last looked, at the top of High Command.

use crate::api::history::changes_since;
use crate::api::queries::queries;
use crate::api::war_samples::use_war_days;
use crate::js;
use crate::state::{View, ui};
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::Panel;
use leptos::prelude::*;
use leptos::task::spawn_local;
use warroom_core::away::{Captures, captures_since, name_list, order_moves};
use warroom_core::format::{fmt_compact, fmt_pct, fmt_time_left};
use warroom_core::major_order::{describe_assignment, task_name, task_percent};
use warroom_core::snapshot::EpisodeStatus;
use warroom_core::time::parse_ms;

fn line(figure: String, tone: &'static str, text: String) -> AnyView {
    view! {
        <li>
            <b class="num" style=format!("color: {tone};")>{figure}</b>
            <span>{text}</span>
        </li>
    }
    .into_any()
}

#[component]
fn Briefing(since: f64) -> impl IntoView {
    let q = queries();
    let captures = RwSignal::new(None::<Captures>);
    spawn_local(async move {
        if let Ok(list) = changes_since(since).await {
            let list: Vec<_> = list.into_iter().map(|c| c.change).collect();
            captures.set(Some(captures_since(&list, since)));
        }
    });
    let days = use_war_days();
    let away = js::now() - since;
    let lines = move || {
        let mut out: Vec<AnyView> = Vec::new();
        if let Some(c) = captures.get() {
            if !c.liberated.is_empty() {
                let n = c.liberated.len();
                let what = format!("planet{} liberated: {}", if n == 1 { "" } else { "s" }, name_list(&c.liberated, 4));
                out.push(line(n.to_string(), "var(--good)", what));
            }
            if !c.lost.is_empty() {
                let n = c.lost.len();
                out.push(line(
                    n.to_string(),
                    "var(--bad)",
                    format!("planet{} lost: {}", if n == 1 { "" } else { "s" }, name_list(&c.lost, 4)),
                ));
            }
        }
        let planets = q.fresh_planets.get();
        let samples = q.order_samples.get();
        for a in q.assignments.get().iter().flat_map(|l| l.iter()) {
            let tasks = describe_assignment(a, planets.as_deref().map(|p| p.as_slice()));
            let samples = samples.as_deref().map_or(&[][..], |s| s.as_slice());
            let moves = order_moves(samples, a.id, &a.progress, since);
            // Order readings reach back a day, so a longer break is measured over less.
            let oldest = samples.iter().filter(|s| s.id == a.id).map(|s| s.at).min_by(f64::total_cmp);
            let over = oldest
                .filter(|o| *o > since + 3_600_000.0)
                .map(|o| format!(" (last {})", fmt_time_left(js::now() - o)));
            for (t, moved) in tasks.iter().zip(moves) {
                let Some(moved) = moved.filter(|m| *m != 0.0) else { continue };
                let (figure, text) = if t.is_planet {
                    ("Done".to_string(), format!("Major Order · {} {}", t.label, task_name(t)))
                } else {
                    let pct = if t.goal > 0.0 { moved / t.goal * 100.0 } else { 0.0 };
                    let text = format!("Major Order · {} now at {}", task_name(t), fmt_pct(task_percent(t), 1));
                    (format!("+{}", fmt_pct(pct, 1)), format!("{text}{}", over.clone().unwrap_or_default()))
                };
                out.push(line(figure, "var(--cta)", text));
            }
        }
        for e in q.episodes.get().iter().flat_map(|l| l.iter()) {
            if e.start > since {
                out.push(line("New".into(), "var(--info)", format!("Galactic Campaign: {}", e.title)));
            }
            if e.end.is_some_and(|end| end > since) && e.status != EpisodeStatus::Active {
                let won = e.status == EpisodeStatus::Won;
                out.push(line(
                    if won { "Won" } else { "Lost" }.into(),
                    if won { "var(--good)" } else { "var(--bad)" },
                    format!("Galactic Campaign over: {}", e.title),
                ));
            }
        }
        let news =
            q.dispatches.with(|d| d.iter().flat_map(|l| l.iter()).filter(|d| parse_ms(&d.published) > since).count());
        if news > 0 {
            out.push(line(
                news.to_string(),
                "var(--info)",
                format!("new dispatch{} in Strohmann News", if news == 1 { "" } else { "es" }),
            ));
        }
        let day_start = since - since.rem_euclid(86_400_000.0);
        let peak = days.data.with(|d| {
            d.iter()
                .flat_map(|l| l.iter())
                .filter(|d| d.day >= day_start)
                .map(|d| d.peak)
                .fold(None, |m: Option<f64>, p| Some(m.map_or(p, |m| m.max(p))))
        });
        if let Some(p) = peak {
            out.push(line(fmt_compact(p), "var(--text-hi)", "peak Helldivers online".into()));
        }
        out
    };
    view! {
        <Panel
            title="While you were away"
            icon=icons::HISTORY
            class="ov-away"
            right=view! {
                <span class="ov-away-for caps num">{format!("{} away", fmt_time_left(away))}</span>
                <button class="ov-away-x" aria-label="Dismiss the briefing" on:click=move |_| ui().away_since.set(None)>
                    <Icon icon=icons::X size=14.0 />
                </button>
            }
            .into_any()
        >
            {move || {
                let list = lines();
                if list.is_empty() {
                    return view! { <p class="ov-away-quiet">"Nothing changed hands and no orders moved."</p> }.into_any();
                }
                view! { <ul class="ov-away-list">{list}</ul> }.into_any()
            }}
            <button class="link ov-away-news caps" on:click=move |_| ui().set_view(View::News)>
                "Open Strohmann News"
            </button>
        </Panel>
    }
}

#[component]
pub fn AwayBriefing() -> impl IntoView {
    let since = ui().away_since;
    move || since.get().map(|s| view! { <Briefing since=s /> })
}
