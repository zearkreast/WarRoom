//! Number and text formatting that matches what the app showed when it ran on JavaScript.

use crate::time::{iso_string, parse_iso, utc_parts};
use regex_lite::Regex;
use std::sync::LazyLock;

/// `Number.prototype.toFixed`: rounds the exact binary value, ties away from zero.
pub fn to_fixed(x: f64, digits: usize) -> String {
    if !x.is_finite() {
        return if x.is_nan() {
            "NaN".into()
        } else if x > 0.0 {
            "Infinity".into()
        } else {
            "-Infinity".into()
        };
    }
    if x.abs() >= 1e21 {
        return js_number(x);
    }
    // Rust rounds ties to even; JavaScript rounds them up, so decide ties from the exact expansion.
    let exact = format!("{:.*}", digits + 40, x.abs());
    let (int_part, frac) = exact.split_once('.').unwrap_or((&exact, ""));
    let keep = &frac[..digits];
    let rest = &frac[digits..];
    let mut num: Vec<u8> = format!("{int_part}{keep}").into_bytes();
    if rest.as_bytes()[0] >= b'5' {
        let mut i = num.len();
        loop {
            if i == 0 {
                num.insert(0, b'1');
                break;
            }
            i -= 1;
            if num[i] == b'9' {
                num[i] = b'0';
            } else {
                num[i] += 1;
                break;
            }
        }
    }
    let s = String::from_utf8(num).unwrap();
    let (i, f) = s.split_at(s.len() - digits);
    let body = if digits == 0 { i.to_string() } else { format!("{i}.{f}") };
    if x < 0.0 { format!("-{body}") } else { body }
}

/// `String(n)` for a JavaScript number.
pub fn js_number(x: f64) -> String {
    if x.is_nan() {
        return "NaN".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "Infinity" } else { "-Infinity" }.into();
    }
    if x == 0.0 {
        return "0".into();
    }
    let abs = x.abs();
    if (1e-7..1e21).contains(&abs) {
        let s = format!("{x}");
        return s;
    }
    // Exponent form, as JavaScript writes very large and very small numbers: "1e+21", "1.5e-7".
    let s = format!("{x:e}");
    let (m, e) = s.split_once('e').unwrap();
    let e: i32 = e.parse().unwrap();
    format!("{m}e{}{}", if e < 0 { "-" } else { "+" }, e.abs())
}

/// Rounds a number's shortest decimal form half away from zero, as `Intl.NumberFormat` does.
fn round_decimal(x: f64, digits: usize) -> f64 {
    let s = format!("{}", x.abs());
    let (int_part, frac) = s.split_once('.').unwrap_or((&s, ""));
    if frac.len() <= digits {
        return x;
    }
    let mut num: Vec<u8> = format!("{int_part}{}", &frac[..digits]).into_bytes();
    if frac.as_bytes()[digits] >= b'5' {
        let mut i = num.len();
        loop {
            if i == 0 {
                num.insert(0, b'1');
                break;
            }
            i -= 1;
            if num[i] == b'9' {
                num[i] = b'0';
            } else {
                num[i] += 1;
                break;
            }
        }
    }
    let s = String::from_utf8(num).unwrap();
    let (i, f) = s.split_at(s.len() - digits);
    let v: f64 = format!("{i}.{f}0").parse().unwrap();
    if x < 0.0 { -v } else { v }
}

