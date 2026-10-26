//! `String.prototype.localeCompare` for the app's English text: ICU root collation, close enough for
//! planet, sector and label names.

use std::cmp::Ordering;

/// Punctuation and symbols in the order ICU's root collation sorts them, all before digits and letters.
const PUNCT: &str = "_-,;:!?.'\"()[]{}@*/\\&#%`^+<=>|~$";

fn base(c: char) -> char {
    let c = c.to_lowercase().next().unwrap_or(c);
    match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => 'a',
        'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' => 'c',
        'ď' | 'đ' => 'd',
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => 'e',
        'ĝ' | 'ğ' | 'ġ' | 'ģ' => 'g',
        'ĥ' | 'ħ' => 'h',
        'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' => 'i',
        'ĵ' => 'j',
        'ķ' => 'k',
        'ĺ' | 'ļ' | 'ľ' | 'ŀ' | 'ł' => 'l',
        'ñ' | 'ń' | 'ņ' | 'ň' => 'n',
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ŏ' | 'ő' => 'o',
        'ŕ' | 'ŗ' | 'ř' => 'r',
        'ś' | 'ŝ' | 'ş' | 'š' => 's',
        'ţ' | 'ť' | 'ŧ' => 't',
        'ù' | 'ú' | 'û' | 'ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => 'u',
        'ŵ' => 'w',
        'ý' | 'ÿ' | 'ŷ' => 'y',
        'ź' | 'ż' | 'ž' => 'z',
        other => other,
    }
}

fn primary(c: char) -> (u8, u32) {
    if c.is_whitespace() {
        (0, 0)
    } else if let Some(i) = PUNCT.find(c) {
        (1, i as u32)
    } else if c.is_ascii_digit() {
        (3, c as u32)
    } else if c.is_alphabetic() {
        (4, base(c) as u32)
    } else {
        (2, c as u32)
    }
}

/// Primary weights of a string; ligatures expand to their letters ("Æ" sorts as "AE").
fn primaries(s: &str) -> impl Iterator<Item = (u8, u32)> + '_ {
    s.chars().flat_map(|c| {
        let expanded: &[char] = match c {
            'æ' | 'Æ' => &['a', 'e'],
            'œ' | 'Œ' => &['o', 'e'],
            'ß' => &['s', 's'],
            _ => return vec![primary(c)],
        };
        expanded.iter().map(|&e| primary(e)).collect()
    })
}

pub fn locale_cmp(a: &str, b: &str) -> Ordering {
    let pa = primaries(a);
    let pb = primaries(b);
    pa.cmp(pb)
        .then_with(|| {
            let accent =
                |s: &str| s.chars().map(|c| base(c) != c.to_lowercase().next().unwrap_or(c)).collect::<Vec<_>>();
            accent(a).cmp(&accent(b))
        })
        .then_with(|| {
            // Lowercase sorts before uppercase.
            let case = |s: &str| s.chars().map(char::is_uppercase).collect::<Vec<_>>();
            case(a).cmp(&case(b))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_like_icu() {
        let mut names =
            vec!["MERAK", "ANGEL'S VENTURE", "ANGEL-1", "angel", "ANGEL", "ÆRUS", "10 KRAK", "Émeria", "EMERIA"];
        names.sort_by(|a, b| locale_cmp(a, b));
        assert_eq!(
            names,
            ["10 KRAK", "ÆRUS", "angel", "ANGEL", "ANGEL-1", "ANGEL'S VENTURE", "EMERIA", "Émeria", "MERAK"]
        );
    }
}
