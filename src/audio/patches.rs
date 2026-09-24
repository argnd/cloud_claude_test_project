//! Instrument presets and the drum kit shared by the music and the sound effects.

use super::seq::DrumHit;
use super::synth::{BASE, Patch, Wave};

// ---------------------------------------------------------------- leads

/// 25% pulse lead with delayed vibrato: the classic 16-bit melody voice.
pub const LEAD: Patch = Patch {
    wave: Wave::Pulse,
    duty: 0.25,
    pwm_depth: 0.06,
    pwm_rate: 0.6,
    attack: 0.008,
    decay: 0.5,
    sustain: 0.7,
    release: 0.16,
    cutoff: 2600.0,
    cutoff_env: 2600.0,
    cutoff_decay: 0.25,
    key_track: 0.5,
    vib_depth: 0.17,
    vib_rate: 5.3,
    vib_delay: 0.22,
    gain: 0.42,
    ..BASE
};

/// Hollow square lead (clarinet-like).
pub const SQUARE: Patch = Patch {
    wave: Wave::Pulse,
    duty: 0.5,
    attack: 0.012,
    decay: 0.4,
    sustain: 0.75,
    release: 0.14,
    cutoff: 2000.0,
    cutoff_env: 1200.0,
    cutoff_decay: 0.2,
    key_track: 0.6,
    vib_depth: 0.14,
    vib_rate: 5.0,
    vib_delay: 0.3,
    gain: 0.38,
    ..BASE
};

/// Narrow pulse with a soft attack: oboe / shawm.
pub const REED: Patch = Patch {
    wave: Wave::Pulse,
    duty: 0.14,
    attack: 0.03,
    decay: 0.5,
    sustain: 0.8,
    release: 0.14,
    cutoff: 2200.0,
    key_track: 0.6,
    vib_depth: 0.15,
    vib_rate: 5.4,
    vib_delay: 0.25,
    gain: 0.4,
    ..BASE
};

/// Breathy triangle flute.
pub const FLUTE: Patch = Patch {
    wave: Wave::Tri,
    noise: 0.05,
    attack: 0.045,
    decay: 0.4,
    sustain: 0.8,
    release: 0.14,
    cutoff: 3200.0,
    key_track: 0.4,
    vib_depth: 0.16,
    vib_rate: 5.1,
    vib_delay: 0.18,
    gain: 0.82,
    ..BASE
};

/// Bowed-string lead: two detuned saws, gentle attack.
pub const STRINGS: Patch = Patch {
    wave: Wave::Saw,
    detune: 7.0,
    attack: 0.12,
    decay: 0.6,
    sustain: 0.85,
    release: 0.35,
    cutoff: 2300.0,
    key_track: 0.5,
    vib_depth: 0.13,
    vib_rate: 5.2,
    vib_delay: 0.3,
    gain: 0.42,
    ..BASE
};

/// Brass stab/section.
pub const BRASS: Patch = Patch {
    wave: Wave::Saw,
    detune: 5.0,
    attack: 0.025,
    decay: 0.5,
    sustain: 0.7,
    release: 0.15,
    cutoff: 900.0,
    cutoff_env: 2600.0,
    cutoff_decay: 0.3,
    key_track: 0.5,
    vib_depth: 0.1,
    vib_rate: 5.5,
    vib_delay: 0.35,
    gain: 0.34,
    ..BASE
};

// ---------------------------------------------------------------- mallets & plucks

/// FM bell (inharmonic ratio, long ring).
pub const BELL: Patch = Patch {
    wave: Wave::Fm,
    fm_ratio: 3.5,
    fm_index: 2.4,
    fm_sustain: 0.15,
    fm_decay: 0.8,
    attack: 0.002,
    decay: 3.0,
    sustain: 0.0,
    release: 1.2,
    gain: 0.52,
    ..BASE
};

/// Soft FM music box / celesta for arpeggios.
pub const CELESTA: Patch = Patch {
    wave: Wave::Fm,
    fm_ratio: 4.0,
    fm_index: 1.4,
    fm_sustain: 0.1,
    fm_decay: 0.25,
    attack: 0.002,
    decay: 1.4,
    sustain: 0.0,
    release: 0.5,
    gain: 0.44,
    ..BASE
};

/// Hollow, dark FM bell (underwater).
pub const DEEP_BELL: Patch = Patch {
    wave: Wave::Fm,
    fm_ratio: 2.0,
    fm_index: 1.6,
    fm_sustain: 0.2,
    fm_decay: 0.5,
    attack: 0.004,
    decay: 2.4,
    sustain: 0.0,
    release: 1.0,
    cutoff: 2200.0,
    gain: 0.5,
    ..BASE
};

