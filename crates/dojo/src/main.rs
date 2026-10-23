//! Dojo: short rounds of point karate.
//! A student trains, then fights. Inspired by the film — an original game,
//! not an official product.

mod cpu;
mod draw;
mod fight;
mod levels;

#[cfg(test)]
mod tunes;

use blip::input::{
    key_held, key_pressed, BLIP_KEY_A, BLIP_KEY_BUTTON2, BLIP_KEY_D, BLIP_KEY_DOWN, BLIP_KEY_F,
    BLIP_KEY_G, BLIP_KEY_LEFT, BLIP_KEY_RIGHT, BLIP_KEY_S, BLIP_KEY_SPACE,
};
#[cfg(not(target_arch = "wasm32"))]
use blip::macroquad::input::KeyCode;
use blip::{
    play_sfx, play_sfx_volume, web, window_conf, Blip, Jukebox, LifeResult, Session, Timer,
    GAME_OVER_MIN_WAIT, BLIP_BLACK, BLIP_ORANGE, BLIP_WHITE, BLIP_YELLOW,
};

use cpu::Brain;
use draw::Popup;
use fight::{Belt, End, Event, Fighter, Intent, Kind, Pose, World, LIVES, WIN_BONUS};
use levels::indoors;

const WIN_W: i32 = 480;
const WIN_H: i32 = 540;

