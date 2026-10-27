//! Major Orders: each task's target and progress, rates from recorded readings, and whether it makes the deadline.

use crate::api::{Assignment, AssignmentTask, Faction, Planet};
use crate::factions::{faction_by_id, faction_label};
use crate::progress::{projected_at, projected_completion, slope_per_hour};
use crate::snapshot::{EpisodeLive, EpisodeStatus, OrderSample, phase_name};

// valueTypes observed in assignment tasks.
const VT_FACTION: i64 = 1;
const VT_GOAL: i64 = 3;
const VT_PLANET: i64 = 12;

// Task types with known meanings; anything else is shown generically.
fn task_label(kind: i64) -> String {
    match kind {
        2 => "Extract samples".into(),
        3 => "Eradicate".into(),
        7 | 9 => "Complete operations".into(),
        11 => "Liberate".into(),
        12 => "Defend".into(),
        13 => "Hold".into(),
        15 => "Liberate more planets than lost".into(),
        _ => format!("Objective (type {kind})"),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TaskView {
    /// True for a planet task (liberate, defend, hold); false for a count.
    pub is_planet: bool,
    pub label: String,
    pub planet: Option<Planet>,
    pub faction: Option<Faction>,
    /// For count tasks: current / goal. For planet tasks: 0/1 completion flag in `done`.
    pub current: f64,
    pub goal: f64,
    pub done: bool,
}

fn value_of(task: &AssignmentTask, value_type: i64) -> Option<f64> {
    let i = task.value_types.iter().position(|v| *v == value_type)?;
    task.values.get(i).copied()
}

pub fn describe_task(task: &AssignmentTask, progress: f64, planets: Option<&[Planet]>) -> TaskView {
    let planet_index = value_of(task, VT_PLANET);
    let faction_id = value_of(task, VT_FACTION);
    let goal = value_of(task, VT_GOAL).unwrap_or(1.0);
    let faction = faction_id.filter(|f| *f != 0.0 && !f.is_nan()).and_then(|f| faction_by_id(f as i64));
    let label = task_label(task.kind);
    // The API fills the planet slot with 0 when a task has no planet.
    let planet = planet_index.filter(|i| *i > 0.0).and_then(|i| planets?.iter().find(|p| p.index as f64 == i));
    if let Some(planet) = planet.filter(|_| matches!(task.kind, 11..=13)) {
        return TaskView {
            is_planet: true,
            label,
            planet: Some(planet.clone()),
            faction,
            current: progress,
            goal: 1.0,
            done: progress >= 1.0,
        };
    }
    TaskView { is_planet: false, label, planet: None, faction, current: progress, goal, done: progress >= goal }
}

pub fn describe_assignment(a: &Assignment, planets: Option<&[Planet]>) -> Vec<TaskView> {
    a.tasks
        .iter()
        .enumerate()
        .map(|(i, t)| describe_task(t, a.progress.get(i).copied().unwrap_or(0.0), planets))
        .collect()
}

/// A count task's progress as 0-100, held at 99.9 until done so it never reads 100% early.
pub fn task_percent(t: &TaskView) -> f64 {
    if t.done {
        100.0
    } else if t.goal > 0.0 {
        ((t.current / t.goal) * 100.0).clamp(0.0, 99.9)
    } else {
        0.0
    }
}

pub fn reward_label(kind: i64) -> &'static str {
    match kind {
        1 => "Warbond Medals",
        2 => "Requisition Slips",
        3 => "Super Credits",
        _ => "Reward",
    }
}

/// The running order's name, taken from its campaign: the assignment itself is titled only "MAJOR ORDER"
/// and its briefing is worded differently, so a name is given only while exactly one order is running.
pub fn order_name(episodes: Option<&[EpisodeLive]>) -> Option<String> {
    let running: Vec<_> = episodes
        .unwrap_or_default()
        .iter()
        .filter(|e| e.status == EpisodeStatus::Active)
        .flat_map(|e| e.phases.iter().filter(|p| p.status == EpisodeStatus::Active))
        .collect();
    if running.len() == 1 { phase_name(running[0]).map(String::from) } else { None }
}

/// Less history than this and a count task's rate isn't trusted.
const MIN_ORDER_SPAN_MS: f64 = 30.0 * 60_000.0;

/// Each task's count per hour, from the readings kept for order `id`; None while there's too little history.
pub fn order_task_rates(samples: Option<&[OrderSample]>, id: i64, tasks: usize) -> Vec<Option<f64>> {
    let mut mine: Vec<&OrderSample> = samples.unwrap_or_default().iter().filter(|s| s.id == id).collect();
    mine.sort_by(|a, b| a.at.total_cmp(&b.at));
    let enough = mine.len() > 1 && mine[mine.len() - 1].at - mine[0].at >= MIN_ORDER_SPAN_MS;
    (0..tasks)
        .map(|i| {
            if !enough {
                return None;
            }
            slope_per_hour(&mine.iter().map(|s| (s.at, s.progress.get(i).copied().unwrap_or(0.0))).collect::<Vec<_>>())
        })
        .collect()
}

/// "NoFront": a target planet nobody can deploy to, so it can't move.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeadlineOutlook {
    Done,
    OnTrack,
    Behind,
    Measuring,
    NoFront,
}

