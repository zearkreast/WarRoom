mod common;

use common::*;
use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
use warroom_core::api::{Dispatch, Faction, Planet};
use warroom_core::campaign_news::{
    EventType, campaign_alerts, campaign_events, campaign_headline, campaign_of, campaign_threads,
};
use warroom_core::data::reward_label;
use warroom_core::faction_marks::{Overlay, faction_marks};
use warroom_core::format::fmt_age_short;
use warroom_core::front_activity::{
    BUCKET_MS, FrontBucket, FrontRow, Reading, front_buckets, latest_run, merge_activity,
};
use warroom_core::galaxy::{
    GALAXY_SCALE, PolarSectors, SectorPlanet, build_polar_sectors, sector_at, sector_territory,
};
use warroom_core::history::OwnerChange;
use warroom_core::planets::{fix_static, is_off_map, is_placeholder, ownership_key};
use warroom_core::snapshot::{
    EpisodeLive, EpisodeStatus, issued_at, merge_episodes, to_episodes, to_live, to_planet, to_static,
};
use warroom_core::ticker::{TickerKind, ticker_items};
use warroom_core::time::iso_string;

fn raw_planets() -> Vec<Planet> {
    planets().into_iter().filter(|p| !is_placeholder(p.index, p.position)).collect()
}

/// Planets with the static fixes applied, as the app builds them.
fn fixed_planets() -> Vec<Planet> {
    raw_planets().iter().map(|p| to_planet(&fix_static(to_static(p)), &to_live(p))).collect()
}

fn sectors_of(planets: &[Planet]) -> PolarSectors {
    let list: Vec<SectorPlanet> =
        planets.iter().map(|p| SectorPlanet { index: p.index, sector: &p.sector, position: p.position }).collect();
    build_polar_sectors(&list)
}

fn campaigns_live() -> Vec<EpisodeLive> {
    let f = episodes();
    merge_episodes(&[], &to_episodes(&f.episodes, f.war_time, f.fetched_at), 0.0)
}

/// Sectors whose cells fall apart into more than one piece.
fn split_sectors(geo: &PolarSectors) -> Vec<String> {
    let tau = PI * 2.0;
    struct Cell<'a> {
        i: usize,
        k: usize,
        sector: &'a str,
        a0: f64,
        a1: f64,
    }
    let cells: Vec<Cell> = geo
        .rings
        .iter()
        .enumerate()
        .flat_map(|(i, ring)| {
            ring.runs.iter().enumerate().map(move |(k, r)| Cell { i, k, sector: &r.sector, a0: r.a0, a1: r.a1 })
        })
        .collect();
    let overlap = |a: &Cell, b: &Cell| [-tau, 0.0, tau].iter().any(|s| a.a1.min(b.a1 + s) - a.a0.max(b.a0 + s) > 1e-6);
    let mut sectors: Vec<&str> = Vec::new();
    for c in &cells {
        if !sectors.contains(&c.sector) {
            sectors.push(c.sector);
        }
    }
    sectors
        .into_iter()
        .filter(|sector| {
            let own: Vec<&Cell> = cells.iter().filter(|c| c.sector == *sector).collect();
            let mut seen: Vec<usize> = vec![0];
            let mut n = 0;
            while n < seen.len() {
                let c = own[seen[n]];
                for (j, d) in own.iter().enumerate() {
                    let touches = (d.i.abs_diff(c.i) == 1 && overlap(c, d)) || (d.i == c.i && d.k.abs_diff(c.k) == 1);
                    if touches && !seen.contains(&j) {
                        seen.push(j);
                    }
                }
                n += 1;
            }
            seen.len() < own.len()
        })
        .map(String::from)
        .collect()
}

