//! Space station history the API doesn't carry: stations it no longer lists, and the DSS's Star of Peace.
//! Text is adapted from the Helldivers Wiki; game dates are SEST. Planet names may be written as
//! `[[planet:Name|label]]` links (see `gww_text`).

use super::*;

pub const PAST_STATIONS: &[StationRecord] = &[StationRecord {
    id: "css",
    short: "CSS",
    name: "Automaton Orbital Superweapon",
    aka: &[
        "Automaton Space Station, to Helldivers",
        "prototype orbital platform, in the campaign briefing",
        "socialist death laser, in The Patriotic Press",
    ],
    status: "Destroyed",
    faction: Faction::Automaton,
    art: "css",
    operation: "Operation Lightning Intercept",
    readout: &[("Struck", "Varylia 5 · Yed Prior"), ("Last seen", "Mort orbit"), ("Destroyed", "27 Jul 2186")],
    note: "A prototype orbital platform the Cyborgs deployed in July 2186. It struck Varylia 5 on 14 July and Yed Prior on 20 July; the DSS heavily damaged it over Mort on 26 July, and it was destroyed in Mort's orbit on 27 July 2186, minutes after the Helldivers liberated X-45, the hub of its facilities.",
    sections: &[
        StationSection {
            heading: "How it struck",
            text: "It appeared in a planet's orbit, carried out a planet-wide kinetic bombardment and vanished, and an Automaton invasion followed each strike. Struck planets burned on the galaxy map but not on the ground. Helldivers fought it by voting the DSS onto the planet it would strike next.",
        },
        StationSection {
            heading: "What it looked like",
            text: "A near-spherical dark hull, ribbed from pole to pole and pricked with red lights, with a collar on top and a short barrel on one flank. A Super Earth propaganda poster painted the Automaton cog and star on its face.",
        },
        StationSection {
            heading: "Afterwards",
            text: "No new Automaton station is recorded since, but the Automatons took [[planet:X-45|X-45]] back on 9 October 2186. Whether the platform was built from the Star of Peace blueprints stolen in January is not known.",
        },
    ],
    timeline: &[
        Dated {
            when: "14 Jul 2186",
            text: "Bombards [[planet:Varylia 5|Varylia 5]] and vanishes; the Automatons invade. Operation Lightning Intercept begins: hold Varylia 5 and fight on [[planet:Marfark|Marfark]], where a new Cyborg megafactory stood.",
        },
        Dated {
            when: "18 Jul 2186",
            text: "Intel says its next test is in the Tanis sector. Helldivers vote the DSS to [[planet:Clasa|Clasa]].",
        },
        Dated {
            when: "20 Jul 2186",
            text: "Strikes [[planet:Yed Prior|Yed Prior]] instead. Helldivers hold it and liberate [[planet:Wezen|Wezen]] to free cut-off SEAF decrypters.",
        },
        Dated {
            when: "23 Jul 2186",
            text: "[[planet:X-45|X-45]] is named the hub of its facilities, and Helldivers are sent to take it.",
        },
        Dated {
            when: "24 Jul 2186",
            text: "Intel seized on [[planet:Choepessa IV|Choepessa IV]] shows the Automatons sharpening their scans of planets with thick fog.",
        },
        Dated { when: "25 Jul 2186", text: "A second vote sends the DSS to [[planet:Mort|Mort]], a thick-fog world." },
        Dated {
            when: "26 Jul 2186",
            text: "It warps into Mort's orbit and the DSS fires on it: heavy damage, but still operational. Mort is not struck, and the Automatons invade it.",
        },
        Dated {
            when: "27 Jul 2186",
            text: "X-45 is liberated; a minute later High Command reports the weapon intercepted before it could strike Mort, then fully destroyed. Its debris rains onto Mort. Helldivers earn the KDM-500 Outrider helmet.",
        },
        Dated { when: "9 Oct 2186", text: "The Automatons retake [[planet:X-45|X-45]]." },
    ],
    sources: &[
        Source {
            title: "Automaton Orbital Superweapon",
            url: "https://helldivers.wiki.gg/wiki/Automaton_Orbital_Superweapon",
        },
        Source { title: "Major Orders", url: "https://helldivers.wiki.gg/wiki/Major_Orders" },
        Source { title: "Battle Tracker", url: "https://helldivers.wiki.gg/wiki/Battle_Tracker" },
    ],
}];

