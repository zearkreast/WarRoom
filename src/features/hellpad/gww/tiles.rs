//! The G.W.W. home page's tiles, and the training course's progress.

use crate::features::hellpad::gww::nav::{icon_url, use_gww_nav};
use crate::features::hellpad::gww::pictures::{company_mark, manual_icon};
use crate::state::settings;
use crate::ui::icons::{self, Icon};
use leptos::prelude::*;
use warroom_core::data::manuals::MANUALS;
use warroom_core::data::sites::{GOV, MINISTRIES};
use warroom_core::data::{Company, Manual, Ministry};
use warroom_core::drill::{Mode, best};
use warroom_core::format::fmt_int;
use warroom_core::gww_nav::{DRILL_ADDRESS, DRILL_TITLE, LOG_ADDRESS, LOG_TITLE};

#[allow(clippy::too_many_arguments)]
pub fn site_tile(
    target: String,
    address: &'static str,
    title: String,
    text: String,
    icon: AnyView,
    color: Option<String>,
    done: Signal<bool>,
    out: bool,
) -> AnyView {
    let nav = use_gww_nav();
    view! {
        <button
            class=move || format!("gww-site {}", if done.get() { "done" } else { "" })
            style=color.map(|c| format!("--site: {c};"))
            on:click=move |e| nav.follow(&target, Some(&e))
        >
            <span class="gww-site-icon">{icon}</span>
            <span class="gww-site-body">
                <span class="gww-address num">
                    {address}
                    {out.then(|| view! {
                        <Icon icon=icons::ARROW_UP_RIGHT size=11.0 />
                        <span class="sr-only">" (leaves the G.W.W.)"</span>
                    })}
                </span>
                <b class="gww-site-title">{title}</b>
                <span class="gww-site-text">{text}</span>
            </span>
            <Show when=move || done.get()>
                <span class="gww-site-done" title="Read">
                    <Icon icon=icons::CHECK size=12.0 label="Read" />
                </span>
            </Show>
        </button>
    }
    .into_any()
}

pub fn company_tile(c: &'static Company) -> AnyView {
    let nav = use_gww_nav();
    let target = format!("company:{}", c.id);
    view! {
        <button class="gww-site corp" style=format!("--site: {};", c.color) on:click=move |e| nav.follow(&target, Some(&e))>
            <span class="gww-corp-logo">{company_mark(c)}</span>
            <span class="gww-site-body">
                <span class="gww-address num">{c.address}</span>
                <b class="gww-site-title">{c.name}</b>
                <span class="gww-site-text">{c.slogan.unwrap_or(c.summary)}</span>
            </span>
        </button>
    }
    .into_any()
}

pub fn manual_target(m: &Manual) -> String {
    format!("manual:{}", m.id)
}

pub fn manual_tile(m: &'static Manual) -> AnyView {
    let id = m.id;
    let read = Signal::derive(move || settings().get(|s| s.gww_read.iter().any(|r| r == id)));
    site_tile(
        manual_target(m),
        m.address,
        m.title.into(),
        m.summary.into(),
        manual_icon(m, 30.0),
        Some("var(--cta)".into()),
        read,
        false,
    )
}

pub fn ministry_tile(m: &'static Ministry) -> AnyView {
    site_tile(
        format!("ministry:{}", m.id),
        m.address,
        m.name.into(),
        m.summary.into(),
        view! { <img src=icon_url(m.icon) alt="" /> }.into_any(),
        Some(m.color.into()),
        Signal::stored(false),
        false,
    )
}

pub(super) fn humanity() -> &'static str {
    MINISTRIES.iter().find(|m| m.id == "humanity").map_or("", |m| m.color)
}

pub fn gov_tile() -> AnyView {
    site_tile(
        "site:gov".into(),
        GOV.address,
        GOV.title.into(),
        GOV.summary.into(),
        view! { <img src=icon_url(GOV.icon) alt="" /> }.into_any(),
        Some("var(--f-humans)".into()),
        Signal::stored(false),
        false,
    )
}

