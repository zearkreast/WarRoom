//! The stratagem drill: rounds of codes to key in against a draining fuse, scored with a streak multiplier.

use crate::stratagems::{Arrow, Permit, STRATAGEMS, Stratagem};
use serde::{Deserialize, Serialize};

/// Seconds on a full fuse.
pub const FUSE: f64 = 10.0;
/// Seconds a completed code gives back, per arrow.
const REFILL_PER_ARROW: f64 = 0.3;
const READY_SECS: f64 = 2.0;
const CLEARED_SECS: f64 = 3.0;
pub const MAX_SCORES: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Every stratagem on file.
    Classic,
    /// The stratagems the player picked.
    Custom,
    /// Every stratagem of the permit colours the player picked.
    Permits,
}

pub const MODES: [Mode; 3] = [Mode::Classic, Mode::Custom, Mode::Permits];

impl Mode {
    pub fn label(self) -> &'static str {
        match self {
            Mode::Classic => "Classic",
            Mode::Custom => "Custom loadout",
            Mode::Permits => "Permit colours",
        }
    }
}

pub fn classic_pool() -> Vec<&'static Stratagem> {
    STRATAGEMS.iter().collect()
}

/// The picked stratagems by id, in the collection's order; unknown ids are skipped.
pub fn custom_pool(ids: &[String]) -> Vec<&'static Stratagem> {
    STRATAGEMS.iter().filter(|s| ids.iter().any(|id| id == s.id)).collect()
}

pub fn permit_pool(permits: &[Permit]) -> Vec<&'static Stratagem> {
    STRATAGEMS.iter().filter(|s| permits.contains(&s.permit())).collect()
}

/// How a pool is described next to a score.
pub fn pool_label(mode: Mode, ids: &[String], permits: &[Permit]) -> String {
    match mode {
        Mode::Classic => format!("All {} stratagems", STRATAGEMS.len()),
        Mode::Custom => match custom_pool(ids).as_slice() {
            [one] => one.short_name().to_string(),
            list => format!("{} stratagems", list.len()),
        },
        Mode::Permits => permits.iter().map(|p| p.colour()).collect::<Vec<_>>().join(", "),
    }
}

pub fn stratagems_per_round(round: u32) -> usize {
    (5 + round as usize).min(15)
}

/// How fast the fuse burns in a round, in fuse seconds per second.
pub fn burn_rate(round: u32) -> f64 {
    (1.0 + 0.08 * round.saturating_sub(1) as f64).min(2.2)
}

/// The score multiplier for a streak of codes keyed without a miss.
pub fn multiplier(streak: u32) -> u32 {
    1 + (streak / 5).min(3)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoundBonus {
    pub round: u32,
    pub clear: u32,
    pub time: u32,
    pub flawless: u32,
}

impl RoundBonus {
    pub fn total(&self) -> u32 {
        self.clear + self.time + self.flawless
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    /// The countdown before a round; seconds left.
    Ready(f64),
    Play,
    /// The round's bonuses on show; seconds left.
    Cleared(f64),
    Over,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Press {
    Ignored,
    Hit,
    Miss,
    /// A code completed, with the points it scored.
    Called(u32),
    /// The round's last code completed, with its points and the round's bonuses.
    Cleared(u32, RoundBonus),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tick {
    None,
    Go,
    Over,
}

/// A small xorshift generator; a drill needs variety, not quality.
#[derive(Clone, Debug)]
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed | 1)
    }

    fn below(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n as u64) as usize
    }
}

#[derive(Clone, Debug)]
pub struct Drill {
    pool: Vec<&'static Stratagem>,
    rng: Rng,
    pub phase: Phase,
    pub round: u32,
    /// What is left of the round; the first is the one being keyed.
    pub queue: Vec<&'static Stratagem>,
    pub keyed: usize,
    pub score: u32,
    /// Fuse seconds left.
    pub fuse: f64,
    pub streak: u32,
    pub best_streak: u32,
    pub called: u32,
    pub hits: u32,
    pub misses: u32,
    round_misses: u32,
    pub last_bonus: Option<RoundBonus>,
}

impl Drill {
    /// Starts on the first round's countdown. The pool must not be empty.
    pub fn new(pool: Vec<&'static Stratagem>, seed: u64) -> Self {
        assert!(!pool.is_empty(), "a drill needs at least one stratagem");
        let mut d = Drill {
            pool,
            rng: Rng::new(seed),
            phase: Phase::Ready(READY_SECS),
            round: 1,
            queue: Vec::new(),
            keyed: 0,
            score: 0,
            fuse: FUSE,
            streak: 0,
            best_streak: 0,
            called: 0,
            hits: 0,
            misses: 0,
            round_misses: 0,
            last_bonus: None,
        };
        d.deal();
        d
    }

