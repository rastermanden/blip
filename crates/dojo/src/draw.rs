//! The room, drawn flat, and the two fighters on top of it.

use blip::{Blip, BlipColor};

use crate::fight::{
    Belt, Fighter, Kind, Pose, World, CRANE_ACTIVE, CRANE_RECOVER, CRANE_START, FLOOR_Y, PUNCH_ACTIVE,
    PUNCH_RECOVER, PUNCH_START,
};

pub const POST_X: f32 = 318.0;

fn rgb(r: u8, g: u8, b: u8) -> BlipColor {
    BlipColor { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a: 1.0 }
}

fn belt_color(belt: Belt) -> BlipColor {
    match belt {
        Belt::White => rgb(236, 232, 220),
        Belt::Yellow => rgb(232, 190, 48),
        Belt::Green => rgb(46, 148, 72),
        Belt::Brown => rgb(128, 74, 40),
        Belt::Black => rgb(28, 26, 32),
    }
}

struct Look {
    gi: BlipColor,
    shade: BlipColor,
    skin: BlipColor,
    hair: BlipColor,
    belt: BlipColor,
}

fn look(f: &Fighter, student: bool) -> Look {
    if student {
        Look {
            gi: rgb(240, 236, 226),
            shade: rgb(198, 188, 170),
            skin: rgb(232, 184, 148),
            hair: rgb(48, 36, 30),
            belt: rgb(232, 196, 72),
        }
    } else if f.belt == Belt::Black {
        Look {
            gi: rgb(44, 46, 58),
            shade: rgb(26, 28, 36),
            skin: rgb(214, 164, 130),
            hair: rgb(20, 16, 16),
            belt: belt_color(Belt::Black),
        }
    } else {
        Look {
            gi: rgb(56, 92, 168),
            shade: rgb(32, 58, 118),
            skin: rgb(214, 164, 130),
            hair: rgb(36, 28, 24),
            belt: belt_color(f.belt),
        }
    }
}

pub fn backdrop(blip: &Blip, tex: &blip::BlipTex) {
    blip.draw_texture(tex, 0.0, 0.0, 480.0, 540.0);
}

pub fn lantern(blip: &Blip, clock: f32) {
    let x = 240.0 + (clock * 1.3).sin() * 10.0;
    blip.draw_line(240.0, 36.0, x, 78.0, rgb(72, 48, 32));
    blip.fill_rect(x - 14.0, 78.0, 28.0, 22.0, rgb(232, 96, 48));
    blip.fill_rect(x - 10.0, 82.0, 20.0, 10.0, rgb(255, 196, 96));
    blip.fill_rect(x - 6.0, 100.0, 12.0, 4.0, rgb(72, 48, 32));
}

pub fn post(blip: &Blip, arm: f32) {
    blip.fill_rect(POST_X - 8.0, FLOOR_Y - 118.0, 16.0, 118.0, rgb(92, 64, 40));
    blip.fill_rect(POST_X - 16.0, FLOOR_Y - 122.0, 32.0, 10.0, rgb(70, 48, 30));
    // A padded arm on a rope, swinging across the student.
    let x = POST_X - 78.0 + arm.sin() * 64.0;
    let y = FLOOR_Y - 96.0 + arm.cos() * 18.0;
    blip.draw_line(POST_X, FLOOR_Y - 100.0, x, y, rgb(120, 96, 64));
    blip.fill_rect(x - 10.0, y - 8.0, 20.0, 16.0, rgb(196, 48, 42));
}

fn ink() -> BlipColor {
    rgb(22, 14, 12)
}

fn tile(blip: &Blip, x: f32, y: f32, w: f32, h: f32, c: BlipColor) {
    blip.fill_rect(x - 1.0, y - 1.0, w + 2.0, h + 2.0, ink());
    blip.fill_rect(x, y, w, h, c);
}

/// Left edge of a bar that runs `len` pixels in the facing direction.
fn fwd(x: f32, s: f32, start: f32, len: f32) -> f32 {
    if s > 0.0 { x + start } else { x - start - len }
}

