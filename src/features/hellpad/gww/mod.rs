//! The G.W.W. (Galactic War Web): the in-universe net, its address bar, search, home page and page router.

pub mod blocks;
pub mod figures;
pub mod live;
pub mod logs;
pub mod nav;
pub mod pages;
pub mod pictures;
pub mod tiles;

use crate::api::history::history;
use crate::js;
use crate::state::settings;
use crate::state::ui;
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::Empty;
use blocks::HeadingRef;
use leptos::html;
use leptos::prelude::*;
use logs::logs_page;
use nav::{GwwNav, icon_url, use_gww_nav};
use pages::{company_page, dir_page, gov_page, holiday_page, holidays_page, manual_page, ministry_page, missing_page};
use pictures::{RAIL_QUERY, ad_box, pick_one};
use tiles::{company_tile, gov_tile, log_tile, manual_tile, ministry_tile, news_tile, site_tile};
use warroom_core::data::manuals::{CHAPTERS, MANUALS, TIPS};
use warroom_core::data::sites::{ADS, COMPANIES, GOV, HOLIDAYS, MINISTRIES};
use warroom_core::data::{Ad, AdShape};
use warroom_core::gww_nav::{GwwPage, LOG_ADDRESS, LOG_TITLE, address_of, normalize, suggested_addresses, target_at};
use warroom_core::gww_text::{next_yearly, plain_text};
use wasm_bindgen::JsCast;

fn key_of(p: &GwwPage) -> String {
    match p {
        GwwPage::Home => "home".into(),
        GwwPage::Manual { id } => format!("manual:{id}"),
        GwwPage::Ministry { id } => format!("ministry:{id}"),
        GwwPage::Company { id } => format!("company:{id}"),
        GwwPage::Holiday { id } => format!("holiday:{id}"),
        GwwPage::Site { id } => format!("site:{id}"),
        GwwPage::Dir { id } => format!("dir:{id}"),
        GwwPage::Missing { address } => format!("missing:{address}"),
    }
}

/// The address bar: shows where the reader is, and takes any address typed into it.
#[component]
fn AddressBar(page: Memo<GwwPage>) -> impl IntoView {
    let nav = use_gww_nav();
    let draft = RwSignal::new(None::<String>);
    let input = NodeRef::<html::Input>::new();
    let list_id = js::unique_id("gww-addresses-");
    let open = move |e: web_sys::SubmitEvent| {
        e.prevent_default();
        let typed = draft.get_untracked().unwrap_or_default();
        match target_at(&typed) {
            Some(target) => nav.follow(&target, None),
            None => nav.go.run(GwwPage::Missing { address: normalize(&typed) }),
        }
        if let Some(i) = input.get_untracked() {
            let _ = i.blur();
        }
    };
    let input_of = |e: &web_sys::Event| e.current_target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
    view! {
        <form class="gww-url num" on:submit=open>
            <Icon icon=icons::LOCK size=12.0 />
            <input
                node_ref=input
                aria-label="Address"
                prop:value=move || draft.get().unwrap_or_else(|| address_of(&page.get()))
                list=list_id.clone()
                spellcheck="false"
                autocomplete="off"
                autocapitalize="off"
                enterkeyhint="go"
                on:focus=move |e| {
                    draft.set(Some(address_of(&page.get_untracked())));
                    if let Some(i) = input_of(&e) {
                        i.select();
                    }
                }
                on:input=move |e| draft.set(Some(event_target_value(&e)))
                on:blur=move |_| draft.set(None)
                on:keydown=move |e: web_sys::KeyboardEvent| {
                    if e.key() != "Escape" {
                        return;
                    }
                    e.stop_propagation();
                    if let Some(i) = input_of(&e) {
                        let _ = i.blur();
                    }
                }
            />
            <datalist id=list_id>{suggested_addresses().into_iter().map(|a| view! { <option value=a></option> }).collect_view()}</datalist>
        </form>
    }
}

fn log_line(count: usize, older: bool) -> String {
    match count {
        0 => "The war log and the FTL jumps.".into(),
        n if older => format!("The war log and the FTL jumps. {n}+ planets have changed hands since records began."),
        1 => "The war log and the FTL jumps. 1 planet has changed hands since records began.".into(),
        n => format!("The war log and the FTL jumps. {n} planets have changed hands since records began."),
    }
}

