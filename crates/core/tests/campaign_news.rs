mod common;
use common::*;
use warroom_core::api::Dispatch;
use warroom_core::campaign_news::{
    EventType, campaign_alerts, campaign_events, campaign_headline, campaign_of, campaign_threads,
};
use warroom_core::snapshot::EpisodeStatus;

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
fn pairs_the_announcements_left_with_the_last_orders_once_the_first_has_dropped_out() {
    // The fixture's oldest dispatch is ARMORED EAGLE's second order announcement.
    let (threads, _) = campaign_threads(&campaigns_live(), &dispatches());
    let t = threads.iter().find(|t| t.campaign.title == "ARMORED EAGLE").unwrap();
    let orders: Vec<(usize, Option<i64>)> = t
        .entries
        .iter()
        .filter_map(|x| {
            let ev = x.event.as_ref().filter(|ev| ev.kind != EventType::Ended)?;
            Some((ev.phase, x.dispatch.as_ref().map(|d| d.id)))
        })
        .collect();
    assert_eq!(orders, [(0, None), (1, Some(3941)), (2, Some(3943))]);
}

#[test]
fn an_announcement_ahead_of_its_order_leaves_the_earlier_pairs_alone() {
    let mut list = campaigns_live();
    let mut ds = dispatches();
    let at = |id: i64, ds: &[Dispatch]| ms(&ds.iter().find(|d| d.id == id).unwrap().published);
    let (first, second) = (at(3941, &ds), at(3943, &ds));
    let e = list.iter_mut().find(|e| e.title == "ARMORED EAGLE").unwrap();
    e.phases[1].issued = Some(first);
    e.phases[2].issued = Some(second);
    ds.push(dispatch(99_999, second + 3.0 * H, "<i=3>NEW MAJOR ORDER</i>\nThe next order."));
    let (threads, _) = campaign_threads(&list, &ds);
    let t = threads.iter().find(|t| t.campaign.title == "ARMORED EAGLE").unwrap();
    let orders: Vec<(usize, Option<i64>)> = t
        .entries
        .iter()
        .filter_map(|x| {
            let ev = x.event.as_ref().filter(|ev| ev.kind != EventType::Ended)?;
            Some((ev.phase, x.dispatch.as_ref().map(|d| d.id)))
        })
        .collect();
    assert_eq!(orders, [(0, None), (1, Some(3941)), (2, Some(3943))]);
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
