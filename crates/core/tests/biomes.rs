use warroom_core::api::{Named, Planet};
use warroom_core::biomes::{biome_slug, planet_look};

#[test]
fn shows_what_a_destroyed_planet_has_become_instead_of_its_old_biome() {
    let planet = |name: &str, biome: &str| Planet {
        name: name.into(),
        biome: Named { name: biome.into(), description: String::new() },
        ..Default::default()
    };
    let look = |p: Planet| planet_look(&p, &[]).map(|(_, art)| art);
    assert_eq!(look(planet("MERIDIA", "Supercolony")), Some("meridian-black-hole"));
    assert_eq!(look(planet("PENTA", "Deadlands")), Some("black-hole"));
    assert_eq!(look(planet("MORADESH", "Desert Dunes")), Some("fractured"));
    assert_eq!(look(planet("FURY", "Magma")), None);
    assert_eq!(biome_slug("Magma"), "magma");
}
