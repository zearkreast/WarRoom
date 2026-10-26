//! The game API's cache sometimes answers an English request with a copy made for a player in another language.

use serde_json::Value;

const ENGLISH: &str = "the of and to is for on with our your are will be by that this from has have it as at their was";
/// Common words of the game's Latin-script languages that English never uses. No one-letter words: timestamps
/// split into "t" and "z".
const OTHER: &str = "de la el los las que del por para con una su sus se al \
    der das und ist den dem mit zu von des für ein eine nicht sich auf \
    le les et du est une pour dans sur au aux avec ce qui \
    il di che della dei delle nel gli \
    os da das dos em um uma não ao pela pelo nas nos \
    na się nie jest oraz że aby";

fn collect<'a>(value: &'a Value, out: &mut Vec<&'a str>) {
    match value {
        Value::String(s) => out.push(s),
        Value::Array(list) => list.iter().for_each(|v| collect(v, out)),
        Value::Object(map) => map.values().for_each(|v| collect(v, out)),
        _ => {}
    }
}

/// The game's markup tags, `<i=3>` and `</i>`, as spaces.
fn strip_tags(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find('<') {
        out.push_str(&rest[..i]);
        let tail = &rest[i..];
        let tag = tail.strip_prefix("</i>").map(|r| (4, r)).or_else(|| {
            let after = tail.strip_prefix("<i")?;
            let after = match after.strip_prefix('=') {
                Some(digits) => {
                    let n = digits.chars().take_while(char::is_ascii_digit).count();
                    if n == 0 {
                        return None;
                    }
                    &digits[n..]
                }
                None => after,
            };
            after.strip_prefix('>').map(|r| (tail.len() - r.len(), r))
        });
        match tag {
            Some((_, r)) => {
                out.push(' ');
                rest = r;
            }
            None => {
                out.push('<');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Whether an API response holds text in a language other than English.
pub fn not_english(body: &Value) -> bool {
    let mut strings = Vec::new();
    collect(body, &mut strings);
    let text = strip_tags(&strings.join(" "));

    let mut letters = 0usize;
    let mut other = 0usize;
    for ch in text.chars().filter(|c| c.is_alphabetic()) {
        letters += 1;
        if !ch.is_ascii_alphabetic() {
            other += 1;
        }
    }
    // Letters beyond A-Z mean another script or accents. A share, not any one: English text names planets like Pöpli IX.
    if other as f64 > letters as f64 * 0.02 {
        return true;
    }

    let english: Vec<&str> = ENGLISH.split(' ').collect();
    let foreign: Vec<&str> = OTHER.split_whitespace().collect();
    let (mut en, mut fo) = (0, 0);
    for word in text.to_lowercase().split(|c: char| !c.is_alphabetic()).filter(|w| !w.is_empty()) {
        if english.contains(&word) {
            en += 1;
        } else if foreign.contains(&word) {
            fo += 1;
        }
    }
    fo >= 3 && fo > en
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn dispatch(message: &str) -> Value {
        json!([{ "id": 1, "published": "2026-10-25T08:01:30Z", "type": 0, "message": message }])
    }

    #[test]
    fn passes_english_planet_names_with_accents_and_bare_timestamps() {
        assert!(!not_english(&dispatch(
            "<i=3>NEW ORDERS</i> Helldivers, hold the line on <i=1>Pöpli IX</i> for Liberty."
        )));
        assert!(!not_english(&json!({ "started": "2024-01-23T20:05:13Z", "factions": ["Humans", "Terminids"] })));
    }

    #[test]
    fn catches_other_languages() {
        for text in [
            "Los Helldivers deben defender la colonia de los ataques del enemigo para la Libertad.",
            "Die Helldivers müssen die Kolonie gegen den Feind verteidigen und nicht weichen.",
            "Les Helldivers doivent défendre la colonie contre les attaques de l'ennemi pour la Liberté.",
            "Gli Helldivers devono difendere la colonia dagli attacchi del nemico per la Libertà.",
            "Os Helldivers devem defender a colônia dos ataques do inimigo pela Liberdade.",
            "Helldiverzy muszą bronić kolonii przed atakami wroga, aby zachować Wolność.",
            "Адские десантники должны защитить колонию от врага.",
            "绝地潜兵必须保卫殖民地，抵御敌人的进攻。",
            "ヘルダイバーは敵の攻撃から植民地を守らなければならない。",
            "헬다이버는 적의 공격으로부터 식민지를 지켜야 한다.",
        ] {
            assert!(not_english(&dispatch(text)), "{text}");
        }
    }
}
