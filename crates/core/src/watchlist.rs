//! Moments worth an alert on pinned planets and the Major Order, found by comparing one look with the next.

const HOUR: f64 = 3_600_000.0;

/// A pinned front at one moment.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FrontLook {
    /// Epoch ms.
    pub at: f64,
    pub rate: Option<f64>,
    pub victory_at: Option<f64>,
    /// Cities waiting on the planet's liberation to open, by id.
    pub waiting: Vec<i64>,
    /// Cities open to Helldivers and not yet freed.
    pub open: Vec<(i64, String)>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FrontAlert {
    VictorySoon(f64),
    LosingGround(f64),
    CityOpens(String),
}

pub fn front_alerts(prev: &FrontLook, next: &FrontLook) -> Vec<FrontAlert> {
    let mut out = Vec::new();
    let soon = |look: &FrontLook| look.victory_at.is_some_and(|v| v > look.at && v - look.at <= HOUR);
    if soon(next) && !soon(prev) {
        out.extend(next.victory_at.map(FrontAlert::VictorySoon));
    }
    if let (Some(was), Some(is)) = (prev.rate, next.rate)
        && was > 0.0
        && is <= 0.0
    {
        out.push(FrontAlert::LosingGround(is));
    }
    for (id, name) in &next.open {
        if prev.waiting.contains(id) {
            out.push(FrontAlert::CityOpens(name.clone()));
        }
    }
    out
}

/// A running Major Order at one moment.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OrderLook {
    /// Epoch ms.
    pub at: f64,
    pub ends: f64,
    /// Some objective won't make the deadline at today's pace.
    pub losing: bool,
    pub done: Vec<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OrderAlert {
    Losing,
    /// Hours left: 6 or 1.
    EndsIn(u8),
    ObjectiveDone(usize),
}

pub fn order_alerts(prev: &OrderLook, next: &OrderLook) -> Vec<OrderAlert> {
    let mut out = Vec::new();
    if next.losing && !prev.losing {
        out.push(OrderAlert::Losing);
    }
    let left = |look: &OrderLook| look.ends - look.at;
    // Only the nearest mark when one look passes both, as after the app slept.
    if left(next) > 0.0
        && let Some(h) = [1u8, 6].into_iter().find(|h| left(prev) > *h as f64 * HOUR && left(next) <= *h as f64 * HOUR)
    {
        out.push(OrderAlert::EndsIn(h));
    }
    for (i, done) in next.done.iter().enumerate() {
        if *done && prev.done.get(i) == Some(&false) {
            out.push(OrderAlert::ObjectiveDone(i));
        }
    }
    out
}
