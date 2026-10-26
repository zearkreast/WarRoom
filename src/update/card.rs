use crate::config::APP_VERSION;
use crate::tauri::is_android;
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::{CtaButton, Meter, MeterSize};
use crate::update::{UpdatePhase, check_for_update, install_update, open_installer, update};
use leptos::prelude::*;
use regex_lite::Regex;
use std::sync::LazyLock;

static LEAD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[>*\-\s]+").unwrap());

/// The first few lines of the release notes, without headings or Markdown marks.
fn summary(notes: &str) -> String {
    notes
        .split('\n')
        .filter(|l| !l.trim_start().starts_with('#'))
        .map(|l| LEAD.replace(l, "").replace("**", "").trim().to_string())
        .filter(|l| !l.is_empty())
        .take(3)
        .collect::<Vec<_>>()
        .join(" · ")
}

/// A newer release, offered at the top of the toast column until it is installed or put off.
#[component]
pub fn UpdateCard() -> impl IntoView {
    let u = update();
    move || {
        if u.dismissed.get() {
            return None;
        }
        let state = u.state.get();
        let version = match &state {
            UpdatePhase::Available { version, .. }
            | UpdatePhase::Downloading { version, .. }
            | UpdatePhase::Downloaded { version }
            | UpdatePhase::Installing { version }
            | UpdatePhase::HandedOff { version } => Some(version.clone()),
            UpdatePhase::Error { version, .. } => version.clone(),
            _ => return None,
        };
        let busy = matches!(state, UpdatePhase::Downloading { .. } | UpdatePhase::Installing { .. });
        let error = matches!(state, UpdatePhase::Error { .. });
        let body = match state {
            UpdatePhase::Available { notes, .. } => view! {
                {(!notes.is_empty()).then(|| view! { <p class="uc-notes">{summary(&notes)}</p> })}
                <div class="uc-actions">
                    <CtaButton icon=icons::DOWNLOAD on_click=Callback::new(|_| install_update())>
                        {if is_android() { "Download APK" } else { "Download and install" }}
                    </CtaButton>
                </div>
            }
            .into_any(),
            UpdatePhase::Downloading { progress, .. } => view! {
                <div class="uc-progress">
                    <Meter value=progress.unwrap_or(0.0) * 100.0 color="var(--cta)".to_string() size=MeterSize::M />
                    <span class="num caps">
                        {match progress {
                            None => "Downloading…".to_string(),
                            Some(p) => format!("Downloading {}%", (p * 100.0).round()),
                        }}
                    </span>
                </div>
            }
            .into_any(),
            UpdatePhase::Downloaded { .. } => view! {
                <p class="uc-notes">
                    "Downloaded. Tap Install in Android's installer; the first time, Android asks you to allow installs from WarCommand."
                </p>
                <div class="uc-actions">
                    <CtaButton icon=icons::PACKAGE_CHECK on_click=Callback::new(|_| open_installer())>
                        "Install"
                    </CtaButton>
                </div>
            }
            .into_any(),
            UpdatePhase::Installing { .. } => {
                view! { <p class="uc-notes">"Installing. The app restarts when it is done."</p> }.into_any()
            }
            UpdatePhase::HandedOff { .. } => view! {
                <p class="uc-notes">
                    "The APK is downloading. Open it from your notifications to install; the first time, Android asks you to allow installs from that app."
                </p>
            }
            .into_any(),
            UpdatePhase::Error { version, message } => {
                let retry = version.is_some();
                view! {
                    <p class="uc-notes">{message}</p>
                    <div class="uc-actions">
                        <CtaButton
                            icon=icons::ROTATE_CW
                            on_click=Callback::new(move |_| if retry { install_update() } else { check_for_update(true) })
                        >
                            "Try again"
                        </CtaButton>
                    </div>
                }
                .into_any()
            }
            _ => ().into_any(),
        };
        Some(view! {
            <section class="update-card" aria-live="polite" aria-label="App update">
                <div class="uc-head">
                    <span class="uc-icon">
                        <Icon icon=icons::DOWNLOAD size=16.0 />
                    </span>
                    <div class="uc-title">
                        <b class="caps">{if error { "Update failed" } else { "Update available" }}</b>
                        <span class="num caps">
                            {version.map(|v| format!("v{v} · "))}
                            "you have v"
                            {APP_VERSION}
                        </span>
                    </div>
                    {(!busy).then(|| view! {
                        <button
                            class="toast-x"
                            aria-label="Remind me next launch"
                            title="Later"
                            on:click=move |_| u.dismissed.set(true)
                        >
                            <Icon icon=icons::X size=16.0 />
                        </button>
                    })}
                </div>
                {body}
            </section>
        })
    }
}