pub fn log_tile(log_text: Signal<String>) -> AnyView {
    (move || {
        site_tile(
            "site:log".into(),
            LOG_ADDRESS,
            LOG_TITLE.into(),
            log_text.get(),
            view! { <Icon icon=icons::HISTORY size=26.0 stroke_width=1.5 /> }.into_any(),
            Some("var(--f-humans)".into()),
            Signal::stored(false),
            false,
        )
    })
    .into_any()
}

pub fn news_tile() -> AnyView {
    site_tile(
        "app:news".into(),
        "se://strohmann.news",
        "Strohmann News".into(),
        "Dispatches and Galactic Campaign news, as they come in.".into(),
        view! { <Icon icon=icons::NEWSPAPER size=26.0 stroke_width=1.5 /> }.into_any(),
        Some("#f074d8".into()),
        Signal::stored(false),
        true,
    )
}

#[component]
pub fn CourseProgress(#[prop(optional)] compact: bool) -> impl IntoView {
    let nav = use_gww_nav();
    let read = Memo::new(move |_| settings().get(|s| s.gww_read.clone()));
    let is_read = move |id: &str| read.with(|r| r.iter().any(|x| x == id));
    let done = Memo::new(move |_| MANUALS.iter().filter(|m| is_read(m.id)).count());
    let next = Memo::new(move |_| MANUALS.iter().position(|m| !is_read(m.id)));
    view! {
        <section class=format!("gww-course {}", if compact { "compact" } else { "" }) aria-label="Recruit's course">
            <div class="gww-course-head">
                <Icon icon=icons::GRADUATION_CAP size=20.0 />
                <div>
                    <b class="caps">"Recruit's course"</b>
                    <span class="num">{move || done.get()}" of "{MANUALS.len()}" training manuals read"</span>
                </div>
                {move || match next.get() {
                    Some(i) => {
                        let m = &MANUALS[i];
                        let target = manual_target(m);
                        view! {
                            <button class="gww-course-next" on:click=move |e| nav.follow(&target, Some(&e))>
                                <span class="caps">{if done.get() == 0 { "Start with" } else { "Continue with" }}</span>
                                <b>{m.title}</b>
                                <Icon icon=icons::ARROW_RIGHT size=14.0 />
                            </button>
                        }
                        .into_any()
                    }
                    None => view! { <span class="gww-course-done caps">"Course complete. Democracy thanks you."</span> }.into_any(),
                }}
            </div>
            <ol class="gww-course-bar" aria-label="Manuals, in reading order">
                {MANUALS
                    .iter()
                    .map(|m| {
                        let target = manual_target(m);
                        view! {
                            <li class=move || if is_read(m.id) { "on" } else { "" }>
                                <button
                                    title=m.title
                                    aria-label=move || format!("{}{}", m.title, if is_read(m.id) { ", read" } else { "" })
                                    on:click=move |e| nav.follow(&target, Some(&e))
                                ></button>
                            </li>
                        }
                    })
                    .collect_view()}
            </ol>
            {(!compact).then(|| view! {
                <Show when=move || { done.get() > 0 }>
                    <button class="link gww-course-reset caps" on:click=move |_| settings().clear_read()>
                        "Start the course over"
                    </button>
                </Show>
            })}
        </section>
    }
}

pub fn drill_tile() -> AnyView {
    let top = Memo::new(move |_| settings().get(|d| best(&d.drill_scores, Mode::Classic).map(|s| s.score)));
    let tile = move || {
        let mut text =
            "Key stratagem codes against the fuse: every stratagem, your own loadout, or by permit colour.".to_string();
        if let Some(s) = top.get() {
            text.push_str(&format!(" Best classic score: {}.", fmt_int(s as f64)));
        }
        site_tile(
            "site:drill".into(),
            DRILL_ADDRESS,
            DRILL_TITLE.into(),
            text,
            view! { <Icon icon=icons::GAMEPAD_2 size=28.0 stroke_width=1.5 /> }.into_any(),
            Some("var(--cta)".into()),
            Signal::stored(false),
            false,
        )
    };
    tile.into_any()
}
