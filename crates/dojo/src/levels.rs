//! Who is waiting, and where the round is fought.

use crate::fight::Belt;

/// Seconds a rival waits between committed punches. Lower is meaner.
pub fn pause(level: i32) -> f32 {
    (0.72 - (level - 1) as f32 * 0.08).max(0.28)
}

pub fn belt(level: i32) -> Belt {
    match (level - 1).rem_euclid(5) {
        0 => Belt::White,
        1 => Belt::Yellow,
        2 => Belt::Green,
        3 => Belt::Brown,
        _ => Belt::Black,
    }
}

/// From the green belt up, a rival answers a punch with a guard.
pub fn blocks(level: i32) -> bool {
    level >= 2
}

/// From the green belt up, a full focus bar becomes a crane kick.
pub fn cranes(level: i32) -> bool {
    level >= 3
}

/// Odd rounds are inside. Even rounds are the yard at dusk.
pub fn indoors(level: i32) -> bool {
    level % 2 == 1
}
