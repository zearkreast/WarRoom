//! An icon for each environmental hazard.

use crate::ui::icons::{self, IconData};
use regex_lite::Regex;
use std::sync::LazyLock;

static ICONS: LazyLock<Vec<(Regex, IconData)>> = LazyLock::new(|| {
    [
        ("heat|fire|scorch|volcan", icons::FLAME),
        ("cold|blizzard|snow|ice|frost", icons::SNOWFLAKE),
        ("rain", icons::CLOUD_RAIN),
        ("fog", icons::CLOUD_FOG),
        ("sand|dust|wind", icons::WIND),
        ("meteor|ember", icons::SPARKLES),
        ("acid|toxic|spore", icons::DROPLETS),
        ("ion|electric|lightning|storm", icons::CLOUD_LIGHTNING),
        ("tremor|quake", icons::ACTIVITY),
        ("sun|solar", icons::SUN),
        ("rock|cliff|terrain", icons::MOUNTAIN),
    ]
    .into_iter()
    .map(|(re, icon)| (Regex::new(&format!("(?i){re}")).unwrap(), icon))
    .collect()
});

pub fn hazard_icon(name: &str) -> IconData {
    ICONS.iter().find(|(re, _)| re.is_match(name)).map_or(icons::TRIANGLE_ALERT, |(_, i)| *i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tells_the_storms_apart() {
        assert_eq!(hazard_icon("Ion Storms"), icons::CLOUD_LIGHTNING);
        assert_eq!(hazard_icon("Acid Storms"), icons::DROPLETS);
        assert_eq!(hazard_icon("Meteor Storms"), icons::SPARKLES);
        assert_eq!(hazard_icon("Sandstorms"), icons::WIND);
        assert_eq!(hazard_icon("Fire Tornadoes"), icons::FLAME);
    }
}
