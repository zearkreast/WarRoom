//! Shared chart styling: hairline solid grid and axes one step off the surface, text in label ink (never
//! series colours), a crosshair tooltip on every chart.

pub const HOUR: f64 = 3_600_000.0;

pub fn fmt_hour_tick(t: f64) -> String {
    crate::js::fmt_time(t)
}

/// Whole-hour ticks every `step_hours` across [from, to].
pub fn hour_ticks(from: f64, to: f64, step_hours: f64) -> Vec<f64> {
    let step = step_hours * HOUR;
    let mut out = Vec::new();
    let mut t = (from / step).ceil() * step;
    while t <= to {
        out.push(t);
        t += step;
    }
    out
}
