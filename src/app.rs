use crate::features::campaigns::page::CampaignsPage;
use crate::features::hellpad::page::HellpadPage;
use crate::features::map::page::MapPage;
use crate::features::news::page::NewsPage;
use crate::features::overview::page::OverviewPage;
use crate::features::settings::page::SettingsPage;
use crate::layout::shell::Shell;
use crate::state::{View, ui};
use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    // A memo, so setting the same view again (opening a Hellpad module) doesn't remount the page.
    let view = Memo::new(move |_| ui().view.get());
    view! {
        <Shell>
            {move || match view.get() {
                View::Campaigns => view! { <CampaignsPage /> }.into_any(),
                View::Map => view! { <MapPage /> }.into_any(),
                View::Hellpad => view! { <HellpadPage /> }.into_any(),
                View::Overview => view! { <OverviewPage /> }.into_any(),
                View::News => view! { <NewsPage /> }.into_any(),
                View::Settings => view! { <SettingsPage /> }.into_any(),
            }}
        </Shell>
    }
}
