//! A planet's war record: owner segments, numbered battles and defense events.

use crate::api::Faction;
use crate::owner_changes::OwnerChange;

/// A stretch of time with one owner.
#[derive(Clone, Debug, PartialEq)]
pub struct OwnerSegment {
    pub owner: Faction,
    pub from: f64,
    pub to: f64,
}

/// A defense event, rebuilt from samples taken while the planet was under attack.
#[derive(Clone, Debug, PartialEq)]
pub struct DefenseEvent {
    pub start: f64,
    pub end: f64,
    /// True while the event is still running.
    pub ongoing: bool,
    pub held: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LogEntry {
    Start { at: f64, owner: Faction },
    Change { at: f64, from: Faction, to: Faction, battle: usize, held_days: f64 },
    Defense { at: f64, enemy: Option<Faction>, battle: usize, ongoing: bool, held: bool },
}

impl LogEntry {
    pub fn at(&self) -> f64 {
        match self {
            LogEntry::Start { at, .. } | LogEntry::Change { at, .. } | LogEntry::Defense { at, .. } => *at,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WarHistory {
    pub record_start: f64,
    pub liberated: usize,
    pub lost: usize,
    pub defenses_held: usize,
    pub defenses_total: usize,
    /// Share of the recorded time Super Earth held the planet, 0-100.
    pub held_free_pct: f64,
    pub segments: Vec<OwnerSegment>,
    pub defenses: Vec<DefenseEvent>,
    /// Newest first.
    pub log: Vec<LogEntry>,
}

/// Two defending samples further apart than this belong to different events.
const EVENT_GAP_MS: f64 = 2.0 * 3_600_000.0;
const DAY_MS: f64 = 86_400_000.0;

/// Groups the timestamps of "defending" samples into events.
pub fn defense_events(
    defending_at: &[f64],
    changes: &[OwnerChange],
    active_event_end: Option<f64>,
    now: f64,
) -> Vec<DefenseEvent> {
    let mut sorted = defending_at.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mut events: Vec<DefenseEvent> = Vec::new();
    for t in sorted {
        match events.last_mut() {
            Some(last) if t - last.end <= EVENT_GAP_MS => last.end = t,
            _ => events.push(DefenseEvent { start: t, end: t, ongoing: false, held: true }),
        }
    }
    for e in &mut events {
        // Lost if Super Earth gave the planet up during (or right after) the event.
        e.held = !changes.iter().any(|c| c.from == Faction::Humans && c.at >= e.start && c.at <= e.end + EVENT_GAP_MS);
    }
    if let Some(last) = events.last_mut()
        && active_event_end.is_some()
        && now - last.end <= EVENT_GAP_MS
    {
        last.ongoing = true;
    }
    events
}

pub struct HistoryInput<'a> {
    pub changes: &'a [OwnerChange],
    pub defenses: &'a [DefenseEvent],
    pub current_owner: Faction,
    pub record_start: f64,
    pub now: f64,
}

/// Builds a planet's war record from its ownership changes (any order) and defense events.
/// `record_start` is when tracking began for this planet; earlier changes are ignored.
pub fn build_war_history(opts: HistoryInput) -> WarHistory {
    let mut changes: Vec<&OwnerChange> = opts.changes.iter().filter(|c| c.at >= opts.record_start).collect();
    changes.sort_by(|a, b| a.at.total_cmp(&b.at));
    let start_owner = changes.first().map_or(opts.current_owner, |c| c.from);

    let mut segments = Vec::new();
    let mut owner = start_owner;
    let mut from = opts.record_start;
    for c in &changes {
        if c.at > from {
            segments.push(OwnerSegment { owner, from, to: c.at });
        }
        owner = c.to;
        from = c.at;
    }
    if opts.now > from {
        segments.push(OwnerSegment { owner, from, to: opts.now });
    }

    let total = (opts.now - opts.record_start).max(1.0);
    let free = crate::format::js_sum(segments.iter().filter(|s| s.owner == Faction::Humans).map(|s| s.to - s.from));

    // Battles are numbered in time order across captures and defenses.
    let mut battles: Vec<LogEntry> = Vec::new();
    let mut prev_at = opts.record_start;
    for c in &changes {
        battles.push(LogEntry::Change {
            at: c.at,
            from: c.from,
            to: c.to,
            battle: 0,
            held_days: (c.at - prev_at) / DAY_MS,
        });
        prev_at = c.at;
    }
    for d in opts.defenses {
        if d.start < opts.record_start {
            continue;
        }
        let enemy = changes.iter().find(|c| c.from == Faction::Humans && c.at >= d.start).map(|c| c.to);
        battles.push(LogEntry::Defense { at: d.start, enemy, battle: 0, ongoing: d.ongoing, held: d.held });
    }
    battles.sort_by(|a, b| a.at().total_cmp(&b.at()));
    for (i, b) in battles.iter_mut().enumerate() {
        match b {
            LogEntry::Change { battle, .. } | LogEntry::Defense { battle, .. } => *battle = i + 1,
            LogEntry::Start { .. } => {}
        }
    }
    let mut log: Vec<LogEntry> = battles.into_iter().rev().collect();
    log.push(LogEntry::Start { at: opts.record_start, owner: start_owner });

    let finished: Vec<&DefenseEvent> =
        opts.defenses.iter().filter(|d| !d.ongoing && d.start >= opts.record_start).collect();
    WarHistory {
        record_start: opts.record_start,
        liberated: changes.iter().filter(|c| c.to == Faction::Humans).count(),
        lost: changes.iter().filter(|c| c.from == Faction::Humans).count(),
        defenses_held: finished.iter().filter(|d| d.held).count(),
        defenses_total: finished.len(),
        held_free_pct: (free / total) * 100.0,
        segments,
        defenses: opts.defenses.to_vec(),
        log,
    }
}

/// "1C 11Y 5M 3W" style duration for long spans such as time deployed.
pub fn fmt_long_duration(seconds: f64) -> String {
    const UNITS: [(&str, f64); 6] = [
        ("C", 100.0 * 365.0 * 86400.0),
        ("Y", 365.0 * 86400.0),
        ("M", 30.0 * 86400.0),
        ("W", 7.0 * 86400.0),
        ("D", 86400.0),
        ("H", 3600.0),
    ];
    let mut rest = seconds.floor().max(0.0);
    let mut parts: Vec<String> = Vec::new();
    for (u, s) in UNITS {
        let n = (rest / s).floor();
        if n != 0.0 || !parts.is_empty() {
            if n != 0.0 {
                parts.push(format!("{n}{u}"));
            }
            rest -= n * s;
        }
        if parts.len() == 4 {
            break;
        }
    }
    if parts.is_empty() { "0H".into() } else { parts.join(" ") }
}
