//! Point karate for one short round.
//!
//! A punch is one point, a crane kick is two, and a crane kick still scores
//! one through a guard. First to three, or the bell. A tie in which the
//! student never scored goes to the rival; any other tie is fought again.

use crate::levels;

/// Where the feet stand, in the virtual canvas. The backdrops meet it.
pub const FLOOR_Y: f32 = 452.0;

pub const ARENA_L: f32 = 52.0;
pub const ARENA_R: f32 = 428.0;
pub const MIN_GAP: f32 = 40.0;
pub const WALK: f32 = 148.0;
pub const HIT_W: f32 = 26.0;

pub const PUNCH_START: f32 = 0.08;
pub const PUNCH_ACTIVE: f32 = 0.10;
pub const PUNCH_RECOVER: f32 = 0.16;
pub const PUNCH_REACH: f32 = 54.0;

pub const CRANE_START: f32 = 0.34;
pub const CRANE_ACTIVE: f32 = 0.12;
pub const CRANE_RECOVER: f32 = 0.36;
pub const CRANE_REACH: f32 = 82.0;

pub const FOCUS_PUNCH: f32 = 0.34;
pub const FOCUS_BLOCK: f32 = 0.28;
pub const FOCUS_WHIFF: f32 = 0.12;
pub const BLOCK_TIRE: f32 = 0.90;
pub const BLOCK_DROP: f32 = 0.32;
pub const HIT_STUN: f32 = 0.30;
pub const DOWN_TIME: f32 = 0.72;
pub const PUSH_PUNCH: f32 = 22.0;
pub const PUSH_CRANE: f32 = 40.0;

