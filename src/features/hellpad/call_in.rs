//! The Call In Super Destroyer stratagem, keyed in on the Hellpad mark, which opens its G.W.W. manual.

use crate::js;
use crate::state::{HellpadModule, View, ui};
use crate::ui::please_wait::{PLEASE_WAIT_MS, PleaseWait};
use leptos::html;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use warroom_core::gww_nav::GwwPage;
use warroom_core::stratagems::{Arrow, CALL_IN, arrow_of, step};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

fn arrow_name(a: Arrow) -> &'static str {
    match a {
        Arrow::Up => "up",
        Arrow::Down => "down",
        Arrow::Left => "left",
        Arrow::Right => "right",
    }
}

/// Opens the G.W.W. on its Super Destroyer manual, in one step so Back returns to the Hellpad.
fn open_manual() {
    let u = ui();
    let there = u.gww.with_untracked(|g| matches!(g.last(), Some(GwwPage::Manual { id }) if id == "super-destroyers"));
    if !there {
        u.gww.update(|g| {
            g.push(GwwPage::Manual { id: "super-destroyers".into() });
            let n = g.len();
            if n > 50 {
                g.drain(..n - 50);
            }
        });
    }
    u.view.set(View::Hellpad);
    u.module.set(Some(HellpadModule::Gww));
    u.options.set(None);
    u.detail.set(None);
}

/// Keying in Call In Super Destroyer: how far it has got, a miss, and the hold once it is complete.
#[derive(Clone, Copy)]
pub struct CallIn {
    pub at: ReadSignal<usize>,
    pub miss: ReadSignal<bool>,
    pub waiting: ReadSignal<bool>,
    set_at: WriteSignal<usize>,
    set_miss: WriteSignal<bool>,
    set_waiting: WriteSignal<bool>,
    done: StoredValue<usize>,
    flash: StoredValue<Option<TimeoutHandle>>,
}

impl CallIn {
    /// Whether the press counted towards the code; idle, only its first arrow does.
    pub fn press(&self, a: Arrow) -> bool {
        let done = self.done.get_value();
        let len = CALL_IN.code.len();
        if done == len {
            return true;
        }
        let next = step(done, a, CALL_IN.code);
        if next == 0 && done == 0 {
            return false;
        }
        if next <= done && done > 0 {
            self.set_miss.set(true);
            if let Some(h) = self.flash.get_value() {
                h.clear();
            }
            let set_miss = self.set_miss;
            self.flash.set_value(
                set_timeout_with_handle(move || set_miss.set(false), std::time::Duration::from_millis(400)).ok(),
            );
        }
        self.done.set_value(next);
        self.set_at.set(next);
        if next == len {
            if let Some(h) = self.flash.get_value() {
                h.clear();
            }
            self.set_miss.set(false);
            self.set_waiting.set(true);
        }
        true
    }

    pub fn reset(&self) {
        self.done.set_value(0);
        self.set_at.set(0);
        self.set_waiting.set(false);
    }
}

pub fn use_call_in() -> CallIn {
    let (at, set_at) = signal(0usize);
    let (miss, set_miss) = signal(false);
    let (waiting, set_waiting) = signal(false);
    let flash = StoredValue::new(None::<TimeoutHandle>);
    on_cleanup(move || {
        if let Some(h) = flash.get_value() {
            h.clear();
        }
    });
    CallIn { at, miss, waiting, set_at, set_miss, set_waiting, done: StoredValue::new(0), flash }
}

const USES_ARROWS: &str =
    "input, textarea, select, [contenteditable='true'], [role='slider'], [role='listbox'], [role='menu']";

