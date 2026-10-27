//! The game's stratagem icons, as traced on the Helldivers Wiki, by the stratagem's name without its designation.

use crate::assets;
use regex_lite::Regex;
use std::sync::LazyLock;

static AUGMENTATION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^ARSENAL AUGMENTATION:\s*").unwrap());
static NOT_SLUG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^a-z0-9]+").unwrap());

/// The stratagem an arsenal augmentation adds, as the game names it ("E/MG-101 HMG EMPLACEMENT").
pub fn augmented_stratagem(effect_name: &str) -> String {
    AUGMENTATION.replace(effect_name, "").into_owned()
}

/// The icon for a stratagem by its full name; Eagles and orbitals carry no designation to drop.
pub fn stratagem_icon(name: &str) -> Option<String> {
    let words: Vec<&str> = name.split_whitespace().collect();
    let plain = match words.first() {
        Some(w) if w.chars().any(|c| c.is_ascii_digit() || c == '/') => &words[1..],
        _ => &words[..],
    };
    let slug = NOT_SLUG.replace_all(&plain.join(" ").to_lowercase(), "-").trim_matches('-').to_string();
    assets::url(&format!("stratagems/{slug}.svg"))
}
