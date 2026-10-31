//! Game knowledge the APIs don't carry: the G.W.W.'s manuals and sites, station history, campaign rewards,
//! the stratagems and the fate of Super Earth's cities.

pub mod manuals;
pub mod sites;
pub mod stations;
pub mod stratagems;

use crate::api::Faction;

pub struct Source {
    pub title: &'static str,
    pub url: &'static str,
}

pub struct TableRow {
    /// Names a picture from the G.W.W. art, drawn at the start of its row.
    pub icon: Option<&'static str>,
    pub cells: &'static [&'static str],
}

pub struct Table {
    pub head: &'static [&'static str],
    pub rows: &'static [TableRow],
    pub note: Option<&'static str>,
}

/// A diagram drawn by the app.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Figure {
    SupplyLines,
    Siege,
    Decay,
    ImpactLive,
}

/// A named kind of something, with its picture.
pub struct KindEntry {
    pub icon: &'static str,
    pub name: &'static str,
    pub text: &'static str,
}

pub struct Faq {
    pub q: &'static str,
    pub a: &'static str,
}

/// A group shown folded: a titled list of "Term: text" points.
pub struct Fold {
    pub title: &'static str,
    pub note: &'static str,
    pub points: &'static [&'static str],
}

pub struct Dated {
    pub when: &'static str,
    pub text: &'static str,
}

pub struct Section {
    pub heading: &'static str,
    pub text: Option<&'static str>,
    /// "Term: text" points get their term set in bold.
    pub points: Option<&'static [&'static str]>,
    pub table: Option<Table>,
    pub figure: Option<Figure>,
    pub kinds: Option<&'static [KindEntry]>,
    pub faq: Option<&'static [Faq]>,
    pub folds: Option<&'static [Fold]>,
    pub timeline: Option<&'static [Dated]>,
    /// Named as the game names them.
    pub products: Option<&'static [&'static str]>,
}

impl Section {
    pub const EMPTY: Section = Section {
        heading: "",
        text: None,
        points: None,
        table: None,
        figure: None,
        kinds: None,
        faq: None,
        folds: None,
        timeline: None,
        products: None,
    };
}

pub struct Chapter {
    pub id: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
}

pub struct Manual {
    pub id: &'static str,
    pub chapter: &'static str,
    pub title: &'static str,
    /// Address shown on the tile and in the address bar, in the Federation's net style.
    pub address: &'static str,
    pub summary: &'static str,
    /// A picture from the G.W.W. art; manuals without one get a drawn icon.
    pub icon: Option<&'static str>,
    pub sections: &'static [Section],
    /// The game's own Training Manual Tips that fit the manual, quoted.
    pub quotes: Option<&'static [&'static str]>,
    /// Tips written for this app, mostly on where to see the mechanic in it.
    pub tips: &'static [&'static str],
    /// Pages to read next, as link targets (`manual:defense`, `app:map`).
    pub related: &'static [&'static str],
    pub sources: Option<&'static [Source]>,
}

/// A link to another site, with what ties the two.
pub struct Tie {
    pub to: &'static str,
    pub how: &'static str,
}

pub struct Ministry {
    pub id: &'static str,
    pub name: &'static str,
    pub address: &'static str,
    pub icon: &'static str,
    pub color: &'static str,
    /// One line for its tile.
    pub summary: &'static str,
    /// The game's Training Manual Tip about the ministry, quoted.
    pub motto: &'static str,
    /// A line from its posters, quoted.
    pub slogan: Option<&'static str>,
    pub sections: &'static [Section],
    pub ties: Option<&'static [Tie]>,
    pub related: Option<&'static [&'static str]>,
    pub ads: Option<&'static [&'static str]>,
    pub source: Source,
}

pub struct Showcase {
    pub image: &'static str,
    pub alt: &'static str,
}

