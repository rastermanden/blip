//! The rival's habits, and the autopilot that plays the student.

use crate::fight::{Fighter, Intent, Pose, World, CRANE_START, PUNCH_ACTIVE, PUNCH_START};
use crate::levels;

pub struct Brain {
    wait: f32,
    focus: f32,
}

impl Brain {
    pub fn new() -> Self {
        Self { wait: 0.4, focus: 0.0 }
    }
}

/// What the rival does this frame. `level` 1 is a white belt who only walks in and punches.
pub fn think(player: &Fighter, rival: &mut Fighter, brain: &mut Brain, level: i32, dt: f32) -> Intent {
    brain.wait -= dt;
    // A visible meter, separate from the student's. Blocking does not feed it.
    brain.focus = (brain.focus + dt * (0.16 + level as f32 * 0.05)).min(1.0);
    rival.focus = brain.focus;

    let dist = (player.x - rival.x).abs();
    let toward: i8 = if player.x < rival.x { -1 } else { 1 };

    if player.pose == Pose::Crane && player.t < CRANE_START && dist < 120.0 && level >= 2 {
        return Intent { dir: -toward, ..Intent::default() };
    }
    let incoming = player.pose == Pose::Punch && player.t < PUNCH_START + PUNCH_ACTIVE;
    if incoming && dist < 96.0 && levels::blocks(level) && rival.drop <= 0.0 {
        return Intent { block: true, ..Intent::default() };
    }
    // 62px is inside a punch, and outside the 40px gap the two bodies keep.
    if dist > 62.0 {
        return Intent { dir: toward, ..Intent::default() };
    }
    if levels::cranes(level)
        && brain.focus >= 1.0
        && matches!(rival.pose, Pose::Idle | Pose::Walk)
        && player.pose == Pose::Block
    {
        brain.focus = 0.0;
        rival.focus = 1.0;
        return Intent { crane: true, ..Intent::default() };
    }
    if brain.wait <= 0.0 && matches!(rival.pose, Pose::Idle | Pose::Walk | Pose::Block) {
        brain.wait = levels::pause(level);
        return Intent { punch: true, ..Intent::default() };
    }
    if levels::blocks(level) && brain.wait > levels::pause(level) * 0.45 {
        return Intent { block: true, ..Intent::default() };
    }
    Intent::default()
}

/// The native autopilot: close, punch, block a punch, crane when the bar is full.
pub fn student(w: &World) -> Intent {
    let dist = (w.rival.x - w.player.x).abs();
    let toward: i8 = if w.rival.x > w.player.x { 1 } else { -1 };
    let incoming = w.rival.pose == Pose::Punch && w.rival.t < PUNCH_START + PUNCH_ACTIVE + 0.05;
    if incoming && dist < 100.0 {
        return Intent { block: true, ..Intent::default() };
    }
    if w.player.focus >= 1.0
        && (48.0..112.0).contains(&dist)
        && matches!(w.player.pose, Pose::Idle | Pose::Walk)
    {
        return Intent { crane: true, ..Intent::default() };
    }
    if dist > 70.0 {
        return Intent { dir: toward, ..Intent::default() };
    }
    if matches!(w.player.pose, Pose::Idle | Pose::Walk) {
        return Intent { punch: true, ..Intent::default() };
    }
    Intent::default()
}

/// Walk up to the post and punch it. `post` is its x.
#[cfg(not(target_arch = "wasm32"))]
pub fn drill(x: f32, pose: Pose, post: f32) -> Intent {
    let dist = (post - x).abs();
    let toward: i8 = if post > x { 1 } else { -1 };
    if dist > 58.0 {
        return Intent { dir: toward, ..Intent::default() };
    }
    if matches!(pose, Pose::Idle | Pose::Walk) {
        Intent { punch: true, ..Intent::default() }
    } else {
        Intent::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fight::End;

    #[test]
    fn a_still_student_loses_the_round() {
        let mut w = World::spar(1);
        let mut brain = Brain::new();
        let mut t = 0.0;
        while w.end == End::None && t < 20.0 {
            let dt = 1.0 / 60.0;
            let cpu = think(&w.player, &mut w.rival, &mut brain, 1, dt);
            w.step(dt, Intent::default(), cpu);
            t += dt;
        }
        assert_eq!(w.end, End::Rival, "still going at {t:.1}s, score {}-{}", w.player.points, w.rival.points);
        assert!(t < 16.0, "a white belt took {t:.1}s to win");
    }
}
