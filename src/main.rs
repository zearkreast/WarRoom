use leptos::prelude::*;
use wasm_bindgen::JsCast;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to(
        document().get_element_by_id("root").unwrap().unchecked_into(),
        || view! { <p>"WarCommand Tracker"</p> },
    )
    .forget();
}