/// Electric-piano-like FM (ratio 1).
pub const EPIANO: Patch = Patch {
    wave: Wave::Fm,
    fm_ratio: 1.0,
    fm_index: 1.8,
    fm_sustain: 0.25,
    fm_decay: 0.6,
    attack: 0.003,
    decay: 2.2,
    sustain: 0.15,
    release: 0.4,
    gain: 0.36,
    ..BASE
};

/// Harp/lute pluck: saw through a fast-closing filter.
pub const HARP: Patch = Patch {
    wave: Wave::Saw,
    attack: 0.002,
    decay: 1.3,
    sustain: 0.0,
    release: 0.35,
    cutoff: 500.0,
    cutoff_env: 3600.0,
    cutoff_decay: 0.16,
    key_track: 0.8,
    gain: 0.36,
    ..BASE
};

/// Short guitar-ish pulse pluck.
pub const PLUCK: Patch = Patch {
    wave: Wave::Pulse,
    duty: 0.32,
    attack: 0.002,
    decay: 0.45,
    sustain: 0.0,
    release: 0.12,
    cutoff: 700.0,
    cutoff_env: 3000.0,
    cutoff_decay: 0.08,
    key_track: 0.7,
    gain: 0.36,
    ..BASE
};

/// Marimba-like soft sine mallet.
pub const MALLET: Patch = Patch {
    wave: Wave::Fm,
    fm_ratio: 4.0,
    fm_index: 0.9,
    fm_sustain: 0.0,
    fm_decay: 0.06,
    attack: 0.002,
    decay: 0.55,
    sustain: 0.0,
    release: 0.2,
    gain: 0.55,
    ..BASE
};

// ---------------------------------------------------------------- pads

/// Detuned saw string pad.
pub const PAD: Patch = Patch {
    wave: Wave::Saw,
    detune: 10.0,
    attack: 0.6,
    decay: 1.5,
    sustain: 0.85,
    release: 1.0,
    cutoff: 1300.0,
    key_track: 0.3,
    vib_depth: 0.05,
    vib_rate: 4.3,
    gain: 0.22,
    ..BASE
};

/// Warm PWM pad.
pub const WARM_PAD: Patch = Patch {
    wave: Wave::Pulse,
    duty: 0.5,
    pwm_depth: 0.22,
    pwm_rate: 0.35,
    detune: 6.0,
    attack: 0.7,
    decay: 1.5,
    sustain: 0.85,
    release: 1.2,
    cutoff: 1000.0,
    key_track: 0.3,
    gain: 0.24,
    ..BASE
};

/// Glassy FM pad (cold, airy).
pub const GLASS_PAD: Patch = Patch {
    wave: Wave::Fm,
    fm_ratio: 2.0,
    fm_index: 1.2,
    fm_sustain: 0.55,
    fm_decay: 1.5,
    detune: 0.0,
    attack: 0.9,
    decay: 2.0,
    sustain: 0.8,
    release: 1.8,
    vib_depth: 0.06,
    vib_rate: 3.1,
    gain: 0.26,
    ..BASE
};

/// Soft choir-like pad (triangles, slow vibrato).
pub const CHOIR: Patch = Patch {
    wave: Wave::Tri,
    detune: 9.0,
    noise: 0.015,
    attack: 0.5,
    decay: 1.0,
    sustain: 0.9,
    release: 0.9,
    cutoff: 2200.0,
    vib_depth: 0.12,
    vib_rate: 4.6,
    vib_delay: 0.2,
    gain: 0.46,
    ..BASE
};

/// Wobbling pad for the fungal caverns (slow, deep vibrato).
pub const WOBBLE_PAD: Patch = Patch {
    wave: Wave::Pulse,
    duty: 0.4,
    pwm_depth: 0.25,
    pwm_rate: 0.9,
    detune: 14.0,
    attack: 0.4,
    decay: 1.0,
    sustain: 0.85,
    release: 0.8,
    cutoff: 900.0,
    key_track: 0.3,
    vib_depth: 0.28,
    vib_rate: 1.6,
    gain: 0.22,
    ..BASE
};

/// Organ (sine plus a fifth above).
pub const ORGAN: Patch = Patch {
    wave: Wave::Sine,
    detune: 702.0,
    attack: 0.03,
    decay: 0.5,
    sustain: 0.9,
    release: 0.2,
    gain: 0.4,
    ..BASE
};

// ---------------------------------------------------------------- basses

/// Round triangle bass.
pub const BASS: Patch = Patch {
    wave: Wave::Tri,
    attack: 0.004,
    decay: 0.4,
    sustain: 0.75,
    release: 0.07,
    gain: 0.5,
    ..BASE
};