pub struct Company {
    pub id: &'static str,
    pub name: &'static str,
    pub address: &'static str,
    pub trade: &'static str,
    /// One line for its tile.
    pub summary: &'static str,
    pub color: &'static str,
    pub logo: Option<&'static str>,
    /// The company's slogan as the game gives it.
    pub slogan: Option<&'static str>,
    /// A line from its Super Destroyer broadcast, quoted.
    pub broadcast: Option<&'static str>,
    /// A Training Manual Tip about the company, quoted.
    pub quote: Option<&'static str>,
    pub showcase: Option<Showcase>,
    pub sections: &'static [Section],
    pub ties: Option<&'static [Tie]>,
    pub related: Option<&'static [&'static str]>,
    pub ads: Option<&'static [&'static str]>,
    pub source: Source,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdShape {
    Wide,
    Tall,
}

impl AdShape {
    pub fn as_str(self) -> &'static str {
        match self {
            AdShape::Wide => "wide",
            AdShape::Tall => "tall",
        }
    }
}

pub enum AdBy {
    /// The advertiser's site, as a link target.
    Site(&'static str),
    /// An advertiser without a site.
    Brand(&'static str),
}

pub struct Ad {
    pub file: &'static str,
    /// What the advert shows and says, for screen readers.
    pub alt: &'static str,
    pub shape: AdShape,
    pub by: AdBy,
}

/// The game's own words, with where they're from.
pub struct Quote {
    pub from: &'static str,
    pub text: &'static str,
}

pub struct Holiday {
    pub id: &'static str,
    pub name: &'static str,
    /// 1-12.
    pub month: u32,
    pub day: u32,
    /// The days it's kept, as the calendar gives them.
    pub when: &'static str,
    pub address: &'static str,
    pub color: &'static str,
    /// One line for its card.
    pub summary: &'static str,
    /// The game's own words about it, quoted.
    pub quotes: &'static [Quote],
    pub sections: &'static [Section],
    pub related: &'static [&'static str],
    pub sources: &'static [Source],
}

pub struct HolidaysSite {
    pub title: &'static str,
    pub address: &'static str,
    pub icon: &'static str,
    pub summary: &'static str,
    pub related: &'static [&'static str],
}

pub struct GovPage {
    pub title: &'static str,
    pub address: &'static str,
    pub icon: &'static str,
    pub summary: &'static str,
    pub sections: &'static [Section],
    pub quotes: &'static [&'static str],
    pub related: &'static [&'static str],
    pub sources: &'static [Source],
}

pub struct CitizensPage {
    pub title: &'static str,
    pub address: &'static str,
    pub icon: &'static str,
    pub summary: &'static str,
    pub broadcasts: &'static [&'static str],
    pub sections: &'static [Section],
    pub related: &'static [&'static str],
    pub sources: &'static [Source],
}

pub struct StationSection {
    pub heading: &'static str,
    pub text: &'static str,
}

/// A station the API doesn't list: retired, destroyed or not yet built; one it lists again should come out of here.
pub struct StationRecord {
    pub id: &'static str,
    /// Short name on the selector.
    pub short: &'static str,
    pub name: &'static str,
    /// Other names it has gone by.
    pub aka: &'static [&'static str],
    /// "Destroyed", "Offline" or "Retired".
    pub status: &'static str,
    pub faction: Faction,
    /// Its emblem and hologram; one drawn for each station.
    pub art: &'static str,
    /// The operation fought over it, which titles its timeline.
    pub operation: &'static str,
    /// Lines under its hologram.
    pub readout: &'static [(&'static str, &'static str)],
    pub note: &'static str,
    pub sections: &'static [StationSection],
    pub timeline: &'static [Dated],
    pub sources: &'static [Source],
}

pub struct StarOfPeace {
    pub status: &'static str,
    pub summary: &'static str,
    pub points: &'static [&'static str],
    pub timeline: &'static [Dated],
    pub sources: &'static [Source],
}

