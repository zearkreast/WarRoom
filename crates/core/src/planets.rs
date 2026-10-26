use crate::api::{Named, Planet, Position};
use crate::snapshot::PlanetStatic;
use std::collections::HashMap;

/// Where the game has moved things the API still files under the old planet id. Meridia was dragged
/// into the Celeste sector. On 12 Aug 2026 Sangis and Alderidge Cove traded names, places, biomes and
/// links: the API has the new places and links but the old names, sectors and biomes. Values are
/// absolute, so they stay right if the API catches up.
pub fn fix_static(mut p: PlanetStatic) -> PlanetStatic {
    match p.index {
        64 => p.sector = "Celeste".into(),
        99 => {
            p.name = "ALDERIDGE COVE".into();
            p.sector = "Omega".into();
            p.biome = Named {
                name: "Deadlands".into(),
                description: "The lifeless grey of this planet is interrupted only by the violet flowers that grow from strange, parasitic outcroppings.".into(),
            };
        }
        273 => {
            p.name = "SANGIS".into();
            p.sector = "Guang".into();
            p.biome = Named {
                name: "Deciduous Autumn Forest".into(),
                description: "Broadleaf woodlands spread across rolling hills, broken by shallow waters that wind through mossy logs, flowers, and stone. (UNOFFICIAL)".into(),
            };
        }
        _ => {}
    }
    p
}

/// Planet indices are not array positions (the list has gaps), so look them up by index.
pub fn by_index(planets: &[Planet]) -> HashMap<i64, &Planet> {
    planets.iter().map(|p| (p.index, p)).collect()
}

/// The API stacks unnamed placeholder worlds on Super Earth at the origin; nothing in the war uses them.
pub fn is_placeholder(index: i64, position: Position) -> bool {
    index != 0 && position.x == 0.0 && position.y == 0.0
}

/// Planets the game has taken off the map while the API still lists them. Calypso vanished after the
/// DSS jumped to it on 19 Aug 2026; the API only dropped its supply lines.
const OFF_MAP: [i64; 1] = [97];

/// Placeholders and planets the game no longer shows: neither is drawn or listed.
pub fn is_off_map(index: i64, position: Position) -> bool {
    is_placeholder(index, position) || OFF_MAP.contains(&index)
}

/// Who holds and who attacks each planet, as one string. Territory and region shapes depend on nothing
/// else that changes, so they are recomputed only when this does, not on every poll.
pub fn ownership_key(planets: &[Planet]) -> String {
    let mut s = String::with_capacity(planets.len() * 2);
    for p in planets {
        s.push_str(&p.current_owner.as_str()[..1]);
        match &p.event {
            Some(e) => s.push_str(&e.faction.as_str()[..1]),
            None => s.push('-'),
        }
    }
    s
}