/// The code shown steady; keying it in greys it, and each correct arrow lights up again.
#[component]
pub fn CallInCode(c: CallIn) -> impl IntoView {
    let boxed = NodeRef::<html::Div>::new();
    // W A S D, being letters, only count with the code itself focused.
    let handler = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
        if e.ctrl_key() || e.meta_key() || e.alt_key() {
            return;
        }
        let Some(a) = arrow_of(&e.key()) else { return };
        let doc = js::document();
        if doc.query_selector("dialog[open]").ok().flatten().is_some() {
            return;
        }
        let active = doc.active_element();
        let on_code = match (&active, boxed.get_untracked()) {
            (Some(el), Some(b)) => el == b.unchecked_ref::<web_sys::Element>(),
            _ => false,
        };
        let in_field = active.as_ref().is_some_and(|el| el.closest(USES_ARROWS).ok().flatten().is_some());
        if !on_code && (!e.key().starts_with("Arrow") || in_field) {
            return;
        }
        if c.press(a) && on_code {
            e.prevent_default();
        }
    });
    let w = js::window();
    let _ = w.add_event_listener_with_callback("keydown", handler.as_ref().unchecked_ref());
    let handler = SendWrapper::new(handler);
    on_cleanup(move || {
        let _ = js::window().remove_event_listener_with_callback("keydown", handler.as_ref().unchecked_ref());
    });
    let len = CALL_IN.code.len();
    let keying = move || {
        let at = c.at.get();
        at > 0 && at < len
    };
    let label =
        format!("{}: {}", CALL_IN.name, CALL_IN.code.iter().map(|a| arrow_name(*a)).collect::<Vec<_>>().join(", "));
    view! {
        <div
            node_ref=boxed
            class=move || format!("hp-code {} {}", if keying() { "keying" } else { "" }, if c.miss.get() { "miss" } else { "" })
            tabindex="0"
            role="group"
            aria-label=label
        >
            <span class="hp-keys" aria-hidden="true">
                {CALL_IN
                    .code
                    .iter()
                    .enumerate()
                    .map(|(i, a)| {
                        let name = arrow_name(*a);
                        view! {
                            <i class=move || format!("hp-key {name} {}", if !keying() || i < c.at.get() { "on" } else { "" })></i>
                        }
                    })
                    .collect_view()}
            </span>
            <span class="sr-only" aria-live="polite">
                {move || if keying() { format!("{} of {len}", c.at.get()) } else { String::new() }}
            </span>
        </div>
    }
}

/// The Hellpad mark's four arrows as keys, for touch screens.
#[component]
pub fn MarkPad(c: CallIn) -> impl IntoView {
    let pad = [(Arrow::Up, "Up"), (Arrow::Right, "Right"), (Arrow::Down, "Down"), (Arrow::Left, "Left")];
    view! {
        <span class="hp-mark-pad">
            {pad
                .map(|(arrow, label)| view! {
                    <button
                        class=format!("hp-mark-key {}", arrow_name(arrow))
                        aria-label=format!("Stratagem key: {label}")
                        on:click=move |_| {
                            c.press(arrow);
                        }
                    ></button>
                })
                .collect_view()}
        </span>
    }
}

/// "Please wait democratically", then the Super Destroyer manual; Esc calls it off.
#[component]
pub fn CallInWait(c: CallIn) -> impl IntoView {
    let dialog = NodeRef::<html::Dialog>::new();
    let timer = StoredValue::new(None::<TimeoutHandle>);
    Effect::new(move |_| {
        let open = c.waiting.get();
        let Some(d) = dialog.get() else { return };
        if let Some(h) = timer.get_value() {
            h.clear();
        }
        if !open {
            if d.open() {
                d.close();
            }
            return;
        }
        let _ = d.show_modal();
        let d2 = SendWrapper::new(d.clone());
        timer.set_value(
            set_timeout_with_handle(
                move || {
                    d2.close();
                    c.reset();
                    open_manual();
                },
                std::time::Duration::from_millis(PLEASE_WAIT_MS as u64),
            )
            .ok(),
        );
    });
    on_cleanup(move || {
        if let Some(h) = timer.get_value() {
            h.clear();
        }
    });
    view! {
        <dialog node_ref=dialog class="briefing waiting" aria-label=CALL_IN.name on:close=move |_| c.reset()>
            <Show when=move || c.waiting.get()>
                <PleaseWait />
            </Show>
        </dialog>
    }
}
