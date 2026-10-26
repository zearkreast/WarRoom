//! Makes the browser's and the phone's Back button step back through the app.

use crate::state::{CityRef, HellpadModule, InfoDialog, View, ui};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use warroom_core::gww_nav::GwwPage;

/// What Back and Forward step through: the screen, the Hellpad module, the G.W.W. page and the open overlay.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Place {
    pub view: View,
    pub module: Option<HellpadModule>,
    pub gww: Vec<GwwPage>,
    pub detail: Option<i64>,
    #[serde(default)]
    pub city: Option<CityRef>,
    pub dialog: Option<InfoDialog>,
    pub dispatch_open: bool,
}

/// A history entry's state: its place, and the place of the entry before it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Entry {
    warroom: u8,
    place: Place,
    prev: Option<Place>,
}

fn entry_of(state: &Value) -> Option<Entry> {
    serde_json::from_value::<Entry>(state.clone()).ok().filter(|e| e.warroom == 1)
}

pub trait HistoryLike {
    fn state(&self) -> Value;
    fn push_state(&mut self, data: Value);
    fn replace_state(&mut self, data: Value);
    fn back(&mut self);
}

/// Keeps the history in step with the app. A new place is pushed; returning to the place before is a step
/// back, so closing an overlay or the G.W.W.'s own Back leaves nothing for Back to reopen. A reload resumes
/// the entry's place.
pub struct Sync<H: HistoryLike> {
    pub history: H,
    entry: Entry,
    /// A step back of ours is in flight; changes made meanwhile are caught up when it lands.
    stepping: bool,
}

impl<H: HistoryLike> Sync<H> {
    /// Starts from the current entry, returning the place a reload should resume, if any.
    pub fn new(mut history: H, current: Place) -> (Self, Option<Place>) {
        match entry_of(&history.state()) {
            Some(entry) => {
                let resume = entry.place.clone();
                (Sync { history, entry, stepping: false }, Some(resume))
            }
            None => {
                let entry = Entry { warroom: 1, place: current, prev: None };
                history.replace_state(serde_json::to_value(&entry).unwrap());
                (Sync { history, entry, stepping: false }, None)
            }
        }
    }

    pub fn on_change(&mut self, now: Place) {
        if self.stepping || now == self.entry.place {
            return;
        }
        if self.entry.prev.as_ref() == Some(&now) {
            self.stepping = true;
            self.history.back();
            return;
        }
        // One planet page opened from another takes its entry, so closing it steps back past both.
        let swapped = now.detail.is_some() && self.entry.place.detail.is_some();
        if swapped && (Place { detail: self.entry.place.detail, ..now.clone() }) == self.entry.place {
            self.entry.place = now;
            self.history.replace_state(serde_json::to_value(&self.entry).unwrap());
            return;
        }
        let prev = std::mem::replace(&mut self.entry.place, now);
        self.entry.prev = Some(prev);
        self.history.push_state(serde_json::to_value(&self.entry).unwrap());
    }

    /// A popstate landed. Returns the place to show, unless it was our own step back, which catches up on
    /// `current` instead.
    pub fn on_pop(&mut self, state: &Value, current: impl FnOnce() -> Place) -> Option<Place> {
        let entry = entry_of(state)?;
        let ours = self.stepping;
        self.stepping = false;
        self.entry = entry;
        if ours {
            self.on_change(current());
            None
        } else {
            Some(self.entry.place.clone())
        }
    }
}

pub fn place_now() -> Place {
    let u = ui();
    Place {
        view: u.view.get_untracked(),
        module: u.module.get_untracked(),
        gww: u.gww.get_untracked(),
        detail: u.detail.get_untracked(),
        city: u.city.get_untracked(),
        dialog: u.dialog.get_untracked(),
        dispatch_open: u.dispatch_open.get_untracked(),
    }
}

