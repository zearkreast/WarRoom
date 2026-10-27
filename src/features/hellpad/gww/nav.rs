//! G.W.W. links (following a target, text with inline links) and the URLs of its pictures.

use crate::api::queries::queries;
use crate::assets;
use crate::state::{HellpadModule, OptionsAnchor, View, global, ui};
use crate::ui::icons::{self, Icon};
use leptos::prelude::*;
use std::sync::{Arc, OnceLock};
use warroom_core::api::{Faction, Planet};
use warroom_core::factions::faction_color;
use warroom_core::gww_nav::{GwwPage, page_of};
use warroom_core::gww_text::{Segment, parse_rich};
use wasm_bindgen::JsCast;

/// A game icon: ministry, campaign type, difficulty, or a space station action.
pub fn icon_url(name: &str) -> Option<String> {
    assets::url(&format!("gww/icons/{name}.svg")).or_else(|| assets::url(&format!("dss-actions/{name}.png")))
}

/// A corporation's logo; some are kept as icons.
pub fn logo_url(name: &str) -> Option<String> {
    assets::url(&format!("gww/logos/{name}.webp"))
        .or_else(|| assets::url(&format!("gww/logos/{name}.svg")))
        .or_else(|| icon_url(name))
}

pub fn ad_url(name: &str) -> Option<String> {
    assets::url(&format!("gww/ads/{name}.webp"))
}

/// Moves the G.W.W. to a page; given to every page through context.
#[derive(Clone, Copy)]
pub struct GwwNav {
    pub go: Callback<GwwPage>,
}

pub fn use_gww_nav() -> GwwNav {
    expect_context::<GwwNav>()
}

pub fn planet_named(name: &str) -> Option<Planet> {
    let name = name.to_lowercase();
    queries().fresh_planets.with_untracked(|p| p.as_ref()?.iter().find(|p| p.name.to_lowercase() == name).cloned())
}

static OWNERS: OnceLock<Memo<Arc<Vec<(String, Faction)>>>> = OnceLock::new();

/// Planet names and owners, changing only when a planet changes hands, so links redraw only then.
fn owners() -> Memo<Arc<Vec<(String, Faction)>>> {
    *OWNERS.get_or_init(|| {
        let q = queries();
        global(|| {
            Memo::new(move |_| {
                Arc::new(q.fresh_planets.with(|p| {
                    p.iter().flat_map(|p| p.iter()).map(|p| (p.name.to_lowercase(), p.current_owner)).collect()
                }))
            })
        })
    })
}

impl GwwNav {
    /// Opens any link target: a G.W.W. page, a page of the app, or a planet (its options placed at `e`).
    pub fn follow(&self, target: &str, e: Option<&web_sys::MouseEvent>) {
        if let Some(page) = page_of(target) {
            self.go.run(page);
            return;
        }
        let id = target.find(':').map_or(target, |i| &target[i + 1..]);
        if target.starts_with("planet:") {
            let Some(p) = planet_named(id) else { return };
            let rect = e
                .and_then(|e| e.current_target())
                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                .map(|el| el.get_bounding_client_rect());
            let pick = |v: Option<f64>, fallback: Option<f64>| {
                v.filter(|v| *v != 0.0).or(fallback.filter(|v| *v != 0.0)).unwrap_or(0.0)
            };
            let x = pick(e.map(|e| e.client_x() as f64), rect.as_ref().map(|r| r.left()));
            let y = pick(e.map(|e| e.client_y() as f64), rect.as_ref().map(|r| r.bottom()));
            ui().open_options(Some(OptionsAnchor { planet: p.index, x, y }));
            return;
        }
        let u = ui();
        match id {
            "stations" => u.open_module(Some(HellpadModule::Stations)),
            "planets" => u.open_module(Some(HellpadModule::Planets)),
            "past" => u.open_module(Some(HellpadModule::PastCampaigns)),
            "news" => u.set_view(View::News),
            "overview" => u.set_view(View::Overview),
            "map" => u.set_view(View::Map),
            "campaigns" => u.set_view(View::Campaigns),
            _ => {}
        }
    }
}

/// Never followed (clicks are handled); it makes the anchor a real link for keyboards and screen readers.
fn href_of(target: &str) -> String {
    format!("#gww/{target}")
}

/// Text with its `[[kind:target|label]]` links made live.
#[component]
pub fn Rich(#[prop(into)] text: String) -> impl IntoView {
    let nav = use_gww_nav();
    let owners = owners();
    move || {
        owners.with(|_| ());
        rich(&text, nav)
    }
}

fn rich(text: &str, nav: GwwNav) -> AnyView {
    parse_rich(text)
        .into_iter()
        .map(|seg| match seg {
            Segment::Text(t) => t.into_any(),
            Segment::Link { kind, target, label } if kind == "planet" => match planet_named(&target) {
                None => label.into_any(),
                Some(p) => {
                    let to = format!("planet:{target}");
                    view! {
                        <a
                            class="gww-link planet"
                            href=href_of(&to)
                            style=format!("color: {};", faction_color(Some(p.current_owner)))
                            title=format!("{}: open its options", p.name)
                            on:click=move |e| {
                                e.prevent_default();
                                nav.follow(&to, Some(&e));
                            }
                        >
                            {label}
                        </a>
                    }
                    .into_any()
                }
            },
            Segment::Link { kind, target, label } => {
                let to = format!("{kind}:{target}");
                let app = kind == "app";
                view! {
                    <a
                        class=format!("gww-link {kind}")
                        href=href_of(&to)
                        on:click=move |e| {
                            e.prevent_default();
                            nav.follow(&to, Some(&e));
                        }
                    >
                        {label}
                        {app.then(|| view! {
                            <Icon icon=icons::ARROW_UP_RIGHT size=11.0 />
                            <span class="sr-only">" (leaves the G.W.W.)"</span>
                        })}
                    </a>
                }
                .into_any()
            }
        })
        .collect_view()
        .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    use warroom_core::data::manuals::MANUALS;
    use warroom_core::data::sites::{ADS, CITIZENS, COMPANIES, GOV, MINISTRIES};

    #[test]
    fn finds_every_picture_its_pages_name() {
        let mut icons: Vec<&str> = Vec::new();
        for m in MANUALS {
            icons.extend(m.icon);
            for s in m.sections {
                icons.extend(s.kinds.unwrap_or(&[]).iter().map(|k| k.icon));
                if let Some(t) = &s.table {
                    icons.extend(t.rows.iter().filter_map(|r| r.icon));
                }
            }
        }
        icons.extend(MINISTRIES.iter().map(|m| m.icon));
        icons.extend([GOV.icon, CITIZENS.icon]);
        let missing: Vec<&str> = icons.into_iter().filter(|i| icon_url(i).is_none()).collect();
        assert!(missing.is_empty(), "icons: {missing:?}");
        let logos = COMPANIES.iter().flat_map(|c| [c.logo, c.showcase.as_ref().map(|s| s.image)]).flatten();
        let missing: Vec<&str> = logos.filter(|l| logo_url(l).is_none()).collect();
        assert!(missing.is_empty(), "logos: {missing:?}");
        let placed = MINISTRIES
            .iter()
            .flat_map(|m| m.ads.unwrap_or(&[]))
            .chain(COMPANIES.iter().flat_map(|c| c.ads.unwrap_or(&[])));
        let missing: Vec<&str> =
            ADS.iter().map(|a| a.file).chain(placed.copied()).filter(|a| ad_url(a).is_none()).collect();
        assert!(missing.is_empty(), "adverts: {missing:?}");
    }
}