pub fn fighter(blip: &Blip, f: &Fighter, student: bool, clock: f32) {
    let look = look(f, student);
    if f.pose == Pose::Down {
        fallen(blip, f, &look);
        return;
    }
    let s = f.face as f32;
    let bob = match f.pose {
        Pose::Walk => (f.t * 11.0).sin() * 2.0,
        Pose::Idle => (clock * 2.2).sin(),
        _ => 0.0,
    };
    let lean = match f.pose {
        Pose::Hit => -s * 8.0,
        Pose::Punch => s * 4.0,
        _ => 0.0,
    };
    let x = f.x.round() + lean;
    let foot = FLOOR_Y + bob;
    let step = if f.pose == Pose::Walk { (f.t * 11.0).sin() * 6.0 } else { 0.0 };
    blip.fill_rect(x - 18.0, FLOOR_Y - 4.0, 36.0, 5.0, BlipColor { r: 0.0, g: 0.0, b: 0.0, a: 0.35 });

    match f.pose {
        Pose::Crane if f.t >= CRANE_START => crane_kick(blip, f, &look, x, foot, s),
        Pose::Crane => crane_ready(blip, &look, x, foot, s, (f.t / CRANE_START).clamp(0.0, 1.0)),
        Pose::Punch => punch(blip, &look, x, foot, s, punch_reach(f.t), step),
        Pose::Block => guard(blip, &look, x, foot, s),
        Pose::Hit => stance(blip, &look, x, foot, s, 0.0, true),
        _ => stance(blip, &look, x, foot, s, step, false),
    }
}

fn punch_reach(t: f32) -> f32 {
    if t < PUNCH_START {
        (t / PUNCH_START).clamp(0.0, 1.0)
    } else if t < PUNCH_START + PUNCH_ACTIVE {
        1.0
    } else {
        1.0 - ((t - PUNCH_START - PUNCH_ACTIVE) / PUNCH_RECOVER).clamp(0.0, 1.0)
    }
}

fn crane_ext(t: f32) -> f32 {
    if t < CRANE_START + CRANE_ACTIVE {
        ((t - CRANE_START) / CRANE_ACTIVE).clamp(0.0, 1.0)
    } else {
        1.0 - ((t - CRANE_START - CRANE_ACTIVE) / CRANE_RECOVER).clamp(0.0, 1.0)
    }
}

fn legs(blip: &Blip, look: &Look, x: f32, foot: f32, s: f32, step: f32) {
    let back = x - s * (8.0 + step);
    let front = x + s * (6.0 - step * 0.3);
    tile(blip, back, foot - 46.0, 8.0, 46.0, look.gi);
    tile(blip, front, foot - 46.0, 8.0, 46.0, look.gi);
    tile(blip, back - 2.0, foot - 5.0, 12.0, 5.0, look.skin);
    tile(blip, front - 2.0, foot - 5.0, 12.0, 5.0, look.skin);
}

fn body(blip: &Blip, look: &Look, x: f32, foot: f32, s: f32) {
    tile(blip, x - 9.0, foot - 84.0, 18.0, 40.0, look.gi);
    blip.fill_rect(x - s * 2.0 - 3.0, foot - 80.0, 5.0, 32.0, look.shade);
    tile(blip, x - 11.0, foot - 50.0, 22.0, 6.0, look.belt);
    tile(blip, x + s * 6.0, foot - 48.0, 4.0, 10.0, look.belt);
}

fn head(blip: &Blip, look: &Look, x: f32, foot: f32, s: f32, hurt: bool) {
    let hy = foot - 98.0;
    blip.fill_circle(x, hy, 12.0, ink());
    blip.fill_circle(x, hy, 10.5, look.skin);
    blip.fill_circle(x, hy - 4.0, 10.0, look.hair);
    blip.fill_circle(x + s * 2.0, hy + 2.0, 8.5, look.skin);
    let eye = rgb(28, 18, 16);
    blip.fill_rect(x + s * 1.0 - 5.0, hy + 1.0, 3.0, 3.0, eye);
    blip.fill_rect(x + s * 1.0 + 2.0, hy + 1.0, 3.0, 3.0, eye);
    if hurt {
        blip.fill_rect(x + s * 9.0, hy + 6.0, 5.0, 2.0, rgb(176, 48, 44));
    }
}

fn fist(blip: &Blip, x: f32, y: f32, look: &Look) {
    tile(blip, x, y, 7.0, 7.0, look.skin);
}

