//! Chart geometry matching Recharts 3: its "nice" axis ticks and d3-shape's linear and monotone-x paths.

use crate::format::{js_number, js_round};

/// Rounds away binary noise the way decimal.js would never produce it (0.1 + 0.2 → 0.3).
fn clean(x: f64) -> f64 {
    if x == 0.0 || !x.is_finite() {
        return x;
    }
    let digits = 12 - (x.abs().log10().floor() as i32 + 1);
    let k = 10f64.powi(digits.clamp(-300, 300));
    (x * k).round() / k
}

fn digit_count(value: f64) -> i32 {
    if value == 0.0 { 1 } else { value.abs().log10().floor() as i32 + 1 }
}

fn adaptive_step(rough: f64, allow_decimals: bool, correction: i32) -> f64 {
    if rough <= 0.0 {
        return 0.0;
    }
    let digits = digit_count(rough);
    let unit = 10f64.powi(digits);
    let ratio = rough / unit;
    let scale = if digits != 1 { 0.05 } else { 0.1 };
    let amended = clean(((ratio / scale).ceil() + correction as f64) * scale);
    let step = clean(amended * unit);
    if allow_decimals { step } else { step.ceil() }
}

fn calculate_step(min: f64, max: f64, count: usize, allow_decimals: bool, correction: i32) -> (f64, f64, f64) {
    if !((max - min) / (count as f64 - 1.0)).is_finite() {
        return (0.0, 0.0, 0.0);
    }
    let step = adaptive_step((max - min) / (count as f64 - 1.0), allow_decimals, correction);
    let middle = if min <= 0.0 && max >= 0.0 {
        0.0
    } else {
        let m = clean((min + max) / 2.0);
        clean(m - m % step)
    };
    let mut below = ((middle - min) / step).ceil() as i64;
    let mut up = ((max - middle) / step).ceil() as i64;
    let scale_count = below + up + 1;
    if scale_count > count as i64 {
        return calculate_step(min, max, count, allow_decimals, correction + 1);
    }
    if scale_count < count as i64 {
        if max > 0.0 {
            up += count as i64 - scale_count;
        } else {
            below += count as i64 - scale_count;
        }
    }
    (step, clean(middle - below as f64 * step), clean(middle + up as f64 * step))
}

fn single_value_ticks(value: f64, count: usize, allow_decimals: bool) -> Vec<f64> {
    let mut step = 1.0;
    let mut middle = value;
    if value.fract() != 0.0 && allow_decimals {
        if value.abs() < 1.0 {
            step = 10f64.powi(digit_count(value) - 1);
            middle = clean((value / step).floor() * step);
        } else if value.abs() > 1.0 {
            middle = value.floor();
        }
    } else if value == 0.0 {
        middle = ((count as f64 - 1.0) / 2.0).floor();
    } else if !allow_decimals {
        middle = value.floor();
    }
    let mid_index = (count as i64 - 1) / 2;
    (0..count as i64).map(|i| clean(middle + (i - mid_index) as f64 * step)).collect()
}

/// Recharts' getNiceTickValues: `count` round ticks covering [min, max], possibly reaching past it.
pub fn nice_ticks(min: f64, max: f64, count: usize, allow_decimals: bool) -> Vec<f64> {
    let count = count.max(2);
    let (lo, hi) = if min > max { (max, min) } else { (min, max) };
    if lo == hi {
        return single_value_ticks(lo, count, allow_decimals);
    }
    let (step, tick_min, tick_max) = calculate_step(lo, hi, count, allow_decimals, 0);
    let end = tick_max + 0.1 * step;
    let mut out = Vec::new();
    let mut v = tick_min;
    while v < end && out.len() < 100_000 {
        out.push(v);
        v = clean(v + step);
        if step == 0.0 {
            break;
        }
    }
    if min > max {
        out.reverse();
    }
    out
}

/// d3-path's default output: coordinates rounded to 3 decimals.
fn n(x: f64) -> String {
    js_number(js_round(x * 1000.0) / 1000.0)
}

