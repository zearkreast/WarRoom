use crate::api::fronts::station_planets;
use crate::api::queries;
use crate::js::{self, num};
use crate::state::{HellpadModule, View, settings, ui};
use crate::ui::emblems::{IconLike, icon_view};
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::use_escape_when;
use leptos::html;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use wasm_bindgen::JsCast;

const SIZE: f64 = 236.0;

/// Radial options for a planet: pin, show on map, station, details.
#[component]
pub fn PlanetOptions() -> impl IntoView {
    let u = ui();
    let options = u.options;
    let first = NodeRef::<html::Button>::new();
    let opener: StoredValue<Option<SendWrapper<web_sys::HtmlElement>>> = StoredValue::new(None);
    let open = Memo::new(move |_| options.with(Option::is_some));

    Effect::new(move |was_open: Option<bool>| {
        let is_open = open.get();
        if is_open && was_open != Some(true) {
            let from = js::document().active_element().and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok());
            opener.set_value(from.map(SendWrapper::new));
        }
        if !is_open && was_open == Some(true) {
            // Back to what opened the menu, unless the choice replaced it or covered it with the planet page.
            if let Some(from) = opener.get_value() {
                let from = from.take();
                let _ = js::window().request_animation_frame(
                    wasm_bindgen::closure::Closure::once_into_js(move || {
                        if from.is_connected() && js::document().query_selector(".overlay").ok().flatten().is_none() {
                            let opts = web_sys::FocusOptions::new();
                            opts.set_prevent_scroll(true);
                            let _ = from.focus_with_options(&opts);
                        }
                    })
                    .unchecked_ref(),
                );
            }
            opener.set_value(None);
        }
        is_open
    });
    Effect::new(move |_| {
        if let Some(b) = first.get() {
            let _ = b.focus();
        }
    });
    use_escape_when(move || ui().open_options(None), move || open.get());

    let stations = station_planets();
    move || {
        let anchor = options.get()?;
        let planet = untrack(|| queries::planet(Some(anchor.planet)))?;
        let index = planet.index;
        let close = move || ui().open_options(None);
        let w = js::window();
        let (vw, vh) = (
            w.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(0.0),
            w.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(0.0),
        );
        let x = anchor.x.max(SIZE / 2.0 + 8.0).min(vw - SIZE / 2.0 - 8.0);
        let y = anchor.y.max(SIZE / 2.0 + 8.0).min(vh - SIZE / 2.0 - 8.0);
        let has_station = stations.with(|s| s.contains(&index));
        let pinned = settings().get(|s| s.pinned.contains(&index));
        let on_map = ui().view.get() == View::Map;
        type Run = Box<dyn Fn() + Send + Sync>;
        let items: Vec<(&str, String, IconLike, Run, bool)> = vec![
            (
                "n",
                if pinned { "Unpin" } else { "Pin to top" }.into(),
                if pinned { icons::PIN_OFF } else { icons::PIN }.into(),
                Box::new(move || {
                    settings().toggle_pin(index);
                    ui().open_options(None);
                }),
                false,
            ),
            ("e", "Details".into(), icons::FILE_TEXT.into(), Box::new(move || ui().open_detail(Some(index))), false),
            // The map is already open there, so its slot centres the planet instead.
            if on_map {
                ("s", "Focus".into(), icons::CROSSHAIR.into(), Box::new(move || ui().focus(index)) as Run, false)
            } else {
                (
                    "s",
                    "Show on map".into(),
                    icons::MAP_PIN.into(),
                    Box::new(move || ui().show_on_map(index)) as Run,
                    false,
                )
            },
            (
                "w",
                "Space station".into(),
                IconLike::Dss,
                Box::new(move || ui().open_module(Some(HellpadModule::Stations))),
                !has_station,
            ),
        ];
        Some(view! {
            <div class="po-scrim" on:click=move |_| close()>
                <div
                    class="po"
                    style=format!(
                        "left: {}px; top: {}px; width: {}px; height: {}px;",
                        num(x - SIZE / 2.0),
                        num(y - SIZE / 2.0),
                        num(SIZE),
                        num(SIZE)
                    )
                    role="menu"
                    aria-label=format!("{} options", planet.name)
                    on:click=|e| e.stop_propagation()
                >
                    {items
                        .into_iter()
                        .enumerate()
                        .map(|(i, (pos, label, icon, run, disabled))| {
                            let title = if disabled { "No space station at this planet".to_string() } else { label.clone() };
                            let node = if i == 0 { first } else { NodeRef::new() };
                            view! {
                                <button
                                    node_ref=node
                                    role="menuitem"
                                    class=format!("po-item po-{pos}")
                                    on:click=move |_| run()
                                    disabled=disabled
                                    title=title
                                >
                                    {icon_view(icon, 22.0, 1.75, None)}
                                    <span>{label}</span>
                                </button>
                            }
                        })
                        .collect_view()}
                    <button class="po-center" on:click=move |_| close() aria-label="Close options">
                        <span class="po-planet">{planet.name.clone()}</span>
                        <span class="po-close caps">
                            <Icon icon=icons::X size=12.0 />
                            " Close"
                        </span>
                    </button>
                </div>
            </div>
        })
    }
}
