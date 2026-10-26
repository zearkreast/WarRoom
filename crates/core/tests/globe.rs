use serde::Deserialize;
use warroom_core::geo::{Orthographic, distance, graticule10, topo_polygons};

#[derive(Deserialize)]
struct Case {
    lon: f64,
    land: String,
    grid: String,
    city: [f64; 2],
    dist: f64,
}

#[test]
fn globe_paths_match_d3() {
    let land = topo_polygons(include_str!("../../../src/assets/geo/land-110m.json"), "land").unwrap();
    let grid = graticule10();
    let cases: Vec<Case> = serde_json::from_str(include_str!("fixtures/globe.json")).unwrap();
    for c in cases {
        let p = Orthographic { scale: 84.0, rotate: [c.lon, -20.0] };
        let at = p.project([-74.0, 40.7]);
        assert!((at[0] - c.city[0]).abs() < 1e-9 && (at[1] - c.city[1]).abs() < 1e-9, "city at {}", c.lon);
        assert!((distance([-74.0, 40.7], [-c.lon, 20.0]) - c.dist).abs() < 1e-12);
        assert_eq!(p.lines_path(&grid), c.grid, "grid at {}", c.lon);
        assert_eq!(p.polygons_path(&land), c.land, "land at {}", c.lon);
    }
}
