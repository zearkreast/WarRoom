//! Biome colours and landscape pictures, and enemy resistance graded from regeneration.

use crate::api::Planet;
use crate::galaxy::{Anomaly, anomaly_of};
use regex_lite::Regex;
use std::sync::LazyLock;

/// Two colours standing in for a biome's look, sky/atmosphere and ground.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BiomeTint {
    pub sky: &'static str,
    pub ground: &'static str,
}

const fn tint(sky: &'static str, ground: &'static str) -> BiomeTint {
    BiomeTint { sky, ground }
}

// The game's 25 biomes by their API names, coloured from their landscapes.
const BIOMES: &[(&str, BiomeTint)] = &[
    ("plains", tint("#1c2a1a", "#8fae5e")),
    ("ionic crimson", tint("#2a0d14", "#c4405a")),
    ("icy glaciers", tint("#13283a", "#a9d2ef")),
    ("desert dunes", tint("#3a2a12", "#d9a54a")),
    ("rocky canyons", tint("#2e1e14", "#b47a4a")),
    ("basic swamp", tint("#16231a", "#6f8f4f")),
    ("tundra", tint("#1d2622", "#9aa88a")),
    ("boneyard", tint("#1a2230", "#c7d3dd")),
    ("ethereal jungle", tint("#1e1430", "#9f7fd0")),
    ("volcanic jungle", tint("#1b2410", "#5fa04a")),
    ("acidic badlands", tint("#1d2410", "#b6d33b")),
    ("haunted swamp", tint("#16141e", "#6e6a86")),
    ("scorched moor", tint("#331a0c", "#d0743a")),
    ("ionic jungle", tint("#0e1f2a", "#4fa8c8")),
    ("deadlands", tint("#1a1a1e", "#8a8a92")),
    ("moon", tint("#1a1c22", "#9a9ca6")),
    ("desert cliffs", tint("#3a2412", "#c98a4a")),
    ("magma", tint("#3a1208", "#e2582a")),
    ("hive world", tint("#24180a", "#c98a2c")),
    ("desert oasis", tint("#2a2a14", "#d0b860")),
    ("deciduous autumn forest", tint("#2a1a0e", "#d08a3a")),
    ("deciduous forest", tint("#16261a", "#6fb05a")),
    ("super earth", tint("#0d1f2e", "#3f8cc2")),
    ("supercolony", tint("#24100a", "#b8582c")),
    ("cyberstan megafactory", tint("#1a1416", "#8a4a4a")),
];

// Names the game may add later still get a fitting colour: matched against the name, first hit wins.
static TINTS: LazyLock<Vec<(Regex, BiomeTint)>> = LazyLock::new(|| {
    [
        (r"magma|volcan|lava|scorch|hellmire", tint("#3a1208", "#e2582a")),
        (r"desert|sand|dune|mesa|canyon", tint("#3a2a12", "#d9a54a")),
        (r"winter|ice|snow|tundra|frost|glacier|arctic", tint("#13283a", "#a9d2ef")),
        (r"swamp|bog|marsh|moor|mire", tint("#16231a", "#7d9b5c")),
        (r"jungle|rainforest|forest|primordial|ethereal", tint("#0f2418", "#4fb06a")),
        (r"toxic|acid|blight|haz", tint("#1d2410", "#b6d33b")),
        (r"crimson|blood|red", tint("#2a0d14", "#c4405a")),
        (r"highland|plateau|plains|grass", tint("#1a2418", "#9cab6e")),
        (r"rock|desolate|barren|waste|crater|moon|asteroid", tint("#1a1c22", "#8a8c96")),
        (r"ocean|sea|water|coast|archipelago", tint("#0d1f2e", "#3f8cc2")),
        (r"colony|hive|super", tint("#24180a", "#c98a2c")),
    ]
    .into_iter()
    .map(|(re, t)| (Regex::new(&format!("(?i){re}")).unwrap(), t))
    .collect()
});

pub const DEFAULT_TINT: BiomeTint = tint("#141a24", "#6b7d90");

/// File name of a biome's landscape picture: its name lowercased, other runs of characters as dashes.
pub fn biome_slug(name: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in name.to_lowercase().chars() {
        if c.is_ascii_lowercase() {
            out.push(c);
            dash = false;
        } else if !dash {
            out.push('-');
            dash = true;
        }
    }
    out
}

pub fn biome_tint(biome: Option<&str>) -> BiomeTint {
    let Some(biome) = biome.filter(|b| !b.is_empty()) else { return DEFAULT_TINT };
    let lower = biome.to_lowercase();
    BIOMES
        .iter()
        .find(|(n, _)| *n == lower)
        .map(|(_, t)| *t)
        .or_else(|| TINTS.iter().find(|(re, _)| re.is_match(biome)).map(|(_, t)| *t))
        .unwrap_or(DEFAULT_TINT)
}

/// What a planet the war has destroyed looks like instead of its biome: colours and picture file.
pub fn look_of(anomaly: Anomaly) -> Option<(BiomeTint, &'static str)> {
    match anomaly {
        Anomaly::Wormhole => Some((tint("#0a0a2a", "#6a5cff"), "meridian-black-hole")),
        Anomaly::Blackhole => Some((tint("#050508", "#7d93ad"), "black-hole")),
        Anomaly::Fractured => Some((tint("#140a10", "#e0553a"), "fractured")),
        Anomaly::Scorched => None,
    }
}

/// A planet's look: the black hole or debris it has become, else None for its biome's.
pub fn planet_look(p: &Planet, effect_ids: &[i64]) -> Option<(BiomeTint, &'static str)> {
    anomaly_of(&p.name, effect_ids).and_then(look_of)
}

/// Enemy regeneration in % of max health per hour.
pub fn regen_pct_per_hour(regen_per_second: f64, max_health: f64) -> f64 {
    if max_health == 0.0 || max_health.is_nan() { 0.0 } else { (regen_per_second / max_health) * 360_000.0 }
}

/// Enemy resistance as the game grades it, from its regeneration in %/h.
pub fn resistance_level(pct_per_hour: f64) -> (&'static str, &'static str) {
    if pct_per_hour < 1.0 {
        ("Low", "var(--good)")
    } else if pct_per_hour < 2.5 {
        ("Average", "var(--warn)")
    } else {
        ("High", "var(--bad)")
    }
}