fn stance(blip: &Blip, look: &Look, x: f32, foot: f32, s: f32, step: f32, hurt: bool) {
    legs(blip, look, x, foot, s, step);
    body(blip, look, x, foot, s);
    // Fists sit beside the ribs, clear of the jacket.
    tile(blip, x - s * 16.0, foot - 62.0, 7.0, 16.0, look.gi);
    tile(blip, x + s * 10.0, foot - 64.0, 7.0, 16.0, look.gi);
    fist(blip, x - s * 16.0, foot - 50.0, look);
    fist(blip, x + s * 10.0, foot - 52.0, look);
    head(blip, look, x, foot, s, hurt);
}

fn punch(blip: &Blip, look: &Look, x: f32, foot: f32, s: f32, reach: f32, step: f32) {
    legs(blip, look, x, foot, s, step);
    let len = 14.0 + reach * 34.0;
    let ax = if s > 0.0 { x + 6.0 } else { x - 6.0 - len };
    tile(blip, ax, foot - 74.0, len, 7.0, look.gi);
    let fist_x = if s > 0.0 { x + 6.0 + len - 2.0 } else { x - 6.0 - len - 5.0 };
    fist(blip, fist_x, foot - 75.0, look);
    body(blip, look, x, foot, s);
    tile(blip, x - s * 12.0, foot - 68.0, 7.0, 14.0, look.gi);
    fist(blip, x - s * 12.0, foot - 56.0, look);
    head(blip, look, x, foot, s, false);
}

fn guard(blip: &Blip, look: &Look, x: f32, foot: f32, s: f32) {
    legs(blip, look, x, foot, s, 0.0);
    body(blip, look, x, foot, s);
    let gx = fwd(x, s, 8.0, 7.0);
    let gx2 = fwd(x, s, 16.0, 7.0);
    tile(blip, gx, foot - 102.0, 7.0, 28.0, look.gi);
    tile(blip, gx2, foot - 96.0, 7.0, 26.0, look.gi);
    fist(blip, gx, foot - 108.0, look);
    fist(blip, gx2, foot - 102.0, look);
    head(blip, look, x, foot, s, false);
}

fn crane_ready(blip: &Blip, look: &Look, x: f32, foot: f32, s: f32, k: f32) {
    tile(blip, x - s * 2.0, foot - 46.0, 8.0, 46.0, look.gi);
    tile(blip, x - s * 4.0, foot - 5.0, 12.0, 5.0, look.skin);
    let knee = foot - 52.0 - k * 22.0;
    tile(blip, x + s * 4.0, knee, 8.0, 20.0, look.gi);
    fist(blip, x + s * 2.0, knee + 14.0, look);
    body(blip, look, x, foot, s);
    let spread = 16.0 + k * 14.0;
    let rear = fwd(x, -s, 8.0, spread);
    let lead = fwd(x, s, 8.0, spread);
    tile(blip, rear, foot - 86.0 - k * 8.0, spread, 6.0, look.gi);
    tile(blip, lead, foot - 90.0 - k * 8.0, spread, 6.0, look.gi);
    fist(blip, fwd(x, -s, 8.0 + spread, 7.0), foot - 88.0 - k * 8.0, look);
    fist(blip, fwd(x, s, 8.0 + spread, 7.0), foot - 92.0 - k * 8.0, look);
    head(blip, look, x, foot, s, false);
}

fn crane_kick(blip: &Blip, f: &Fighter, look: &Look, x: f32, foot: f32, s: f32) {
    let ext = crane_ext(f.t);
    tile(blip, x - s * 2.0, foot - 46.0, 8.0, 46.0, look.gi);
    tile(blip, x - s * 4.0, foot - 5.0, 12.0, 5.0, look.skin);
    // Thigh lifts, then the shin shoots out at head height.
    tile(blip, x + s * 2.0, foot - 78.0, 8.0, 28.0, look.gi);
    let len = 16.0 + ext * 46.0;
    let shin_y = foot - 96.0 - ext * 8.0;
    let shin_x = if s > 0.0 { x + 8.0 } else { x - 8.0 - len };
    tile(blip, shin_x, shin_y, len, 7.0, look.gi);
    let toe = if s > 0.0 { x + 8.0 + len - 2.0 } else { x - 8.0 - len - 8.0 };
    tile(blip, toe, shin_y - 2.0, 10.0, 8.0, look.skin);
    if ext > 0.4 {
        blip.draw_line_ex(toe - s * 22.0, shin_y + 12.0, toe - s * 4.0, shin_y + 4.0, 2.0, rgb(255, 214, 120));
    }
    body(blip, look, x, foot, s);
    tile(blip, fwd(x, -s, 8.0, 20.0), foot - 96.0, 20.0, 6.0, look.gi);
    tile(blip, fwd(x, s, 8.0, 16.0), foot - 100.0, 16.0, 6.0, look.gi);
    fist(blip, fwd(x, -s, 28.0, 7.0), foot - 98.0, look);
    fist(blip, fwd(x, s, 24.0, 7.0), foot - 102.0, look);
    head(blip, look, x, foot, s, false);
}