#[test]
fn files_meridia_under_celeste_and_puts_sangis_and_alderidge_cove_back() {
    let planets = fixed_planets();
    let by_name: HashMap<&str, &Planet> = planets.iter().map(|p| (p.name.as_str(), p)).collect();
    assert_eq!(by_name["MERIDIA"].sector, "Celeste");
    assert_eq!((by_name["SANGIS"].index, by_name["SANGIS"].sector.as_str()), (273, "Guang"));
    assert_eq!((by_name["ALDERIDGE COVE"].index, by_name["ALDERIDGE COVE"].sector.as_str()), (99, "Omega"));
}

#[test]
fn leaves_no_sector_in_pieces_once_corrected() {
    let raw = split_sectors(&sectors_of(&raw_planets()));
    for s in ["Umlaut", "Guang", "Omega"] {
        assert!(raw.iter().any(|r| r == s), "{s} in {raw:?}");
    }
    assert_eq!(split_sectors(&sectors_of(&fixed_planets())), Vec::<String>::new());
}

#[test]
fn gives_two_sectors_one_boundary_angle_in_every_ring_where_their_planets_allow_it() {
    let geo = sectors_of(&fixed_planets());
    let mut angles: HashMap<String, HashSet<String>> = HashMap::new();
    for ring in &geo.rings {
        if ring.runs.len() < 2 {
            continue;
        }
        for (k, run) in ring.runs.iter().enumerate() {
            let prev = &ring.runs[(k + ring.runs.len() - 1) % ring.runs.len()];
            let a = warroom_core::format::to_fixed(((run.a0 % (2.0 * PI)) + 2.0 * PI) % (2.0 * PI), 4);
            angles.entry(format!("{}|{}", prev.sector, run.sector)).or_default().insert(a);
        }
    }
    let shared = angles.values().filter(|s| s.len() == 1).count();
    assert!(shared as f64 / angles.len() as f64 > 0.9);
}

#[test]
fn puts_each_enemys_emblem_inside_its_own_territory_in_the_sectors_overlay() {
    let planets = fixed_planets();
    let geo = sectors_of(&planets);
    let territory = sector_territory(&planets);
    let marks = faction_marks(&planets, Overlay::Sectors, Some(&geo));
    assert!(!marks.is_empty());
    for m in &marks {
        let sector = sector_at(&geo, m.x / GALAXY_SCALE, -m.y / GALAXY_SCALE).unwrap_or("");
        let t = territory.iter().find(|(s, _)| s == sector).and_then(|(_, t)| *t);
        assert_eq!(t.map(|t| t.faction), Some(m.faction));
    }
}

#[test]
fn puts_each_enemys_emblem_on_its_largest_group_of_planets_in_the_other_overlays() {
    let planets = fixed_planets();
    for overlay in [Overlay::Regions, Overlay::Galaxy] {
        let marks = faction_marks(&planets, overlay, None);
        let mut got: Vec<Faction> = marks.iter().map(|m| m.faction).collect();
        got.sort();
        let mut held: Vec<Faction> = planets
            .iter()
            .filter(|p| p.current_owner != Faction::Humans)
            .map(|p| p.current_owner)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        held.sort();
        assert_eq!(got, held);
        for m in &marks {
            let d = |p: &Planet| (p.position.x * 500.0 - m.x).hypot(-p.position.y * 500.0 - m.y);
            let mut nearest = &planets[0];
            for p in &planets[1..] {
                if d(p) < d(nearest) {
                    nearest = p;
                }
            }
            assert_eq!(nearest.current_owner, m.faction);
        }
    }
}

#[test]
fn only_changes_the_ownership_key_when_an_owner_or_an_attack_changes() {
    let planets = fixed_planets();
    let key = ownership_key(&planets);
    let healed: Vec<Planet> = planets
        .iter()
        .cloned()
        .map(|mut p| {
            p.health = 1.0;
            p
        })
        .collect();
    assert_eq!(ownership_key(&healed), key);
    let mut flipped = planets.clone();
    flipped[3].current_owner = Faction::Automaton;
    assert_ne!(ownership_key(&flipped), key);
}

