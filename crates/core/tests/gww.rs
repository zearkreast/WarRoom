mod common;

use common::*;
use std::collections::HashSet;
use warroom_core::data::manuals::{CHAPTERS, MANUALS, TIPS};
use warroom_core::data::sites::{ADS, CITIZENS, COMPANIES, GOV, HOLIDAYS, HOLIDAYS_SITE, MINISTRIES};
use warroom_core::data::stations::{PAST_STATIONS, STAR_OF_PEACE};
use warroom_core::data::{AdBy, Section};
use warroom_core::gww_nav::{addresses, app_page_title, page_of, suggested_addresses, target_at};
use warroom_core::gww_text::{Segment, links_in, next_yearly, parse_rich, plain_text};

fn all_sections() -> Vec<&'static Section> {
    MANUALS
        .iter()
        .flat_map(|m| m.sections)
        .chain(MINISTRIES.iter().flat_map(|m| m.sections))
        .chain(COMPANIES.iter().flat_map(|c| c.sections))
        .chain(GOV.sections)
        .chain(CITIZENS.sections)
        .chain(HOLIDAYS.iter().flat_map(|h| h.sections))
        .collect()
}

fn all_text() -> Vec<&'static str> {
    let mut out = Vec::new();
    for s in all_sections() {
        out.push(s.text.unwrap_or(""));
        out.extend(s.points.unwrap_or_default());
        out.extend(s.table.as_ref().map(|t| t.rows).unwrap_or_default().iter().flat_map(|r| r.cells.iter().copied()));
        out.extend(s.kinds.unwrap_or_default().iter().map(|k| k.text));
        out.extend(s.faq.unwrap_or_default().iter().map(|f| f.a));
        out.extend(s.timeline.unwrap_or_default().iter().map(|d| d.text));
        out.extend(s.folds.unwrap_or_default().iter().flat_map(|f| f.points.iter().copied()));
    }
    out.extend(MANUALS.iter().flat_map(|m| m.tips.iter().copied()));
    out
}

fn targets() -> Vec<String> {
    let mut out: Vec<String> = all_text().into_iter().flat_map(links_in).collect();
    out.extend(MANUALS.iter().flat_map(|m| m.related.iter().map(|s| s.to_string())));
    for (related, ties) in
        MINISTRIES.iter().map(|m| (m.related, m.ties)).chain(COMPANIES.iter().map(|c| (c.related, c.ties)))
    {
        out.extend(related.unwrap_or_default().iter().map(|s| s.to_string()));
        out.extend(ties.unwrap_or_default().iter().map(|t| t.to.to_string()));
    }
    out.extend(GOV.related.iter().chain(CITIZENS.related).map(|s| s.to_string()));
    out.extend(ADS.iter().filter_map(|a| match a.by {
        AdBy::Site(s) => Some(s.to_string()),
        AdBy::Brand(_) => None,
    }));
    out
}

fn planet_names() -> HashSet<String> {
    planets().iter().map(|p| p.name.to_lowercase()).collect()
}

#[test]
fn places_every_advert_it_names_and_links_a_sites_own_advert_there() {
    let placed: Vec<&str> = MINISTRIES
        .iter()
        .flat_map(|m| m.ads.unwrap_or_default())
        .chain(COMPANIES.iter().flat_map(|c| c.ads.unwrap_or_default()))
        .copied()
        .collect();
    assert_eq!(placed.iter().filter(|f| !ADS.iter().any(|a| a.file == **f)).count(), 0);
    // A site's own advert shows on its site, so it has to link there.
    let site_of = |f: &str| {
        MINISTRIES
            .iter()
            .find(|m| m.ads.is_some_and(|a| a.contains(&f)))
            .map(|m| m.id)
            .or_else(|| COMPANIES.iter().find(|c| c.ads.is_some_and(|a| a.contains(&f))).map(|c| c.id))
    };
    let wrong: Vec<&str> = ADS
        .iter()
        .filter(|a| match site_of(a.file) {
            None => false,
            Some(id) => !matches!(a.by, AdBy::Site(s) if s.ends_with(&format!(":{id}"))),
        })
        .map(|a| a.file)
        .collect();
    assert!(wrong.is_empty(), "{wrong:?}");
}

#[test]
fn gives_every_site_its_own_address_and_id() {
    let mut list: Vec<&str> = MANUALS.iter().map(|m| m.address).collect();
    list.extend(MINISTRIES.iter().map(|m| m.address));
    list.extend(COMPANIES.iter().map(|c| c.address));
    list.extend(HOLIDAYS.iter().map(|h| h.address));
    list.extend([GOV.address, CITIZENS.address, HOLIDAYS_SITE.address]);
    assert_eq!(list.iter().collect::<HashSet<_>>().len(), list.len());
    assert_eq!(MANUALS.iter().map(|m| m.id).collect::<HashSet<_>>().len(), MANUALS.len());
    assert_eq!(MINISTRIES.iter().map(|m| m.id).collect::<HashSet<_>>().len(), MINISTRIES.len());
    assert_eq!(COMPANIES.iter().map(|m| m.id).collect::<HashSet<_>>().len(), COMPANIES.len());
    assert_eq!(HOLIDAYS.iter().map(|h| h.id).collect::<HashSet<_>>().len(), HOLIDAYS.len());
}

