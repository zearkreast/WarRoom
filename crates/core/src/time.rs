//! Epoch-millisecond times, read and written as JavaScript's `Date.parse` and `toISOString` do.

const DAY_MS: f64 = 86_400_000.0;

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Year, month (1-12) and day of a day count since 1970-01-01.
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Epoch ms of a UTC calendar time.
pub fn utc_ms(y: i64, mo: u32, d: u32, h: u32, mi: u32, s: u32, ms: u32) -> f64 {
    let days = days_from_civil(y, mo as i64, d as i64);
    days as f64 * DAY_MS + (h as f64 * 3_600_000.0) + (mi as f64 * 60_000.0) + (s as f64 * 1000.0) + ms as f64
}

fn digits(s: &[u8], at: usize, n: usize) -> Option<u32> {
    let part = s.get(at..at + n)?;
    part.iter().try_fold(0u32, |acc, c| c.is_ascii_digit().then(|| acc * 10 + (c - b'0') as u32))
}

/// An ISO 8601 date or date-time as epoch ms; None where `Date.parse` would give NaN.
/// Times without an offset are read as UTC.
pub fn parse_iso(s: &str) -> Option<f64> {
    let b = s.trim().as_bytes();
    let y = digits(b, 0, 4)? as i64;
    if b.get(4) != Some(&b'-') {
        return None;
    }
    let mo = digits(b, 5, 2)?;
    if b.get(7) != Some(&b'-') {
        return None;
    }
    let d = digits(b, 8, 2)?;
    if !(1..=12).contains(&mo) || !(1..=31).contains(&d) {
        return None;
    }
    if b.len() == 10 {
        return Some(utc_ms(y, mo, d, 0, 0, 0, 0));
    }
    if !matches!(b.get(10), Some(b'T' | b't' | b' ')) {
        return None;
    }
    let h = digits(b, 11, 2)?;
    if b.get(13) != Some(&b':') {
        return None;
    }
    let mi = digits(b, 14, 2)?;
    let mut i = 16;
    let mut sec = 0;
    let mut ms = 0;
    if b.get(i) == Some(&b':') {
        sec = digits(b, i + 1, 2)?;
        i += 3;
        if b.get(i) == Some(&b'.') || b.get(i) == Some(&b',') {
            i += 1;
            let start = i;
            while b.get(i).is_some_and(u8::is_ascii_digit) {
                i += 1;
            }
            if i == start {
                return None;
            }
            let frac = &b[start..i.min(start + 3)];
            ms = frac.iter().fold(0, |acc, c| acc * 10 + (c - b'0') as u32) * 10u32.pow(3 - frac.len() as u32);
        }
    }
    if h > 24 || mi > 59 || sec > 59 {
        return None;
    }
    let mut offset_min: i64 = 0;
    match b.get(i) {
        None => {}
        Some(b'Z' | b'z') => i += 1,
        Some(&c @ (b'+' | b'-')) => {
            let oh = digits(b, i + 1, 2)? as i64;
            i += 3;
            let mut om = 0;
            if b.get(i) == Some(&b':') {
                om = digits(b, i + 1, 2)? as i64;
                i += 3;
            } else if let Some(v) = digits(b, i, 2) {
                om = v as i64;
                i += 2;
            }
            offset_min = (oh * 60 + om) * if c == b'-' { -1 } else { 1 };
        }
        _ => return None,
    }
    if i != b.len() {
        return None;
    }
    Some(utc_ms(y, mo, d, h, mi, sec, ms) - offset_min as f64 * 60_000.0)
}

/// `Date.parse`, NaN when unreadable.
pub fn parse_ms(s: &str) -> f64 {
    parse_iso(s).unwrap_or(f64::NAN)
}

/// Calendar parts of an epoch-ms time in UTC.
pub struct Utc {
    pub year: i64,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub ms: u32,
}

pub fn utc_parts(ms: f64) -> Utc {
    let total = ms.floor() as i64;
    let days = total.div_euclid(86_400_000);
    let rest = total.rem_euclid(86_400_000);
    let (year, month, day) = civil_from_days(days);
    Utc {
        year,
        month,
        day,
        hour: (rest / 3_600_000) as u32,
        minute: (rest / 60_000 % 60) as u32,
        second: (rest / 1000 % 60) as u32,
        ms: (rest % 1000) as u32,
    }
}

/// `new Date(ms).toISOString()`.
pub fn iso_string(ms: f64) -> String {
    let u = utc_parts(ms);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z", u.year, u.month, u.day, u.hour, u.minute, u.second, u.ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_iso_times() {
        let t = parse_iso("2026-10-26T16:06:07.123Z").unwrap();
        assert_eq!(iso_string(t), "2026-10-26T16:06:07.123Z");
        assert_eq!(parse_iso("2026-10-26T16:06:07.123456+00:00"), Some(t));
        assert_eq!(parse_iso("2026-10-26T18:06:07.123+02:00"), Some(t));
        assert_eq!(parse_iso("1970-01-01"), Some(0.0));
        assert_eq!(parse_iso("2024-02-29T00:00:00Z"), Some(1_709_164_800_000.0));
        assert_eq!(parse_iso("nonsense"), None);
        assert_eq!(iso_string(-1.0), "1969-12-31T23:59:59.999Z");
    }
}