#[test]
fn dates_a_launch_and_an_end_for_every_finished_campaign_newest_first() {
    let campaigns = campaigns_live();
    let events = campaign_events(&campaigns, &[]);
    let ended = campaigns.iter().filter(|e| e.status != EpisodeStatus::Active).count();
    assert_eq!(events.iter().filter(|e| e.kind == EventType::Ended).count(), ended);
    assert_eq!(events.iter().filter(|e| e.kind == EventType::Launched).count(), campaigns.len());
    assert!(events.windows(2).all(|w| w[0].at >= w[1].at));
    let lost = events.iter().find(|e| e.kind == EventType::Ended && e.campaign.title == "BLAZING ELECTORATE").unwrap();
    assert_eq!(campaign_headline(lost), "BLAZING ELECTORATE failed");
}

#[test]
fn alerts_on_campaign_launches_and_outcomes_once_never_on_the_first_load() {
    let campaigns = campaigns_live();
    let (fresh, keys) = campaign_alerts(&campaigns, &[]);
    assert!(fresh.is_empty());
    let not_orders: Vec<_> =
        campaign_events(&campaigns, &[]).into_iter().filter(|e| e.kind != EventType::Order).collect();
    assert_eq!(keys.len(), not_orders.len());
    assert!(campaign_alerts(&campaigns, &keys).0.is_empty());
    let latest = &not_orders[0];
    let seen: Vec<String> = keys.iter().filter(|k| **k != latest.key).cloned().collect();
    let fresh: Vec<String> = campaign_alerts(&campaigns, &seen).0.into_iter().map(|e| e.key).collect();
    assert_eq!(fresh, std::slice::from_ref(&latest.key));
}

fn dispatch(id: i64, published: f64, message: &str) -> Dispatch {
    Dispatch { id, published: iso_string(published), kind: 0, message: message.into() }
}

#[test]
fn gathers_a_campaigns_orders_and_dispatches_on_one_thread() {
    let campaigns = campaigns_live();
    let e = campaigns.iter().find(|c| c.status == EpisodeStatus::Active).unwrap_or(campaigns.last().unwrap());
    let at = |h: f64| e.start + h * H;
    let list = [
        dispatch(1, at(0.5), "<i=3>NEW MAJOR ORDER</i> First"),
        dispatch(2, at(30.0), "<i=3>MAJOR ORDER FAILED</i> Too few samples"),
        dispatch(3, at(31.0), "<i=3>STRATEGIC ALERT</i> Incineration Corps sighted"),
    ];
    let (threads, claimed) = campaign_threads(&campaigns, &list);
    let mut claimed: Vec<i64> = claimed.into_iter().collect();
    claimed.sort();
    assert_eq!(claimed, [1, 2]);
    let mine = threads.iter().find(|t| t.campaign.id == e.id).unwrap();
    let ids: Vec<i64> = mine.entries.iter().filter_map(|x| x.dispatch.as_ref().map(|d| d.id)).collect();
    assert_eq!(ids, [1, 2]);
    // The first order's announcement is told on the launch, not as a second line.
    assert_eq!(mine.entries[0].event.as_ref().map(|e| e.kind), Some(EventType::Launched));
    assert_eq!(mine.entries[0].dispatch.as_ref().map(|d| d.id), Some(1));
    assert!(mine.entries.windows(2).all(|w| w[0].at <= w[1].at));
}

#[test]
fn doesnt_let_an_order_the_game_hasnt_named_yet_claim_every_major_order_headline() {
    let mut sorted = campaigns_live();
    sorted.sort_by(|a, b| a.start.total_cmp(&b.start));
    let older = sorted[sorted.len() - 2].clone();
    let mut unnamed = sorted[sorted.len() - 1].clone();
    let mut extra = unnamed.phases[0].clone();
    extra.title = "MAJOR ORDER".into();
    extra.status = EpisodeStatus::Active;
    unnamed.phases.push(extra);
    let old = dispatch(9, older.start + H, "<i=3>MAJOR ORDER WON</i> Done");
    let (threads, _) = campaign_threads(&[older, unnamed.clone()], &[old]);
    let t = threads.iter().find(|t| t.campaign.id == unnamed.id).unwrap();
    assert!(!t.entries.iter().any(|x| x.dispatch.as_ref().is_some_and(|d| d.id == 9)));
}