fn group(title: &'static str, count: usize, all: Option<&'static str>, children: AnyView) -> AnyView {
    let nav = use_gww_nav();
    view! {
        <section class="gww-group">
            <h3 class="gww-group-title caps">
                {title}
                " "
                <span class="num">{count}</span>
                {all.map(|all| view! {
                    <button class="gww-group-all" on:click=move |e| nav.follow(all, Some(&e))>
                        "View all "
                        <Icon icon=icons::ARROW_RIGHT size=12.0 />
                    </button>
                })}
            </h3>
            <div class="gww-sites">{children}</div>
        </section>
    }
    .into_any()
}

/// One of the game's Training Manual Tips, with a button for the next.
#[component]
fn TipTicker() -> impl IntoView {
    let at = RwSignal::new((js_sys::Math::random() * TIPS.len() as f64).floor() as usize);
    view! {
        <div class="gww-ticker">
            <span class="gww-ticker-label caps">
                <Icon icon=icons::LIGHTBULB size=14.0 />
                " Training manual tip"
            </span>
            {move || view! { <p class="gww-ticker-text" aria-live="polite">{TIPS[at.get()]}</p> }}
            <button class="gww-ticker-next" aria-label="Next tip" title="Next tip" on:click=move |_| at.update(|i| *i = (*i + 1) % TIPS.len())>
                <Icon icon=icons::REFRESH_CW size=14.0 />
            </button>
        </div>
    }
}

#[component]
fn HolidayChip() -> impl IntoView {
    let nav = use_gww_nav();
    let now = js::use_now(60.0 * 60_000.0);
    let days: Vec<(u32, u32)> = HOLIDAYS.iter().map(|h| (h.month, h.day)).collect();
    let next = Memo::new(move |_| {
        let (y, m, d) = js::local_today(now.get());
        next_yearly(&days, y, m, d)
    });
    move || {
        next.get().map(|(i, n)| {
            let when = match n {
                0 => "today".to_string(),
                1 => "tomorrow".to_string(),
                n => format!("in {n} days"),
            };
            view! {
                <button class=format!("gww-holiday caps {}", if n <= 1 { "soon" } else { "" }) on:click=move |e| nav.follow("site:holidays", Some(&e))>
                    <Icon icon=icons::CALENDAR_DAYS size=13.0 />
                    " "
                    {HOLIDAYS[i].name}
                    " "
                    {when}
                </button>
            }
        })
    }
}

fn hit<S: AsRef<str>>(q: &str, texts: &[S]) -> bool {
    q.is_empty() || texts.iter().any(|t| plain_text(t.as_ref()).to_lowercase().contains(q))
}

#[derive(Clone, Copy, PartialEq)]
enum Official {
    Gov,
    Log,
    News,
    Ministry(usize),
}

fn official_tile(o: Official, log_text: Signal<String>) -> AnyView {
    match o {
        Official::Gov => gov_tile(),
        Official::Log => log_tile(log_text),
        Official::News => news_tile(),
        Official::Ministry(i) => ministry_tile(&MINISTRIES[i]),
    }
}

fn section_lines(sections: &[warroom_core::data::Section]) -> impl Iterator<Item = String> + '_ {
    sections.iter().map(|s| format!("{} {}", s.heading, s.text.unwrap_or("")))
}

fn official_hits(q: &str, log_text: &str) -> Vec<Official> {
    let mut out = Vec::new();
    let mut gov: Vec<String> = vec![GOV.title.into(), GOV.summary.into(), GOV.address.into()];
    gov.extend(section_lines(GOV.sections));
    if hit(q, &gov) {
        out.push(Official::Gov);
    }
    if hit(q, &[LOG_TITLE, log_text, LOG_ADDRESS, "war log", "FTL jumps"]) {
        out.push(Official::Log);
    }
    if hit(q, &["Strohmann News", "Dispatches and campaign news", "se://strohmann.news"]) {
        out.push(Official::News);
    }
    for (i, m) in MINISTRIES.iter().enumerate() {
        let mut texts = vec![m.name, m.summary, m.motto, m.address];
        texts.extend(m.sections.iter().map(|s| s.text.unwrap_or("")));
        if hit(q, &texts) {
            out.push(Official::Ministry(i));
        }
    }
    out
}

