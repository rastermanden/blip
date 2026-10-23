// Compiled by build.rs, and by the game for its tests: each uses part of this file.
#![allow(dead_code)]

use blip::cosy::{self, Tone, Voice, H as HOLD};
use blip::song::{major, Chord, Groove, Song, H, R};
use blip::Asset;

// ---- music: an original yo-scale tune, not a film theme ------------------
// D E G A B, plucked. QUIET under the title and the drill, SPAR once the
// round starts. The film's score is not used.

const D: Chord = major(50);
const G: Chord = major(55);
const A: Chord = major(57);

pub fn quiet_song() -> Song<'static> {
    Song {
        bpm: 86.0,
        melody: &[
            [62, H, 67, H, 69, H, 67, H],
            [62, H, H, H, 64, H, R, R],
            [69, H, 71, H, 74, H, 71, H],
            [69, H, H, H, R, R, 67, H],
            [67, H, 64, H, 62, H, 64, H],
            [67, H, H, H, 69, H, R, R],
            [71, H, 69, H, 67, H, 64, H],
            [62, H, H, H, R, R, R, R],
        ],
        chords: &[[D, D], [D, A], [G, G], [D, D], [G, G], [A, A], [D, G], [D, D]],
        lead: Voice::Pluck,
        lead_vol: 0.46,
        harmony: Some((Voice::Bell, -12)),
        bass: [0, R, 7, R],
        groove: Groove::Brush,
        seed: 0xD0D0_0001,
        ..Song::DEFAULT
    }
}

pub fn spar_song() -> Song<'static> {
    Song {
        bpm: 126.0,
        melody: &[
            [62, 64, 67, 69, 71, 69, 67, 64],
            [74, H, 71, H, 69, H, 67, H],
            [69, 71, 74, 71, 69, 67, 64, 62],
            [67, H, H, H, 64, H, R, R],
            [71, 69, 67, 69, 71, 74, 76, H],
            [74, H, 71, H, 69, H, 71, H],
            [67, 69, 71, 74, 71, 69, 67, 64],
            [62, H, H, H, R, R, R, R],
        ],
        chords: &[[D, D], [G, G], [A, A], [D, D], [G, A], [D, D], [A, G], [D, D]],
        lead: Voice::Pluck,
        lead_vol: 0.40,
        harmony: Some((Voice::Bell, -12)),
        bass: [0, 12, 0, 12],
        groove: Groove::Backbeat,
        seed: 0xD0D0_0002,
        ..Song::DEFAULT
    }
}

pub fn quiet_wav() -> Vec<u8> {
    quiet_song().render()
}

pub fn spar_wav() -> Vec<u8> {
    spar_song().render()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_tracks_render_the_exposed_song_data() {
        assert_eq!(quiet_wav(), quiet_song().render());
        assert_eq!(spar_wav(), spar_song().render());
    }
}

fn punch_sfx() -> Vec<u8> {
    let thud: Vec<f32> = cosy::glide(0.09, 160.0, 50.0, Tone::Sine, 2.4, 0.0).iter().map(|v| v * 0.8).collect();
    cosy::sfx(&cosy::mix(thud, &cosy::run(&[43], 0.04, Voice::Pluck, 0.35), 0.0, 1.0))
}

fn block_sfx() -> Vec<u8> {
    let wood: Vec<f32> = cosy::glide(0.07, 420.0, 180.0, Tone::Pulse(0.2), 1.6, 0.0).iter().map(|v| v * 0.45).collect();
    cosy::sfx(&cosy::mix(wood, &cosy::run(&[60, 67], 0.03, Voice::Pluck, 0.4), 0.0, 1.0))
}

fn crane_sfx() -> Vec<u8> {
    let whoosh: Vec<f32> = cosy::glide(0.22, 280.0, 1400.0, Tone::Tri, 1.5, 20.0).iter().map(|v| v * 0.35).collect();
    cosy::sfx(&cosy::mix(whoosh, &cosy::run(&[72, 79], 0.06, Voice::Pluck, 0.4), 0.04, 1.0))
}

fn point_sfx() -> Vec<u8> {
    cosy::jingle_with(&[76, 81, HOLD], 0.07, Voice::Bell, 0, None)
}

fn round_sfx() -> Vec<u8> {
    cosy::jingle_with(&[62, HOLD, 67, HOLD, 74, HOLD, HOLD], 0.09, Voice::Pluck, 0, Some((Voice::Bell, -1)))
}

fn game_over_sfx() -> Vec<u8> {
    cosy::jingle_with(&[69, HOLD, 65, HOLD, 62, HOLD, 57, HOLD, HOLD], 0.12, Voice::Pluck, 0, Some((Voice::Bell, -2)))
}

pub fn sounds() -> Vec<Asset> {
    vec![
        ("sounds/punch.wav", punch_sfx()),
        ("sounds/block.wav", block_sfx()),
        ("sounds/crane.wav", crane_sfx()),
        ("sounds/point.wav", point_sfx()),
        ("sounds/round.wav", round_sfx()),
        ("sounds/game_over.wav", game_over_sfx()),
    ]
}

pub fn music() -> Vec<(String, Vec<u8>)> {
    blip::xm::tunes(&[("QUIET", &quiet_wav), ("SPAR", &spar_wav)])
}
