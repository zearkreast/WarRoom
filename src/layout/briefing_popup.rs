use crate::alerts::{Toast, ToastKind, toasts};
use crate::features::news::card::MarkupText;
use crate::js::num;
use crate::ui::emblems::{FactionEmblem, SealArt, SealGradient};
use crate::ui::please_wait::{PLEASE_WAIT_MS, PleaseWait};
use crate::ui::primitives::KeyButton;
use leptos::html;
use leptos::prelude::*;
use warroom_core::dispatch::{DispatchKind, dispatch_kind};
use warroom_core::format::dispatch_title;

fn verdict(k: DispatchKind) -> (&'static str, &'static str) {
    match k {
        DispatchKind::Victory => ("Victory", "var(--good)"),
        DispatchKind::Defeat => ("Failure", "var(--bad)"),
        DispatchKind::Order => ("Major order", "var(--cta)"),
        DispatchKind::Alert => ("Alert", "var(--warn)"),
        DispatchKind::Advisory => ("Advisory", "var(--info)"),
        DispatchKind::News => ("Dispatch", "var(--info)"),
    }
}

fn verdict_of(t: &Toast) -> DispatchKind {
    match t.kind {
        ToastKind::Capture => DispatchKind::Victory,
        ToastKind::Loss => DispatchKind::Defeat,
        ToastKind::Defense => DispatchKind::Alert,
        ToastKind::Order => DispatchKind::Order,
        ToastKind::Dispatch => dispatch_kind(t.message.as_deref().unwrap_or(&t.body)),
    }
}

const CLOTH: &str = "M74 14C210 -4 330 40 470 22s130 6 150 14V206c-40-14-90-24-160-8S200 214 74 196Z";

/// A flag on a pole against a smoky sky, drawn for the app, bearing the game's Super Earth emblem.
#[component]
fn Banner() -> impl IntoView {
    let embers = (0..14).map(|i| ((i * 47 + 13) % 640, 3 + (i % 5), 1.0 + (i % 3) as f64 * 0.6));
    view! {
        <svg class="bp-banner" viewBox="0 0 640 240" preserveAspectRatio="xMidYMid slice" aria-hidden="true">
            <defs>
                <linearGradient id="bp-sky" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stop-color="#0a0b10" />
                    <stop offset="55%" stop-color="#2a1a14" />
                    <stop offset="100%" style="stop-color: color-mix(in srgb, var(--bp-tone) 28%, #3a2418);" />
                </linearGradient>
                <linearGradient id="bp-cloth" x1="0" y1="0" x2="1" y2="0">
                    <stop offset="0%" stop-color="#15224a" />
                    <stop offset="22%" stop-color="#1f3264" />
                    <stop offset="40%" stop-color="#121c3e" />
                    <stop offset="62%" stop-color="#22376b" />
                    <stop offset="82%" stop-color="#111a3a" />
                    <stop offset="100%" stop-color="#1b2c5a" />
                </linearGradient>
                <SealGradient id="bp-seal" />
                <filter id="bp-smoke" x="-20%" y="-50%" width="140%" height="200%">
                    <feGaussianBlur stdDeviation="18" />
                </filter>
            </defs>
            <rect width="640" height="240" fill="url(#bp-sky)" />
            <g filter="url(#bp-smoke)" opacity="0.55">
                <ellipse cx="520" cy="210" rx="160" ry="40" fill="#5a3a2a" />
                <ellipse cx="150" cy="230" rx="180" ry="30" fill="#4a2c22" />
                <ellipse cx="380" cy="60" rx="140" ry="26" fill="#1a1416" />
            </g>
            <rect x="66" y="0" width="8" height="240" fill="#c9c2b4" />
            <g class="bp-flag">
                <path d=CLOTH fill="url(#bp-cloth)" />
                <g transform="translate(345 113)">
                    <SealArt size=158.0 gradient="bp-seal" />
                </g>
            </g>
            {embers
                .enumerate()
                .map(|(i, (x, d, r))| {
                    view! {
                        <circle
                            class="bp-ember"
                            cx=x.to_string()
                            cy="240"
                            r=num(r)
                            style=format!("animation-duration: {d}s; animation-delay: {}s;", num(-(i as f64) * 0.7))
                        />
                    }
                })
                .collect_view()}
        </svg>
    }
}

/// Opens from a toast: a short "please wait democratically" hold, then the full briefing.
#[component]
pub fn BriefingPopup() -> impl IntoView {
    let t = toasts();
    let dialog = NodeRef::<html::Dialog>::new();
    let (ready, set_ready) = signal(false);
    let timer = StoredValue::new_local(None::<TimeoutHandle>);
    Effect::new(move |_| {
        let briefing = t.briefing.get();
        let Some(d) = dialog.get() else { return };
        if briefing.is_some() && !d.open() {
            let _ = d.show_modal();
        }
        if briefing.is_none() && d.open() {
            d.close();
        }
        set_ready.set(false);
        if let Some(h) = timer.get_value() {
            h.clear();
        }
        if briefing.is_some() {
            let h = set_timeout_with_handle(
                move || set_ready.set(true),
                std::time::Duration::from_millis(PLEASE_WAIT_MS as u64),
            )
            .ok();
            timer.set_value(h);
        }
    });
    let verdict_now = move || t.briefing.with(|b| b.as_ref().map(|b| verdict(verdict_of(b))));
    let close = move || t.close_briefing();
    view! {
        <dialog
            node_ref=dialog
            class=move || format!("briefing {}", if ready.get() { "" } else { "waiting" })
            aria-label=move || verdict_now().map_or("Briefing", |v| v.0)
            on:close=move |_| close()
            on:click=move |e| {
                if e.target() == e.current_target() {
                    close()
                }
            }
            style=move || verdict_now().map(|v| format!("--bp-tone: {};", v.1))
        >
            {move || {
                let b = t.briefing.get()?;
                let (word, _) = verdict(verdict_of(&b));
                if !ready.get() {
                    return Some(view! { <PleaseWait /> }.into_any());
                }
                let is_dispatch = b.kind == ToastKind::Dispatch && b.message.is_some();
                let heading = if is_dispatch { dispatch_title(b.message.as_deref().unwrap_or_default()) } else { b.title.clone() };
                let text = b.message.clone().unwrap_or_else(|| b.body.clone());
                Some(
                    view! {
                        <article class="bp-card">
                            <Banner />
                            <header class="bp-head">
                                <FactionEmblem faction="Humans" size=30.0 color="var(--text-hi)" />
                                <h2>{word}</h2>
                            </header>
                            <div class="bp-body">
                                <h3>{heading}</h3>
                                <MarkupText message=text class="bp-text" whole=!is_dispatch />
                            </div>
                        </article>
                        <div class="bp-foot">
                            <KeyButton k="ESC" on_click=Callback::new(move |_| close())>
                                "Continue"
                            </KeyButton>
                        </div>
                    }
                        .into_any(),
                )
            }}
        </dialog>
    }
}
