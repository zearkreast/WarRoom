//! The game's faction and sub-faction emblems, as traced on the Helldivers Wiki; every shape fills with `currentColor`.

use crate::assets;
use crate::js::{num, unique_id};
use crate::ui::icons::{Icon, IconData};
use leptos::prelude::*;
use regex_lite::Regex;
use std::sync::LazyLock;
use warroom_core::api::Faction;
use warroom_core::factions::faction_color;

struct Art {
    bx: [f64; 4],
    /// Inner SVG markup, or a white-on-clear picture used as a mask.
    body: Option<String>,
    mask: Option<String>,
}

fn art_of(slug: &str) -> Option<Art> {
    if slug == "invasion-fleet" {
        return Some(Art { bx: [0.0, 0.0, 128.0, 128.0], body: None, mask: assets::url("emblems/invasion-fleet.png") });
    }
    let svg = assets::svg(&format!("emblems/{slug}.svg"))?;
    let vb = svg.split("viewBox=\"").nth(1)?.split('"').next()?;
    let nums: Vec<f64> = vb.split_whitespace().filter_map(|n| n.parse().ok()).collect();
    let open_end = svg.find("<svg")? + svg[svg.find("<svg")?..].find('>')? + 1;
    let close = svg.rfind("</svg>")?;
    let mut body = svg[open_end..close].to_string();
    if let (Some(a), Some(b)) = (body.find("<desc>"), body.find("</desc>")) {
        body.replace_range(a..b + "</desc>".len(), "");
    }
    Some(Art { bx: [nums[0], nums[1], nums[2], nums[3]], body: Some(body), mask: None })
}

fn faction_art(name: &str) -> Option<&'static str> {
    match name {
        "Humans" => Some("super-earth"),
        "Terminids" => Some("terminids"),
        "Automaton" => Some("automaton"),
        "Illuminate" => Some("illuminate"),
        _ => None,
    }
}

// Effect names as the game writes them. Hive Lords and Dragonroaches are units with no emblem of their own.
static SUBFACTION_ART: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    [
        ("predator strain", "predator-strain"),
        ("spore burst strain", "spore-burst-strain"),
        ("rupture strain", "rupture-strain"),
        ("jet brigade", "jet-brigade"),
        ("incineration corps", "incineration-corps"),
        ("cyborg", "cyborgs"),
        ("appropriators", "appropriators"),
        ("vote snatchers", "vote-snatchers"),
        ("mindless masses", "mindless-masses"),
        ("invasion fleet|great host", "invasion-fleet"),
    ]
    .into_iter()
    .map(|(re, art)| (Regex::new(&format!("(?i){re}")).unwrap(), art))
    .collect()
});

/// The emblem for a sub-faction effect, by its name; None for units that have none.
pub fn subfaction_art(name: &str) -> Option<&'static str> {
    SUBFACTION_ART.iter().find(|(re, _)| re.is_match(name)).map(|(_, art)| *art)
}

pub fn has_emblem(faction: &str) -> bool {
    faction_art(faction).is_some()
}