/// The DSS's planet-collapsing weapon, from Operation Free Space on.
pub const STAR_OF_PEACE: StarOfPeace = StarOfPeace {
    status: "Unarmed",
    summary: "The Democracy Space Station's planet-collapsing weapon: a beam of ionized Dark Fluid. Its one shot turned [[planet:Penta|Penta]] into a black hole and spent every drop of Dark Fluid Super Earth had made; it has been unarmed since.",
    points: &[
        "Weapon: a macro-scale laser whose gain medium is synthesized Dark Fluid. Heatsinks of iridium-infused superalloy, taken from the Automatons on the magma planets, shield the station from its own heat.",
        "Fuel: Dark Fluid is made, very inefficiently, from E-711, which only Terminid Hive Worlds yield. Its one firing used all of it and damaged the DSS.",
        "Built: at Pandora Base, a secret Ministry of Expansion foundry on [[planet:Afoyay Bay|Afoyay Bay]], by dissidents from the Center for the Confinement of Dissidence.",
        "Targets: Helldivers voted between [[planet:Penta|Penta]] (Automatons), [[planet:Valmox|Valmox]] (Illuminate) and [[planet:Zagon Prime|Zagon Prime]] (Terminids). The planet next to the target had to be Super Earth's when it fired.",
    ],
    timeline: &[
        Dated {
            when: "11 Dec 2185",
            text: "Operation Free Space begins: Helldivers hold [[planet:Afoyay Bay|Afoyay Bay]], [[planet:Fury|Fury]], [[planet:K|K]] and [[planet:Mox|Mox]] so Pandora Base can be built.",
        },
        Dated {
            when: "18 Dec 2185",
            text: "Construction of Pandora Base begins, and High Command reveals what it is for: the Star of Peace.",
        },
        Dated {
            when: "2 Jan 2186",
            text: "Helldivers gather E-711 on [[planet:Omicron|Omicron]]. The order fails on 8 January: too little to fire. By 14 January there is barely enough Dark Fluid for one shot.",
        },
        Dated {
            when: "8 Jan 2186",
            text: "Heatsinks are taken from the Automatons on Fury, K and Mox. Strohmann News announces the weapon; on 14 January a test fire crushes a small moon.",
        },
        Dated { when: "16 Jan 2186", text: "The target vote opens, with the DSS offline to prepare. Penta wins." },
        Dated {
            when: "18 Jan 2186",
            text: "At 21:28 the Star of Peace fires on [[planet:Penta|Penta]], which collapses into a stable black hole. All the Dark Fluid is spent and the DSS is damaged. Penta's fall cuts [[planet:Merak|Merak]]'s supply lines.",
        },
        Dated {
            when: "21 Jan 2186",
            text: "A data breach at the firing is suspected, and the DSS is locked down for a Ministry of Truth inquiry while Helldivers hold Afoyay Bay and clear a way out through [[planet:Valmox|Valmox]] and [[planet:Myrium|Myrium]]. On 27 January the inquiry confirms the full blueprints were downloaded, and repairs resume.",
        },
        Dated {
            when: "3 Feb 2186",
            text: "The blueprints are found on a terminal on [[planet:Clasa|Clasa]]. Fearing a socialist Star of Peace, Super Earth marches on [[planet:Cyberstan|Cyberstan]]. On 10 February High Command names the Cyborgs as the thieves; Operation Valid Pretext fails on 21 February.",
        },
        Dated {
            when: "9 Jun 2186",
            text: "An order to re-arm it with E-711 from Omicron fails. The Star of Peace remains unarmed, and the DSS serves without it.",
        },
    ],
    sources: &[
        Source { title: "Democracy Space Station", url: "https://helldivers.wiki.gg/wiki/Democracy_Space_Station" },
        Source { title: "Major Orders", url: "https://helldivers.wiki.gg/wiki/Major_Orders" },
        Source { title: "Conventional Black Hole", url: "https://helldivers.wiki.gg/wiki/Conventional_Black_Hole" },
        Source { title: "Pandora Base", url: "https://helldivers.wiki.gg/wiki/Pandora_Base" },
    ],
};
