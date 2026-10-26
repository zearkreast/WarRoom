//! Pictures shipped in src/assets, served at assets/ next to the page.

mod list {
    include!(concat!(env!("OUT_DIR"), "/assets.rs"));
}

/// The URL of an asset by its path under src/assets, if it exists.
pub fn url(path: &str) -> Option<String> {
    list::FILES.binary_search(&path).ok().map(|_| format!("assets/{path}"))
}

/// The raw text of an SVG asset, for inlining.
pub fn svg(path: &str) -> Option<&'static str> {
    EMBLEMS.iter().find(|(p, _)| *p == path).map(|(_, s)| *s)
}

macro_rules! emblems {
    ($($name:literal),* $(,)?) => {
        &[$((concat!("emblems/", $name, ".svg"), include_str!(concat!("assets/emblems/", $name, ".svg")))),*]
    };
}

/// The faction and sub-faction emblems, inlined so they take the colour of the text around them.
const EMBLEMS: &[(&str, &str)] = emblems![
    "appropriators",
    "automaton",
    "automaton-station",
    "cyborgs",
    "dss",
    "illuminate",
    "incineration-corps",
    "jet-brigade",
    "mindless-masses",
    "predator-strain",
    "rupture-strain",
    "spore-burst-strain",
    "super-earth",
    "terminids",
    "vote-snatchers",
];
