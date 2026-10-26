use crate::alerts::{ToastKind, notify};
use crate::api::queries::queries;
use crate::config::APP_VERSION;
use crate::features::map::options::{dispatch_options, overlay_options, planet_options, quality_options};
use crate::js::{self, num};
use crate::state::{AlertSettings, Popups, settings};
use crate::tauri;
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::{Panel, Stepper};
use crate::update::{UpdatePhase, check_for_update, update};
use leptos::prelude::*;
use serde::Serialize;
use warroom_core::campaign_filter::{MIN_PLAYERS, MIN_RATE};
use warroom_core::format::{fmt_int, js_number};
use warroom_core::time::iso_string;

#[component]
fn Row(#[prop(into)] title: String, #[prop(into)] text: String, #[prop(optional)] children: Option<Children>) -> impl IntoView {
    view! {
        <div class="set-row">
            <div class="set-text">
                <b>{title}</b>
                <span>{text}</span>
            </div>
            {children.map(|c| c())}
        </div>
    }
}

fn update_text(p: &UpdatePhase) -> &'static str {
    match p {
        UpdatePhase::Idle => "The app looks for a new release a few seconds after it starts, then every six hours.",
        UpdatePhase::Checking => "Checking for a new release…",
        UpdatePhase::Current => "You have the latest release.",
        UpdatePhase::Available { .. } => "A new release is ready: see the card at the top right.",
        UpdatePhase::Downloading { .. } => "Downloading the new release…",
        UpdatePhase::Installing { .. } => "Installing; the app restarts when it is done.",
        UpdatePhase::HandedOff { .. } => "The APK is downloading; open it from your notifications to install.",
        UpdatePhase::Error { .. } => "The last check failed.",
    }
}

#[component]
fn UpdateRow() -> impl IntoView {
    let title = format!("Version {APP_VERSION}");
    if !tauri::is_tauri() {
        return view! { <Row title=title text="The web version is always the latest; there is nothing to install." /> }.into_any();
    }
    let state = update().state;
    (move || {
        let phase = state.get();
        let checking = phase == UpdatePhase::Checking;
        view! {
            <Row title=title.clone() text=update_text(&phase)>
                <button class="hud-btn" on:click=move |_| check_for_update(true) disabled=checking>
                    <Icon icon=icons::REFRESH_CW size=14.0 />
                    " Check now"
                </button>
            </Row>
        }
    })
    .into_any()
}

#[component]
fn OnOff(#[prop(into)] label: String, #[prop(into)] value: Signal<bool>, on_change: impl Fn(bool) + Clone + Send + Sync + 'static) -> impl IntoView {
    view! {
        <Stepper
            label=label
            options=vec![(true, "On"), (false, "Off")]
            value=value
            on_change=move |v: bool| on_change(v)
        />
    }
}

const ALERTS: [(&str, &str, &str); 5] = [
    ("orders", "Major Orders and campaigns", "When High Command issues a new order, and when a Galactic Campaign starts or ends."),
    ("dispatches", "New dispatches", "Messages from Super Earth."),
    ("captures", "Planets liberated or lost", "Whenever a planet changes hands."),
    ("defenses", "New defenses", "When an enemy attacks a Super Earth planet."),
    ("system", "System notifications", "Also show alerts outside the app, as OS notifications."),
];

fn alert_value(a: &AlertSettings, key: &str) -> bool {
    match key {
        "orders" => a.orders,
        "dispatches" => a.dispatches,
        "captures" => a.captures,
        "defenses" => a.defenses,
        _ => a.system,
    }
}

