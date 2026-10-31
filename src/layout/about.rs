//! About: what the app is, where its data comes from, privacy, and credits.

use crate::config::{APP_NAME, APP_VERSION};
use crate::ui::icons;
use crate::ui::primitives::{OutLink, Panel};
use leptos::prelude::*;
use warroom_core::data::manuals::{WIKI, WIKI_LICENCE};
use warroom_core::data::sites::GWW_SOURCES;

#[component]
pub fn About() -> impl IntoView {
    view! {
        <Panel title=format!("{APP_NAME} · v{APP_VERSION}") icon=icons::INFO>
            <div class="about">
                <p>
                    "A companion for the Helldivers 2 Galactic War: what is being fought, where, how fast it moves, what the Major Order needs, and what has changed."
                </p>
                <h3>"Where the data comes from"</h3>
                <p>
                    "A collector polls the community Helldivers 2 API (api.helldivers2.dev) every one to five minutes, and the game's own API for Galactic Campaigns and planet effects, and stores snapshots in a shared database. The app only reads that database, so every user sees the same history. It reaches back to the war's first days: who held each planet since the first capture on 10 February 2024, Helldivers online and campaign progress since 2 March 2024, and the space station's jumps since it first deployed. What came before the collector first ran, on 21 October 2026, was filled in from the game's public war data. Planet ownership changes, campaign progress, galaxy-wide telemetry and the space station's jumps are kept; that record is what powers each planet's war history."
                </p>
                <h3>"Freshness"</h3>
                <p>
                    "The status strip under the top bar turns to "
                    <b style="color: var(--warn);">"STALE"</b>
                    " when no new data has arrived for ten minutes, and to "
                    <b style="color: var(--bad);">"LINK LOST"</b>
                    " when the database can't be reached. Either way the last known data stays on screen."
                </p>
                <h3>"Privacy"</h3>
                <p>
                    "The app collects no personal data: no account, analytics, ads or tracking. Settings and pins stay on this device, alerts are worked out here, and the only thing downloaded is war data from the project's read-only database. The full policy is in PRIVACY.md at github.com/zearkreast/WarRoom."
                </p>
                <h3>"Not affiliated"</h3>
                <p>
                    "A fan project, not affiliated with Arrowhead Game Studios or Sony Interactive Entertainment. Map artwork is drawn for the app. The faction and sub-faction emblems and the DSS glyph (as traced by Helldivers Wiki editors), the DSS's summary hologram, the biome landscapes and the black hole and fractured planet pictures are the game's, taken from the Helldivers Wiki (helldivers.wiki.gg), and belong to their owners. Effect names come from the Helldivers 2 community's data set (github.com/helldivers-2/json, MIT licence)."
                </p>
                <h3>"Credits"</h3>
                <p>
                    "The G.W.W.'s training manuals, sites and holiday pages are adapted from the"
                    " "
                    <OutLink href=WIKI.home.url>{WIKI.home.title}</OutLink>
                    " and written to read as the Federation's own net, so they carry no source lines; the wiki pages they draw on are listed below. The wiki's text is shared under"
                    " "
                    <OutLink href=WIKI_LICENCE.url>{WIKI_LICENCE.title}</OutLink>
                    ", and so are the pages adapted from it. Training manual tips, slogans, decrees, dispatches, carols, poster and broadcast lines are quoted from the game, as collected on the wiki."
                </p>
                <ul class="about-sources">
                    {GWW_SOURCES.iter().map(|s| view! { <li><OutLink href=s.url>{s.title}</OutLink></li> }).collect_view()}
                </ul>
                <p>
                    "The ministry, campaign, difficulty and stratagem icons and the SEAF emblem (vector tracings by the wiki's contributors), corporation logos, product pictures and city adverts are the game's, taken from the Helldivers Wiki, and belong to Arrowhead Game Studios and Sony Interactive Entertainment. Martian Armory's logo is drawn for the app."
                </p>
            </div>
        </Panel>
    }
}