pub const ROUND_TIME: f32 = 26.0;
pub const WIN_POINTS: i32 = 3;
pub const WIN_BONUS: i32 = 1000;
pub const LIVES: i32 = 3;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Pose {
    Idle,
    Walk,
    Punch,
    Block,
    Crane,
    Hit,
    Down,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Belt {
    White,
    Yellow,
    Green,
    Brown,
    Black,
}

impl Belt {
    pub fn name(self) -> &'static str {
        match self {
            Belt::White => "WHITE",
            Belt::Yellow => "YELLOW",
            Belt::Green => "GREEN",
            Belt::Brown => "BROWN",
            Belt::Black => "BLACK",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum End {
    None,
    Player,
    Rival,
    Draw,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Kind {
    Punch,
    Block,
    Crane,
    Break,
}

#[derive(Copy, Clone, Debug)]
pub struct Event {
    pub kind: Kind,
    pub score: i32,
    pub x: f32,
}

#[derive(Copy, Clone, Default)]
pub struct Intent {
    pub dir: i8,
    pub block: bool,
    pub punch: bool,
    pub crane: bool,
}

#[derive(Clone, Debug)]
pub struct Fighter {
    pub x: f32,
    pub face: i8,
    pub pose: Pose,
    pub t: f32,
    pub points: i32,
    pub focus: f32,
    pub stun: f32,
    pub down: f32,
    pub guard: f32,
    pub drop: f32,
    /// How long "OPEN" stays up after a guard falls.
    pub open: f32,
    pub belt: Belt,
    scored: bool,
}

impl Fighter {
    pub(crate) fn new(x: f32, belt: Belt) -> Self {
        Self {
            x,
            face: 1,
            pose: Pose::Idle,
            t: 0.0,
            points: 0,
            focus: 0.0,
            stun: 0.0,
            down: 0.0,
            guard: 0.0,
            drop: 0.0,
            open: 0.0,
            belt,
            scored: false,
        }
    }

    pub fn attacking(&self) -> bool {
        match self.pose {
            Pose::Punch => self.t < punch_len(),
            Pose::Crane => self.t < crane_len(),
            _ => false,
        }
    }

    /// The drill and the tests move one fighter without a whole round.
    pub fn drive(&mut self, input: Intent, dt: f32) {
        advance(self, input, dt);
        self.x = self.x.clamp(ARENA_L, ARENA_R);
    }

    /// True while a punch or crane would meet `target`'s x.
    pub fn meets(&self, target: f32) -> bool {
        if self.scored || !in_window(self.pose, self.t, 1.0 / 60.0) {
            return false;
        }
        covers(self, target)
    }
}

fn punch_len() -> f32 {
    PUNCH_START + PUNCH_ACTIVE + PUNCH_RECOVER
}

fn crane_len() -> f32 {
    CRANE_START + CRANE_ACTIVE + CRANE_RECOVER
}

pub struct World {
    pub player: Fighter,
    pub rival: Fighter,
    pub time: f32,
    pub end: End,
    pub flash: f32,
    pub flash_kind: Kind,
    pub events: Vec<Event>,
}

impl World {
    pub fn spar(level: i32) -> Self {
        Self {
            player: Fighter::new(128.0, Belt::White),
            rival: Fighter::new(352.0, levels::belt(level)),
            time: ROUND_TIME,
            end: End::None,
            flash: 0.0,
            flash_kind: Kind::Punch,
            events: Vec::new(),
        }
    }

    pub fn step(&mut self, dt: f32, player: Intent, rival: Intent) {
        if self.end != End::None {
            return;
        }
        self.events.clear();
        self.time -= dt;
        self.flash = (self.flash - dt).max(0.0);
        aim(&mut self.player, self.rival.x);
        aim(&mut self.rival, self.player.x);
        advance(&mut self.player, player, dt);
        advance(&mut self.rival, rival, dt);
        self.player.x = self.player.x.clamp(ARENA_L, ARENA_R);
        self.rival.x = self.rival.x.clamp(ARENA_L, ARENA_R);
        separate(&mut self.player, &mut self.rival);
        aim(&mut self.player, self.rival.x);
        aim(&mut self.rival, self.player.x);

        let blows = [
            consider(true, &self.player, &self.rival, dt),
            consider(false, &self.rival, &self.player, dt),
        ];
        for blow in blows.into_iter().flatten() {
            self.apply(blow);
        }
        settle_whiff(&mut self.player);
        settle_whiff(&mut self.rival);
        self.finish();
    }

    fn apply(&mut self, blow: Blow) {
        let (attacker, defender) = if blow.to_player {
            (&mut self.rival, &mut self.player)
        } else {
            (&mut self.player, &mut self.rival)
        };
        attacker.scored = true;
        let push = if blow.crane { PUSH_CRANE } else { PUSH_PUNCH };
        if blow.blocked && !blow.crane {
            defender.focus = (defender.focus + FOCUS_BLOCK).min(1.0);
            defender.x += attacker.face as f32 * push * 0.45;
            self.events.push(Event { kind: Kind::Block, score: 50, x: defender.x });
        } else if blow.blocked {
            attacker.points += 1;
            defender.drop = BLOCK_DROP;
            defender.guard = 0.0;
            defender.open = 0.5;
            defender.stun = HIT_STUN * 0.7;
            defender.pose = Pose::Hit;
            defender.t = 0.0;
            defender.x += attacker.face as f32 * push;
            self.flash = 0.55;
            self.flash_kind = Kind::Break;
            self.events.push(Event { kind: Kind::Break, score: 250, x: defender.x });
        } else if blow.crane {
            attacker.points += 2;
            defender.down = DOWN_TIME;
            defender.stun = 0.0;
            defender.pose = Pose::Down;
            defender.t = 0.0;
            defender.x += attacker.face as f32 * push;
            self.flash = 0.7;
            self.flash_kind = Kind::Crane;
            self.events.push(Event { kind: Kind::Crane, score: 500, x: defender.x });
        } else {
            attacker.points += 1;
            attacker.focus = (attacker.focus + FOCUS_PUNCH).min(1.0);
            defender.stun = HIT_STUN;
            defender.pose = Pose::Hit;
            defender.t = 0.0;
            defender.scored = true;
            defender.x += attacker.face as f32 * push;
            self.flash = 0.5;
            self.flash_kind = Kind::Punch;
            self.events.push(Event { kind: Kind::Punch, score: 100, x: defender.x });
        }
        defender.x = defender.x.clamp(ARENA_L, ARENA_R);
        separate(&mut self.player, &mut self.rival);
    }

    fn finish(&mut self) {
        if self.player.points >= WIN_POINTS {
            self.end = End::Player;
        } else if self.rival.points >= WIN_POINTS {
            self.end = End::Rival;
        } else if self.time <= 0.0 {
            self.end = if self.player.points > self.rival.points {
                End::Player
            } else if self.player.points == 0 || self.player.points < self.rival.points {
                End::Rival
            } else {
                End::Draw
            };
        }
    }
}

fn aim(f: &mut Fighter, other: f32) {
    f.face = if other >= f.x { 1 } else { -1 };
}

fn advance(f: &mut Fighter, input: Intent, dt: f32) {
    f.stun = (f.stun - dt).max(0.0);
    f.down = (f.down - dt).max(0.0);
    f.drop = (f.drop - dt).max(0.0);
    f.open = (f.open - dt).max(0.0);
    if f.down > 0.0 {
        f.pose = Pose::Down;
        f.t += dt;
        return;
    }
    if f.stun > 0.0 {
        f.pose = Pose::Hit;
        f.t += dt;
        return;
    }
    if f.attacking() {
        f.t += dt;
        if !f.attacking() {
            f.pose = Pose::Idle;
            f.t = 0.0;
        }
        return;
    }
    if input.crane && f.focus >= 1.0 {
        f.focus = 0.0;
        f.pose = Pose::Crane;
        f.t = 0.0;
        f.scored = false;
        f.guard = 0.0;
        return;
    }
    if input.punch {
        f.pose = Pose::Punch;
        f.t = 0.0;
        f.scored = false;
        f.guard = 0.0;
        return;
    }
    if input.block && f.drop <= 0.0 {
        if f.pose != Pose::Block {
            f.t = 0.0;
        }
        f.pose = Pose::Block;
        f.t += dt;
        f.guard += dt;
        if f.guard >= BLOCK_TIRE {
            f.guard = 0.0;
            f.drop = BLOCK_DROP;
            f.open = 0.5;
            f.pose = Pose::Idle;
            f.t = 0.0;
        }
        return;
    }
    f.guard = 0.0;
    let dir = input.dir.clamp(-1, 1);
    if dir != 0 {
        f.x += dir as f32 * WALK * dt;
        if f.pose != Pose::Walk {
            f.t = 0.0;
        }
        f.pose = Pose::Walk;
        f.t += dt;
    } else {
        if f.pose != Pose::Idle {
            f.t = 0.0;
        }
        f.pose = Pose::Idle;
        f.t += dt;
    }
}

struct Blow {
    to_player: bool,
    crane: bool,
    blocked: bool,
}

fn in_window(pose: Pose, t: f32, dt: f32) -> bool {
    let (start, len) = match pose {
        Pose::Punch => (PUNCH_START, PUNCH_ACTIVE),
        Pose::Crane => (CRANE_START, CRANE_ACTIVE),
        _ => return false,
    };
    let prev = t - dt;
    prev < start + len && t >= start
}

/// The limb covers every x from the hip out to the fist, so a long kick
/// still lands when the rival is standing inside its reach.
fn covers(attacker: &Fighter, target: f32) -> bool {
    let reach = if attacker.pose == Pose::Crane { CRANE_REACH } else { PUNCH_REACH };
    let sign = attacker.face as f32;
    let near = attacker.x + sign * 16.0;
    let far = attacker.x + sign * reach;
    target >= near.min(far) - HIT_W && target <= near.max(far) + HIT_W
}

fn consider(attacker_is_player: bool, attacker: &Fighter, defender: &Fighter, dt: f32) -> Option<Blow> {
    if attacker.scored || defender.down > 0.0 || !in_window(attacker.pose, attacker.t, dt) {
        return None;
    }
    if !covers(attacker, defender.x) {
        return None;
    }
    let crane = attacker.pose == Pose::Crane;
    let blocked = defender.pose == Pose::Block && defender.drop <= 0.0;
    Some(Blow { to_player: !attacker_is_player, crane, blocked })
}

fn settle_whiff(f: &mut Fighter) {
    let end = match f.pose {
        Pose::Punch => PUNCH_START + PUNCH_ACTIVE,
        Pose::Crane => CRANE_START + CRANE_ACTIVE,
        _ => return,
    };
    if f.t >= end && !f.scored {
        f.scored = true;
        if f.pose == Pose::Punch {
            f.focus = (f.focus + FOCUS_WHIFF).min(1.0);
        }
    }
}

fn separate(a: &mut Fighter, b: &mut Fighter) {
    let d = b.x - a.x;
    if d.abs() >= MIN_GAP {
        return;
    }
    let push = (MIN_GAP - d.abs()) * 0.5;
    let sign = if d >= 0.0 { 1.0 } else { -1.0 };
    a.x -= sign * push;
    b.x += sign * push;
    a.x = a.x.clamp(ARENA_L, ARENA_R);
    b.x = b.x.clamp(ARENA_L, ARENA_R);
    let d = b.x - a.x;
    if d.abs() < MIN_GAP {
        if a.x <= ARENA_L + 0.5 {
            b.x = (a.x + MIN_GAP).min(ARENA_R);
        } else {
            a.x = (b.x - MIN_GAP).max(ARENA_L);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frames(w: &mut World, n: i32, p: Intent, r: Intent) {
        for _ in 0..n {
            w.step(1.0 / 60.0, p, r);
        }
    }

    fn close(w: &mut World) {
        w.player.x = 200.0;
        w.rival.x = 246.0;
    }

    #[test]
    fn a_punch_in_range_scores_once() {
        let mut w = World::spar(1);
        close(&mut w);
        w.step(1.0 / 60.0, Intent { punch: true, ..Intent::default() }, Intent::default());
        assert_eq!(w.player.points, 0);
        frames(&mut w, 30, Intent::default(), Intent::default());
        assert_eq!(w.player.points, 1);
        assert_eq!(w.rival.points, 0);
        frames(&mut w, 40, Intent::default(), Intent::default());
        assert_eq!(w.player.points, 1);
    }

    #[test]
    fn a_punch_out_of_range_misses_and_builds_a_little_focus() {
        let mut w = World::spar(1);
        w.player.x = 120.0;
        w.rival.x = 320.0;
        w.step(1.0 / 60.0, Intent { punch: true, ..Intent::default() }, Intent::default());
        frames(&mut w, 40, Intent::default(), Intent::default());
        assert_eq!(w.player.points, 0);
        assert!(w.player.focus >= FOCUS_WHIFF - 0.001);
    }

    #[test]
    fn a_guard_stops_a_punch_and_fills_focus() {
        let mut w = World::spar(1);
        close(&mut w);
        let block = Intent { block: true, ..Intent::default() };
        w.step(1.0 / 60.0, Intent { punch: true, ..Intent::default() }, block);
        frames(&mut w, 30, Intent::default(), block);
        assert_eq!(w.player.points, 0);
        assert!(w.rival.focus >= FOCUS_BLOCK - 0.001);
    }

    #[test]
    fn a_held_guard_drops() {
        let mut w = World::spar(1);
        let block = Intent { block: true, ..Intent::default() };
        frames(&mut w, 70, block, Intent::default());
        assert!(w.player.drop > 0.0 || w.player.open > 0.0);
        assert_ne!(w.player.pose, Pose::Block);
    }

    #[test]
    fn crane_needs_focus_and_scores_two() {
        let mut w = World::spar(1);
        close(&mut w);
        w.step(1.0 / 60.0, Intent { crane: true, ..Intent::default() }, Intent::default());
        assert_ne!(w.player.pose, Pose::Crane);
        w.player.focus = 1.0;
        w.step(1.0 / 60.0, Intent { crane: true, ..Intent::default() }, Intent::default());
        assert_eq!(w.player.focus, 0.0);
        assert_eq!(w.player.pose, Pose::Crane);
        frames(&mut w, 40, Intent::default(), Intent::default());
        assert_eq!(w.player.points, 2);
        assert!(w.rival.down > 0.0 || w.rival.pose == Pose::Down || w.end != End::None);
    }

    #[test]
    fn crane_through_a_guard_scores_one_and_opens_it() {
        let mut w = World::spar(1);
        close(&mut w);
        w.player.focus = 1.0;
        let block = Intent { block: true, ..Intent::default() };
        w.step(1.0 / 60.0, Intent { crane: true, ..Intent::default() }, block);
        frames(&mut w, 40, Intent::default(), block);
        assert_eq!(w.player.points, 1);
        assert!(w.rival.drop > 0.0 || w.rival.stun > 0.0 || w.rival.open > 0.0);
    }

    #[test]
    fn three_points_end_the_round() {
        let mut w = World::spar(1);
        w.player.points = 2;
        close(&mut w);
        w.step(1.0 / 60.0, Intent { punch: true, ..Intent::default() }, Intent::default());
        frames(&mut w, 30, Intent::default(), Intent::default());
        assert_eq!(w.end, End::Player);
    }

    #[test]
    fn the_bell_with_no_score_goes_to_the_rival() {
        let mut w = World::spar(1);
        w.time = 0.01;
        w.step(1.0 / 60.0, Intent::default(), Intent::default());
        assert_eq!(w.end, End::Rival);
    }

    #[test]
    fn a_tied_score_above_zero_is_fought_again() {
        let mut w = World::spar(1);
        w.player.points = 1;
        w.rival.points = 1;
        w.time = 0.01;
        w.step(1.0 / 60.0, Intent::default(), Intent::default());
        assert_eq!(w.end, End::Draw);
    }
}
