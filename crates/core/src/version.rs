//! Version and release tag comparison for the update check.

use std::cmp::Ordering;

fn strip_v(v: &str) -> &str {
    v.strip_prefix(['v', 'V']).unwrap_or(v)
}

/// A version or release tag without its pre-release or build label: "v0.3.0-beta" → "0.3.0".
pub fn release_number(v: &str) -> &str {
    strip_v(v).split(['-', '+']).next().unwrap_or("")
}

/// `parseInt(s, 10) || 0`.
fn leading_int(s: &str) -> i64 {
    let s = s.trim_start();
    let (sign, rest) = match s.strip_prefix('-') {
        Some(r) => (-1, r),
        None => (1, s.strip_prefix('+').unwrap_or(s)),
    };
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    digits.parse::<i64>().map_or(0, |n| sign * n)
}

/// The numeric parts of a version or release tag: "v0.2.8-alpha" → [0, 2, 8].
pub fn version_parts(v: &str) -> Vec<i64> {
    release_number(v).split('.').map(leading_int).collect()
}

/// The pre-release label after the dash ("beta.2" in "0.3.0-beta.2"), or "" for a full release.
fn pre_release(v: &str) -> String {
    let base = strip_v(v).split('+').next().unwrap_or("");
    base.split('-').skip(1).collect::<Vec<_>>().join("-")
}

/// `localeCompare` with numeric collation, for pre-release labels: "alpha" < "beta" < "rc", "beta.2" < "beta.10".
fn natural_cmp(a: &str, b: &str) -> Ordering {
    fn runs(s: &str) -> Vec<(bool, String)> {
        let mut out: Vec<(bool, String)> = Vec::new();
        for c in s.chars() {
            let digit = c.is_ascii_digit();
            match out.last_mut() {
                Some((d, run)) if *d == digit => run.push(c),
                _ => out.push((digit, c.to_string())),
            }
        }
        out
    }
    let (ra, rb) = (runs(a), runs(b));
    for (x, y) in ra.iter().zip(rb.iter()) {
        let ord = match (x.0, y.0) {
            (true, true) => {
                let (nx, ny) = (x.1.trim_start_matches('0'), y.1.trim_start_matches('0'));
                nx.len().cmp(&ny.len()).then_with(|| nx.cmp(ny))
            }
            (false, false) => x.1.to_lowercase().cmp(&y.1.to_lowercase()),
            // Digits sort before letters.
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
        };
        if ord != Ordering::Equal {
            return ord;
        }
    }
    ra.len().cmp(&rb.len())
}

/// Whether `candidate` is a later version than `current`, as semver orders them: by number first, then a full
/// release after its own pre-releases, and pre-releases by label ("alpha" < "beta" < "rc", "beta.2" < "beta.10").
pub fn is_newer(candidate: &str, current: &str) -> bool {
    let a = version_parts(candidate);
    let b = version_parts(current);
    for i in 0..a.len().max(b.len()) {
        let d = a.get(i).copied().unwrap_or(0) - b.get(i).copied().unwrap_or(0);
        if d != 0 {
            return d > 0;
        }
    }
    let (pa, pb) = (pre_release(candidate), pre_release(current));
    if pa == pb {
        return false;
    }
    if pa.is_empty() || pb.is_empty() {
        return pa.is_empty();
    }
    natural_cmp(&pa, &pb) == Ordering::Greater
}