fn set_place(p: Place) {
    let u = ui();
    u.view.set(p.view);
    u.module.set(p.module);
    // An unchanged G.W.W. page list is left alone, or the G.W.W. would take it for navigation.
    if u.gww.get_untracked() != p.gww {
        u.gww.set(p.gww);
    }
    u.detail.set(p.detail);
    u.city.set(p.city);
    u.dialog.set(p.dialog);
    u.dispatch_open.set(p.dispatch_open);
    u.options.set(None);
}

mod browser {
    use super::*;
    use crate::js;
    use std::cell::RefCell;
    use wasm_bindgen::JsValue;
    use wasm_bindgen::prelude::*;

    pub struct Browser;

    fn to_js(v: &Value) -> JsValue {
        js_sys::JSON::parse(&v.to_string()).unwrap_or(JsValue::NULL)
    }

    pub fn from_js(v: &JsValue) -> Value {
        js_sys::JSON::stringify(v)
            .ok()
            .and_then(|s| s.as_string())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(Value::Null)
    }

    impl HistoryLike for Browser {
        fn state(&self) -> Value {
            js::window().history().ok().and_then(|h| h.state().ok()).map_or(Value::Null, |s| from_js(&s))
        }
        fn push_state(&mut self, data: Value) {
            if let Ok(h) = js::window().history() {
                let _ = h.push_state(&to_js(&data), "");
            }
        }
        fn replace_state(&mut self, data: Value) {
            if let Ok(h) = js::window().history() {
                let _ = h.replace_state(&to_js(&data), "");
            }
        }
        fn back(&mut self) {
            if let Ok(h) = js::window().history() {
                let _ = h.back();
            }
        }
    }

    thread_local! {
        static SYNC: RefCell<Option<Sync<Browser>>> = const { RefCell::new(None) };
    }

    /// Puts back the place a reload left, before the first render so the default screen never shows,
    /// then follows every change of place.
    pub fn start() {
        let (sync, resume) = Sync::new(Browser, place_now());
        if let Some(p) = resume {
            set_place(p);
        }
        SYNC.with_borrow_mut(|s| *s = Some(sync));
        let u = ui();
        Effect::new(move |_| {
            let now = Place {
                view: u.view.get(),
                module: u.module.get(),
                gww: u.gww.get(),
                detail: u.detail.get(),
                city: u.city.get(),
                dialog: u.dialog.get(),
                dispatch_open: u.dispatch_open.get(),
            };
            SYNC.with_borrow_mut(|s| s.as_mut().map(|s| s.on_change(now)));
        });
        let on_pop = Closure::<dyn Fn(web_sys::PopStateEvent)>::new(|e: web_sys::PopStateEvent| {
            let state = from_js(&e.state());
            let place = SYNC.with_borrow_mut(|s| s.as_mut().and_then(|s| s.on_pop(&state, place_now)));
            if let Some(p) = place {
                set_place(p);
            }
        });
        let _ = js::window().add_event_listener_with_callback("popstate", on_pop.as_ref().unchecked_ref());
        on_pop.forget();
    }
}

