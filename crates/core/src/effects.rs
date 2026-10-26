use regex_lite::Regex;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// What an effect is to the player, which decides where it shows:
/// subfaction, threat, seaf and poi get a map pawn; look and gloom change how the planet is drawn;
/// the rest only appear in the planet's operational parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EffectKind {
    Subfaction,
    Threat,
    Seaf,
    Poi,
    Dss,
    Arsenal,
    Condition,
    Event,
    Look,
    Gloom,
    Hidden,
}

impl EffectKind {
    pub fn as_str(self) -> &'static str {
        match self {
            EffectKind::Subfaction => "subfaction",
            EffectKind::Threat => "threat",
            EffectKind::Seaf => "seaf",
            EffectKind::Poi => "poi",
            EffectKind::Dss => "dss",
            EffectKind::Arsenal => "arsenal",
            EffectKind::Condition => "condition",
            EffectKind::Event => "event",
            EffectKind::Look => "look",
            EffectKind::Gloom => "gloom",
            EffectKind::Hidden => "hidden",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlanetEffect {
    pub id: i64,
    pub kind: EffectKind,
    pub name: String,
    pub description: String,
}

#[derive(Deserialize)]
struct Named {
    name: String,
    description: String,
}

/// The community's list of galactic effect names, by id.
static NAMES: LazyLock<HashMap<i64, Named>> = LazyLock::new(|| {
    let raw: HashMap<String, Named> = serde_json::from_str(include_str!("../data/planetEffects.json")).unwrap();
    raw.into_iter().filter_map(|(k, v)| Some((k.parse().ok()?, v))).collect()
});

/// Names for galactic effect ids the community list (planetEffects.json) doesn't carry yet, and fixes
/// to the ones it gets wrong. Unnamed ids are left off the map and the planet page until added here.
/// Ids seen without a name on 24 Oct 2026: 1272 (Mastia), 1315–1317 (Esker, Erson Sands, Socorro III),
/// 1318–1320 (Zagon Prime, Omicron), 1327 (Grafmere), 1363 (Aurora Bay),
/// 1365 (Chort Bay), 1366 (Turing), 1418 (Calypso).
fn override_of(id: i64) -> Option<(&'static str, &'static str)> {
    match id {
        // Named from the game's own planet tooltip, 25 Oct 2026.
        1333 => Some((
            "E-711 EXTRACTION FACILITY",
            "Extensive mining and drilling operations are taking place to comb this former Hive World for all traces of the highly valuable substance known as E-711.",
        )),
        _ => None,
    }
}

// Each enemy unit comes as a pair, one id per side of the fight; both name the same thing.
const KINDS: &[(EffectKind, &[i64])] = &[
    (
        EffectKind::Subfaction,
        &[
            1202, 1203, 1243, 1244, 1245, 1248, 1249, 1269, 1303, 1306, 1307, 1308, 1309, 1310, 1360, 1361, 1377, 1378,
            1379, 1380, 1386, 1402, 1403, 1413, 1414,
        ],
    ),
    (EffectKind::Threat, &[1283, 1285, 1288, 1293, 1355, 1357, 1359]),
    (EffectKind::Seaf, &[1400, 1401]),
    (EffectKind::Dss, &[1209, 1210, 1211, 1214, 1215, 1216, 1217, 1238, 1261, 1262]),
    (EffectKind::Condition, &[1373, 1374, 1375]),
    (EffectKind::Event, &[1204, 1240, 1242, 1329, 1407, 1412]),
    (EffectKind::Look, &[1177, 1228, 1229, 1230, 1241, 1252, 1311, 1352, 1376]),
    (EffectKind::Gloom, &GLOOM_IDS),
    (EffectKind::Hidden, &[1212, 1213, 1222, 1246, 1406]),
];

const ORDER: [EffectKind; 10] = [
    EffectKind::Subfaction,
    EffectKind::Threat,
    EffectKind::Seaf,
    EffectKind::Poi,
    EffectKind::Dss,
    EffectKind::Arsenal,
    EffectKind::Condition,
    EffectKind::Event,
    EffectKind::Look,
    EffectKind::Gloom,
];

/// Planets inside or on the edge of the Gloom; the map draws its cloud over them.
pub const GLOOM_IDS: [i64; 5] = [1186, 1187, 1188, 1190, 1193];

static SIDE_SUFFIX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\s*\((enemies|marker|old map marker)\)\s*$").unwrap());
static MARKUP_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<i=\d+>|</i>").unwrap());

fn kind_of(id: i64) -> Option<EffectKind> {
    KINDS.iter().find(|(_, ids)| ids.contains(&id)).map(|(k, _)| *k)
}

fn describe(id: i64) -> Option<PlanetEffect> {
    let (name, description) = match override_of(id) {
        Some((n, d)) => (n, d),
        None => NAMES.get(&id).map(|n| (n.name.as_str(), n.description.as_str()))?,
    };
    let kind = kind_of(id).unwrap_or(if name.to_uppercase().starts_with("ARSENAL AUGMENTATION") {
        EffectKind::Arsenal
    } else {
        EffectKind::Poi
    });
    // "(Enemies)" marks the enemy-side twin of an id; the player knows them by one name.
    Some(PlanetEffect {
        id,
        kind,
        name: SIDE_SUFFIX.replace(name, "").into_owned(),
        description: MARKUP_TAG.replace_all(description, "").into_owned(),
    })
}

/// A planet's effects as the player sees them: named, one per name, ordered as the game lists them.
pub fn planet_effects(ids: &[i64]) -> Vec<PlanetEffect> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for &id in ids {
        let Some(e) = describe(id) else { continue };
        if e.kind == EffectKind::Hidden || !seen.insert(e.name.to_lowercase()) {
            continue;
        }
        out.push(e);
    }
    let rank = |k: EffectKind| ORDER.iter().position(|o| *o == k).map_or(-1, |i| i as i64);
    out.sort_by_key(|e| rank(e.kind));
    out
}

/// Effects that earn a pawn beside the planet on the map.
pub fn has_pawn(e: &PlanetEffect) -> bool {
    matches!(e.kind, EffectKind::Subfaction | EffectKind::Threat | EffectKind::Seaf | EffectKind::Poi)
}