fn manual_hits(q: &str) -> Vec<usize> {
    MANUALS
        .iter()
        .enumerate()
        .filter(|(_, m)| {
            let mut texts: Vec<String> = vec![m.title.into(), m.summary.into(), m.address.into()];
            for s in m.sections {
                texts.push(s.heading.into());
                texts.push(s.text.unwrap_or("").into());
                texts.extend(s.points.unwrap_or(&[]).iter().map(|p| p.to_string()));
                texts.extend(s.faq.unwrap_or(&[]).iter().map(|f| format!("{} {}", f.q, f.a)));
                for f in s.folds.unwrap_or(&[]) {
                    texts.push(f.title.into());
                    texts.extend(f.points.iter().map(|p| p.to_string()));
                }
            }
            hit(q, &texts)
        })
        .map(|(i, _)| i)
        .collect()
}

fn company_hits(q: &str) -> Vec<usize> {
    COMPANIES
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            let mut texts: Vec<String> =
                vec![c.name.into(), c.trade.into(), c.summary.into(), c.slogan.unwrap_or("").into(), c.address.into()];
            texts.extend(
                c.sections.iter().map(|s| format!("{} {}", s.text.unwrap_or(""), s.products.unwrap_or(&[]).join(" "))),
            );
            hit(q, &texts)
        })
        .map(|(i, _)| i)
        .collect()
}

fn other_than(list: &[&'static Ad], not: Option<&'static Ad>) -> Vec<&'static Ad> {
    list.iter().copied().filter(|a| not.is_none_or(|n| !std::ptr::eq(*a, n))).collect()
}

#[component]
fn Home(log_text: Signal<String>) -> impl IntoView {
    let query = RwSignal::new(String::new());
    let landscape: Vec<&'static Ad> = ADS.iter().filter(|a| a.shape == AdShape::Wide).collect();
    let banner = pick_one(&landscape).copied();
    let tall = pick_one(&ADS.iter().filter(|a| a.shape == AdShape::Tall).collect::<Vec<_>>()).copied();
    let second = pick_one(&other_than(&landscape, banner)).copied();
    let wide = js::use_media(RAIL_QUERY);
    let q = Memo::new(move |_| query.with(|s| s.trim().to_lowercase()));
    let searching = Memo::new(move |_| q.with(|q| !q.is_empty()));
    let official = Memo::new(move |_| q.with(|q| log_text.with(|l| official_hits(q, l))));
    let manuals = Memo::new(move |_| q.with(|q| manual_hits(q)));
    let companies = Memo::new(move |_| q.with(|q| company_hits(q)));
    let none = Memo::new(move |_| {
        official.with(Vec::is_empty) && manuals.with(Vec::is_empty) && companies.with(Vec::is_empty)
    });
    view! {
        <div class="gww-home">
            <header class="gww-mast">
                <img class="gww-mast-mark" src=icon_url("super-earth") alt="" />
                <div class="gww-mast-text">
                    <h2 class="gww-mast-title" tabindex="-1">"Galactic Wide Web"</h2>
                    <p class="caps">"The Federation's net · verified by the Ministry of Truth"</p>
                    <HolidayChip />
                </div>
                <label class="gww-search">
                    <Icon icon=icons::SEARCH size=14.0 />
                    <input
                        type="search"
                        placeholder="Search the G.W.W."
                        aria-label="Search the Galactic Wide Web"
                        prop:value=move || query.get()
                        on:input=move |e| query.set(event_target_value(&e))
                    />
                </label>
            </header>
            <TipTicker />
            <div class="gww-home-grid">
                <div class="gww-dir">
                    <Show when=move || none.get()>
                        <Empty>"No site on the G.W.W. matches “"{move || query.get()}"”."</Empty>
                    </Show>
                    {move || {
                        let list = official.get();
                        (!list.is_empty()).then(|| {
                            let n = list.len();
                            group("Super Earth", n, Some("dir:gov"), list.into_iter().map(|o| official_tile(o, log_text)).collect_view().into_any())
                        })
                    }}
                    {move || {
                        let list = manuals.get();
                        (searching.get() && !list.is_empty()).then(|| {
                            let body = list.iter().map(|&i| manual_tile(&MANUALS[i])).collect_view().into_any();
                            group("Training manuals", list.len(), Some("dir:manuals"), body)
                        })
                    }}
                    {move || (!searching.get()).then(|| {
                        let read = Signal::derive(|| {
                            settings().get(|s| MANUALS.iter().filter(|m| s.gww_read.iter().any(|r| r == m.id)).count())
                        });
                        let tile = move || {
                            let n = read.get();
                            site_tile(
                                "dir:manuals".into(),
                                "se://hc/manuals",
                                "Training manuals".into(),
                                format!(
                                    "How the Second Galactic War works, in {} chapters and {} manuals. {n} of {} read.",
                                    CHAPTERS.len(),
                                    MANUALS.len(),
                                    MANUALS.len()
                                ),
                                view! { <Icon icon=icons::GRADUATION_CAP size=28.0 stroke_width=1.5 /> }.into_any(),
                                Some("var(--cta)".into()),
                                Signal::stored(n == MANUALS.len()),
                                false,
                            )
                        };
                        group("Training manuals", MANUALS.len(), None, tile.into_any())
                    })}
                    {move || (!searching.get()).then(|| banner.map(|a| ad_box(a, None, false)))}
                    {move || {
                        let list = companies.get();
                        (!list.is_empty()).then(|| {
                            group("Corporations", list.len(), Some("dir:corps"), list.iter().map(|&i| company_tile(&COMPANIES[i])).collect_view().into_any())
                        })
                    }}
                    {move || (!searching.get() && !wide.get()).then(|| second.map(|a| ad_box(a, None, false)))}
                </div>
                {move || {
                    wide.get()
                        .then_some(tall)
                        .flatten()
                        .map(|a| view! {
                            <aside class="gww-rail" aria-label="Sponsored">
                                {ad_box(a, None, true)}
                            </aside>
                        })
                }}
            </div>
        </div>
    }
}