/// Soft plucked saw bass.
pub const SAW_BASS: Patch = Patch {
    wave: Wave::Saw,
    attack: 0.003,
    decay: 0.5,
    sustain: 0.55,
    release: 0.07,
    cutoff: 330.0,
    cutoff_env: 1300.0,
    cutoff_decay: 0.1,
    key_track: 0.6,
    gain: 0.55,
    ..BASE
};

/// Punchy pulse bass for battles.
pub const PULSE_BASS: Patch = Patch {
    wave: Wave::Pulse,
    duty: 0.3,
    attack: 0.002,
    decay: 0.3,
    sustain: 0.6,
    release: 0.05,
    cutoff: 700.0,
    cutoff_env: 1800.0,
    cutoff_decay: 0.09,
    key_track: 0.6,
    gain: 0.44,
    ..BASE
};

/// Heavy, gritty bass for the forge (saw + detuned partner).
pub const HEAVY_BASS: Patch = Patch {
    wave: Wave::Saw,
    detune: -8.0,
    attack: 0.003,
    decay: 0.35,
    sustain: 0.6,
    release: 0.06,
    cutoff: 420.0,
    cutoff_env: 1600.0,
    cutoff_decay: 0.08,
    key_track: 0.5,
    gain: 0.5,
    ..BASE
};

/// Deep sine sub for drones.
pub const SUB: Patch = Patch {
    wave: Wave::Sine,
    attack: 0.4,
    decay: 1.0,
    sustain: 0.9,
    release: 1.5,
    gain: 0.34,
    ..BASE
};

// ---------------------------------------------------------------- drums

const KICK: Patch = Patch {
    wave: Wave::Sine,
    pitch_env: 26.0,
    pitch_decay: 0.028,
    attack: 0.001,
    decay: 0.3,
    sustain: 0.0,
    release: 0.05,
    gain: 1.0,
    ..BASE
};

const SNARE: Patch = Patch {
    wave: Wave::Tri,
    noise: 0.72,
    pitch_env: 7.0,
    pitch_decay: 0.02,
    attack: 0.001,
    decay: 0.2,
    sustain: 0.0,
    release: 0.05,
    cutoff: 7000.0,
    highpass: 180.0,
    gain: 0.75,
    ..BASE
};

const HAT: Patch = Patch {
    wave: Wave::Noise,
    attack: 0.001,
    decay: 0.05,
    sustain: 0.0,
    release: 0.02,
    highpass: 7000.0,
    gain: 0.4,
    ..BASE
};

const OPEN_HAT: Patch = Patch {
    decay: 0.32,
    release: 0.1,
    gain: 0.3,
    ..HAT
};

const CRASH: Patch = Patch {
    wave: Wave::Noise,
    attack: 0.001,
    decay: 1.8,
    sustain: 0.0,
    release: 0.6,
    highpass: 3000.0,
    cutoff: 11000.0,
    gain: 0.42,
    ..BASE
};

const TOM: Patch = Patch {
    wave: Wave::Tri,
    noise: 0.08,
    pitch_env: 7.0,
    pitch_decay: 0.06,
    attack: 0.001,
    decay: 0.4,
    sustain: 0.0,
    release: 0.08,
    cutoff: 2500.0,
    gain: 0.9,
    ..BASE
};

const TIMPANI: Patch = Patch {
    wave: Wave::Sine,
    noise: 0.04,
    pitch_env: 1.5,
    pitch_decay: 0.12,
    attack: 0.002,
    decay: 1.5,
    sustain: 0.0,
    release: 0.4,
    cutoff: 1200.0,
    gain: 1.05,
    ..BASE
};

const ANVIL: Patch = Patch {
    wave: Wave::Fm,
    fm_ratio: 2.76,
    fm_index: 3.4,
    fm_sustain: 0.3,
    fm_decay: 0.12,
    attack: 0.001,
    decay: 0.8,
    sustain: 0.0,
    release: 0.2,
    highpass: 400.0,
    gain: 0.34,
    ..BASE
};

const DRIP: Patch = Patch {
    wave: Wave::Sine,
    pitch_env: -14.0,
    pitch_decay: 0.014,
    attack: 0.001,
    decay: 0.1,
    sustain: 0.0,
    release: 0.03,
    gain: 0.32,
    ..BASE
};

const RIM: Patch = Patch {
    wave: Wave::Pulse,
    noise: 0.5,
    attack: 0.001,
    decay: 0.035,
    sustain: 0.0,
    release: 0.02,
    highpass: 1500.0,
    gain: 0.34,
    ..BASE
};

