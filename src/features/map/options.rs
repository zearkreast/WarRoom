//! The choices for each map setting, with their labels.

use crate::state::{MapBeacon, MapDispatches, MapOverlay, MapPlanets, MapQuality};

pub fn overlay_options() -> Vec<(MapOverlay, &'static str)> {
    vec![(MapOverlay::Regions, "Regions"), (MapOverlay::Sectors, "Sectors"), (MapOverlay::Galaxy, "Galaxy")]
}

pub fn quality_options() -> Vec<(MapQuality, &'static str)> {
    vec![(MapQuality::Min, "Minimal"), (MapQuality::Normal, "Normal"), (MapQuality::Max, "Maximum")]
}

pub fn dispatch_options() -> Vec<(MapDispatches, &'static str)> {
    vec![
        (MapDispatches::Three, "Last 3"),
        (MapDispatches::Five, "Last 5"),
        (MapDispatches::Ten, "Last 10"),
        (MapDispatches::Recent, "72 hours"),
    ]
}

pub fn planet_options() -> Vec<(MapPlanets, &'static str)> {
    vec![(MapPlanets::Markers, "Markers"), (MapPlanets::Spheres, "Spheres")]
}

pub fn beacon_options() -> Vec<(MapBeacon, &'static str)> {
    vec![(MapBeacon::Ping, "Ping"), (MapBeacon::Glow, "Glow"), (MapBeacon::Beacon, "Beacon"), (MapBeacon::Off, "Off")]
}
