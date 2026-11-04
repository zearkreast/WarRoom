//! The planet detail's jump bar: a link to each section on the page, the one in view lit.

use crate::js;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;

/// A section the bar links to; sections that render nothing on a planet are left off the bar.
#[component]
pub fn Sec(label: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="pd-sec" data-label=label>
            {children()}
        </div>
    }
}

fn shown_sections(root: &web_sys::Element) -> Vec<web_sys::HtmlElement> {
    let Ok(list) = root.query_selector_all(".pd-sec[data-label]") else { return Vec::new() };
    (0..list.length())
        .filter_map(|i| list.item(i)?.dyn_into::<web_sys::HtmlElement>().ok())
        .filter(|s| s.offset_height() > 0)
        .collect()
}

/// Marks the sections that have come into view, so each rises in as it's first reached; a batch reached
/// together rises one after another.
fn reveal(bar: &web_sys::Element, root: &web_sys::Element, list: &[web_sys::HtmlElement]) {
    if let Some(page) = bar.parent_element() {
        let _ = page.class_list().add_1("staged");
    }
    let bottom = root.get_bounding_client_rect().bottom();
    let mut k = 0;
    for s in list {
        if s.class_list().contains("seen") || s.get_bounding_client_rect().top() > bottom - 24.0 {
            continue;
        }
        let _ = s.style().set_property("--d", &format!("{}ms", k * 80));
        let _ = s.class_list().add_1("seen");
        k += 1;
    }
}

#[component]
pub fn SectionBar() -> impl IntoView {
    let nav = NodeRef::<leptos::html::Nav>::new();
    let labels = RwSignal::new(Vec::<String>::new());
    let current = RwSignal::new(0usize);
    // A linked section stays lit until the reader scrolls themselves: near the page's end it can't reach the
    // top, and the scroll rule would light the last section instead.
    let picked = StoredValue::new(None::<usize>);
    let scan = move || {
        let Some(bar) = nav.get_untracked() else { return };
        let Some(root) = bar.closest(".overlay-scroll").ok().flatten() else { return };
        let list = shown_sections(&root);
        reveal(&bar, &root, &list);
        let found: Vec<String> = list.iter().map(|s| s.get_attribute("data-label").unwrap_or_default()).collect();
        // A section counts as reached once its top passes a little below the bar.
        let line = bar.get_bounding_client_rect().bottom() + 48.0;
        let at_end = f64::from(root.scroll_top() + root.client_height()) >= f64::from(root.scroll_height()) - 2.0;
        let i = if let Some(i) = picked.get_value() {
            i
        } else if at_end {
            list.len().saturating_sub(1)
        } else {
            list.iter().rposition(|s| s.get_bounding_client_rect().top() <= line).unwrap_or(0)
        };
        if labels.with_untracked(|l| *l != found) {
            labels.set(found);
        }
        if current.get_untracked() != i {
            current.set(i);
        }
    };
    type Listeners = Vec<(&'static str, Closure<dyn Fn()>)>;
    let listeners: StoredValue<Option<SendWrapper<(web_sys::Element, Listeners)>>, LocalStorage> =
        StoredValue::new_local(None);
    Effect::new(move |_| {
        let Some(bar) = nav.get() else { return };
        let Some(root) = bar.closest(".overlay-scroll").ok().flatten() else { return };
        let mut list: Listeners = vec![("scroll", Closure::<dyn Fn()>::new(scan))];
        for event in ["wheel", "touchstart", "keydown"] {
            list.push((event, Closure::<dyn Fn()>::new(move || picked.set_value(None))));
        }
        for (event, f) in &list {
            let _ = root.add_event_listener_with_callback(event, f.as_ref().unchecked_ref());
        }
        listeners.set_value(Some(SendWrapper::new((root, list))));
        scan();
    });
    on_cleanup(move || {
        if let Some(l) = listeners.try_update_value(Option::take).flatten() {
            let (root, list) = &*l;
            for (event, f) in list {
                let _ = root.remove_event_listener_with_callback(event, f.as_ref().unchecked_ref());
            }
        }
    });
    // Sections fill in as their data arrives, so the list is checked again now and then.
    let tick = js::use_now(1500.0);
    Effect::new(move |_| {
        tick.track();
        scan();
    });
    let jump = move |i: usize| {
        let Some(root) = nav.get_untracked().and_then(|b| b.closest(".overlay-scroll").ok().flatten()) else {
            return;
        };
        if let Some(s) = shown_sections(&root).get(i) {
            picked.set_value(Some(i));
            current.set(i);
            let opts = web_sys::ScrollIntoViewOptions::new();
            opts.set_behavior(if js::prefers_reduced_motion() {
                web_sys::ScrollBehavior::Auto
            } else {
                web_sys::ScrollBehavior::Smooth
            });
            opts.set_block(web_sys::ScrollLogicalPosition::Start);
            s.scroll_into_view_with_scroll_into_view_options(&opts);
        }
    };
    let items = move || labels.get().into_iter().enumerate().collect::<Vec<_>>();
    view! {
        <nav class="pd-nav" node_ref=nav aria-label="Sections">
            <For each=items key=|x| x.clone() let:item>
                <button
                    class=move || format!("pd-nav-item caps {}", if current.get() == item.0 { "on" } else { "" })
                    aria-current=move || (current.get() == item.0).then_some("true")
                    on:click=move |_| jump(item.0)
                >
                    {item.1.clone()}
                </button>
            </For>
        </nav>
    }
}
