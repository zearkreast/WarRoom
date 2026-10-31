//! Stratagems: their codes keyed in with WASD or the arrow keys, and what the collection says of each.

pub use crate::data::stratagems::{CALL_IN, STRATAGEMS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arrow {
    Up,
    Down,
    Left,
    Right,
}

impl Arrow {
    pub fn as_str(self) -> &'static str {
        match self {
            Arrow::Up => "up",
            Arrow::Down => "down",
            Arrow::Left => "left",
            Arrow::Right => "right",
        }
    }
}

/// The permit a stratagem is issued under, which gives its icon its colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Permit {
    Offensive,
    Supply,
    Defensive,
    Mission,
}

pub const PERMITS: [Permit; 4] = [Permit::Offensive, Permit::Supply, Permit::Defensive, Permit::Mission];

impl Permit {
    pub fn as_str(self) -> &'static str {
        match self {
            Permit::Offensive => "offensive",
            Permit::Supply => "supply",
            Permit::Defensive => "defensive",
            Permit::Mission => "mission",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Permit::Offensive => "Offensive",
            Permit::Supply => "Supply",
            Permit::Defensive => "Defensive",
            Permit::Mission => "Mission",
        }
    }

    pub fn colour(self) -> &'static str {
        match self {
            Permit::Offensive => "Red",
            Permit::Supply => "Blue",
            Permit::Defensive => "Green",
            Permit::Mission => "Yellow",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Orbital,
    Eagle,
    SupportWeapon,
    Backpack,
    Vehicle,
    Sentry,
    Emplacement,
    /// Issued by the Super Destroyer on every mission.
    Ship,
    /// Issued for a mission's objectives.
    Objective,
}

pub const KINDS: [Kind; 9] = [
    Kind::Orbital,
    Kind::Eagle,
    Kind::SupportWeapon,
    Kind::Backpack,
    Kind::Vehicle,
    Kind::Sentry,
    Kind::Emplacement,
    Kind::Ship,
    Kind::Objective,
];

impl Kind {
    pub fn permit(self) -> Permit {
        match self {
            Kind::Orbital | Kind::Eagle => Permit::Offensive,
            Kind::SupportWeapon | Kind::Backpack | Kind::Vehicle => Permit::Supply,
            Kind::Sentry | Kind::Emplacement => Permit::Defensive,
            Kind::Ship | Kind::Objective => Permit::Mission,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Kind::Orbital => "Orbital strikes",
            Kind::Eagle => "Eagle strikes",
            Kind::SupportWeapon => "Support weapons",
            Kind::Backpack => "Backpacks",
            Kind::Vehicle => "Vehicles",
            Kind::Sentry => "Sentries",
            Kind::Emplacement => "Emplacements",
            Kind::Ship => "Ship",
            Kind::Objective => "Objectives",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Uses {
    Unlimited,
    Times(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unlock {
    /// Bought with Requisition Slips from a ship module once a Helldiver reaches the level; 0 slips is free.
    Requisition {
        module: &'static str,
        level: u8,
        slips: u32,
    },
    Warbond {
        warbond: &'static str,
        page: u8,
        medals: u16,
    },
    /// Handed to every Helldiver for a campaign or an event.
    Reward(&'static str),
    /// Comes with the mission, not the loadout.
    Issued,
    Unreleased,
}

#[derive(Debug, PartialEq)]
pub struct Stratagem {
    /// Names its icon and stands for it in saved settings.
    pub id: &'static str,
    pub name: &'static str,
    pub code: &'static [Arrow],
    pub kind: Kind,
    /// Seconds.
    pub cooldown: Option<f32>,
    /// Seconds from the throw to the strike or the drop.
    pub call_in: Option<f32>,
    pub uses: Option<Uses>,
    pub unlock: Unlock,
    pub traits: &'static [&'static str],
}

impl Stratagem {
    pub fn permit(&self) -> Permit {
        self.kind.permit()
    }

    /// The name without its designation ("RS-422 Railgun" is "Railgun"); Eagles and orbitals carry none.
    pub fn short_name(&self) -> &'static str {
        short_name(self.name)
    }

    /// The designation ("RS-422"), if the name carries one.
    pub fn designation(&self) -> Option<&'static str> {
        let first = self.name.split_whitespace().next()?;
        is_designation(first).then_some(first)
    }
}

fn is_designation(word: &str) -> bool {
    word.chars().any(|c| c.is_ascii_digit() || c == '/')
}

pub fn short_name(name: &str) -> &str {
    match name.split_once(' ') {
        Some((first, rest)) if is_designation(first) => rest,
        _ => name,
    }
}

pub fn by_id(id: &str) -> Option<&'static Stratagem> {
    STRATAGEMS.iter().find(|s| s.id == id)
}

/// How far a code is keyed in after one more arrow: the longest start of the code the last presses spell.
pub fn step(at: usize, a: Arrow, code: &[Arrow]) -> usize {
    let mut keyed: Vec<Arrow> = code[..at.min(code.len())].to_vec();
    keyed.push(a);
    for n in (1..=keyed.len().min(code.len())).rev() {
        if keyed[keyed.len() - n..].iter().zip(code).all(|(k, c)| k == c) {
            return n;
        }
    }
    0
}

/// The arrow a key dials: W A S D or the arrow keys, as in the game.
pub fn arrow_of(key: &str) -> Option<Arrow> {
    match key.to_lowercase().as_str() {
        "w" | "arrowup" => Some(Arrow::Up),
        "a" | "arrowleft" => Some(Arrow::Left),
        "s" | "arrowdown" => Some(Arrow::Down),
        "d" | "arrowright" => Some(Arrow::Right),
        _ => None,
    }
}
