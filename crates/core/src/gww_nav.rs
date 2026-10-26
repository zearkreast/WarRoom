//! The G.W.W.'s pages and addresses, and the link targets that open them.

use crate::data::manuals::MANUALS;
use crate::data::sites::{CITIZENS, COMPANIES, GOV, HOLIDAYS, HOLIDAYS_SITE, MINISTRIES};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum GwwPage {
    Home,
    Manual {
        id: String,
    },
    Ministry {
        id: String,
    },
    Company {
        id: String,
    },
    Holiday {
        id: String,
    },
    Site {
        id: String,
    },
    Dir {
        id: String,
    },
    /// What the address bar opens for an address no site answers at.
    Missing {
        address: String,
    },
}

const HOME: &str = "se://gww";
pub const LOG_ADDRESS: &str = "se://hc/logs";
pub const LOG_TITLE: &str = "Galactic logs";
const SITES: [(&str, &str); 4] =
    [("gov", GOV.address), ("citizens", CITIZENS.address), ("holidays", HOLIDAYS_SITE.address), ("log", LOG_ADDRESS)];
const DIR_ADDRESSES: [(&str, &str); 3] =
    [("manuals", "se://hc/manuals"), ("gov", "se://gov/directory"), ("corps", "se://corp")];

/// Directory ids and their titles.
pub const DIRS: [(&str, &str); 3] =
    [("manuals", "Training manuals"), ("gov", "Super Earth"), ("corps", "Corporations")];

/// `app:` link targets and their titles.
pub const APP_PAGES: [(&str, &str); 7] = [
    ("news", "Strohmann News"),
    ("overview", "High Command overview"),
    ("map", "Galaxy map"),
    ("campaigns", "Active planets"),
    ("stations", "Space stations"),
    ("past", "Past campaigns"),
    ("planets", "Planet index"),
];

pub fn app_page_title(id: &str) -> Option<&'static str> {
    APP_PAGES.iter().find(|(k, _)| *k == id).map(|(_, t)| *t)
}

fn lookup(list: &[(&'static str, &'static str)], id: &str) -> Option<&'static str> {
    list.iter().find(|(k, _)| *k == id).map(|(_, v)| *v)
}

pub fn address_of(p: &GwwPage) -> String {
    match p {
        GwwPage::Manual { id } => MANUALS.iter().find(|m| m.id == id).map_or(HOME, |m| m.address).into(),
        GwwPage::Ministry { id } => MINISTRIES.iter().find(|m| m.id == id).map_or(HOME, |m| m.address).into(),
        GwwPage::Company { id } => COMPANIES.iter().find(|c| c.id == id).map_or(HOME, |c| c.address).into(),
        GwwPage::Holiday { id } => HOLIDAYS.iter().find(|h| h.id == id).map_or(HOME, |h| h.address).into(),
        GwwPage::Site { id } => lookup(&SITES, id).unwrap_or(HOME).into(),
        GwwPage::Dir { id } => lookup(&DIR_ADDRESSES, id).unwrap_or(HOME).into(),
        GwwPage::Missing { address } => address.clone(),
        GwwPage::Home => HOME.into(),
    }
}

/// Every address on the G.W.W., with the link target it opens. An address listed twice keeps its first place
/// and its last target: Strohmann News is a corporation, but its address opens the tracker's news page.
pub fn addresses() -> Vec<(String, String)> {
    let mut all = vec![(HOME.to_string(), "home".to_string())];
    all.extend(SITES.iter().map(|(id, a)| (a.to_string(), format!("site:{id}"))));
    all.extend(DIR_ADDRESSES.iter().map(|(id, a)| (a.to_string(), format!("dir:{id}"))));
    all.extend(MANUALS.iter().map(|m| (m.address.to_string(), format!("manual:{}", m.id))));
    all.extend(MINISTRIES.iter().map(|m| (m.address.to_string(), format!("ministry:{}", m.id))));
    all.extend(COMPANIES.iter().map(|c| (c.address.to_string(), format!("company:{}", c.id))));
    all.extend(HOLIDAYS.iter().map(|h| (h.address.to_string(), format!("holiday:{}", h.id))));
    all.push(("se://strohmann.news".into(), "app:news".into()));
    let mut out: Vec<(String, String)> = Vec::with_capacity(all.len());
    for (address, target) in all {
        match out.iter_mut().find(|(a, _)| *a == address) {
            Some(seen) => seen.1 = target,
            None => out.push((address, target)),
        }
    }
    out
}

