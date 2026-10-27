//! Stratagem codes keyed in with WASD or the arrow keys.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arrow {
    Up,
    Down,
    Left,
    Right,
}

pub struct Stratagem {
    pub name: &'static str,
    pub code: &'static [Arrow],
}

/// Keyed in on the Hellpad masthead, it opens the G.W.W.'s Super Destroyer manual.
pub const CALL_IN: Stratagem = Stratagem {
    name: "Call In Super Destroyer",
    code: &[Arrow::Up, Arrow::Up, Arrow::Down, Arrow::Down, Arrow::Left, Arrow::Right, Arrow::Left, Arrow::Right],
};

/// How far a code is keyed in after one more arrow: the longest start of the code the last presses spell.
pub fn step(at: usize, a: Arrow, code: &[Arrow]) -> usize {
    let mut keyed: Vec<Arrow> = code[..at.min(code.len())].to_vec();
    keyed.push(a);
    for n in (1..=keyed.len().min(code.len())).rev() {
        if keyed[keyed.len() - n..].iter().zip(code).all(|(k, c)| k == c) {
            return n;
        }
    }
    0
}

/// The arrow a key dials: W A S D or the arrow keys, as in the game.
pub fn arrow_of(key: &str) -> Option<Arrow> {
    match key.to_lowercase().as_str() {
        "w" | "arrowup" => Some(Arrow::Up),
        "a" | "arrowleft" => Some(Arrow::Left),
        "s" | "arrowdown" => Some(Arrow::Down),
        "d" | "arrowright" => Some(Arrow::Right),
        _ => None,
    }
}
