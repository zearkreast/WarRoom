mod common;
use common::*;
use warroom_core::effects::{EffectKind, planet_effects};
use warroom_core::format::parse_markup;
use warroom_core::news_feed::plain_text;
use warroom_core::snapshot::to_effects_live;

fn on(index: i64) -> Vec<warroom_core::effects::PlanetEffect> {
    let live = to_effects_live(&status_effects(), 0.0);
    planet_effects(&live.planets.iter().find(|p| p.index == index).map(|p| p.ids.clone()).unwrap_or_default())
}

#[test]
fn names_each_sub_faction_once_though_the_game_lists_it_once_per_side() {
    let fury: Vec<(EffectKind, String)> = on(261).into_iter().map(|e| (e.kind, e.name)).collect();
    assert_eq!(
        fury,
        [
            (EffectKind::Subfaction, "THE INCINERATION CORPS".to_string()),
            (EffectKind::Seaf, "Heavy SEAF Presence".to_string())
        ]
    );
}

#[test]
fn files_cities_the_training_ground_and_tcs_towers_as_points_of_interest() {
    let live = to_effects_live(&status_effects(), 0.0);
    let pois: Vec<String> = live
        .planets
        .iter()
        .flat_map(|p| planet_effects(&p.ids).into_iter().filter(|e| e.kind == EffectKind::Poi).map(|e| e.name))
        .collect();
    for name in
        ["NEW HOPE CITY", "NEW ASPIRATION CITY", "NEW YEARNING CITY", "HELLDIVER TRAINING FACILITIES", "PANDORA BASE"]
    {
        assert!(pois.iter().any(|p| p == name), "{name}");
    }
    assert_eq!(pois.iter().filter(|n| *n == "TERMINID CONTROL SYSTEM+").count(), 14);
}

#[test]
fn keeps_the_effects_war_events_grant_for_a_while_with_when_they_end() {
    let at = ms("2026-10-26T23:57:00Z");
    let status: warroom_core::api::WarStatusRaw = serde_json::from_value(serde_json::json!({
        "time": 85_118_800,
        "planetActiveEffects": [],
        "spaceStations": [],
        "globalEvents": [
            { "effectIds": [], "planetIndices": [], "expireTime": 85_331_008 },
            { "effectIds": [1207], "planetIndices": [123, 75], "expireTime": 85_331_977 },
            { "effectIds": [1231], "planetIndices": [75], "expireTime": 85_000_000 },
        ],
    }))
    .unwrap();
    let timed = to_effects_live(&status, at).timed;
    assert_eq!(timed.len(), 1);
    assert_eq!((timed[0].ids.clone(), timed[0].planets.clone()), (vec![1207], vec![75, 123]));
    assert_eq!(timed[0].until, ms("2026-10-29T11:10:00Z"));
    let none = warroom_core::api::WarStatusRaw { global_events: Vec::new(), ..status };
    let live = to_effects_live(&none, at);
    assert!(live.timed.is_empty());
    assert!(serde_json::to_value(&live).unwrap().get("timed").is_none());
}

#[test]
fn leaves_out_unnamed_ids_and_bookkeeping_like_the_campaign_blocker() {
    let live = to_effects_live(&status_effects(), 0.0);
    let zagon = live.planets.iter().find(|p| p.ids.contains(&1318)).unwrap();
    assert!(!planet_effects(&zagon.ids).iter().any(|e| e.name == "CAMPAIGN BLOCKER"));
    assert!(planet_effects(&[1318, 1319]).is_empty());
}

#[test]
fn names_scavenger_rampage_as_the_games_tooltip_does() {
    let [e]: [_; 1] = planet_effects(&[1429]).try_into().unwrap();
    assert_eq!(
        (e.kind, e.group.as_deref(), e.name.as_str()),
        (EffectKind::Threat, Some("PERILOUS POPULATION"), "SCAVENGER RAMPAGE")
    );
    let hot: Vec<String> =
        parse_markup(&e.description).into_iter().filter(|s| s.style.is_some()).map(|s| s.text).collect();
    assert_eq!(hot, ["Scavenger", "2,900% higher"]);
    assert_eq!(
        plain_text(&e.description),
        "Terrestrial scans indicate that Scavenger threat density is 2,900% higher than normal."
    );
    assert_eq!(planet_effects(&[1333, 1209]).into_iter().map(|e| e.group).collect::<Vec<_>>(), [None, None]);
}

#[test]
fn shows_scavenger_rampage_on_the_planets_its_war_event_grants_it_to() {
    let status: warroom_core::api::WarStatusRaw = serde_json::from_value(serde_json::json!({
        "time": 85_118_800,
        "planetActiveEffects": [],
        "spaceStations": [],
        "globalEvents": [{ "effectIds": [1429], "planetIndices": [171], "expireTime": 85_331_977 }],
    }))
    .unwrap();
    let [t]: [_; 1] = to_effects_live(&status, ms("2026-10-26T23:57:00Z")).timed.try_into().unwrap();
    assert_eq!((t.ids.clone(), t.planets.clone()), (vec![1429], vec![171]));
    assert_eq!(t.until, ms("2026-10-29T11:10:00Z"));
    let names: Vec<String> = planet_effects(&t.ids).into_iter().map(|e| e.name).collect();
    assert_eq!(names, ["SCAVENGER RAMPAGE"]);
}

#[test]
fn keeps_the_games_highlights_in_descriptions() {
    let [e]: [_; 1] = planet_effects(&[1209]).try_into().unwrap();
    assert_eq!(e.name, "EAGLE STORM");
    assert_eq!(e.description, "The DSS deploys <i=1>Eagle Strafing Runs</i> during missions.");
    assert_eq!(plain_text(&e.description), "The DSS deploys Eagle Strafing Runs during missions.");
}