/// Pages reached only through Managed Democracy: they open when typed, but aren't suggested.
pub fn is_gov_subpage(target: &str) -> bool {
    target == "site:citizens" || target == "site:holidays" || target.starts_with("holiday:")
}

/// The addresses the address bar suggests.
pub fn suggested_addresses() -> Vec<String> {
    addresses().into_iter().filter(|(_, t)| !is_gov_subpage(t)).map(|(a, _)| a).collect()
}

/// Typed addresses may skip the scheme or end in a slash.
pub fn normalize(typed: &str) -> String {
    let a = typed.trim().to_lowercase();
    let a = a.trim_end_matches('/');
    if a.starts_with("se://") { a.to_string() } else { format!("se://{}", a.trim_start_matches('/')) }
}

/// The link target a typed address opens, if any site answers at it.
pub fn target_at(typed: &str) -> Option<String> {
    let a = normalize(typed);
    addresses().into_iter().find(|(addr, _)| *addr == a).map(|(_, t)| t)
}

fn split(target: &str) -> (&str, &str) {
    match target.find(':') {
        Some(i) => (&target[..i], &target[i + 1..]),
        None => ("", target),
    }
}

/// The G.W.W. page a link target opens, or None for targets outside it.
pub fn page_of(target: &str) -> Option<GwwPage> {
    if target == "home" {
        return Some(GwwPage::Home);
    }
    let (kind, id) = split(target);
    let id_s = id.to_string();
    match kind {
        "manual" if MANUALS.iter().any(|m| m.id == id) => Some(GwwPage::Manual { id: id_s }),
        "ministry" if MINISTRIES.iter().any(|m| m.id == id) => Some(GwwPage::Ministry { id: id_s }),
        "company" if COMPANIES.iter().any(|c| c.id == id) => Some(GwwPage::Company { id: id_s }),
        "holiday" if HOLIDAYS.iter().any(|h| h.id == id) => Some(GwwPage::Holiday { id: id_s }),
        "site" if matches!(id, "log" | "gov" | "citizens" | "holidays") => Some(GwwPage::Site { id: id_s }),
        "dir" if lookup(&DIRS, id).is_some() => Some(GwwPage::Dir { id: id_s }),
        _ => None,
    }
}

pub fn title_of(target: &str) -> String {
    if target == "home" {
        return "G.W.W. home".into();
    }
    let (_, id) = split(target);
    match page_of(target) {
        Some(GwwPage::Manual { .. }) => MANUALS.iter().find(|m| m.id == id).unwrap().title.into(),
        Some(GwwPage::Ministry { .. }) => MINISTRIES.iter().find(|m| m.id == id).unwrap().name.into(),
        Some(GwwPage::Company { .. }) => COMPANIES.iter().find(|c| c.id == id).unwrap().name.into(),
        Some(GwwPage::Holiday { .. }) => HOLIDAYS.iter().find(|h| h.id == id).unwrap().name.into(),
        Some(GwwPage::Site { id }) => match id.as_str() {
            "gov" => GOV.title.into(),
            "citizens" => CITIZENS.title.into(),
            "holidays" => HOLIDAYS_SITE.title.into(),
            _ => LOG_TITLE.into(),
        },
        Some(GwwPage::Dir { id }) => lookup(&DIRS, &id).unwrap_or_default().into(),
        _ if target.starts_with("app:") => app_page_title(id).unwrap_or(id).into(),
        _ => id.into(),
    }
}