#[test]
fn matches_order_names_as_whole_words_and_only_while_their_campaign_runs() {
    let mut e = campaigns_live()[0].clone();
    e.title = "CELESTIAL FENCE".into();
    e.start = 0.0;
    e.end = Some(10.0 * H);
    let mut phase = e.phases[0].clone();
    phase.title = "DEPLOYMENT".into();
    e.phases = vec![phase];
    let list = [e.clone()];
    let d = |h: f64, message: &str| dispatch(h as i64, h * H, message);
    assert_eq!(campaign_of(&d(5.0, "<i=3>WARFRONT UPDATE</i> DEPLOYMENT is under way"), &list), Some(&e));
    assert_eq!(campaign_of(&d(5.0, "<i=3>ARSENAL AUGMENTATION</i> for all Helldiver deployments"), &list), None);
    assert_eq!(campaign_of(&d(500.0, "<i=3>WARFRONT UPDATE</i> DEPLOYMENT is under way"), &list), None);
}

#[test]
fn never_dates_an_order_after_its_campaign_ended_or_before_it_began() {
    let campaigns = campaigns_live();
    let ended = campaigns.iter().find(|c| c.status != EpisodeStatus::Active).unwrap();
    let end = ended.end.unwrap();
    assert_eq!(issued_at(ended.start, ended.end, Some(end + 1.0)), None);
    assert_eq!(issued_at(ended.start, ended.end, Some(ended.start - 1.0)), None);
    assert_eq!(issued_at(ended.start, ended.end, Some(ended.start + 1.0)), Some(ended.start + 1.0));
}

#[test]
fn names_known_rewards_and_falls_back_to_the_type_then_a_generic_label() {
    assert_eq!(reward_label(campaigns_live()[0].reward), "M-103 Supply FRV");
    assert_eq!(reward_label(Some(2659326411)), "Stratagem");
    assert_eq!(reward_label(None), "Campaign reward");
}

const T0: f64 = 1_000.0 * BUCKET_MS;

fn bucket(at: f64, n: f64) -> FrontBucket {
    FrontBucket { at, terminids: n, automatons: 0.0, illuminate: 0.0, players: n }
}

fn reading(at: f64, split: f64) -> Reading {
    Reading { at, impact: 0.01, players: 5000.0, terminids: split, automatons: 0.0, illuminate: 0.0 }
}

#[test]
fn uses_the_buckets_until_the_first_reading_then_each_readings_own_split() {
    let rows = merge_activity(
        &[bucket(T0, 100.0), bucket(T0 + BUCKET_MS, 200.0), bucket(T0 + 2.0 * BUCKET_MS, 999.0)],
        &[reading(T0 + BUCKET_MS / 2.0, 4000.0)],
        T0 + 2.0 * BUCKET_MS + 60_000.0,
    );
    let got: Vec<(f64, f64, Option<f64>)> = rows.iter().map(|r| (r.at - T0, r.terminids, r.online)).collect();
    assert_eq!(got, [(0.0, 100.0, None), (BUCKET_MS / 2.0, 4000.0, Some(5000.0))]);
}

#[test]
fn draws_from_the_readings_alone_once_they_cover_an_hour() {
    let rows = merge_activity(
        &[bucket(T0, 100.0), bucket(T0 + 8.0 * BUCKET_MS, 200.0)],
        &[reading(T0 + 4.0 * BUCKET_MS, 4000.0), reading(T0 + 8.0 * BUCKET_MS, 4100.0)],
        T0 + 10.0 * BUCKET_MS,
    );
    assert_eq!(rows.iter().map(|r| r.terminids).collect::<Vec<_>>(), [4000.0, 4100.0]);
}

