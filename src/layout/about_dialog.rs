use crate::layout::about::About;
use crate::layout::legend::Legend;
use crate::state::{InfoDialog, ui};
use crate::ui::primitives::KeyButton;
use leptos::html;
use leptos::prelude::*;

/// About or the icons legend in a modal over whatever page is open; ESC, the button or a click outside closes it.
#[component]
pub fn AboutDialog() -> impl IntoView {
    let u = ui();
    let dialog = NodeRef::<html::Dialog>::new();
    Effect::new(move |_| {
        let open = u.dialog.get();
        let Some(d) = dialog.get() else { return };
        if open.is_some() && !d.open() {
            let _ = d.show_modal();
            // showModal focuses Close, which scrolls a tall dialog to its end on short screens.
            d.set_scroll_top(0);
        }
        if open.is_none() && d.open() {
            d.close();
        }
    });
    view! {
        <dialog
            node_ref=dialog
            class=move || format!("about-dialog {}", if u.dialog.get() == Some(InfoDialog::Legend) { "wide" } else { "" })
            aria-label=move || if u.dialog.get() == Some(InfoDialog::Legend) { "Icons legend" } else { "About" }
            on:close=move |_| u.show_dialog(None)
            on:click=move |e| {
                if e.target() == e.current_target() {
                    u.show_dialog(None)
                }
            }
        >
            {move || {
                u.dialog.get().map(|d| {
                    view! {
                        {match d {
                            InfoDialog::About => view! { <About /> }.into_any(),
                            InfoDialog::Legend => view! { <Legend /> }.into_any(),
                        }}
                        <div class="about-foot">
                            <KeyButton k="ESC" on_click=Callback::new(move |_| u.show_dialog(None))>
                                "Close"
                            </KeyButton>
                        </div>
                    }
                })
            }}
        </dialog>
    }
}
