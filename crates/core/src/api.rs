//! Shapes of the Helldivers 2 community API and the official game API, as the app and collector use them.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Drops `null` object members, so a field the API sends as null reads as missing: its default.
fn strip_nulls(v: &mut Value) {
    match v {
        Value::Object(map) => {
            map.retain(|_, x| !x.is_null());
            map.values_mut().for_each(strip_nulls);
        }
        Value::Array(list) => list.iter_mut().for_each(strip_nulls),
        _ => {}
    }
}

/// Reads JSON leniently: null or missing fields take their defaults.
pub fn from_value<T: DeserializeOwned>(mut v: Value) -> Result<T, serde_json::Error> {
    strip_nulls(&mut v);
    serde_json::from_value(v)
}

pub fn from_str<T: DeserializeOwned>(s: &str) -> Result<T, serde_json::Error> {
    from_value(serde_json::from_str(s)?)
}

/// Whole floats as integers (`5`, not `5.0`), the number format every stored snapshot uses.
pub fn js_value(mut v: Value) -> Value {
    fn walk(v: &mut Value) {
        match v {
            Value::Number(n) => {
                if let Some(f) =
                    n.as_f64().filter(|f| n.is_f64() && f.fract() == 0.0 && f.abs() < 9_007_199_254_740_992.0)
                {
                    *v = Value::from(f as i64);
                }
            }
            Value::Array(list) => list.iter_mut().for_each(walk),
            Value::Object(map) => map.values_mut().for_each(walk),
            _ => {}
        }
    }
    walk(&mut v);
    v
}

/// `JSON.stringify` of a value: whole floats written as integers.
pub fn to_js_json<T: Serialize>(value: &T) -> String {
    js_value(serde_json::to_value(value).expect("serializable")).to_string()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Faction {
    #[default]
    Humans,
    Terminids,
    Automaton,
    Illuminate,
    /// Anything the API may add later; shown as "Unknown".
    #[serde(other)]
    Unknown,
}

impl Faction {
    pub const ENEMIES: [Faction; 3] = [Faction::Terminids, Faction::Automaton, Faction::Illuminate];
    pub const ALL: [Faction; 4] = [Faction::Humans, Faction::Terminids, Faction::Automaton, Faction::Illuminate];

    pub fn as_str(self) -> &'static str {
        match self {
            Faction::Humans => "Humans",
            Faction::Terminids => "Terminids",
            Faction::Automaton => "Automaton",
            Faction::Illuminate => "Illuminate",
            Faction::Unknown => "Unknown",
        }
    }

    pub fn parse(s: &str) -> Option<Faction> {
        Faction::ALL.into_iter().find(|f| f.as_str() == s)
    }
}