fn page_view(page: &GwwPage, heading: HeadingRef, log_text: Signal<String>) -> Option<AnyView> {
    Some(match page {
        GwwPage::Manual { id } => manual_page(MANUALS.iter().find(|m| m.id == id)?, heading),
        GwwPage::Ministry { id } => ministry_page(MINISTRIES.iter().find(|m| m.id == id)?, heading),
        GwwPage::Company { id } => company_page(COMPANIES.iter().find(|c| c.id == id)?, heading),
        GwwPage::Holiday { id } => holiday_page(HOLIDAYS.iter().find(|h| h.id == id)?, heading),
        GwwPage::Site { id } if id == "log" => logs_page(heading),
        GwwPage::Site { id } if id == "holidays" => holidays_page(heading),
        GwwPage::Site { id } => gov_page(id, heading),
        GwwPage::Dir { id } => dir_page(id, heading, log_text),
        GwwPage::Missing { address } => missing_page(address.clone(), heading),
        GwwPage::Home => return None,
    })
}

#[component]
pub fn GwwModule() -> impl IntoView {
    let stack = ui().gww;
    let changes = history().changes;
    let older = history().older;
    let page = Memo::new(move |_| stack.with(|s| s.last().cloned().unwrap_or(GwwPage::Home)));
    let depth = Memo::new(move |_| stack.with(Vec::len));
    let root = NodeRef::<html::Div>::new();
    let heading = StoredValue::new(None::<HeadingRef>);
    // Where each history entry was scrolled to, restored on coming back to the same page.
    let scrolls = StoredValue::new(Vec::<Option<(String, i32)>>::new());
    let scroller = move || {
        root.get_untracked()
            .and_then(|r| r.closest(".scroll").ok().flatten())
            .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    };
    let record = move || {
        let Some(top) = scroller().map(|b| b.scroll_top()) else { return };
        let (len, key) = stack.with_untracked(|s| (s.len(), s.last().map(key_of).unwrap_or_else(|| "home".into())));
        if len == 0 {
            return;
        }
        scrolls.update_value(|v| {
            if v.len() < len {
                v.resize(len, None);
            }
            v[len - 1] = Some((key, top));
        });
    };
    let go = Callback::new(move |p: GwwPage| {
        record();
        let mut now = stack.get_untracked();
        scrolls.update_value(|v| v.truncate(now.len()));
        now.push(p);
        let over = now.len().saturating_sub(50);
        now.drain(..over);
        ui().set_gww(now);
    });
    provide_context(GwwNav { go });
    let back = move || {
        let now = stack.get_untracked();
        if now.len() > 1 {
            record();
            ui().set_gww(now[..now.len() - 1].to_vec());
        }
    };
    let home = move || {
        if page.get_untracked() != GwwPage::Home {
            go.run(GwwPage::Home);
        }
    };
    // Browser Back and Forward change the stack from outside, so positions are kept as they change.
    Effect::new(move |_| {
        let Some(b) = scroller() else { return };
        let on_scroll = wasm_bindgen::closure::Closure::<dyn Fn()>::new(record);
        let _ = b.add_event_listener_with_callback("scroll", on_scroll.as_ref().unchecked_ref());
        let held = send_wrapper::SendWrapper::new((b, on_scroll));
        on_cleanup(move || {
            let (b, f) = &*held;
            let _ = b.remove_event_listener_with_callback("scroll", f.as_ref().unchecked_ref());
        });
    });
    // Not on opening: only navigation scrolls and moves focus.
    Effect::new(move |opened: Option<()>| {
        stack.track();
        if opened.is_none() {
            return;
        }
        let key = key_of(&page.get_untracked());
        let saved = scrolls.with_value(|s| s.get(depth.get_untracked().wrapping_sub(1)).cloned().flatten());
        match (scroller(), saved) {
            (Some(b), Some((k, top))) if k == key => b.set_scroll_top(top),
            _ => {
                if let Some(r) = root.get_untracked() {
                    let opts = web_sys::ScrollIntoViewOptions::new();
                    opts.set_block(web_sys::ScrollLogicalPosition::Start);
                    r.scroll_into_view_with_scroll_into_view_options(&opts);
                }
            }
        }
        let target: Option<web_sys::HtmlElement> = heading
            .get_value()
            .and_then(|h| h.get_untracked())
            .map(|h| h.unchecked_into())
            .or_else(|| root.get_untracked()?.query_selector(".gww-mast-title").ok().flatten()?.dyn_into().ok());
        if let Some(el) = target {
            let opts = web_sys::FocusOptions::new();
            opts.set_prevent_scroll(true);
            let _ = el.focus_with_options(&opts);
        }
        record();
    });
    let log_text = Signal::derive(move || log_line(changes.with(Vec::len), older.get()));
    let view_key = Memo::new(move |_| (depth.get(), key_of(&page.get())));
    view! {
        <div class="gww" node_ref=root>
            <div class="gww-bar">
                <button class="gww-nav" aria-label="Back" title="Back" disabled=move || depth.get() < 2 on:click=move |_| back()>
                    <Icon icon=icons::ARROW_LEFT size=16.0 />
                </button>
                <button
                    class="gww-nav"
                    aria-label="G.W.W. home"
                    title="Home"
                    disabled=move || page.with(|p| *p == GwwPage::Home)
                    on:click=move |_| home()
                >
                    <Icon icon=icons::HOUSE size=16.0 />
                </button>
                <AddressBar page=page />
                <span class="gww-brand-tag caps" aria-hidden="true">
                    "G.W.W."
                </span>
            </div>
            {move || {
                depth.with(|_| ());
                view! { <div class="gww-load" aria-hidden="true"></div> }
            }}
            {move || {
                view_key.with(|_| ());
                let h = HeadingRef::new();
                let body = page_view(&page.get_untracked(), h, log_text);
                heading.set_value(body.is_some().then_some(h));
                view! { <div class="gww-view">{body.unwrap_or_else(|| view! { <Home log_text=log_text /> }.into_any())}</div> }
            }}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::log_line;

    #[test]
    fn counts_the_war_log_as_a_floor_while_older_changes_are_unread() {
        assert_eq!(
            log_line(37, false),
            "The war log and the FTL jumps. 37 planets have changed hands since records began."
        );
        assert_eq!(
            log_line(100, true),
            "The war log and the FTL jumps. 100+ planets have changed hands since records began."
        );
    }
}