/// A Galactic Campaign reward: what the in-game Control Center calls its type, and the item, once known.
pub struct CampaignReward {
    pub kind: &'static str,
    pub name: Option<&'static str>,
}

/// Galactic Campaign rewards by the item id the game sends with each campaign. The game API doesn't
/// name these items, so they are listed here by hand; add new campaigns' rewards as they are announced.
pub fn campaign_reward(mix_id: i64) -> Option<CampaignReward> {
    let (kind, name) = match mix_id {
        2768520552 => ("Stratagem", Some("M-103 Supply FRV")), // Census Thunder
        2232169940 => ("Primary weapon", Some("R-4 Hyena")),   // Celestial Fence
        1407130184 => ("Helmet", Some("KDM-500 Outrider")),    // Lightning Intercept
        3421347422 => ("Armor", Some("KDM-500 Outrider")),     // Permanent Enclosure
        2547900634 => ("Helmet", Some("IX-Voidwalker")),       // Void Piercer
        2961949068 => ("Stratagem", Some("Eagle Gas Strike")), // Counterdissident Hammer
        3890208935 => ("Cape", Some("Message of Morale")),     // Blazing Electorate
        1618361436 => ("Stratagem", Some("TD-110 Maelstrom")), // Armored Eagle
        2237630340 => ("Tactical action", Some("Eagle Gas Storm")), // Epic Purge
        2659326411 => ("Stratagem", None),                     // Memoriam Eternal
        _ => return None,
    };
    Some(CampaignReward { kind, name })
}

/// Display name for a campaign's reward: the item if known, else its type, else a generic label.
pub fn reward_label(mix_id: Option<i64>) -> &'static str {
    match mix_id.and_then(campaign_reward) {
        Some(r) => r.name.unwrap_or(r.kind),
        None => "Campaign reward",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CityFateState {
    Destroyed,
    Rebuilt,
    Intact,
}

pub struct CityFate {
    pub state: CityFateState,
    /// The name it was rebuilt under.
    pub name: Option<&'static str>,
    pub note: &'static str,
    /// Longitude and latitude of the real-world city it stands on.
    pub at: [f64; 2],
}

/// What became of Super Earth's seven sector capitals in the Illuminate invasion of 2185, by region id.
/// The API still lists them as they stood before it.
pub fn super_earth_city(region: i64) -> Option<&'static CityFate> {
    SUPER_EARTH_CITIES.get(usize::try_from(region).ok()?)
}

pub const SUPER_EARTH_CITIES: [CityFate; 7] = [
    CityFate {
        state: CityFateState::Destroyed,
        name: None,
        note: "The first megacity the Great Host attacked, razed on the invasion's second day.",
        at: [33.0, 30.5],
    },
    CityFate {
        state: CityFateState::Destroyed,
        name: None,
        note: "Razed four days into the invasion, the second megacity to fall.",
        at: [4.35, 50.85],
    },
    CityFate {
        state: CityFateState::Rebuilt,
        name: Some("GUN"),
        note: "Razed four days into the invasion. The first megacity rebuilt, renamed Gun by a Helldiver vote of 72.5%.",
        at: [-58.4, -34.6],
    },
    CityFate {
        state: CityFateState::Rebuilt,
        name: Some("NEW YORK SUPREME"),
        note: "Razed on the invasion's ninth day. Rebuilt as New York Supreme, the name that won the Helldiver vote over Bernard.",
        at: [-74.0, 40.7],
    },
    CityFate {
        state: CityFateState::Destroyed,
        name: None,
        note: "Razed on the invasion's ninth day.",
        at: [103.8, 1.3],
    },
    CityFate {
        state: CityFateState::Intact,
        name: None,
        note: "The federal capital held against the Great Host's last assault, in which the President fell.",
        at: [18.07, 59.33],
    },
    CityFate {
        state: CityFateState::Intact,
        name: None,
        note: "Among the first cities attacked, and held for the whole invasion.",
        at: [121.5, 31.2],
    },
];