impl std::fmt::Display for Faction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Statistics {
    pub missions_won: f64,
    pub missions_lost: f64,
    pub mission_time: f64,
    pub terminid_kills: f64,
    pub automaton_kills: f64,
    pub illuminate_kills: f64,
    pub bullets_fired: f64,
    pub bullets_hit: f64,
    pub time_played: f64,
    pub deaths: f64,
    pub revives: f64,
    pub friendlies: f64,
    pub mission_success_rate: f64,
    pub accuracy: f64,
    pub player_count: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct War {
    pub started: String,
    pub ended: String,
    pub now: String,
    pub client_version: String,
    pub factions: Vec<Faction>,
    pub impact_multiplier: f64,
    pub statistics: Statistics,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PlanetEvent {
    pub id: i64,
    pub event_type: i64,
    pub faction: Faction,
    pub health: f64,
    pub max_health: f64,
    pub start_time: String,
    pub end_time: String,
    pub campaign_id: i64,
    pub joint_operation_ids: Vec<i64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Region {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub health: Option<f64>,
    pub max_health: f64,
    pub size: String,
    /// Enemy regeneration in health per second, while the region is fought over.
    pub regen_per_second: Option<f64>,
    /// The share of the planet's health left when the region opens: it opens once the planet is (1 − factor)
    /// liberated, so 1 means open from the start. Absent from snapshots of older collectors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_factor: Option<f64>,
    pub is_available: bool,
    pub players: f64,
    /// Progress trend in %/h from the collector; only on campaign planets, null while measuring.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Named {
    pub name: String,
    pub description: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Planet {
    pub index: i64,
    pub name: String,
    pub sector: String,
    pub biome: Named,
    pub hazards: Vec<Named>,
    pub hash: i64,
    pub position: Position,
    pub waypoints: Vec<i64>,
    pub max_health: f64,
    pub health: f64,
    pub disabled: bool,
    pub initial_owner: Faction,
    pub current_owner: Faction,
    pub regen_per_second: f64,
    pub event: Option<PlanetEvent>,
    pub statistics: Statistics,
    pub attacking: Vec<i64>,
    pub regions: Vec<Region>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Campaign {
    pub id: i64,
    pub planet: Planet,
    #[serde(rename = "type")]
    pub kind: i64,
    pub count: i64,
    pub faction: Faction,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AssignmentTask {
    #[serde(rename = "type")]
    pub kind: i64,
    pub values: Vec<f64>,
    pub value_types: Vec<i64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Reward {
    #[serde(rename = "type")]
    pub kind: i64,
    pub amount: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Assignment {
    pub id: i64,
    pub progress: Vec<f64>,
    pub title: Option<String>,
    pub briefing: Option<String>,
    pub description: Option<String>,
    pub tasks: Vec<AssignmentTask>,
    pub reward: Option<Reward>,
    pub rewards: Vec<Reward>,
    pub expiration: String,
    pub flags: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Dispatch {
    pub id: i64,
    pub published: String,
    #[serde(rename = "type")]
    pub kind: i64,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TacticalActionCost {
    pub id: String,
    pub item_mix_id: i64,
    pub target_value: f64,
    pub current_value: f64,
    pub delta_per_second: f64,
    pub max_donation_ammount: f64,
    pub max_donation_period_seconds: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TacticalAction {
    pub id32: i64,
    pub media_id32: i64,
    pub name: String,
    pub description: String,
    pub strategic_description: String,
    /// 1 preparing, 2 active, 3 on cooldown (0 or other: unavailable).
    pub status: i64,
    pub status_expire: String,
    pub costs: Vec<TacticalActionCost>,
    pub effect_ids: Vec<i64>,
}

/// A Democracy Space Station, from /api/v1/space-stations.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SpaceStation {
    pub id32: i64,
    pub planet: Planet,
    /// When the vote on the next destination closes, i.e. the next FTL jump.
    pub election_end: String,
    pub flags: i64,
    pub tactical_actions: Vec<TacticalAction>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MixReward {
    pub mix_id: i64,
    pub amount: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct EpisodePhaseRaw {
    pub intro_title: String,
    pub intro_message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outro_message: Option<String>,
    pub status: i64,
    pub rewards: Vec<MixReward>,
}

/// A story campaign as the official game API returns it; times are war seconds.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct EpisodeRaw {
    pub id32: i64,
    pub title: String,
    pub description: String,
    /// Faction code: 1 Super Earth (several fronts), 2 Terminids, 3 Automatons, 4 Illuminate.
    pub race: i64,
    pub status: i64,
    pub start_war_time: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_war_time: Option<f64>,
    pub phases: Vec<EpisodePhaseRaw>,
    pub rewards: Vec<MixReward>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PlanetActiveEffect {
    pub index: i64,
    pub galactic_effect_id: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct StationEffects {
    pub id32: i64,
    pub planet_index: i64,
    pub active_effect_ids: Vec<i64>,
}

/// The parts of the official game API's war status the collector keeps: effects on planets and stations.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WarStatusRaw {
    /// War time, in seconds; `expire_time` counts in the same clock.
    pub time: f64,
    pub planet_active_effects: Vec<PlanetActiveEffect>,
    pub space_stations: Vec<StationEffects>,
    /// War events; one with effects grants them to its planets until it expires.
    pub global_events: Vec<GlobalEvent>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GlobalEvent {
    #[serde(deserialize_with = "null_default")]
    pub effect_ids: Vec<i64>,
    #[serde(deserialize_with = "null_default")]
    pub planet_indices: Vec<i64>,
    pub expire_time: f64,
}

fn null_default<'de, D: serde::Deserializer<'de>, T: Default + Deserialize<'de>>(d: D) -> Result<T, D::Error> {
    Ok(Option::<T>::deserialize(d)?.unwrap_or_default())
}
