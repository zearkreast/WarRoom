//! The game's stratagem icons, as traced on the Helldivers Wiki, by the stratagem's name without its designation.

use crate::assets;
use leptos::prelude::*;
use regex_lite::Regex;
use std::sync::LazyLock;
use warroom_core::stratagems::{Arrow, Permit, Stratagem};

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

/// A stratagem's icon by its id in the collection.
pub fn icon_of(s: &Stratagem) -> Option<String> {
    assets::url(&format!("stratagems/{}.svg", s.id))
}

pub fn permit_color(p: Permit) -> &'static str {
    match p {
        Permit::Offensive => "var(--p-offensive)",
        Permit::Supply => "var(--p-supply)",
        Permit::Defensive => "var(--p-defensive)",
        Permit::Mission => "var(--p-mission)",
    }
}

/// The code as lit arrows, read out as words.
pub fn code_view(code: &'static [Arrow]) -> AnyView {
    let words = code.iter().map(|a| a.as_str()).collect::<Vec<_>>().join(", ");
    view! {
        <span class="hp-keys" role="img" aria-label=format!("Code: {words}")>
            {code.iter().map(|a| view! { <i class=format!("hp-key on {}", a.as_str())></i> }).collect_view()}
        </span>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::icon_of;
    use warroom_core::stratagems::STRATAGEMS;

    #[test]
    fn finds_an_icon_for_every_stratagem() {
        let missing: Vec<&str> = STRATAGEMS.iter().filter(|s| icon_of(s).is_none()).map(|s| s.id).collect();
        assert!(missing.is_empty(), "no icon for {missing:?}");
    }
}