#[test]
fn falls_back_to_the_buckets_for_a_reading_saved_without_a_split() {
    let rows = merge_activity(
        &[bucket(T0, 100.0), bucket(T0 + BUCKET_MS, 200.0)],
        &[reading(T0 + BUCKET_MS / 2.0, 0.0)],
        T0 + 3.0 * BUCKET_MS,
    );
    assert_eq!(rows.last().unwrap().terminids, 150.0);
}

#[test]
fn ticker_shows_the_last_day_newest_first_with_captures_and_losses() {
    let now = ms("2026-10-23T12:00:00Z");
    let d = |id: i64, hours_ago: f64| dispatch(id, now - hours_ago * H, &format!("<i=3>NEWS {id}</i>\n\nBody"));
    let change = OwnerChange {
        planet: 7,
        planet_name: "Fury".into(),
        from: Faction::Automaton,
        to: Faction::Humans,
        at: now - 1_800_000.0,
    };
    let items = ticker_items(&(1..=6).map(|i| d(i, i as f64)).collect::<Vec<_>>(), &[change], &[], now);
    assert_eq!(
        (items[0].kind, items[0].text.as_str(), items[0].planet),
        (TickerKind::Liberated, "Fury liberated", Some(7))
    );
    assert_eq!(items.len(), 7);
}

#[test]
fn ticker_tops_a_quiet_day_up_with_older_news() {
    let now = ms("2026-10-23T12:00:00Z");
    let d = |id: i64, hours_ago: f64| dispatch(id, now - hours_ago * H, &format!("<i=3>NEWS {id}</i>\n\nBody"));
    let items = ticker_items(&[d(1, 30.0), d(2, 50.0), d(3, 2.0)], &[], &[], now);
    assert_eq!(items.iter().map(|i| i.text.as_str()).collect::<Vec<_>>(), ["NEWS 3", "NEWS 1", "NEWS 2"]);
    assert_eq!(fmt_age_short(now - 90.0 * 60_000.0, now), "1h");
}

fn front_rows() -> Vec<FrontRow> {
    let at = |q: f64| iso_string(ms("2026-10-23T12:00:00Z") + q * BUCKET_MS);
    let row = |q, owner, planet, players| FrontRow { bucket: at(q), owner, planet, players };
    vec![
        row(0.0, 3, None, 100.0),
        row(4.0, 3, None, 200.0),
        row(4.0, 1, Some(9), 50.0),
        row(5.0, 2, None, 300.0),
        row(5.0, 1, Some(8), 70.0),
    ]
}

#[test]
fn credits_defenses_to_their_attacker_and_drops_ones_it_cant_place() {
    let buckets = front_buckets(&front_rows(), |p| (p == 9).then_some(Faction::Illuminate));
    assert_eq!((buckets[1].automatons, buckets[1].illuminate, buckets[1].players), (200.0, 50.0, 250.0));
    assert_eq!((buckets[2].terminids, buckets[2].players), (300.0, 300.0));
}

#[test]
fn starts_the_chart_after_the_latest_gap_in_the_data() {
    let buckets = front_buckets(&front_rows(), |_| None);
    let run: Vec<f64> = latest_run(&buckets).iter().map(|b| b.at).collect();
    let at = |q: f64| ms("2026-10-23T12:00:00Z") + q * BUCKET_MS;
    assert_eq!(run, [at(4.0), at(5.0)]);
}

#[test]
fn drops_calypso_which_the_game_removed_but_the_api_still_lists() {
    let planets = planets();
    let calypso = planets.iter().find(|p| p.name == "CALYPSO").unwrap();
    assert!(is_off_map(calypso.index, calypso.position));
    let fixed = fixed_planets();
    assert!(!is_off_map(fixed[1].index, fixed[1].position));
}