impl DeadlineOutlook {
    pub fn label(self) -> &'static str {
        match self {
            DeadlineOutlook::Done => "Complete",
            DeadlineOutlook::OnTrack => "Winning",
            DeadlineOutlook::Behind => "Losing",
            DeadlineOutlook::Measuring => "Measuring",
            DeadlineOutlook::NoFront => "No front",
        }
    }

    pub fn tone(self) -> &'static str {
        match self {
            DeadlineOutlook::Done | DeadlineOutlook::OnTrack => "var(--good)",
            DeadlineOutlook::Behind | DeadlineOutlook::NoFront => "var(--bad)",
            DeadlineOutlook::Measuring => "var(--label)",
        }
    }

    /// The name the stylesheet and tests use: "done", "on-track", "behind", "measuring", "no-front".
    pub fn as_str(self) -> &'static str {
        match self {
            DeadlineOutlook::Done => "done",
            DeadlineOutlook::OnTrack => "on-track",
            DeadlineOutlook::Behind => "behind",
            DeadlineOutlook::Measuring => "measuring",
            DeadlineOutlook::NoFront => "no-front",
        }
    }
}

pub fn task_name(t: &TaskView) -> String {
    match &t.planet {
        Some(p) => p.name.clone(),
        None => match t.faction {
            Some(f) => format!("{} {}", t.label, faction_label(Some(f))),
            None => t.label.clone(),
        },
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TaskOutlook {
    pub outlook: DeadlineOutlook,
    /// Percent it stands at by the deadline, 0-100.
    pub end: Option<f64>,
    /// When it reaches 100% at the current rate, if that's before the deadline.
    pub eta: Option<f64>,
    /// Trend in %/h.
    pub rate: Option<f64>,
}

/// The whole order at the current pace: it is won only if every objective makes the deadline.
pub fn order_verdict(tasks: &[TaskView], outlooks: &[TaskOutlook]) -> Option<(String, &'static str)> {
    let short: Vec<String> = tasks
        .iter()
        .zip(outlooks)
        .filter(|(_, o)| matches!(o.outlook, DeadlineOutlook::Behind | DeadlineOutlook::NoFront))
        .map(|(t, _)| task_name(t))
        .collect();
    if !short.is_empty() {
        return Some((format!("Losing: {} won't make the deadline at this pace", short.join(", ")), "var(--bad)"));
    }
    if outlooks.iter().all(|o| matches!(o.outlook, DeadlineOutlook::Done | DeadlineOutlook::OnTrack)) {
        return Some(("Winning: every objective makes the deadline at this pace".into(), "var(--good)"));
    }
    outlooks
        .iter()
        .any(|o| o.outlook == DeadlineOutlook::Measuring)
        .then(|| ("Measuring progress rates…".into(), "var(--label)"))
}

/// A task's outlook from its percent and its trend in %/h.
pub fn task_outlook(percent: f64, rate: Option<f64>, now: f64, deadline: f64) -> TaskOutlook {
    let (outlook, end) = deadline_outlook(percent, rate, now, deadline);
    let eta = if outlook == DeadlineOutlook::Done { None } else { projected_completion(percent, rate, now) };
    TaskOutlook { outlook, end, eta: eta.filter(|e| *e < deadline), rate }
}

/// Where a target ends up by the deadline if the current rate holds; `end` is clamped to 0-100.
pub fn deadline_outlook(percent: f64, rate: Option<f64>, now: f64, deadline: f64) -> (DeadlineOutlook, Option<f64>) {
    if percent >= 100.0 {
        return (DeadlineOutlook::Done, Some(100.0));
    }
    match projected_at(percent, rate, now, deadline) {
        None => (DeadlineOutlook::Measuring, None),
        Some(end) => {
            (if end >= 100.0 { DeadlineOutlook::OnTrack } else { DeadlineOutlook::Behind }, Some(end.clamp(0.0, 100.0)))
        }
    }
}
