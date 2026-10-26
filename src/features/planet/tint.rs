use crate::assets;
use warroom_core::api::Planet;
use warroom_core::biomes::{biome_slug, biome_tint, planet_look};

/// Two colours standing in for a planet's look, and its landscape picture as a CSS url().
#[derive(Clone, Debug, PartialEq)]
pub struct Tint {
    pub sky: &'static str,
    pub ground: &'static str,
    pub art: Option<String>,
}

pub fn biome_art(biome: &str) -> Option<String> {
    assets::url(&format!("biomes/{}.png", biome_slug(biome))).map(|u| format!("url(\"{u}\")"))
}

/// A planet's colours and picture: its biome's, or the black hole or debris it has become.
pub fn planet_tint(p: &Planet, effect_ids: &[i64]) -> Tint {
    if let Some((t, file)) = planet_look(p, effect_ids) {
        let art = assets::url(&format!("looks/{file}.png")).map(|u| format!("url(\"{u}\")"));
        return Tint { sky: t.sky, ground: t.ground, art };
    }
    let name = (!p.biome.name.is_empty()).then_some(p.biome.name.as_str());
    let t = biome_tint(name);
    Tint { sky: t.sky, ground: t.ground, art: name.and_then(biome_art) }
}