/// d3-shape's linear curve through one run of points.
fn linear(out: &mut String, pts: &[(f64, f64)], join: bool) {
    for (i, (x, y)) in pts.iter().enumerate() {
        out.push(if i == 0 && !join { 'M' } else { 'L' });
        out.push_str(&n(*x));
        out.push(',');
        out.push_str(&n(*y));
    }
}

fn sign(x: f64) -> f64 {
    if x < 0.0 { -1.0 } else { 1.0 }
}

/// d3-shape's curveMonotoneX through one run of points.
fn monotone(out: &mut String, pts: &[(f64, f64)], join: bool) {
    let (mut x0, mut y0, mut x1, mut y1, mut t0) = (f64::NAN, f64::NAN, f64::NAN, f64::NAN, f64::NAN);
    let mut state = 0;
    let bezier = |out: &mut String, x0: f64, y0: f64, x1: f64, y1: f64, t0: f64, t1: f64| {
        let dx = (x1 - x0) / 3.0;
        out.push_str(&format!(
            "C{},{},{},{},{},{}",
            n(x0 + dx),
            n(y0 + dx * t0),
            n(x1 - dx),
            n(y1 - dx * t1),
            n(x1),
            n(y1)
        ));
    };
    let slope3 = |x0: f64, y0: f64, x1: f64, y1: f64, x2: f64, y2: f64| {
        let (h0, h1) = (x1 - x0, x2 - x1);
        let d0 = if h0 != 0.0 {
            h0
        } else if h1 < 0.0 {
            -0.0
        } else {
            0.0
        };
        let d1 = if h1 != 0.0 {
            h1
        } else if h0 < 0.0 {
            -0.0
        } else {
            0.0
        };
        let (s0, s1) = ((y1 - y0) / d0, (y2 - y1) / d1);
        let p = (s0 * h1 + s1 * h0) / (h0 + h1);
        let v = (sign(s0) + sign(s1)) * s0.abs().min(s1.abs()).min(0.5 * p.abs());
        if v.is_nan() { 0.0 } else { v }
    };
    let slope2 = |x0: f64, y0: f64, x1: f64, y1: f64, t: f64| {
        let h = x1 - x0;
        if h != 0.0 { (3.0 * (y1 - y0) / h - t) / 2.0 } else { t }
    };
    for &(x, y) in pts {
        let mut t1 = f64::NAN;
        if x == x1 && y == y1 {
            continue;
        }
        match state {
            0 => {
                state = 1;
                out.push(if join { 'L' } else { 'M' });
                out.push_str(&format!("{},{}", n(x), n(y)));
            }
            1 => state = 2,
            2 => {
                state = 3;
                t1 = slope3(x0, y0, x1, y1, x, y);
                bezier(out, x0, y0, x1, y1, slope2(x0, y0, x1, y1, t1), t1);
            }
            _ => {
                t1 = slope3(x0, y0, x1, y1, x, y);
                bezier(out, x0, y0, x1, y1, t0, t1);
            }
        }
        x0 = x1;
        x1 = x;
        y0 = y1;
        y1 = y;
        t0 = t1;
    }
    match state {
        2 => out.push_str(&format!("L{},{}", n(x1), n(y1))),
        3 => bezier(out, x0, y0, x1, y1, t0, slope2(x0, y0, x1, y1, t0)),
        _ => {}
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Curve {
    Linear,
    Monotone,
}

fn runs(pts: &[Option<(f64, f64)>]) -> Vec<Vec<(f64, f64)>> {
    let mut out: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut cur = Vec::new();
    for p in pts {
        match p {
            Some(p) => cur.push(*p),
            None if !cur.is_empty() => out.push(std::mem::take(&mut cur)),
            None => {}
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// An SVG path through the points, broken at the gaps (`None`), like d3's line with `defined`.
pub fn line_path(pts: &[Option<(f64, f64)>], curve: Curve) -> String {
    let mut out = String::new();
    for run in runs(pts) {
        match curve {
            Curve::Linear => linear(&mut out, &run, false),
            Curve::Monotone => monotone(&mut out, &run, false),
        }
        if run.len() == 1 {
            out.push('Z');
        }
    }
    out
}

/// A filled SVG path between the points and their base, like d3's area: along the top, back along the base.
pub fn area_path(pts: &[Option<(f64, f64, f64)>], curve: Curve) -> String {
    let mut out = String::new();
    let tops: Vec<Option<(f64, f64)>> = pts.iter().map(|p| p.map(|(x, y, _)| (x, y))).collect();
    let bases: Vec<Option<(f64, f64)>> = pts.iter().map(|p| p.map(|(x, _, b)| (x, b))).collect();
    for (top, base) in runs(&tops).into_iter().zip(runs(&bases)) {
        let back: Vec<_> = base.into_iter().rev().collect();
        match curve {
            Curve::Linear => {
                linear(&mut out, &top, false);
                linear(&mut out, &back, true);
            }
            Curve::Monotone => {
                monotone(&mut out, &top, false);
                monotone(&mut out, &back, true);
            }
        }
        out.push('Z');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nice_ticks_match_recharts() {
        let cases: [(f64, f64, &[f64]); 8] = [
            (0.0, 37971.0, &[0.0, 9500.0, 19000.0, 28500.0, 38000.0]),
            (0.0, 100.0, &[0.0, 25.0, 50.0, 75.0, 100.0]),
            (0.0, 0.3, &[0.0, 0.075, 0.15, 0.225, 0.3]),
            (0.0, 0.0, &[0.0, 1.0, 2.0, 3.0, 4.0]),
            (0.0, 7.0, &[0.0, 2.0, 4.0, 6.0, 8.0]),
            (120000.0, 163000.0, &[120000.0, 135000.0, 150000.0, 165000.0, 180000.0]),
            (0.0123, 0.0456, &[0.0095, 0.019, 0.0285, 0.038, 0.0475]),
            (0.0, 1234567.0, &[0.0, 350000.0, 700000.0, 1050000.0, 1400000.0]),
        ];
        for (min, max, want) in cases {
            assert_eq!(nice_ticks(min, max, 5, true), want, "{min}..{max}");
        }
    }

    fn pts(p: &[(f64, f64)]) -> Vec<Option<(f64, f64)>> {
        p.iter().copied().map(Some).collect()
    }

    #[test]
    fn paths_match_d3() {
        assert_eq!(
            line_path(&pts(&[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)]), Curve::Monotone),
            "M0,0C0.333,0.5,0.667,1,1,1C1.333,1,1.667,0.5,2,0"
        );
        assert_eq!(
            line_path(&pts(&[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0), (3.0, 5.0), (4.0, 5.0)]), Curve::Monotone),
            "M0,0C0.333,0.5,0.667,1,1,1C1.333,1,1.667,0,2,0C2.333,0,2.667,5,3,5C3.333,5,3.667,5,4,5"
        );
        assert_eq!(line_path(&pts(&[(0.0, 0.0), (1.0, 1.0)]), Curve::Monotone), "M0,0L1,1");
        assert_eq!(line_path(&pts(&[(0.0, 3.0)]), Curve::Monotone), "M0,3Z");
        assert_eq!(
            line_path(&[Some((0.0, 1.0)), Some((1.0, 2.0)), None, Some((3.0, 4.0))], Curve::Linear),
            "M0,1L1,2M3,4Z"
        );
        assert_eq!(
            area_path(
                &[Some((0.0, 0.0, 10.0)), Some((1.0, 1.0, 10.0)), Some((2.0, 0.0, 10.0)), Some((3.0, 4.0, 10.0))],
                Curve::Monotone
            ),
            "M0,0C0.333,0.5,0.667,1,1,1C1.333,1,1.667,0,2,0C2.333,0,2.667,2,3,4L3,10C2.667,10,2.333,10,2,10C1.667,10,1.333,10,1,10C0.667,10,0.333,10,0,10Z"
        );
        assert_eq!(
            area_path(&[Some((0.0, 0.0, 10.0)), Some((1.0, 1.0, 10.0)), Some((2.0, 0.0, 10.0))], Curve::Linear),
            "M0,0L1,1L2,0L2,10L1,10L0,10Z"
        );
        assert_eq!(
            area_path(&[Some((0.0, 1.0, 5.0)), Some((1.0, 2.0, 5.0)), None, Some((3.0, 4.0, 5.0))], Curve::Linear),
            "M0,1L1,2L1,5L0,5ZM3,4L3,5Z"
        );
    }
}
