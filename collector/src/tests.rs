//! The jobs end to end, against a recording stand-in for Supabase and canned API answers.

use crate::hd2::{Api, ApiError};
use crate::http::{Request, Response, Transport, TransportError};
use crate::push::{Push, Topic};
use crate::recorder::Recorder;
use crate::supabase::Supabase;
use crate::tasks::{Collector, Job, campaign_news};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use warroom_core::api::{Planet, from_str};
use warroom_core::format::dispatch_title;
use warroom_core::snapshot::{
    EpisodeLive, EpisodeStatus, OrderSample, add_order_samples, campaign_tally, merge_episodes, to_campaign_live,
    to_episodes, to_live, to_planet, to_static,
};
use warroom_core::time::iso_string;

macro_rules! fixture {
    ($name:literal) => {
        include_str!(concat!("../../crates/core/tests/fixtures/", $name))
    };
}

fn raw(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

fn planets() -> Vec<Planet> {
    from_str(fixture!("planets.json")).unwrap()
}

#[derive(Clone, Debug)]
struct Call {
    method: &'static str,
    path: String,
    prefer: Option<String>,
    body: Value,
}

type Calls = Rc<RefCell<Vec<Call>>>;

/// A Supabase stand-in that records every request and answers selects with `rows`, by "table:kind" first.
struct FakeDb {
    rows: HashMap<String, Value>,
    calls: Calls,
    /// Paths answered 404, as for a table whose migration isn't applied.
    missing: Vec<&'static str>,
}

impl Transport for FakeDb {
    fn send(&self, req: Request) -> Result<Response, TransportError> {
        let path = req.url.replace("https://x.supabase.co/rest/v1/", "");
        assert!(req.headers.contains(&("apikey", "sb_secret_test".into())));
        let prefer = req.headers.iter().find(|(k, _)| *k == "Prefer").map(|(_, v)| v.clone());
        let body = req.body.as_deref().map_or(Value::Null, raw);
        self.calls.borrow_mut().push(Call { method: req.method, path: path.clone(), prefer, body });
        let table = path.split('?').next().unwrap().to_string();
        let kind = path.split("kind=eq.").nth(1).map(|k| k.split('&').next().unwrap().to_string());
        let reply = |status, body: String| Ok(Response { status, headers: Vec::new(), body });
        if self.missing.iter().any(|m| table == *m) {
            return reply(404, "{}".into());
        }
        if req.method == "GET" {
            let key = format!("{table}:{}", kind.unwrap_or_default());
            let rows = self.rows.get(&key).or_else(|| self.rows.get(&table)).cloned().unwrap_or(json!([]));
            return reply(200, rows.to_string());
        }
        if table.starts_with("rpc/") {
            return reply(200, "3".into());
        }
        reply(201, String::new())
    }
}

fn fake_db(rows: &[(&str, Value)]) -> (Supabase, Calls) {
    fake_db_missing(rows, &[])
}

fn fake_db_missing(rows: &[(&str, Value)], missing: &[&'static str]) -> (Supabase, Calls) {
    let calls = Calls::default();
    let fake = FakeDb {
        rows: rows.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
        calls: calls.clone(),
        missing: missing.to_vec(),
    };
    (Supabase::new("https://x.supabase.co", "sb_secret_test", Box::new(fake)), calls)
}

/// Answers by the last part of the path.
struct FakeApi(HashMap<String, Result<Value, ApiError>>);

impl Api for FakeApi {
    fn get(&self, path: &str) -> Result<Value, ApiError> {
        let name = path.rsplit('/').next().unwrap();
        self.0.get(name).cloned().unwrap_or_else(|| panic!("unexpected {path}"))
    }
}

fn api_data() -> HashMap<String, Result<Value, ApiError>> {
    [
        ("planets", fixture!("planets.json")),
        ("campaigns", fixture!("campaigns.json")),
        ("war", fixture!("war.json")),
        ("assignments", fixture!("assignments.json")),
        ("dispatches", fixture!("dispatches.json")),
        ("space-stations", fixture!("space-stations.json")),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), Ok(raw(v))))
    .collect()
}