    /// A fresh round's stratagems, never the same one twice running while the pool has another.
    fn deal(&mut self) {
        let n = self.pool.len();
        let mut last: Option<usize> = None;
        self.queue = (0..stratagems_per_round(self.round))
            .map(|_| {
                let i = match last {
                    Some(l) if n > 1 => {
                        let i = self.rng.below(n - 1);
                        if i >= l { i + 1 } else { i }
                    }
                    _ => self.rng.below(n),
                };
                last = Some(i);
                self.pool[i]
            })
            .collect();
        self.keyed = 0;
        self.fuse = FUSE;
        self.round_misses = 0;
    }

    pub fn current(&self) -> Option<&'static Stratagem> {
        self.queue.first().copied()
    }

    /// Of every arrow pressed, the share that was right; 1 before any press.
    pub fn accuracy(&self) -> f64 {
        let all = self.hits + self.misses;
        if all == 0 { 1.0 } else { self.hits as f64 / all as f64 }
    }

    pub fn tick(&mut self, dt: f64) -> Tick {
        match self.phase {
            Phase::Ready(left) if left - dt <= 0.0 => {
                self.phase = Phase::Play;
                Tick::Go
            }
            Phase::Ready(left) => {
                self.phase = Phase::Ready(left - dt);
                Tick::None
            }
            Phase::Play => {
                self.fuse -= dt * burn_rate(self.round);
                if self.fuse <= 0.0 {
                    self.fuse = 0.0;
                    self.phase = Phase::Over;
                    Tick::Over
                } else {
                    Tick::None
                }
            }
            Phase::Cleared(left) if left - dt <= 0.0 => {
                self.round += 1;
                self.deal();
                self.phase = Phase::Ready(READY_SECS);
                Tick::None
            }
            Phase::Cleared(left) => {
                self.phase = Phase::Cleared(left - dt);
                Tick::None
            }
            Phase::Over => Tick::None,
        }
    }

    pub fn press(&mut self, a: Arrow) -> Press {
        if self.phase != Phase::Play {
            return Press::Ignored;
        }
        let Some(s) = self.current() else { return Press::Ignored };
        if s.code[self.keyed] != a {
            self.keyed = 0;
            self.misses += 1;
            self.round_misses += 1;
            self.streak = 0;
            return Press::Miss;
        }
        self.hits += 1;
        self.keyed += 1;
        if self.keyed < s.code.len() {
            return Press::Hit;
        }
        self.streak += 1;
        self.best_streak = self.best_streak.max(self.streak);
        self.called += 1;
        let points = 10 * s.code.len() as u32 * multiplier(self.streak);
        self.score += points;
        self.fuse = (self.fuse + REFILL_PER_ARROW * s.code.len() as f64).min(FUSE);
        self.queue.remove(0);
        self.keyed = 0;
        if !self.queue.is_empty() {
            return Press::Called(points);
        }
        let bonus = RoundBonus {
            round: self.round,
            clear: 50 + 25 * self.round,
            time: (self.fuse / FUSE * 100.0).round() as u32,
            flawless: if self.round_misses == 0 { 100 } else { 0 },
        };
        self.score += bonus.total();
        self.last_bonus = Some(bonus);
        self.phase = Phase::Cleared(CLEARED_SECS);
        Press::Cleared(points, bonus)
    }
}

/// A finished drill, as kept on this device.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DrillScore {
    pub mode: Mode,
    pub score: u32,
    /// The round the drill ended in.
    pub round: u32,
    pub called: u32,
    /// Milliseconds since the epoch.
    pub at: f64,
    pub pool: String,
}

/// Files a score among its mode's best, keeping the top MAX_SCORES of each; its place from 1, if it made it.
pub fn record(scores: &mut Vec<DrillScore>, s: DrillScore) -> Option<usize> {
    if s.score == 0 {
        return None;
    }
    let mode = s.mode;
    let at = s.at;
    scores.push(s);
    scores.sort_by(|a, b| b.score.cmp(&a.score).then(a.at.total_cmp(&b.at)));
    let mut kept = 0;
    scores.retain(|x| {
        if x.mode != mode {
            return true;
        }
        kept += 1;
        kept <= MAX_SCORES
    });
    scores.iter().filter(|x| x.mode == mode).position(|x| x.at == at).map(|i| i + 1)
}

pub fn best(scores: &[DrillScore], mode: Mode) -> Option<&DrillScore> {
    scores.iter().filter(|s| s.mode == mode).max_by(|a, b| a.score.cmp(&b.score).then(b.at.total_cmp(&a.at)))
}