fn conf() -> blip::macroquad::window::Conf {
    window_conf("DOJO", WIN_W, WIN_H)
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Phase {
    Title,
    Ready,
    Fight,
    Banner,
    Drill,
    Over,
}

#[derive(Copy, Clone)]
enum Banner {
    Win,
    Lose,
    Draw,
}

struct Drill {
    fighter: Fighter,
    left: f32,
    hits: i32,
    got: bool,
    arm: f32,
}

struct Sounds {
    punch: blip::BlipSound,
    block: blip::BlipSound,
    crane: blip::BlipSound,
    point: blip::BlipSound,
    round: blip::BlipSound,
    game_over: blip::BlipSound,
}

struct Game {
    phase: Phase,
    world: World,
    brain: Brain,
    demo: World,
    demo_brain: Brain,
    drill: Drill,
    sess: Session,
    level: i32,
    focus: f32,
    banner: Banner,
    wait: Timer,
    pops: Vec<Popup>,
    clock: f32,
    reported: bool,
}

impl Game {
    fn new() -> Self {
        Self {
            phase: Phase::Title,
            world: World::spar(1),
            brain: Brain::new(),
            demo: World::spar(2),
            demo_brain: Brain::new(),
            drill: Drill {
                fighter: Fighter::new(160.0, Belt::White),
                left: 0.0,
                hits: 0,
                got: false,
                arm: 0.0,
            },
            sess: Session::new(LIVES),
            level: 1,
            focus: 0.0,
            banner: Banner::Win,
            wait: Timer::default(),
            pops: Vec::new(),
            clock: 0.0,
            reported: false,
        }
    }

    fn begin_round(&mut self) {
        self.world = World::spar(self.level);
        self.world.player.focus = self.focus;
        self.brain = Brain::new();
        self.phase = Phase::Ready;
        self.wait.start(0.75);
    }

    fn begin_drill(&mut self) {
        self.drill = Drill {
            fighter: Fighter::new(150.0, Belt::White),
            left: 6.4,
            hits: 0,
            got: false,
            arm: 0.0,
        };
        self.phase = Phase::Drill;
    }

    fn track(&self) -> usize {
        if self.phase == Phase::Fight { 1 } else { 0 }
    }
}

fn read_player() -> Intent {
    let left = key_held(BLIP_KEY_A) || key_held(BLIP_KEY_LEFT);
    let right = key_held(BLIP_KEY_D) || key_held(BLIP_KEY_RIGHT);
    Intent {
        dir: match (left, right) {
            (true, false) => -1,
            (false, true) => 1,
            _ => 0,
        },
        block: key_held(BLIP_KEY_S) || key_held(BLIP_KEY_DOWN),
        punch: key_pressed(BLIP_KEY_SPACE) || key_pressed(BLIP_KEY_F),
        crane: key_pressed(BLIP_KEY_BUTTON2) || key_pressed(BLIP_KEY_G),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn hold_intent(intent: Intent, keys: &mut Vec<KeyCode>) {
    if intent.dir < 0 {
        keys.push(BLIP_KEY_LEFT);
    }
    if intent.dir > 0 {
        keys.push(BLIP_KEY_RIGHT);
    }
    if intent.block {
        keys.push(BLIP_KEY_DOWN);
    }
    if intent.punch {
        keys.push(BLIP_KEY_F);
    }
    if intent.crane {
        keys.push(BLIP_KEY_G);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn autopilot(g: &Game) {
    if !blip::bot::active() {
        return;
    }
    let mut keys = Vec::new();
    match g.phase {
        Phase::Title | Phase::Over => keys.push(BLIP_KEY_SPACE),
        Phase::Ready | Phase::Banner => {}
        Phase::Fight => hold_intent(cpu::student(&g.world), &mut keys),
        Phase::Drill => {
            hold_intent(cpu::drill(g.drill.fighter.x, g.drill.fighter.pose, draw::POST_X), &mut keys);
        }
    }
    blip::bot::hold(&keys);
}

#[cfg(target_arch = "wasm32")]
fn autopilot(_g: &Game) {}

fn hear(sfx: &Sounds, events: &[Event]) {
    for e in events {
        match e.kind {
            Kind::Block => play_sfx(&sfx.block),
            Kind::Crane => {
                play_sfx(&sfx.crane);
                play_sfx(&sfx.point);
            }
            Kind::Break => {
                play_sfx(&sfx.punch);
                play_sfx(&sfx.point);
            }
            Kind::Punch => {
                play_sfx(&sfx.punch);
                play_sfx(&sfx.point);
            }
        }
        if e.score > 0 {
            web::haptic();
        }
    }
}

fn note_swings(sfx: &Sounds, world: &World) {
    for f in [&world.player, &world.rival] {
        if f.t != 0.0 {
            continue;
        }
        if f.pose == Pose::Crane {
            play_sfx_volume(&sfx.crane, 0.45);
        } else if f.pose == Pose::Punch {
            play_sfx_volume(&sfx.punch, 0.28);
        }
    }
}

fn push_pops(g: &mut Game, events: &[Event]) {
    for e in events {
        if e.score > 0 {
            g.sess.add_score(e.score);
            g.pops.push(Popup { n: e.score, x: e.x - 12.0, y: 300.0, t: 0.55 });
        }
    }
}

fn tick_pops(g: &mut Game, dt: f32) {
    for p in &mut g.pops {
        p.t -= dt;
        p.y -= 28.0 * dt;
    }
    g.pops.retain(|p| p.t > 0.0);
}

fn step_demo(g: &mut Game, dt: f32) {
    let rival = cpu::think(&g.demo.player, &mut g.demo.rival, &mut g.demo_brain, 2, dt);
    let student = cpu::student(&g.demo);
    g.demo.step(dt, student, rival);
    if g.demo.end != End::None {
        g.demo = World::spar(2);
        g.demo_brain = Brain::new();
    }
}

fn update_title(g: &mut Game, dt: f32) {
    step_demo(g, dt);
    if key_pressed(BLIP_KEY_SPACE) || key_pressed(BLIP_KEY_F) {
        web::spend_coin();
        g.sess.reset(LIVES);
        g.level = 1;
        g.focus = 0.0;
        g.reported = false;
        g.begin_round();
    }
}

fn update_ready(g: &mut Game, dt: f32) {
    if g.wait.tick(dt) {
        g.phase = Phase::Fight;
    }
}

fn update_fight(g: &mut Game, dt: f32, sfx: &Sounds) {
    let cpu = cpu::think(&g.world.player, &mut g.world.rival, &mut g.brain, g.level, dt);
    g.world.step(dt, read_player(), cpu);
    note_swings(sfx, &g.world);
    let events: Vec<Event> = g.world.events.clone();
    hear(sfx, &events);
    push_pops(g, &events);
    if g.world.end == End::None {
        return;
    }
    play_sfx(&sfx.round);
    g.banner = match g.world.end {
        End::Player => Banner::Win,
        End::Draw => Banner::Draw,
        _ => Banner::Lose,
    };
    if g.world.end == End::Player {
        g.sess.add_score(WIN_BONUS + g.level * 100);
        g.level += 1;
        g.sess.next_level();
    }
    g.focus = g.world.player.focus;
    g.phase = Phase::Banner;
    g.wait.start(1.15);
}

fn update_banner(g: &mut Game, dt: f32, sfx: &Sounds) {
    if !g.wait.tick(dt) {
        return;
    }
    match g.banner {
        Banner::Win => g.begin_drill(),
        Banner::Draw => g.begin_round(),
        Banner::Lose => {
            if g.sess.lose_life() == LifeResult::GameOver {
                g.phase = Phase::Over;
                g.wait.start(GAME_OVER_MIN_WAIT);
                g.reported = true;
                play_sfx(&sfx.game_over);
                web::report_score(g.sess.score);
            } else {
                g.focus = 0.0;
                g.begin_round();
            }
        }
    }
}

fn update_drill(g: &mut Game, dt: f32, sfx: &Sounds) {
    let intent = read_player();
    g.drill.arm += dt * 2.6;
    g.drill.fighter.face = if draw::POST_X >= g.drill.fighter.x { 1 } else { -1 };
    g.drill.fighter.drive(
        Intent { dir: intent.dir, punch: intent.punch, block: intent.block, crane: false },
        dt,
    );
    if g.drill.fighter.meets(draw::POST_X) && !g.drill.got {
        g.drill.got = true;
        g.drill.hits += 1;
        g.sess.add_score(50);
        play_sfx(&sfx.punch);
        play_sfx(&sfx.point);
        g.pops.push(Popup { n: 50, x: draw::POST_X - 12.0, y: 320.0, t: 0.45 });
    }
    if g.drill.fighter.pose != Pose::Punch {
        g.drill.got = false;
    }
    // The padded arm crosses the student. A block keeps your place.
    let struck = g.drill.arm.sin() > 0.86 && (g.drill.fighter.x - (draw::POST_X - 48.0)).abs() < 34.0;
    if struck && g.drill.fighter.pose != Pose::Block && g.drill.fighter.pose != Pose::Punch {
        g.drill.fighter.x -= 26.0;
        play_sfx_volume(&sfx.block, 0.6);
    }
    g.drill.left -= dt;
    if g.drill.left <= 0.0 {
        g.focus = if g.drill.hits >= 3 { 1.0 } else { (g.drill.hits as f32) * 0.34 };
        g.begin_round();
    }
}

fn update_over(g: &mut Game, dt: f32) {
    g.wait.tick(dt);
    if g.wait.active() {
        return;
    }
    if key_pressed(BLIP_KEY_SPACE) || key_pressed(BLIP_KEY_F) {
        web::spend_coin();
        g.sess.reset(LIVES);
        g.level = 1;
        g.focus = 0.0;
        g.reported = false;
        g.begin_round();
    }
}

fn indoors_now(g: &Game) -> bool {
    match g.phase {
        Phase::Title | Phase::Drill => true,
        _ => indoors(g.level.max(1)),
    }
}

fn draw_world(blip: &Blip, g: &Game, room: &blip::BlipTex, yard: &blip::BlipTex) {
    let inside = indoors_now(g);
    draw::backdrop(blip, if inside { room } else { yard });
    if inside {
        draw::lantern(blip, g.clock);
    }
    match g.phase {
        Phase::Title => {
            draw::fighter(blip, &g.demo.rival, false, g.clock);
            draw::fighter(blip, &g.demo.player, true, g.clock);
        }
        Phase::Drill => {
            draw::post(blip, g.drill.arm);
            draw::fighter(blip, &g.drill.fighter, true, g.clock);
        }
        _ => {
            draw::fighter(blip, &g.world.rival, false, g.clock);
            draw::fighter(blip, &g.world.player, true, g.clock);
            if g.phase == Phase::Fight || g.phase == Phase::Ready {
                draw::hud_fight(blip, &g.world, g.clock);
            }
        }
    }
    draw::popups(blip, &g.pops);
}

fn draw_title(blip: &Blip, g: &Game) {
    blip.fill_rect(40.0, 78.0, 400.0, 168.0, blip::BlipColor { r: 0.08, g: 0.05, b: 0.03, a: 0.55 });
    blip.draw_centered("DOJO", 96.0, 6.0, BLIP_YELLOW);
    blip.draw_centered("INSPIRED BY THE FILM", 148.0, 2.0, BLIP_WHITE);
    blip.draw_centered("PUNCH  BLOCK  CRANE", 176.0, 2.0, BLIP_ORANGE);
    if (g.clock * 2.0) as i32 % 2 == 0 {
        blip.draw_centered("PRESS FIRE", 210.0, 2.0, BLIP_WHITE);
    }
    blip.draw_hi(&web::high_score(), 250.0, BLIP_ORANGE);
}

fn draw_ready(blip: &Blip, g: &Game) {
    blip.draw_centered(&format!("ROUND {}", g.level), 150.0, 3.0, BLIP_YELLOW);
    blip.draw_centered(levels::belt(g.level).name(), 188.0, 3.0, BLIP_WHITE);
    blip.draw_centered("READY", 230.0, 2.0, BLIP_ORANGE);
}

fn draw_banner(blip: &Blip, g: &Game) {
    let (word, color) = match g.banner {
        Banner::Win => ("YOU WIN", BLIP_YELLOW),
        Banner::Lose => ("DOWN", blip::BlipColor { r: 0.9, g: 0.28, b: 0.2, a: 1.0 }),
        Banner::Draw => ("DRAW", BLIP_WHITE),
    };
    blip.draw_centered(word, 180.0, 4.0, color);
    if matches!(g.banner, Banner::Win) {
        blip.draw_centered("NOW DRILL", 230.0, 2.0, BLIP_WHITE);
    }
}

fn draw_drill(blip: &Blip, g: &Game) {
    blip.draw_centered("DRILL", 40.0, 3.0, BLIP_YELLOW);
    blip.draw_centered("PUNCH THE POST", 78.0, 2.0, BLIP_WHITE);
    blip.draw_centered("BLOCK THE ARM", 100.0, 2.0, BLIP_ORANGE);
    blip.draw_number(g.drill.hits, 24.0, 40.0, 3.0, BLIP_WHITE);
    blip.draw_number(g.drill.left.ceil().max(0.0) as i32, 430.0, 40.0, 3.0, BLIP_YELLOW);
}

#[blip::macroquad::main(conf)]
async fn main() {
    let mut blip = Blip::new(WIN_W, WIN_H);
    let mut g = Game::new();
    web::set_players(0);

    let room = blip::load_png(blip::png!("dojo"));
    let yard = blip::load_png(blip::png!("yard"));
    let sfx = Sounds {
        punch: blip::audio::load_sound(blip::wav!("punch")).await,
        block: blip::audio::load_sound(blip::wav!("block")).await,
        crane: blip::audio::load_sound(blip::wav!("crane")).await,
        point: blip::audio::load_sound(blip::wav!("point")).await,
        round: blip::audio::load_sound(blip::wav!("round")).await,
        game_over: blip::audio::load_sound(blip::wav!("game_over")).await,
    };
    let mut music = Jukebox::new("DOJO", &["QUIET", "SPAR"]);

    loop {
        let dt = blip.delta_time;
        g.clock += dt;
        autopilot(&g);
        match g.phase {
            Phase::Title => update_title(&mut g, dt),
            Phase::Ready => update_ready(&mut g, dt),
            Phase::Fight => update_fight(&mut g, dt, &sfx),
            Phase::Banner => update_banner(&mut g, dt, &sfx),
            Phase::Drill => update_drill(&mut g, dt, &sfx),
            Phase::Over => update_over(&mut g, dt),
        }
        tick_pops(&mut g, dt);
        music.play(g.track());

        blip.clear(BLIP_BLACK);
        draw_world(&blip, &g, &room, &yard);
        if g.phase != Phase::Title && g.phase != Phase::Over {
            blip.draw_hud(g.sess.score, g.sess.lives);
        }
        match g.phase {
            Phase::Title => draw_title(&blip, &g),
            Phase::Ready => draw_ready(&blip, &g),
            Phase::Banner => draw_banner(&blip, &g),
            Phase::Drill => draw_drill(&blip, &g),
            Phase::Over => {
                let ready = !g.wait.active();
                blip.draw_game_over(g.sess.score, &web::high_score(), BLIP_ORANGE, BLIP_YELLOW, BLIP_WHITE, ready);
            }
            Phase::Fight => {}
        }

        blip.next_frame(60).await;
    }
}