fn api() -> Box<dyn Api> {
    Box::new(FakeApi(api_data()))
}

fn collector(db: Supabase, now: f64) -> Collector {
    Collector::new(api(), db, Recorder::default(), Box::new(move || now), None, None)
}

fn kinds(calls: &Calls, method: &str) -> Vec<String> {
    calls
        .borrow()
        .iter()
        .filter(|c| c.method == method && c.path.starts_with("snapshots"))
        .map(|c| c.body[0]["kind"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn rebuilds_a_full_planet_from_its_static_and_live_halves() {
    for p in planets() {
        let back = to_planet(&to_static(&p), &to_live(&p));
        assert_eq!((back.name.as_str(), back.health, back.current_owner), (p.name.as_str(), p.health, p.current_owner));
        assert_eq!(back.statistics.player_count, p.statistics.player_count);
        assert_eq!(
            back.regions.iter().map(|r| r.id).collect::<Vec<_>>(),
            p.regions.iter().map(|r| r.id).collect::<Vec<_>>()
        );
    }
}

#[test]
fn is_much_smaller_than_the_raw_api_response() {
    let raw_len = fixture!("planets.json").len();
    let live = warroom_core::api::to_js_json(&planets().iter().map(to_live).collect::<Vec<_>>()).len();
    assert!(live < raw_len / 2);
}

#[test]
fn puts_each_regions_rate_on_its_campaign_planet() {
    let campaigns: Vec<warroom_core::api::Campaign> = from_str(fixture!("campaigns.json")).unwrap();
    let c = campaigns.iter().find(|x| !x.planet.regions.is_empty()).unwrap();
    let live = to_campaign_live(c, Some(1.0), |region| Some(region as f64 + 0.5));
    let rates: Vec<Option<f64>> = live.planet.regions.iter().map(|r| r.rate).collect();
    assert_eq!(rates, c.planet.regions.iter().map(|r| Some(r.id as f64 + 0.5)).collect::<Vec<_>>());
    assert_eq!(to_campaign_live(c, Some(1.5), |_| None).rate, Some(1.5));
}

#[test]
fn measures_a_regions_rate_from_the_readings_of_its_planet() {
    let (db, _) = fake_db(&[]);
    let mut recorder = Recorder::default();
    let mut p = planets().into_iter().find(|x| !x.regions.is_empty()).unwrap();
    p.regions[0].max_health = 100_000.0;
    let region = p.regions[0].id;
    for m in (0..=20).step_by(5) {
        p.regions[0].health = Some(100_000.0 - m as f64 * 1000.0);
        recorder.ingest(&db, std::slice::from_ref(&p), m as f64 * 60_000.0).unwrap();
    }
    assert!((recorder.region_rate(p.index, region).unwrap() - 60.0).abs() < 1e-5);
    assert_eq!(recorder.region_rate(p.index, region + 99), None);
}

#[test]
fn stores_helldivers_on_a_planet_with_no_campaign_and_a_busy_city_at_most_every_few_minutes() {
    let (db, calls) = fake_db(&[]);
    let mut recorder = Recorder::default();
    let mut idle = planets().into_iter().find(|x| !x.regions.is_empty()).unwrap();
    idle.health = idle.max_health;
    idle.event = None;
    idle.statistics.player_count = 4321.0;
    idle.regions[0].players = 12.0;
    for m in [0.0, 1.0, 5.0] {
        recorder.ingest(&db, std::slice::from_ref(&idle), m * 60_000.0).unwrap();
    }
    let calls = calls.borrow();
    let rows = |path: &str| -> Vec<Value> {
        calls.iter().filter(|c| c.path == path).flat_map(|c| c.body.as_array().unwrap().clone()).collect()
    };
    assert_eq!(rows("planet_players").iter().map(|r| r["players"].as_f64().unwrap()).collect::<Vec<_>>(), [4321.0; 3]);
    assert!(rows("samples").is_empty());
    let cities: Vec<String> = rows("region_samples").iter().map(|r| r["at"].as_str().unwrap().to_string()).collect();
    assert_eq!(cities, [iso_string(0.0), iso_string(5.0 * 60_000.0)]);
}

#[test]
fn skips_the_activity_writes_rather_than_failing_before_their_migration_is_applied() {
    let missing = ["planet_players", "region_samples", "rpc/note_war_day"];
    let (db, _) = fake_db_missing(&[], &missing);
    let mut recorder = Recorder::default();
    let mut idle = planets().remove(0);
    idle.statistics.player_count = 10.0;
    assert_eq!(recorder.ingest(&db, &[idle], 0.0).unwrap().samples, 0);
    let (db, _) = fake_db_missing(&[], &missing);
    let r = collector(db, 0.0).run(Job::War);
    assert!(r.ok && r.message.contains("no war_days table yet"), "{}", r.message);
}

#[test]
fn keeps_a_day_of_major_order_readings_only_for_running_orders() {
    let day = 24.0 * 3_600_000.0;
    let s = |id, at, p| OrderSample { id, at, progress: vec![p] };
    let kept = [s(1, 0.0, 1.0), s(1, day / 2.0, 5.0), s(2, day / 2.0, 9.0)];
    let order = warroom_core::api::Assignment { id: 1, progress: vec![8.0], ..Default::default() };
    assert_eq!(add_order_samples(&kept, &[order], day + 1.0), [s(1, day / 2.0, 5.0), s(1, day + 1.0, 8.0)]);
}

#[test]
fn carries_on_from_the_stored_major_order_readings_after_a_restart() {
    let assignments = raw(fixture!("assignments.json"));
    let id = assignments[0]["id"].as_i64().unwrap();
    let (db, calls) = fake_db(&[("snapshots", json!([{ "data": [{ "id": id, "at": 999_000, "progress": [0] }] }]))]);
    let mut c = collector(db, 1_000_000.0);
    assert!(c.run(Job::Assignments).ok);
    let calls = calls.borrow();
    let write = calls.iter().find(|c| c.path == "snapshots" && c.body[0]["kind"] == "order_samples").unwrap();
    let ats: Vec<i64> = write.body[0]["data"].as_array().unwrap().iter().map(|s| s["at"].as_i64().unwrap()).collect();
    let mut want = vec![999_000];
    want.extend(assignments.as_array().unwrap().iter().map(|_| 1_000_000));
    assert_eq!(ats, want);
}

#[test]
fn logs_a_stations_ftl_jump_when_its_planet_changes_starting_from_the_stored_snapshot() {
    let station = raw(fixture!("space-stations.json"))[0].clone();
    let (id, planet) = (station["id32"].as_i64().unwrap(), station["planet"]["index"].as_i64().unwrap());
    let (db, calls) = fake_db(&[("snapshots", json!([{ "data": [{ "id32": id, "planetIndex": planet + 1 }] }]))]);
    let mut c = collector(db, 5_000_000.0);
    assert!(c.run(Job::Stations).message.ends_with("1 FTL jump"));
    let jumps: Vec<Call> = calls.borrow().iter().filter(|c| c.path == "station_jumps").cloned().collect();
    assert_eq!(
        jumps[0].body,
        json!([{ "station": id, "at": iso_string(5_000_000.0), "from_planet": planet + 1, "to_planet": planet }])
    );
    c.run(Job::Stations);
    assert_eq!(calls.borrow().iter().filter(|c| c.path == "station_jumps").count(), 1);
}

#[test]
fn writes_snapshots_samples_and_owners_on_a_first_run() {
    let (db, calls) = fake_db(&[]);
    let mut recorder = Recorder::default();
    recorder.load(&db, 0.0).unwrap();
    let mut c = Collector::new(api(), db, recorder, Box::new(|| 1_000_000.0), None, None);
    for job in c.jobs() {
        let r = c.run(job);
        assert!(r.ok, "{}: {}", r.name, r.message);
    }
    assert_eq!(
        kinds(&calls, "POST"),
        [
            "planets",
            "planets_static",
            "campaigns",
            "war",
            "assignments",
            "order_samples",
            "dispatches",
            "space_stations"
        ]
    );
    let calls = calls.borrow();
    let war = &calls.iter().find(|c| c.path == "war_samples").unwrap().body;
    assert_eq!(war.as_array().unwrap().len(), 1);
    assert_eq!(war[0]["players"], raw(fixture!("war.json"))["statistics"]["playerCount"]);
    let fronts = war[0]["terminids"].as_f64().unwrap()
        + war[0]["automatons"].as_f64().unwrap()
        + war[0]["illuminate"].as_f64().unwrap();
    assert!(fronts > 0.0);
    for c in calls.iter().filter(|c| c.path == "snapshots") {
        assert!(c.prefer.as_deref().unwrap().contains("merge-duplicates"));
    }
    let samples: Vec<&Call> = calls.iter().filter(|c| c.path == "samples").collect();
    assert!(!samples.is_empty());
    assert!(samples[0].prefer.as_deref().unwrap().contains("ignore-duplicates"));
    assert!(samples[0].body.as_array().unwrap().iter().all(|r| (1..=4).contains(&r["owner"].as_i64().unwrap())));
    let owners = &calls.iter().find(|c| c.path == "planet_owners").unwrap().body;
    assert_eq!(owners.as_array().unwrap().len(), planets().len());
    assert!(!calls.iter().any(|c| c.path == "owner_changes"));
    assert!(calls.iter().any(|c| c.path == "rpc/compact_samples"));
    assert!(calls.iter().any(|c| c.path == "rpc/compact_war_samples"));
    assert_eq!(
        calls.iter().find(|c| c.path == "rpc/note_war_day").unwrap().body,
        json!({ "reading_at": iso_string(1_000_000.0), "online": raw(fixture!("war.json"))["statistics"]["playerCount"] })
    );
}

#[test]
fn records_a_capture_and_skips_unchanged_static_data_and_dispatches() {
    let (db, calls) = fake_db(&[]);
    let mut c = collector(db, 1_000_000.0);
    c.run(Job::Planets);
    c.run(Job::Dispatches);
    calls.borrow_mut().clear();

    let mut captured = raw(fixture!("planets.json"));
    for p in captured.as_array_mut().unwrap() {
        if p["index"] == 261 {
            p["currentOwner"] = json!("Humans");
            p["health"] = p["maxHealth"].clone();
        }
    }
    let mut data = api_data();
    data.insert("planets".into(), Ok(captured));
    c.api = Box::new(FakeApi(data));
    c.now = Box::new(|| 2_000_000.0);
    assert!(c.run(Job::Planets).message.contains("1 owner change"));
    {
        let calls = calls.borrow();
        let change = &calls.iter().find(|c| c.path == "owner_changes").unwrap().body;
        assert_eq!(
            change,
            &json!([{ "planet": 261, "planet_name": "FURY", "from_owner": 3, "to_owner": 1, "at": iso_string(2_000_000.0) }])
        );
        assert_eq!(
            calls.iter().find(|c| c.path == "planet_owners").unwrap().body,
            json!([{ "planet": 261, "owner": 1 }])
        );
        assert!(!kinds_in(&calls).contains(&"planets_static".to_string()));
    }
    calls.borrow_mut().clear();
    assert_eq!(c.run(Job::Dispatches).message, "unchanged");
    assert!(calls.borrow().is_empty());
}

fn kinds_in(calls: &[Call]) -> Vec<String> {
    calls.iter().filter(|c| c.path == "snapshots").map(|c| c.body[0]["kind"].as_str().unwrap().to_string()).collect()
}

#[test]
fn rewrites_the_dispatches_when_the_newest_ones_text_changes_under_the_same_id() {
    let (db, calls) = fake_db(&[]);
    let mut c = collector(db, 1_000_000.0);
    c.run(Job::Dispatches);
    let mut corrected = raw(fixture!("dispatches.json"));
    for d in corrected.as_array_mut().unwrap() {
        d["message"] = json!(format!("{} (corrected)", d["message"].as_str().unwrap()));
    }
    let mut data = api_data();
    data.insert("dispatches".into(), Ok(corrected));
    c.api = Box::new(FakeApi(data));
    calls.borrow_mut().clear();
    c.run(Job::Dispatches);
    assert_eq!(calls.borrow().iter().filter(|c| c.path.starts_with("snapshots")).count(), 1);
}

#[test]
fn restores_owners_after_a_restart_so_it_doesnt_invent_captures() {
    let owners: Vec<Value> = planets()
        .iter()
        .map(|p| json!({ "planet": p.index, "owner": warroom_core::factions::faction_id(p.current_owner) }))
        .collect();
    let (db, calls) = fake_db(&[("planet_owners", Value::Array(owners))]);
    let mut recorder = Recorder::default();
    recorder.load(&db, 0.0).unwrap();
    assert_eq!(recorder.known_planets(), planets().len());
    let mut c = Collector::new(api(), db, recorder, Box::new(|| 1_000_000.0), None, None);
    c.run(Job::Planets);
    assert!(!calls.borrow().iter().any(|c| c.path == "owner_changes" || c.path == "planet_owners"));
}

#[test]
fn reports_a_rate_limit_with_a_retry_time() {
    let (db, _) = fake_db(&[]);
    let mut data = api_data();
    data.insert("war".into(), Err(ApiError::RateLimited { retry_after_ms: 20_000.0 }));
    let mut c = Collector::new(Box::new(FakeApi(data)), db, Recorder::default(), Box::new(|| 0.0), None, None);
    let r = c.run(Job::War);
    assert_eq!((r.ok, r.retry_at), (false, Some(20_000.0)));
}

fn raw_episodes() -> Value {
    json!([
        {
            "id32": 1, "title": "OLD", "description": "", "race": 4, "status": 3, "startWarTime": 1_000, "endWarTime": 2_000,
            "phases": [{ "introTitle": "A", "introMessage": "", "outroMessage": "lost", "status": 3, "rewards": [] }],
            "rewards": [{ "mixId": 7, "amount": 1 }],
        },
        {
            "id32": 2, "title": "NOW", "description": "d", "race": 1, "status": 0, "startWarTime": 9_000,
            "phases": [{ "introTitle": "B", "introMessage": "brief", "status": 0, "rewards": [{ "mixId": 897894480, "amount": 40 }] }],
            "rewards": [],
        },
    ])
}

fn game(episodes: Value, status: Value) -> Box<dyn Api> {
    Box::new(FakeApi(
        [("WarID", json!({ "id": 801 })), ("801", json!({ "episodes": episodes })), ("Status", status)]
            .into_iter()
            .map(|(k, v)| (k.to_string(), Ok(v)))
            .collect(),
    ))
}

fn episodes_collector(db: Supabase, episodes: Value, status: Value) -> Collector {
    Collector::new(api(), db, Recorder::default(), Box::new(|| 50_000_000.0), Some(game(episodes, status)), None)
}

#[test]
fn dates_war_second_timestamps_against_the_war_clock_and_maps_outcomes_faction_and_reward() {
    let list = to_episodes(&from_str::<Vec<_>>(&raw_episodes().to_string()).unwrap(), 10_000.0, 50_000_000.0);
    let (old, now) = (&list[0], &list[1]);
    assert_eq!(
        (old.status, old.faction, old.reward),
        (EpisodeStatus::Lost, warroom_core::api::Faction::Illuminate, Some(7))
    );
    assert_eq!((old.start, old.end), (50_000_000.0 - 9_000_000.0, Some(50_000_000.0 - 8_000_000.0)));
    assert_eq!((old.phases[0].outro.as_str(), old.phases[0].issued), ("lost", Some(50_000_000.0 - 9_000_000.0)));
    assert_eq!(
        (now.status, now.faction, now.end, now.reward),
        (EpisodeStatus::Active, warroom_core::api::Faction::Humans, None, None)
    );
    assert_eq!(
        (now.phases[0].title.as_str(), now.phases[0].status, now.phases[0].medals),
        ("B", EpisodeStatus::Active, Some(40.0))
    );
}

#[test]
fn maps_the_live_apis_campaigns_won_by_majority_lost_by_majority() {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Fixture {
        episodes: Vec<warroom_core::api::EpisodeRaw>,
        war_time: f64,
        fetched_at: f64,
    }
    let f: Fixture = from_str(fixture!("episodes.json")).unwrap();
    let list = to_episodes(&f.episodes, f.war_time, f.fetched_at);
    for e in list.iter().filter(|x| x.status != EpisodeStatus::Active) {
        let t = campaign_tally(e);
        assert_eq!(e.status, if t.won > t.lost { EpisodeStatus::Won } else { EpisodeStatus::Lost });
    }
    assert_eq!(list.iter().find(|e| e.title == "BLAZING ELECTORATE").unwrap().status, EpisodeStatus::Lost);
    assert!(list.iter().all(|e| e.reward.is_some()));
}

#[test]
fn keeps_campaigns_the_game_stops_listing_and_stamps_new_major_orders_when_they_appear() {
    let raw_list: Vec<warroom_core::api::EpisodeRaw> = from_str(&raw_episodes().to_string()).unwrap();
    let stored = to_episodes(&raw_list, 10_000.0, 50_000_000.0);
    let mut grown = raw_list[1].clone();
    grown.phases.push(warroom_core::api::EpisodePhaseRaw { intro_title: "C".into(), ..Default::default() });
    let next = to_episodes(&[grown], 20_000.0, 60_000_123.0);
    let merged = merge_episodes(&stored, &next, 60_000_123.0);
    assert_eq!(merged.iter().map(|e| e.id).collect::<Vec<_>>(), [1, 2]);
    assert_eq!(merged[1].start, stored[1].start);
    assert_eq!(
        merged[1].phases.iter().map(|p| p.issued).collect::<Vec<_>>(),
        [Some(stored[1].start), Some(60_000_123.0)]
    );
    // Nothing stored yet: orders after the first predate the collector, so their date is unknown.
    assert_eq!(merge_episodes(&[], &next, 1.0)[0].phases[1].issued, None);
}

#[test]
fn reads_the_stored_archive_once_then_writes_only_when_something_changed() {
    let raw_list: Vec<warroom_core::api::EpisodeRaw> = from_str(&raw_episodes().to_string()).unwrap();
    let stored = to_episodes(&raw_list, 10_000.0, 50_000_000.0);
    let (db, calls) = fake_db(&[("snapshots", json!([{ "data": serde_json::to_value(&stored).unwrap() }]))]);
    let mut c = episodes_collector(db, raw_episodes(), json!({ "time": 10_000 }));
    assert_eq!(c.run(Job::Episodes).message, "unchanged, 1 active");
    assert_eq!(c.run(Job::Episodes).message, "unchanged, 1 active");
    assert_eq!(calls.borrow().iter().filter(|c| c.method == "GET").count(), 1);
    assert_eq!(kinds(&calls, "POST"), ["planet_effects"]);
}

#[test]
fn keeps_the_planet_effects_from_the_status_it_already_reads_without_another_request() {
    let (db, calls) = fake_db(&[]);
    let status = json!({
        "time": 10_000,
        "planetActiveEffects": [
            { "index": 5, "galacticEffectId": 1249 },
            { "index": 5, "galacticEffectId": 1248 },
            { "index": 2, "galacticEffectId": 1395 },
        ],
        "spaceStations": [{ "id32": 9, "planetIndex": 5, "activeEffectIds": [1238] }],
    });
    let mut c = episodes_collector(db, raw_episodes(), status);
    c.run(Job::Episodes);
    let calls = calls.borrow();
    let written = calls.iter().find(|c| c.method == "POST" && c.body[0]["kind"] == "planet_effects").unwrap();
    assert_eq!(
        written.body[0]["data"],
        json!({ "planets": [{ "index": 2, "ids": [1395] }, { "index": 5, "ids": [1248, 1249] }], "stations": [{ "id": 9, "ids": [1238] }] })
    );
}

#[test]
fn dates_a_new_order_only_on_a_running_campaign_it_already_knew_and_drops_impossible_dates() {
    let raw_list: Vec<warroom_core::api::EpisodeRaw> = from_str(&raw_episodes().to_string()).unwrap();
    let stored = to_episodes(&raw_list, 10_000.0, 50_000_000.0);
    let mut ended = raw_list[0].clone();
    ended.phases.push(warroom_core::api::EpisodePhaseRaw {
        intro_title: "Late".into(),
        status: 3,
        ..Default::default()
    });
    let merged = merge_episodes(
        &stored,
        &to_episodes(&[ended.clone(), raw_list[1].clone()], 20_000.0, 60_000_000.0),
        60_000_000.0,
    );
    assert_eq!(merged[0].phases[1].issued, None);
    let mut broken = stored[0].clone();
    let mut late = broken.phases[0].clone();
    late.issued = Some(60_000_000.0);
    broken.phases.push(late);
    let again = merge_episodes(&[broken], &to_episodes(&[ended], 20_000.0, 60_000_000.0), 70_000_000.0);
    assert_eq!(again[0].phases[1].issued, None);
}

#[test]
fn writes_an_episodes_snapshot_only_when_the_game_api_client_is_given() {
    let (db, calls) = fake_db(&[]);
    let mut c = episodes_collector(db, raw_episodes(), json!({ "time": 10_000 }));
    assert_eq!(c.run(Job::Episodes).message, "2 campaigns, 1 active");
    assert!(kinds(&calls, "POST").contains(&"episodes".to_string()));
    let (db, _) = fake_db(&[]);
    assert!(!collector(db, 0.0).jobs().contains(&Job::Episodes));
}

/// Records what would have been pushed.
struct Recorded(Rc<RefCell<Vec<(Topic, String, String)>>>);

impl Push for Recorded {
    fn push(&mut self, topic: Topic, title: &str, body: &str) -> Result<(), String> {
        self.0.borrow_mut().push((topic, title.into(), body.into()));
        Ok(())
    }
}

struct Failing;

impl Push for Failing {
    fn push(&mut self, _: Topic, _: &str, _: &str) -> Result<(), String> {
        Err("FCM send 503".into())
    }
}

fn pushing(db: Supabase, push: Box<dyn Push>) -> Collector {
    Collector::new(api(), db, Recorder::default(), Box::new(|| 1_000_000.0), None, Some(push))
}

#[test]
fn pushes_a_major_order_missing_from_the_stored_snapshot_once() {
    let (db, _) = fake_db(&[("snapshots:assignments", json!([{ "data": [] }]))]);
    let sent = Rc::new(RefCell::new(Vec::new()));
    let mut c = pushing(db, Box::new(Recorded(sent.clone())));
    assert_eq!(c.run(Job::Assignments).message, "1 orders, 1/1 pushed");
    assert_eq!(c.run(Job::Assignments).message, "1 orders");
    let briefing = raw(fixture!("assignments.json"))[0]["briefing"].as_str().unwrap().to_string();
    assert_eq!(*sent.borrow(), [(Topic::Orders, "New Major Order".to_string(), briefing)]);
}

#[test]
fn pushes_nothing_on_a_fresh_setup_with_nothing_stored() {
    let (db, _) = fake_db(&[]);
    let sent = Rc::new(RefCell::new(Vec::new()));
    let mut c = pushing(db, Box::new(Recorded(sent.clone())));
    c.run(Job::Assignments);
    c.run(Job::Dispatches);
    assert!(sent.borrow().is_empty());
}

#[test]
fn pushes_the_dispatches_newer_than_the_stored_ones_oldest_first() {
    let mut newest: Vec<Value> = raw(fixture!("dispatches.json")).as_array().unwrap().clone();
    newest.sort_by_key(|d| std::cmp::Reverse(d["id"].as_i64().unwrap()));
    let (db, _) = fake_db(&[("snapshots:dispatches", json!([{ "data": newest[2..] }]))]);
    let sent = Rc::new(RefCell::new(Vec::new()));
    let mut c = pushing(db, Box::new(Recorded(sent.clone())));
    assert_eq!(c.run(Job::Dispatches).message, format!("newest #{}, 2/2 pushed", newest[0]["id"]));
    assert_eq!(c.run(Job::Dispatches).message, "unchanged");
    let want: Vec<(Topic, String, String)> = [&newest[1], &newest[0]]
        .iter()
        .map(|d| (Topic::Dispatches, "New dispatch".to_string(), dispatch_title(d["message"].as_str().unwrap())))
        .collect();
    assert_eq!(*sent.borrow(), want);
}

#[test]
fn pushes_a_campaign_launch_under_the_orders_setting() {
    let raw_list: Vec<warroom_core::api::EpisodeRaw> = from_str(&raw_episodes().to_string()).unwrap();
    let stored = to_episodes(&raw_list[..1], 10_000.0, 50_000_000.0);
    let (db, _) = fake_db(&[("snapshots", json!([{ "data": serde_json::to_value(&stored).unwrap() }]))]);
    let sent = Rc::new(RefCell::new(Vec::new()));
    let game = game(raw_episodes(), json!({ "time": 10_000 }));
    let push = Box::new(Recorded(sent.clone()));
    let mut c = Collector::new(api(), db, Recorder::default(), Box::new(|| 50_000_000.0), Some(game), Some(push));
    assert_eq!(c.run(Job::Episodes).message, "2 campaigns, 1 active, 1/1 pushed");
    assert_eq!(*sent.borrow(), [(Topic::Orders, "New Galactic Campaign".to_string(), "New campaign: NOW".to_string())]);
}

#[test]
fn logs_a_failed_push_instead_of_failing_the_job_whose_snapshot_is_already_written() {
    let (db, calls) = fake_db(&[("snapshots:assignments", json!([{ "data": [] }]))]);
    let mut c = pushing(db, Box::new(Failing));
    let r = c.run(Job::Assignments);
    assert_eq!((r.ok, r.message.as_str()), (true, "1 orders, 0/1 pushed"));
    assert!(kinds(&calls, "POST").contains(&"assignments".to_string()));
}

#[test]
fn words_campaign_launches_and_outcomes_as_the_app_does() {
    let e = |id, title: &str, status| EpisodeLive { id, title: title.into(), status, ..Default::default() };
    use EpisodeStatus::*;
    let news = campaign_news(
        &[e(1, "EPIC PURGE", Active), e(2, "OLD", Lost)],
        &[e(1, "EPIC PURGE", Won), e(2, "OLD", Lost), e(3, "MEMORIAM ETERNAL", Active)],
    );
    assert_eq!(
        news,
        [
            ("Galactic Campaign over".to_string(), "EPIC PURGE won".to_string()),
            ("New Galactic Campaign".to_string(), "New campaign: MEMORIAM ETERNAL".to_string()),
        ]
    );
}