fn alert_flag<'a>(a: &'a mut AlertSettings, key: &str) -> &'a mut bool {
    match key {
        "orders" => &mut a.orders,
        "dispatches" => &mut a.dispatches,
        "captures" => &mut a.captures,
        "defenses" => &mut a.defenses,
        _ => &mut a.system,
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Diagnostics {
    version: &'static str,
    last_data: Option<String>,
    user_agent: String,
    viewport: String,
    settings: DiagnosticSettings,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DiagnosticSettings {
    popups: Popups,
    map_overlay: crate::state::MapOverlay,
    map_quality: crate::state::MapQuality,
    map_planets: crate::state::MapPlanets,
    show_inactive: bool,
    pinned: usize,
}

#[component]
pub fn SettingsPage() -> impl IntoView {
    let s = settings();
    let last_update = queries().last_update;
    let copied = RwSignal::new(false);
    let pinned = Memo::new(move |_| s.get(|s| s.pinned.len()));

    let copy_diagnostics = move |_| {
        let w = js::window();
        let dim = |v: Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>| v.ok().and_then(|v| v.as_f64()).unwrap_or(0.0);
        let info = s.untracked(|d| Diagnostics {
            version: APP_VERSION,
            last_data: last_update.get_untracked().map(iso_string),
            user_agent: js::user_agent(),
            viewport: format!("{}x{}@{}", num(dim(w.inner_width())), num(dim(w.inner_height())), js_number(w.device_pixel_ratio())),
            settings: DiagnosticSettings {
                popups: d.popups,
                map_overlay: d.map_overlay,
                map_quality: d.map_quality,
                map_planets: d.map_planets,
                show_inactive: d.show_inactive,
                pinned: d.pinned.len(),
            },
        });
        if let Ok(text) = serde_json::to_string_pretty(&info) {
            js::copy_text(&text);
        }
        copied.set(true);
        set_timeout(move || copied.set(false), std::time::Duration::from_millis(2000));
    };

    view! {
        <div class="page scroll starfield">
            <div class="page-inner settings-page">
                <h1 class="page-title">"Settings"</h1>
                <Panel title="Interface" icon=icons::SETTINGS>
                    <Row
                        title="Stalled planets"
                        text=format!(
                            "Fronts under {} Helldivers that gain less than +{}%/h. Major Order targets and pinned planets always show.",
                            fmt_int(MIN_PLAYERS),
                            js_number(MIN_RATE)
                        )
                    >
                        <Stepper
                            label="Stalled planets"
                            options=vec![(false, "Hidden"), (true, "Shown")]
                            value=Signal::derive(move || s.get(|s| s.show_inactive))
                            on_change=move |v| s.set(|s| s.show_inactive = v)
                        />
                    </Row>
                </Panel>
                <Panel title="Galaxy map" icon=icons::SETTINGS>
                    <Row
                        title="Map overlay"
                        text="Regions draws merged enemy fronts; Sectors shows the sector grid with enemy territory hatched; Galaxy shows supply lines only."
                    >
                        <Stepper
                            label="Map overlay"
                            options=overlay_options()
                            value=Signal::derive(move || s.get(|s| s.map_overlay))
                            on_change=move |v| s.set(|s| s.map_overlay = v)
                        />
                    </Row>
                    <Row title="Planets" text="Glowing markers, or shaded spheres tinted by biome like the in-game map.">
                        <Stepper
                            label="Planet style"
                            options=planet_options()
                            value=Signal::derive(move || s.get(|s| s.map_planets))
                            on_change=move |v| s.set(|s| s.map_planets = v)
                        />
                    </Row>
                    <Row
                        title="Render quality"
                        text="Minimal keeps the look but stops the pulsing markers and draws fewer, softer stars. Maximum draws the most stars at full sharpness."
                    >
                        <Stepper
                            label="Render quality"
                            options=quality_options()
                            value=Signal::derive(move || s.get(|s| s.map_quality))
                            on_change=move |v| s.set(|s| s.map_quality = v)
                        />
                    </Row>
                    <Row
                        title="Map dispatches"
                        text="How many dispatches the map's Dispatches box lists: the latest few whatever their age, or every one from the last 72 hours."
                    >
                        <Stepper
                            label="Map dispatches"
                            options=dispatch_options()
                            value=Signal::derive(move || s.get(|s| s.map_dispatches))
                            on_change=move |v| s.set(|s| s.map_dispatches = v)
                        />
                    </Row>
                </Panel>
                <Panel title="Notifications" icon=icons::BELL_RING>
                    <Row title="Popups" text="Which in-app popups appear. Priority hides dispatches.">
                        <Stepper
                            label="Popups"
                            options=vec![(Popups::All, "All"), (Popups::Priority, "Priority"), (Popups::None, "None")]
                            value=Signal::derive(move || s.get(|s| s.popups))
                            on_change=move |v| s.set(|s| s.popups = v)
                        />
                    </Row>
                    {ALERTS
                        .iter()
                        .map(|(key, title, text)| {
                            let key = *key;
                            view! {
                                <Row title=*title text=*text>
                                    <OnOff
                                        label=*title
                                        value=Signal::derive(move || s.get(|s| alert_value(&s.alerts, key)))
                                        on_change=move |v| s.set(|s| *alert_flag(&mut s.alerts, key) = v)
                                    />
                                </Row>
                            }
                        })
                        .collect_view()}
                    {tauri::has_tray().then(|| view! {
                        <Row
                            title="Keep running in the tray"
                            text="Closing the window leaves the app in the tray, so alerts keep coming. Quit from the tray icon."
                        >
                            <OnOff
                                label="Keep running in the tray"
                                value=Signal::derive(move || s.get(|s| s.close_to_tray))
                                on_change=move |v| s.set(|s| s.close_to_tray = v)
                            />
                        </Row>
                    })}
                    <div class="set-actions">
                        <button
                            class="hud-btn"
                            on:click=move |_| {
                                let system = s.untracked(|s| s.alerts.system);
                                notify(ToastKind::Order, "Test alert", "Alerts are working, Helldiver.", system, None)
                            }
                        >
                            <Icon icon=icons::BELL_RING size=14.0 />
                            " Send test alert"
                        </button>
                    </div>
                </Panel>
                <Panel title="Updates" icon=icons::DOWNLOAD>
                    <UpdateRow />
                </Panel>
                <div class="set-footer">
                    <button class="hud-btn" on:click=copy_diagnostics>
                        <Icon icon=icons::CLIPBOARD_COPY size=14.0 />
                        {move || if copied.get() { " Copied" } else { " Copy diagnostics" }}
                    </button>
                    <button class="hud-btn set-cta" on:click=move |_| s.reset_unread()>
                        <Icon icon=icons::ROTATE_CCW size=14.0 />
                        " Reset hints and unread"
                    </button>
                    <button class="hud-btn" on:click=move |_| s.clear_pins() disabled=move || pinned.get() == 0>
                        <Icon icon=icons::PIN_OFF size=14.0 />
                        " Clear pinned planets"
                        {move || (pinned.get() > 0).then(|| format!(" ({})", pinned.get()))}
                    </button>
                </div>
                <p class="set-version caps num">{format!("WarCommand Tracker v{APP_VERSION} · changes apply immediately")}</p>
            </div>
        </div>
    }
}
