use crate::assets;
use regex_lite::Regex;
use std::sync::LazyLock;
use warroom_core::api::TacticalAction;
use warroom_core::effects::{EffectKind, PlanetEffect};
use warroom_core::time::parse_ms;

/// Every hologram by file name; the map masks its scanlines to each one's outline.
pub const PAWNS: [&str; 18] = [
    "bot_cyborgs",
    "bot_incincorp",
    "bot_jetbrigade",
    "bug_dragonroach",
    "bug_hivelord",
    "bug_predator",
    "bug_rupture",
    "bug_sporeburst",
    "dss_active",
    "dss_cooldown",
    "dss_default",
    "dss_offline",
    "ill_appropriators",
    "ill_greathost",
    "ill_mindlessmasses",
    "ill_spearhead",
    "ill_votesnatchers",
    "seaf",
];

pub fn pawn_href(key: &str) -> Option<String> {
    assets::url(&format!("pawns/{key}.png"))
}

fn re(p: &str) -> Regex {
    Regex::new(&format!("(?i){p}")).unwrap()
}

/// Sub-faction names, as the effect list spells them, to their hologram.
static UNITS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    [
        ("jet brigade", "bot_jetbrigade"),
        ("incineration corps", "bot_incincorp"),
        ("cyborgs", "bot_cyborgs"),
        ("predator strain", "bug_predator"),
        ("spore burst strain", "bug_sporeburst"),
        ("rupture strain", "bug_rupture"),
        ("dragonroach", "bug_dragonroach"),
        ("hive lord", "bug_hivelord"),
        ("great host", "ill_greathost"),
        ("mindless masses", "ill_mindlessmasses"),
        ("appropriators", "ill_appropriators"),
        ("vote snatchers", "ill_votesnatchers"),
        ("spearhead", "ill_spearhead"),
    ]
    .into_iter()
    .map(|(p, k)| (re(p), k))
    .collect()
});

#[derive(Clone, Debug, PartialEq)]
pub struct Art {
    pub key: String,
    pub href: String,
}

/// The hologram drawn over the map for an effect, if it has one; threats and points of interest don't.
pub fn pawn_art(e: &PlanetEffect) -> Option<Art> {
    let key = match e.kind {
        EffectKind::Seaf => Some("seaf"),
        EffectKind::Subfaction => UNITS.iter().find(|(re, _)| re.is_match(&e.name)).map(|(_, k)| *k),
        _ => None,
    }?;
    Some(Art { key: key.into(), href: pawn_href(key)? })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DssState {
    Active,
    Cooldown,
    Offline,
    Default,
}

impl DssState {
    pub fn as_str(self) -> &'static str {
        match self {
            DssState::Active => "active",
            DssState::Cooldown => "cooldown",
            DssState::Offline => "offline",
            DssState::Default => "default",
        }
    }

    /// The station's light in each state, on the map and in its hologram on the station page.
    pub fn tone(self) -> &'static str {
        match self {
            DssState::Active => "var(--cta)",
            DssState::Cooldown => "var(--bad)",
            DssState::Offline => "#8d949b",
            DssState::Default => "var(--f-humans)",
        }
    }
}

/// The station's hologram: yellow while an action is deployed, red while one cools down, grey when none
/// can be prepared, deployed or cooled down (the API has no offline flag of its own), blue otherwise.
pub fn dss_state(actions: &[TacticalAction], now: f64) -> DssState {
    let live = |a: &TacticalAction| parse_ms(&a.status_expire) > now;
    if actions.iter().any(|a| a.status == 2 && live(a)) {
        return DssState::Active;
    }
    if actions.iter().any(|a| a.status == 3 && live(a)) {
        return DssState::Cooldown;
    }
    if !actions.iter().any(|a| (1..=3).contains(&a.status)) {
        return DssState::Offline;
    }
    DssState::Default
}

pub fn dss_art(state: DssState) -> Option<Art> {
    let key = format!("dss_{}", state.as_str());
    Some(Art { href: pawn_href(&key)?, key })
}

/// Tactical action names, as the station spells them, to their icon.
static ACTIONS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    [("eagle storm", "eagle_storm"), ("orbital blockade", "orbital_blockade"), ("heavy ordnance", "heavy_ordnance")]
        .into_iter()
        .map(|(p, k)| (re(p), k))
        .collect()
});

#[derive(Clone, Debug, PartialEq)]
pub struct DssInfo {
    pub state: DssState,
    /// Actions deployed right now that have an icon.
    pub active: Vec<Art>,
}

pub fn dss_info(actions: &[TacticalAction], now: f64) -> DssInfo {
    let active = actions
        .iter()
        .filter(|a| a.status == 2 && parse_ms(&a.status_expire) > now)
        .filter_map(|a| {
            let key = ACTIONS.iter().find(|(re, _)| re.is_match(&a.name)).map(|(_, k)| *k)?;
            Some(Art { key: key.into(), href: assets::url(&format!("dss-actions/{key}.png"))? })
        })
        .collect();
    DssInfo { state: dss_state(actions, now), active }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_every_hologram_and_action_badge() {
        let missing: Vec<&str> = PAWNS.iter().copied().filter(|k| pawn_href(k).is_none()).collect();
        assert!(missing.is_empty(), "{missing:?}");
        for key in ["eagle_storm", "orbital_blockade", "heavy_ordnance"] {
            assert!(assets::url(&format!("dss-actions/{key}.png")).is_some(), "{key}");
        }
    }
}