const SHAKER: Patch = Patch {
    wave: Wave::Noise,
    attack: 0.012,
    decay: 0.07,
    sustain: 0.0,
    release: 0.03,
    highpass: 5000.0,
    gain: 0.22,
    ..BASE
};

const GONG: Patch = Patch {
    wave: Wave::Fm,
    fm_ratio: 1.41,
    fm_index: 2.6,
    fm_sustain: 0.35,
    fm_decay: 1.2,
    attack: 0.004,
    decay: 4.5,
    sustain: 0.0,
    release: 1.5,
    cutoff: 3000.0,
    gain: 0.42,
    ..BASE
};

const SWELL: Patch = Patch {
    wave: Wave::Noise,
    attack: 1.1,
    decay: 0.1,
    sustain: 1.0,
    release: 0.06,
    highpass: 2500.0,
    cutoff: 10000.0,
    gain: 0.22,
    ..BASE
};

const CHIME: Patch = Patch {
    wave: Wave::Fm,
    fm_ratio: 3.5,
    fm_index: 1.5,
    fm_sustain: 0.0,
    fm_decay: 0.2,
    attack: 0.001,
    decay: 0.7,
    sustain: 0.0,
    release: 0.3,
    gain: 0.16,
    ..BASE
};

const TAMBOURINE: Patch = Patch {
    wave: Wave::Metal,
    attack: 0.002,
    decay: 0.13,
    sustain: 0.0,
    release: 0.04,
    highpass: 5000.0,
    gain: 0.16,
    ..BASE
};

const CLAP: Patch = Patch {
    wave: Wave::Noise,
    attack: 0.002,
    decay: 0.09,
    sustain: 0.0,
    release: 0.03,
    highpass: 1100.0,
    cutoff: 5000.0,
    gain: 0.5,
    ..BASE
};

fn hit(patch: Patch, note: f32, vel: f32, pan: f32, len: f32) -> Option<DrumHit> {
    Some(DrumHit {
        patch,
        note,
        vel,
        pan,
        len,
    })
}

/// The drum kit: one character per sound (upper case = accented).
///
/// `k` kick, `s` snare, `h` closed hat, `o` open hat, `c` crash, `t m u` toms
/// (low, mid, high), `p` timpani (`P` low), `a` anvil (`A` low), `d` drip (`D`
/// high drip), `x` rim, `n` shaker, `g` gong, `w` cymbal swell, `b` chime,
/// `j` tambourine, `f` clap.
pub fn kit(c: char) -> Option<DrumHit> {
    match c {
        'k' => hit(KICK, 33.0, 0.9, 0.0, 0.05),
        'K' => hit(KICK, 33.0, 1.15, 0.0, 0.05),
        's' => hit(SNARE, 55.0, 0.75, 0.05, 0.05),
        'S' => hit(SNARE, 55.0, 1.0, 0.05, 0.05),
        'h' => hit(HAT, 110.0, 0.55, 0.3, 0.02),
        'H' => hit(HAT, 110.0, 0.85, 0.3, 0.02),
        'o' => hit(OPEN_HAT, 110.0, 0.7, 0.3, 0.2),
        'c' => hit(CRASH, 110.0, 0.9, -0.35, 0.3),
        'C' => hit(CRASH, 110.0, 1.1, 0.35, 0.3),
        't' => hit(TOM, 43.0, 0.85, -0.3, 0.1),
        'm' => hit(TOM, 48.0, 0.8, 0.0, 0.1),
        'u' => hit(TOM, 53.0, 0.75, 0.3, 0.1),
        'p' => hit(TIMPANI, 38.0, 0.9, -0.1, 0.4),
        'P' => hit(TIMPANI, 33.0, 1.0, -0.1, 0.4),
        'a' => hit(ANVIL, 86.0, 0.8, 0.3, 0.1),
        'A' => hit(ANVIL, 79.0, 0.95, -0.25, 0.1),
        'd' => hit(DRIP, 88.0, 0.8, -0.45, 0.05),
        'D' => hit(DRIP, 95.0, 0.7, 0.5, 0.05),
        'x' => hit(RIM, 84.0, 0.7, -0.15, 0.02),
        'n' => hit(SHAKER, 110.0, 0.7, -0.35, 0.03),
        'g' => hit(GONG, 40.0, 0.9, 0.0, 1.0),
        'w' => hit(SWELL, 110.0, 0.8, 0.0, 1.1),
        'b' => hit(CHIME, 96.0, 0.8, 0.4, 0.1),
        'j' => hit(TAMBOURINE, 100.0, 0.8, 0.35, 0.05),
        'f' => hit(CLAP, 100.0, 0.8, -0.1, 0.03),
        _ => None,
    }
}
