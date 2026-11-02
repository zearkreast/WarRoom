/// Fronts with at least this many Helldivers always show, moving or not.
pub const MIN_PLAYERS: f64 = 5000.0;

/// Net %/h after enemy regen; below it a front is holding (+6%/h gross can meet -5.6%/h regen).
pub const MIN_RATE: f64 = 0.4;

/// The API's campaign types seen on ordinary fronts: a liberation and a defense.
const LIBERATION: i64 = 0;
const DEFENSE: i64 = 4;

/// Any other campaign type, such as Recon: too rare to go unseen.
pub fn is_rare_campaign(kind: i64) -> bool {
    kind != LIBERATION && kind != DEFENSE
}

pub struct FrontActivity {
    pub players: f64,
    /// Net progress in %/h, None while the collector is still measuring.
    pub rate: Option<f64>,
    pub in_major_order: bool,
    pub pinned: bool,
    /// A defense, a rare campaign, or a front a gambit or circumvallation runs through.
    pub strategic: bool,
}

/// A front too small to matter that isn't gaining ground.
pub fn is_stalled(f: &FrontActivity) -> bool {
    if f.in_major_order || f.pinned || f.strategic || f.players >= MIN_PLAYERS {
        return false;
    }
    f.rate.is_none_or(|r| r < MIN_RATE)
}
