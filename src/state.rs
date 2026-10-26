use crate::js;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::OnceLock;
use warroom_core::gww_nav::GwwPage;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum View {
    Campaigns,
    Map,
    Hellpad,
    Overview,
    News,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HellpadModule {
    Planets,
    Stations,
    Gww,
    Campaigns,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InfoDialog {
    About,
    Legend,
}

/// Where the planet options popover is anchored, in viewport pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OptionsAnchor {
    pub planet: i64,
    pub x: f64,
    pub y: f64,
}

/// What the app is showing. Not persisted; Back and Forward step through it (see nav_history).
#[derive(Clone, Copy)]
pub struct Ui {
    pub view: RwSignal<View>,
    pub module: RwSignal<Option<HellpadModule>>,
    /// Planet highlighted on the map.
    pub selected_planet: RwSignal<Option<i64>>,
    /// Bumped on every request to centre the map on the selected planet, so repeating one still flies.
    pub focus_seq: RwSignal<u64>,
    /// Planet whose detail overlay is open.
    pub detail: RwSignal<Option<i64>>,
    pub options: RwSignal<Option<OptionsAnchor>>,
    pub dispatch_open: RwSignal<bool>,
    /// The About or Icons legend dialog, when one is open.
    pub dialog: RwSignal<Option<InfoDialog>>,
    /// The G.W.W.'s page history, kept while the app runs so leaving the module and coming back resumes it.
    pub gww: RwSignal<Vec<GwwPage>>,
}

impl Ui {
    pub fn set_view(&self, view: View) {
        self.view.set(view);
        self.options.set(None);
        self.module.set(None);
    }

    pub fn open_module(&self, module: Option<HellpadModule>) {
        self.view.set(View::Hellpad);
        self.module.set(module);
        self.options.set(None);
        self.detail.set(None);
    }

    pub fn select(&self, planet: Option<i64>) {
        self.selected_planet.set(planet);
    }

    pub fn show_on_map(&self, planet: i64) {
        self.view.set(View::Map);
        self.selected_planet.set(Some(planet));
        self.focus_seq.update(|n| *n += 1);
        self.detail.set(None);
        self.options.set(None);
    }

    pub fn focus(&self, planet: i64) {
        self.selected_planet.set(Some(planet));
        self.focus_seq.update(|n| *n += 1);
        self.options.set(None);
    }

    pub fn open_detail(&self, planet: Option<i64>) {
        self.detail.set(planet);
        self.options.set(None);
    }

    pub fn open_options(&self, anchor: Option<OptionsAnchor>) {
        self.options.set(anchor);
    }

    pub fn toggle_dispatch(&self, open: Option<bool>) {
        self.dispatch_open.update(|o| *o = open.unwrap_or(!*o));
    }

    pub fn show_dialog(&self, dialog: Option<InfoDialog>) {
        self.dialog.set(dialog);
    }

    pub fn set_gww(&self, stack: Vec<GwwPage>) {
        self.gww.set(stack);
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AlertSettings {
    pub system: bool,
    pub orders: bool,
    pub dispatches: bool,
    pub captures: bool,
    pub defenses: bool,
}

impl Default for AlertSettings {
    fn default() -> Self {
        AlertSettings { system: true, orders: true, dispatches: true, captures: true, defenses: true }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MapOverlay {
    #[default]
    Regions,
    Sectors,
    Galaxy,
}

impl MapOverlay {
    pub fn as_str(self) -> &'static str {
        match self {
            MapOverlay::Regions => "regions",
            MapOverlay::Sectors => "sectors",
            MapOverlay::Galaxy => "galaxy",
        }
    }

    /// The overlay as the faction marks are placed for it.
    pub fn core(self) -> warroom_core::faction_marks::Overlay {
        use warroom_core::faction_marks::Overlay;
        match self {
            MapOverlay::Regions => Overlay::Regions,
            MapOverlay::Sectors => Overlay::Sectors,
            MapOverlay::Galaxy => Overlay::Galaxy,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MapQuality {
    Max,
    #[default]
    Normal,
    Min,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MapPlanets {
    #[default]
    Markers,
    Spheres,
}

impl MapQuality {
    pub fn as_str(self) -> &'static str {
        match self {
            MapQuality::Max => "max",
            MapQuality::Normal => "normal",
            MapQuality::Min => "min",
        }
    }
}

impl MapPlanets {
    pub fn as_str(self) -> &'static str {
        match self {
            MapPlanets::Markers => "markers",
            MapPlanets::Spheres => "spheres",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Popups {
    #[default]
    All,
    Priority,
    None,
}

/// How many dispatches the map's Dispatches box lists, or all from the last 72 hours.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MapDispatches {
    #[serde(rename = "3")]
    Three,
    #[default]
    #[serde(rename = "5")]
    Five,
    #[serde(rename = "10")]
    Ten,
    #[serde(rename = "72h")]
    Recent,
}

impl MapDispatches {
    pub fn as_str(self) -> &'static str {
        match self {
            MapDispatches::Three => "3",
            MapDispatches::Five => "5",
            MapDispatches::Ten => "10",
            MapDispatches::Recent => "72h",
        }
    }
}

/// Everything the app remembers between launches, stored as the React app stored it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SettingsData {
    pub alerts: AlertSettings,
    pub popups: Popups,
    pub map_overlay: MapOverlay,
    pub map_quality: MapQuality,
    pub map_planets: MapPlanets,
    pub show_inactive: bool,
    /// Whether the Galactic control ring is unfolded on the planet index.
    pub planets_control_open: bool,
    /// Whether the Major Order strip on Active planets is unfolded.
    pub mo_strip_open: bool,
    /// Whether the Theater of War disc turns by itself.
    pub tw_spin: bool,
    pub map_dispatches: MapDispatches,
    /// Desktop: closing the window hides it to the tray, so alerts keep coming.
    pub close_to_tray: bool,
    pub pinned: Vec<i64>,
    /// Views whose first-run hint has been dismissed.
    pub hints_seen: Vec<String>,
    /// Ids already alerted on, so a restart doesn't repeat them.
    pub seen_assignments: Vec<i64>,
    pub seen_dispatch: i64,
    pub seen_campaign_events: Vec<String>,
    /// App version whose changelog the user has opened.
    pub seen_changelog: String,
    /// G.W.W. training manuals read to the end, by id.
    pub gww_read: Vec<String>,
    /// Time of the newest Strohmann News item on the last visit; 0 before the first.
    pub news_seen_at: f64,
}

impl Default for SettingsData {
    fn default() -> Self {
        SettingsData {
            alerts: AlertSettings::default(),
            popups: Popups::All,
            map_overlay: MapOverlay::Regions,
            map_quality: MapQuality::Normal,
            map_planets: MapPlanets::Markers,
            show_inactive: false,
            planets_control_open: true,
            mo_strip_open: false,
            tw_spin: !js::prefers_reduced_motion(),
            map_dispatches: MapDispatches::Five,
            close_to_tray: true,
            pinned: Vec::new(),
            hints_seen: Vec::new(),
            seen_assignments: Vec::new(),
            seen_dispatch: 0,
            seen_campaign_events: Vec::new(),
            seen_changelog: String::new(),
            gww_read: Vec::new(),
            news_seen_at: 0.0,
        }
    }
}

const STORAGE_KEY: &str = "warcommand-settings";

/// Stored settings over the defaults, key by key, so one unreadable value doesn't lose the rest.
fn load() -> SettingsData {
    let defaults = SettingsData::default();
    let Some(stored) = js::local_get(STORAGE_KEY).and_then(|s| serde_json::from_str::<Value>(&s).ok()) else {
        return defaults;
    };
    let Some(Value::Object(state)) = stored.get("state").cloned() else { return defaults };
    let mut merged = serde_json::to_value(&defaults).unwrap();
    for (k, v) in state {
        let mut trial = merged.clone();
        trial[&k] = v;
        if serde_json::from_value::<SettingsData>(trial.clone()).is_ok() {
            merged = trial;
        }
    }
    serde_json::from_value(merged).unwrap_or(defaults)
}

fn save(data: &SettingsData) {
    let stored = serde_json::json!({ "state": data, "version": 0 });
    js::local_set(STORAGE_KEY, &stored.to_string());
}

#[derive(Clone, Copy)]
pub struct Settings {
    pub data: RwSignal<SettingsData>,
}

impl Settings {
    fn change(&self, f: impl FnOnce(&mut SettingsData)) {
        self.data.update(|d| {
            f(d);
            save(d);
        });
    }

    pub fn get<T>(&self, f: impl FnOnce(&SettingsData) -> T) -> T {
        self.data.with(f)
    }

    pub fn untracked<T>(&self, f: impl FnOnce(&SettingsData) -> T) -> T {
        self.data.with_untracked(f)
    }

    pub fn set(&self, f: impl FnOnce(&mut SettingsData)) {
        self.change(f);
    }

    pub fn toggle_pin(&self, planet: i64) {
        self.change(|d| match d.pinned.iter().position(|p| *p == planet) {
            Some(i) => {
                d.pinned.remove(i);
            }
            None => d.pinned.push(planet),
        });
    }

    pub fn clear_pins(&self) {
        self.change(|d| d.pinned.clear());
    }

    pub fn dismiss_hint(&self, id: &str) {
        self.change(|d| {
            if !d.hints_seen.iter().any(|h| h == id) {
                d.hints_seen.push(id.into());
            }
        });
    }

    pub fn reset_unread(&self) {
        self.change(|d| {
            d.seen_changelog.clear();
            d.hints_seen.clear();
        });
    }

    pub fn mark_seen(&self, assignments: Vec<i64>, dispatch: i64) {
        self.change(|d| {
            d.seen_assignments = assignments;
            d.seen_dispatch = dispatch;
        });
    }

    pub fn mark_campaign_seen(&self, keys: Vec<String>) {
        self.change(|d| d.seen_campaign_events = keys);
    }

    pub fn mark_changelog(&self, version: &str) {
        self.change(|d| d.seen_changelog = version.into());
    }

    pub fn mark_read(&self, manual: &str) {
        if self.untracked(|d| d.gww_read.iter().any(|m| m == manual)) {
            return;
        }
        self.change(|d| d.gww_read.push(manual.into()));
    }

    pub fn clear_read(&self) {
        self.change(|d| d.gww_read.clear());
    }

    pub fn mark_news_seen(&self, at: f64) {
        self.change(|d| d.news_seen_at = d.news_seen_at.max(at));
    }
}

/// Runs `f` under an owner that is never disposed, for app-wide memos first asked for inside a component.
pub fn global<T>(f: impl FnOnce() -> T) -> T {
    let caller = Owner::current();
    // new_root also makes the root the current owner, so the caller's owner is put back by hand.
    let owner = Owner::new_root(None);
    let out = owner.with(f);
    match caller {
        Some(c) => c.set(),
        None => owner.clone().unset(),
    }
    std::mem::forget(owner);
    out
}

struct Stores {
    ui: Ui,
    settings: Settings,
}

static STORES: OnceLock<Stores> = OnceLock::new();

/// Creates the app-wide stores; called once, before anything renders, so no component owns them.
pub fn init() {
    let ui = Ui {
        view: RwSignal::new(View::Campaigns),
        module: RwSignal::new(None),
        selected_planet: RwSignal::new(None),
        focus_seq: RwSignal::new(0),
        detail: RwSignal::new(None),
        options: RwSignal::new(None),
        dispatch_open: RwSignal::new(false),
        dialog: RwSignal::new(None),
        gww: RwSignal::new(vec![GwwPage::Home]),
    };
    let settings = Settings { data: RwSignal::new(load()) };
    let _ = STORES.set(Stores { ui, settings });
}

pub fn ui() -> Ui {
    STORES.get().expect("stores initialised").ui
}

pub fn settings() -> Settings {
    STORES.get().expect("stores initialised").settings
}