pub use browser::start;

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn at(f: impl FnOnce(&mut Place)) -> Place {
        let mut p = Place {
            view: View::Campaigns,
            module: None,
            gww: vec![GwwPage::Home],
            detail: None,
            city: None,
            dialog: None,
            dispatch_open: false,
        };
        f(&mut p);
        p
    }

    fn home() -> Place {
        at(|_| {})
    }

    #[derive(Default)]
    struct Book {
        entries: Vec<Value>,
        i: usize,
        pending: usize,
    }

    /// A session history whose back() lands only when `land()` is called, as popstate arrives later in a browser.
    #[derive(Clone, Default)]
    struct Fake(Rc<RefCell<Book>>);

    impl HistoryLike for Fake {
        fn state(&self) -> Value {
            let b = self.0.borrow();
            b.entries.get(b.i).cloned().unwrap_or(Value::Null)
        }
        fn push_state(&mut self, data: Value) {
            let mut b = self.0.borrow_mut();
            let i = b.i;
            b.entries.truncate(i + 1);
            b.entries.push(data);
            b.i += 1;
        }
        fn replace_state(&mut self, data: Value) {
            let mut b = self.0.borrow_mut();
            let i = b.i;
            b.entries[i] = data;
        }
        fn back(&mut self) {
            self.0.borrow_mut().pending += 1;
        }
    }

    struct App {
        h: Fake,
        sync: Sync<Fake>,
        place: Place,
    }

    impl App {
        fn new(h: &Fake, first: Place) -> App {
            if h.0.borrow().entries.is_empty() {
                h.0.borrow_mut().entries.push(Value::Null);
            }
            let (sync, resume) = Sync::new(h.clone(), first.clone());
            App { h: h.clone(), sync, place: resume.unwrap_or(first) }
        }
        fn go(&mut self, p: Place) {
            self.place = p.clone();
            self.sync.on_change(p);
        }
        fn pop(&mut self) {
            let state = self.h.state();
            let current = self.place.clone();
            if let Some(p) = self.sync.on_pop(&state, || current) {
                self.place = p;
            }
        }
        fn land(&mut self) {
            loop {
                {
                    let mut b = self.h.0.borrow_mut();
                    if b.pending == 0 {
                        break;
                    }
                    b.pending -= 1;
                    b.i -= 1;
                }
                self.pop();
            }
        }
        fn browser_back(&mut self) {
            self.h.0.borrow_mut().i -= 1;
            self.pop();
        }
        fn index(&self) -> usize {
            self.h.0.borrow().i
        }
    }

    #[test]
    fn pushes_each_new_place_and_restores_it_on_back() {
        let h = Fake::default();
        let mut a = App::new(&h, home());
        a.go(at(|p| p.view = View::Map));
        a.go(at(|p| {
            p.view = View::Map;
            p.detail = Some(7);
        }));
        assert_eq!(a.index(), 2);
        a.browser_back();
        assert_eq!(a.place, at(|p| p.view = View::Map));
        a.browser_back();
        assert_eq!(a.place, home());
    }

    #[test]
    fn turns_a_return_to_the_place_before_into_a_step_back_not_a_new_entry() {
        let h = Fake::default();
        let mut a = App::new(&h, home());
        let gww = || {
            at(|p| {
                p.view = View::Hellpad;
                p.module = Some(HellpadModule::Gww);
            })
        };
        a.go(gww());
        let mut manual = gww();
        manual.gww.push(GwwPage::Manual { id: "basics".into() });
        a.go(manual);
        a.go(gww());
        a.land();
        assert_eq!(a.index(), 1);
        assert_eq!(h.0.borrow().entries.len(), 3);
        assert_eq!(a.place, gww());
    }

    #[test]
    fn lets_one_planet_page_opened_from_another_take_its_entry() {
        let h = Fake::default();
        let mut a = App::new(&h, home());
        a.go(at(|p| p.detail = Some(261)));
        a.go(at(|p| p.detail = Some(154)));
        assert_eq!(a.index(), 1);
        a.go(home());
        a.land();
        assert_eq!(a.index(), 0);
        assert_eq!(a.place, home());
    }

    #[test]
    fn catches_up_on_a_change_made_while_its_step_back_is_in_flight() {
        let h = Fake::default();
        let mut a = App::new(&h, home());
        a.go(at(|p| p.dialog = Some(InfoDialog::About)));
        a.go(home());
        a.go(at(|p| p.view = View::News));
        a.land();
        assert_eq!(a.index(), 1);
        assert_eq!(a.place, at(|p| p.view = View::News));
        a.browser_back();
        assert_eq!(a.place, home());
    }

    #[test]
    fn resumes_the_entrys_place_after_a_reload() {
        let h = Fake::default();
        let mut a = App::new(&h, home());
        a.go(at(|p| p.view = View::Overview));
        let mut again = App::new(&h, home());
        assert_eq!(again.place, at(|p| p.view = View::Overview));
        again.go(home());
        again.land();
        assert_eq!(again.index(), 0);
    }
}