/// An emblem as SVG content, centred on (0, 0) and `size` units across, for use inside another SVG.
/// `art` is a faction name or a sub-faction art key; it takes the colour of `color`, else the faction's.
#[component]
pub fn EmblemArt(
    #[prop(into)] art: String,
    size: f64,
    #[prop(optional, into)] color: MaybeProp<String>,
    #[prop(optional, into)] class: MaybeProp<String>,
    #[prop(optional, into)] style: MaybeProp<String>,
) -> impl IntoView {
    let (color, class, style) = (color.get_untracked(), class.get_untracked(), style.get_untracked());
    let slug = faction_art(&art).unwrap_or(&art).to_string();
    let a = art_of(&slug)?;
    let [x, y, w, h] = a.bx;
    let s = size / w.max(h);
    let transform = format!("translate({},{}) scale({}) translate({},{})", num(-(w * s) / 2.0), num(-(h * s) / 2.0), num(s), num(-x), num(-y));
    let tone = color.or_else(|| faction_art(&art).map(|_| faction_color(Faction::parse(&art)).to_string()));
    // --u: a 24-grid unit in the emblem's own units, for strokes drawn over it.
    let mut look = style.unwrap_or_default();
    if let Some(t) = &tone {
        look.push_str(&format!("color: {t}; "));
    }
    look.push_str(&format!("--u: {};", num(w.max(h) / 24.0)));
    Some(match (a.mask, a.body) {
        (Some(mask), _) => {
            let id = unique_id("emblem-mask-");
            view! {
                <g class=class style=look transform=transform>
                    <mask id=id.clone()>
                        <image href=mask x=num(x) y=num(y) width=num(w) height=num(h) />
                    </mask>
                    <rect
                        x=num(x)
                        y=num(y)
                        width=num(w)
                        height=num(h)
                        style="fill: currentColor; stroke: none;"
                        mask=format!("url(#{id})")
                    />
                </g>
            }
            .into_any()
        }
        (None, body) => {
            view! { <g class=class style=look transform=transform inner_html=body.unwrap_or_default()></g> }.into_any()
        }
    })
}

/// The game paints the Super Earth emblem cream over gold: the globe pale, the arcs beneath it gold.
#[component]
pub fn SealGradient(#[prop(into)] id: String) -> impl IntoView {
    view! {
        <linearGradient id=id x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#f4e4c1" />
            <stop offset="58%" stop-color="#ead09a" />
            <stop offset="100%" stop-color="#d9a521" />
        </linearGradient>
    }
}

/// The Super Earth emblem in the game's cream and gold, for use inside another SVG with a SealGradient.
#[component]
pub fn SealArt(size: f64, #[prop(into)] gradient: String) -> impl IntoView {
    view! { <EmblemArt art="Humans" size=size class="emblem-seal" style=format!("--seal: url(#{gradient}); ") /> }
}

/// The Super Earth emblem as the game shows it, at least 36px so its globe and arcs stay apart.
#[component]
pub fn SuperEarthSeal(#[prop(default = 40.0)] size: f64) -> impl IntoView {
    let id = unique_id("seal-");
    view! {
        <svg class="emblem" width=num(size) height=num(size) viewBox="-12 -12 24 24" aria-hidden="true">
            <defs>
                <SealGradient id=id.clone() />
            </defs>
            <SealArt size=24.0 gradient=id />
        </svg>
    }
}

#[component]
pub fn FactionEmblem(
    #[prop(into)] faction: String,
    #[prop(default = 16.0)] size: f64,
    #[prop(optional, into)] color: MaybeProp<String>,
) -> impl IntoView {
    has_emblem(&faction).then(|| {
        view! {
            <svg class="emblem" width=num(size) height=num(size) viewBox="-12 -12 24 24" aria-hidden="true">
                <EmblemArt art=faction size=24.0 color=color />
            </svg>
        }
    })
}

/// A sub-faction's emblem by its effect name, or its faction's when the unit has none.
#[component]
pub fn SubfactionEmblem(
    #[prop(into)] name: String,
    #[prop(into)] faction: String,
    #[prop(default = 16.0)] size: f64,
    #[prop(optional, into)] color: MaybeProp<String>,
) -> impl IntoView {
    let art = subfaction_art(&name).map(String::from).unwrap_or_else(|| faction.clone());
    let color = color.get_untracked().unwrap_or_else(|| faction_color(Faction::parse(&faction)).to_string());
    view! {
        <svg class="emblem" width=num(size) height=num(size) viewBox="-12 -12 24 24" aria-hidden="true">
            <EmblemArt art=art size=24.0 color=color />
        </svg>
    }
}