fn group_thousands(int_digits: &str) -> String {
    let mut out = String::new();
    for (i, c) in int_digits.chars().enumerate() {
        if i > 0 && (int_digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// A number as `Intl.NumberFormat("en-US", { maximumFractionDigits })` writes it.
pub fn fmt_number(n: f64, max_fraction: usize) -> String {
    number(n, max_fraction, 4)
}

/// `min_grouped`: fewest integer digits that get thousands separators (compact notation starts at 5).
fn number(n: f64, max_fraction: usize, min_grouped: usize) -> String {
    if !n.is_finite() {
        return if n.is_nan() {
            "NaN".into()
        } else if n > 0.0 {
            "∞".into()
        } else {
            "-∞".into()
        };
    }
    let r = round_decimal(n, max_fraction);
    let negative = r < 0.0 || (r == 0.0 && r.is_sign_negative());
    let s = format!("{:.*}", max_fraction, r.abs());
    let (i, f) = s.split_once('.').unwrap_or((&s, ""));
    let f = f.trim_end_matches('0');
    let i = if i.len() >= min_grouped { group_thousands(i) } else { i.to_string() };
    let body = if f.is_empty() { i } else { format!("{i}.{f}") };
    if negative { format!("-{body}") } else { body }
}

/// `Math.round`: halves round up, towards +∞.
pub fn js_round(n: f64) -> f64 {
    let r = (n + 0.5).floor();
    if r == 0.0 && n < 0.0 { -0.0 } else { r }
}

pub fn fmt_int(n: f64) -> String {
    fmt_number(js_round(n), 0)
}

/// `Intl.NumberFormat("en-US", { notation: "compact", maximumFractionDigits: 1 })`, with short ("1.2K") or
/// long ("1.2 thousand") unit names.
fn compact(n: f64, long: bool) -> String {
    if !n.is_finite() || n == 0.0 {
        return fmt_number(n, 1);
    }
    const SHORT: [&str; 5] = ["", "K", "M", "B", "T"];
    const LONG: [&str; 5] = ["", " thousand", " million", " billion", " trillion"];
    let mut unit = ((n.abs().log10().floor() as i32) / 3).clamp(0, 4) as usize;
    loop {
        let scaled = round_decimal(n / 1000f64.powi(unit as i32), 1);
        if scaled.abs() >= 1000.0 && unit < 4 {
            unit += 1;
            continue;
        }
        let names = if long { LONG } else { SHORT };
        return format!("{}{}", number(scaled, 1, 5), names[unit]);
    }
}

pub fn fmt_compact(n: f64) -> String {
    compact(n, false)
}

/// "1.2 million", as the Major Order bars read numbers out.
pub fn fmt_compact_long(n: f64) -> String {
    compact(n, true)
}

pub fn fmt_pct(n: f64, digits: usize) -> String {
    format!("{}%", to_fixed(n, digits))
}

pub fn fmt_rate(rate: Option<f64>) -> String {
    match rate {
        None => "—".into(),
        Some(r) => format!("{}{}%/h", if r > 0.0 { "+" } else { "" }, to_fixed(r, 2)),
    }
}

/// "2d 4h", "47h 19m", "12m". Negative or zero durations render as "now".
pub fn fmt_duration(ms: f64) -> String {
    if ms <= 0.0 || ms.is_nan() {
        return "now".into();
    }
    let m = (ms / 60_000.0).floor() as i64;
    let h = m / 60;
    let d = h / 24;
    if d >= 3 {
        format!("{d}d {}h", h % 24)
    } else if h >= 1 {
        format!("{h}h {}m", m % 60)
    } else {
        format!("{m}m")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MarkupSpan {
    pub text: String,
    /// Highlight style id from the game's `<i=N>` tags, or None for plain text.
    pub style: Option<i64>,
}

static MARKUP: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)<i=(\d+)>(.*?)</i>").unwrap());

/// Splits the game's dispatch markup (`<i=3>TITLE</i>`) into styled spans.
pub fn parse_markup(message: &str) -> Vec<MarkupSpan> {
    let mut spans = Vec::new();
    let mut last = 0;
    for m in MARKUP.captures_iter(message) {
        let whole = m.get(0).unwrap();
        if whole.start() > last {
            spans.push(MarkupSpan { text: message[last..whole.start()].to_string(), style: None });
        }
        spans.push(MarkupSpan { text: m[2].to_string(), style: Some(m[1].parse().unwrap_or(i64::MAX)) });
        last = whole.end();
    }
    if last < message.len() {
        spans.push(MarkupSpan { text: message[last..].to_string(), style: None });
    }
    spans
}

/// `String.prototype.trim`.
pub fn js_trim(s: &str) -> &str {
    s.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}')
}

/// First highlighted span (usually the headline), else the first line.
pub fn dispatch_title(message: &str) -> String {
    let spans = parse_markup(message);
    match spans.iter().find(|s| s.style.is_some()) {
        Some(head) => js_trim(&head.text).to_string(),
        None => js_trim(message.split('\n').next().unwrap_or("")).to_string(),
    }
}

/// Super Earth Standard Time: UTC, with the in-game calendar running 160 years ahead.
pub const SEST_YEAR_OFFSET: i64 = 160;

pub fn sest_clock(ms: f64) -> String {
    iso_string(ms)[11..19].to_string()
}

pub fn sest_date(ms: f64) -> String {
    let u = utc_parts(ms);
    format!("{:02}/{:02}/{}", u.day, u.month, u.year + SEST_YEAR_OFFSET)
}

/// Day of the Galactic War, counting the start day as day 1.
pub fn war_day(started_iso: &str, now: f64) -> i64 {
    match parse_iso(started_iso) {
        Some(start) => ((now - start) / 86_400_000.0).floor() as i64 + 1,
        None => 0,
    }
}

fn plural(n: i64, unit: &str) -> String {
    format!("{n} {unit}{} ago", if n == 1 { "" } else { "s" })
}

/// "just now", "12 minutes ago", "3 hours ago", "2 days ago".
pub fn fmt_ago(at: f64, now: f64) -> String {
    let m = ((now - at) / 60_000.0).floor() as i64;
    if m < 1 {
        return "just now".into();
    }
    if m < 60 {
        return plural(m, "minute");
    }
    let h = m / 60;
    if h < 24 {
        return plural(h, "hour");
    }
    plural(h / 24, "day")
}

/// Compact age for tight spaces: "now", "12m", "3h", "2d".
pub fn fmt_age_short(at: f64, now: f64) -> String {
    let m = ((now - at) / 60_000.0).floor() as i64;
    if m < 1 {
        return "now".into();
    }
    if m < 60 {
        return format!("{m}m");
    }
    let h = m / 60;
    if h < 24 { format!("{h}h") } else { format!("{}d", h / 24) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_fixed_rounds_like_javascript() {
        assert_eq!(to_fixed(12.5, 0), "13");
        assert_eq!(to_fixed(-2.5, 0), "-3");
        assert_eq!(to_fixed(0.125, 2), "0.13");
        assert_eq!(to_fixed(1.005, 2), "1.00");
        assert_eq!(to_fixed(99.999, 1), "100.0");
        assert_eq!(to_fixed(-0.001, 2), "-0.00");
        assert_eq!(to_fixed(-0.0, 1), "0.0");
        assert_eq!(to_fixed(3.0, 2), "3.00");
    }

    #[test]
    fn formats_numbers_like_intl() {
        assert_eq!(fmt_int(1234567.4), "1,234,567");
        assert_eq!(fmt_int(-2.5), "-2");
        assert_eq!(fmt_int(999.5), "1,000");
        assert_eq!(fmt_compact(1234.0), "1.2K");
        assert_eq!(fmt_compact(999_950.0), "1M");
        assert_eq!(fmt_compact(999.96), "1K");
        assert_eq!(fmt_compact(12.34), "12.3");
        assert_eq!(fmt_compact(1.5e9), "1.5B");
        assert_eq!(fmt_compact(-45_678.0), "-45.7K");
        assert_eq!(fmt_compact(2e15), "2000T");
        assert_eq!(fmt_compact_long(1_250_000.0), "1.3 million");
        assert_eq!(fmt_number(1.005, 2), "1.01");
    }

    #[test]
    fn formats_durations_and_ages() {
        assert_eq!(fmt_duration(0.0), "now");
        assert_eq!(fmt_duration(12.0 * 60_000.0), "12m");
        assert_eq!(fmt_duration((47.0 * 60.0 + 19.0) * 60_000.0), "47h 19m");
        assert_eq!(fmt_duration((76.0 * 60.0) * 60_000.0), "3d 4h");
        assert_eq!(fmt_ago(0.0, 60_000.0), "1 minute ago");
        assert_eq!(fmt_age_short(0.0, 3.0 * 86_400_000.0), "3d");
    }

    #[test]
    fn parses_markup() {
        let spans = parse_markup("<i=3>TITLE</i>\nBody <i=1>hot</i> end");
        assert_eq!(spans.len(), 4);
        assert_eq!(spans[0], MarkupSpan { text: "TITLE".into(), style: Some(3) });
        assert_eq!(dispatch_title("<i=3> TITLE </i>\nrest"), "TITLE");
        assert_eq!(dispatch_title("First line\nSecond"), "First line");
        assert_eq!(sest_date(0.0), "01/01/2130");
        assert_eq!(sest_clock(3_723_000.0), "01:02:03");
    }
}