fn fallen(blip: &Blip, f: &Fighter, look: &Look) {
    let x = f.x.round();
    let s = f.face as f32;
    let y = FLOOR_Y - 16.0;
    blip.fill_rect(x - 30.0, FLOOR_Y - 4.0, 70.0, 5.0, BlipColor { r: 0.0, g: 0.0, b: 0.0, a: 0.35 });
    let body_x = if s > 0.0 { x - 18.0 } else { x - 28.0 };
    tile(blip, body_x, y, 46.0, 14.0, look.gi);
    tile(blip, x - 8.0, y + 4.0, 16.0, 6.0, look.belt);
    tile(blip, x - s * 36.0, y + 2.0, 16.0, 8.0, look.gi);
    head(blip, look, x + s * 32.0, FLOOR_Y + 78.0, s, true);
}

pub fn hud_fight(blip: &Blip, w: &World, clock: f32) {
    let secs = w.time.max(0.0).ceil() as i32;
    let clock_color = if secs <= 5 { rgb(232, 72, 48) } else { rgb(255, 244, 220) };
    blip.draw_centered(&format!("0:{secs:02}"), 34.0, 2.0, clock_color);
    let full = w.player.focus >= 1.0 && (clock * 6.0).sin() > 0.0;
    pip(blip, 196.0, w.player.focus, full);
    let belt = w.rival.belt.name();
    blip.draw_text(belt, 468.0 - belt.len() as f32 * 12.0, 36.0, 2.0, belt_color(w.rival.belt));
    over_head(blip, &w.player, rgb(255, 244, 220));
    over_head(blip, &w.rival, rgb(255, 214, 160));
    if w.player.open > 0.0 {
        blip.draw_text_outlined("OPEN", w.player.x - 24.0, FLOOR_Y - 172.0, 2.0, rgb(232, 72, 48), ink());
    }
    if w.rival.open > 0.0 {
        blip.draw_text_outlined("OPEN", w.rival.x - 24.0, FLOOR_Y - 172.0, 2.0, rgb(232, 72, 48), ink());
    }
    if w.flash > 0.0 {
        let word = match w.flash_kind {
            Kind::Crane => "CRANE",
            Kind::Break => "BREAK",
            Kind::Block => "BLOCK",
            Kind::Punch => "POINT",
        };
        blip.draw_centered_outlined(word, 240.0, 118.0, 3.0, rgb(255, 236, 160), ink());
    }
}

fn over_head(blip: &Blip, f: &Fighter, c: BlipColor) {
    let label = f.points.to_string();
    let w = label.len() as f32 * 12.0;
    blip.draw_text_outlined(&label, f.x - w * 0.5, FLOOR_Y - 158.0, 2.0, c, ink());
}

fn pip(blip: &Blip, x: f32, focus: f32, full: bool) {
    for i in 0..8 {
        let on = focus >= (i as f32 + 1.0) / 8.0;
        let c = if full {
            rgb(255, 220, 80)
        } else if on {
            rgb(232, 120, 48)
        } else {
            rgb(70, 52, 40)
        };
        blip.fill_rect(x + i as f32 * 11.0, 56.0, 8.0, 8.0, c);
    }
}

pub struct Popup {
    pub n: i32,
    pub x: f32,
    pub y: f32,
    pub t: f32,
}

pub fn popups(blip: &Blip, pops: &[Popup]) {
    for p in pops {
        if p.t > 0.0 {
            blip.draw_number(p.n, p.x, p.y, 2.0, rgb(255, 236, 160));
        }
    }
}