/// The game's own DSS glyph, drawn like a Lucide icon in the current colour.
#[component]
pub fn DssIcon(
    #[prop(default = 24.0)] size: f64,
    #[prop(optional, into)] class: MaybeProp<String>,
    #[prop(optional, into)] color: MaybeProp<String>,
) -> impl IntoView {
    let class = format!("emblem {}", class.get_untracked().unwrap_or_default());
    let style = color.get_untracked().map(|c| format!("color: {c};"));
    view! {
        <svg class=class width=num(size) height=num(size) viewBox="-12 -12 24 24" style=style aria-hidden="true">
            <EmblemArt art="dss" size=24.0 />
        </svg>
    }
}

/// A space station's emblem: the DSS glyph, or the Automaton cog and star the enemy station wore.
#[component]
pub fn StationEmblem(
    #[prop(into)] station: String,
    #[prop(default = 16.0)] size: f64,
    #[prop(optional, into)] color: MaybeProp<String>,
) -> impl IntoView {
    let art = if station == "dss" { "dss" } else { "automaton-station" };
    view! {
        <svg class="emblem" width=num(size) height=num(size) viewBox="-12 -12 24 24" aria-hidden="true">
            <EmblemArt art=art size=24.0 color=color />
        </svg>
    }
}

/// The Hellpad's mark, the four stratagem arrows a Helldiver keys in on it; drawn like a Lucide icon.
#[component]
pub fn HellpadMark(#[prop(default = 24.0)] size: f64, #[prop(optional, into)] class: MaybeProp<String>) -> impl IntoView {
    let class = class.get_untracked();
    // Heavier lines at dock size, where the mast's would thin out to a hairline.
    let small = size < 40.0;
    view! {
        <svg class=class width=num(size) height=num(size) viewBox="0 0 64 64" aria-hidden="true">
            <path
                d="M14 4h36l10 10v36L50 60H14L4 50V14Z"
                fill="none"
                stroke="currentColor"
                stroke-width=if small { "5" } else { "2.5" }
            />
            <path d="M18 9h28l9 9v28l-9 9H18l-9-9V18Z" fill="currentColor" opacity="0.08" />
            <g fill="currentColor">
                <path d="M32 12l8 9h-16Z" />
                <path d="M52 32l-9 8v-16Z" />
                <path d="M32 52l-8-9h16Z" />
                <path d="M12 32l9-8v16Z" />
            </g>
            <rect x="28" y="28" width="8" height="8" fill="none" stroke="currentColor" stroke-width=if small { "4" } else { "2" } />
        </svg>
    }
}

/// App mark: a shield with a star, drawn in the CTA colour.
#[component]
pub fn AppMark(#[prop(default = 26.0)] size: f64) -> impl IntoView {
    view! {
        <svg width=num(size) height=num(size) viewBox="0 0 32 32" aria-hidden="true">
            <path d="M16 2 29 8v8c0 7-5.5 12-13 14C8.5 28 3 23 3 16V8z" fill="none" stroke="var(--cta)" stroke-width="2" />
            <path d="m16 9 2.2 4.6 5 .6-3.7 3.4 1 5-4.5-2.5-4.5 2.5 1-5-3.7-3.4 5-.6z" fill="var(--cta)" />
        </svg>
    }
}

/// Anything drawn like a Lucide icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconLike {
    Lucide(IconData),
    Dss,
    Hellpad,
}

impl From<IconData> for IconLike {
    fn from(icon: IconData) -> Self {
        IconLike::Lucide(icon)
    }
}

/// Draws an icon at `size` with `stroke_width` lines and an extra class.
pub fn icon_view(icon: IconLike, size: f64, stroke_width: f64, class: Option<String>) -> AnyView {
    match icon {
        IconLike::Lucide(data) => view! { <Icon icon=data size=size stroke_width=stroke_width class=class /> }.into_any(),
        IconLike::Dss => view! { <DssIcon size=size class=class /> }.into_any(),
        IconLike::Hellpad => view! { <HellpadMark size=size class=class /> }.into_any(),
    }
}