#[test]
fn links_only_to_pages_tracker_views_and_planets_that_exist() {
    let planets = planet_names();
    let broken: Vec<String> = targets()
        .into_iter()
        .filter(|t| {
            let id = &t[t.find(':').map_or(0, |i| i + 1)..];
            if t.starts_with("planet:") {
                !planets.contains(&id.to_lowercase())
            } else if t.starts_with("app:") {
                app_page_title(id).is_none()
            } else {
                page_of(t).is_none()
            }
        })
        .collect();
    assert!(broken.is_empty(), "{broken:?}");
}

#[test]
fn names_only_planets_that_exist_in_the_station_histories() {
    let planets = planet_names();
    let mut text: Vec<&str> = vec![STAR_OF_PEACE.summary];
    text.extend(STAR_OF_PEACE.points);
    text.extend(STAR_OF_PEACE.timeline.iter().map(|e| e.text));
    for s in PAST_STATIONS {
        text.push(s.note);
        text.extend(s.sections.iter().map(|x| x.text));
        text.extend(s.timeline.iter().map(|e| e.text));
    }
    let bad: Vec<String> = text
        .into_iter()
        .flat_map(links_in)
        .filter(|t| !t.starts_with("planet:") || !planets.contains(&t[7..].to_lowercase()))
        .collect();
    assert!(bad.is_empty(), "{bad:?}");
}

#[test]
fn opens_a_typed_address_with_or_without_its_scheme_and_every_address_opens_a_page() {
    assert_eq!(target_at("se://gov/truth").as_deref(), Some("ministry:truth"));
    assert_eq!(target_at("  GOV/Truth/ ").as_deref(), Some("ministry:truth"));
    assert_eq!(target_at("se://gww").as_deref(), Some("home"));
    assert_eq!(target_at("se://nowhere"), None);
    assert_eq!(target_at("strohmann.news").as_deref(), Some("app:news"));
    assert_eq!(target_at("se://gov/holidays/liberty-day").as_deref(), Some("holiday:liberty-day"));
    assert!(
        !suggested_addresses().iter().any(|a| a.starts_with("se://gov/citizens") || a.starts_with("se://gov/holidays"))
    );
    let all = addresses();
    assert_eq!(all.iter().filter(|(a, _)| a == "se://strohmann.news").count(), 1);
    let dead: Vec<String> = addresses()
        .into_iter()
        .filter(|(a, _)| {
            let t = target_at(a).unwrap();
            t != "home" && !t.starts_with("app:") && page_of(&t).is_none()
        })
        .map(|(a, _)| a)
        .collect();
    assert!(dead.is_empty(), "{dead:?}");
}

#[test]
fn puts_every_manual_in_a_chapter_and_the_chapters_in_reading_order() {
    let order: Vec<&str> = CHAPTERS.iter().map(|c| c.id).collect();
    let seq: Vec<Option<usize>> = MANUALS.iter().map(|m| order.iter().position(|c| *c == m.chapter)).collect();
    assert!(seq.iter().all(Option::is_some));
    let mut sorted = seq.clone();
    sorted.sort();
    assert_eq!(sorted, seq);
    assert!(!TIPS.is_empty());
}

#[test]
fn splits_links_out_of_text() {
    let text = |t: &str| Segment::Text(t.into());
    let link = |k: &str, t: &str, l: &str| Segment::Link { kind: k.into(), target: t.into(), label: l.into() };
    assert_eq!(
        parse_rich("See [[manual:defense|gambits]] and [[app:map|the map]]."),
        [text("See "), link("manual", "defense", "gambits"), text(" and "), link("app", "map", "the map"), text(".")]
    );
    assert_eq!(parse_rich("[[nope:x|plain]] stays"), [text("plain"), text(" stays")]);
    assert_eq!(plain_text("On [[planet:Tien Kwan|Tien Kwan]]."), "On Tien Kwan.");
}

#[test]
fn counts_down_to_the_next_yearly_day() {
    let days = [(10, 26), (4, 3)];
    assert_eq!(next_yearly(&days, 2026, 10, 25), Some((0, 1)));
    assert_eq!(next_yearly(&days, 2026, 10, 26), Some((0, 0)));
    assert_eq!(next_yearly(&days, 2026, 10, 27), Some((1, 158)));
}
