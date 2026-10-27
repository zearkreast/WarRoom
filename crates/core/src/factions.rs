//! Faction names, colours and ids.

use crate::api::Faction;

pub struct FactionInfo {
    pub label: &'static str,
    /// CSS variable holding this faction's color.
    pub color: &'static str,
}

pub fn info(f: Faction) -> Option<FactionInfo> {
    let (label, color) = match f {
        Faction::Humans => ("Super Earth", "var(--f-humans)"),
        Faction::Terminids => ("Terminids", "var(--f-terminids)"),
        Faction::Automaton => ("Automatons", "var(--f-automaton)"),
        Faction::Illuminate => ("Illuminate", "var(--f-illuminate)"),
        Faction::Unknown => return None,
    };
    Some(FactionInfo { label, color })
}

/// Faction ids as used in assignment task values (valueType 1) and the official API's race codes.
pub fn faction_by_id(id: i64) -> Option<Faction> {
    match id {
        1 => Some(Faction::Humans),
        2 => Some(Faction::Terminids),
        3 => Some(Faction::Automaton),
        4 => Some(Faction::Illuminate),
        _ => None,
    }
}

/// Faction → id, for the smallint owner columns in Supabase.
pub fn faction_id(f: Faction) -> i64 {
    match f {
        Faction::Humans => 1,
        Faction::Terminids => 2,
        Faction::Automaton => 3,
        Faction::Illuminate => 4,
        Faction::Unknown => 0,
    }
}

pub fn faction_color(f: Option<Faction>) -> &'static str {
    f.and_then(info).map_or("var(--text-dim)", |i| i.color)
}

pub fn faction_label(f: Option<Faction>) -> &'static str {
    f.and_then(info).map_or("Unknown", |i| i.label)
}
