//! Sound effects: each `Sfx` is a short score of voice triggers (pitch
//! sweeps, noise bursts, arpeggios) played on a dedicated voice pool.

use super::Sfx;
use super::patches;
use super::synth::{BASE, Echo, EchoParams, NoteOn, Patch, VoicePool, Wave};

const VOICES: usize = 28;
const QUEUE: usize = 192;
const KINDS: usize = 64;

#[derive(Clone, Copy)]
struct Pending {
    at: u64,
    patch: Patch,
    on: NoteOn,
}

pub struct SfxPlayer {
    rate: f32,
    voices: VoicePool,
    queue: Vec<Pending>,
    echo: Echo,
    send_l: Vec<f32>,
    send_r: Vec<f32>,
    clock: u64,
    last: [u64; KINDS],
    counter: u32,
}

impl SfxPlayer {
    pub fn new(rate: f32, max_block: usize) -> SfxPlayer {
        let mut echo = Echo::new(rate, 0.5);
        echo.set(
            &EchoParams {
                time_l: 0.16,
                time_r: 0.23,
                feedback: 0.35,
                cross: 0.4,
                damp_hz: 3500.0,
                level: 0.5,
                diffuse: true,
            },
            rate,
        );
        SfxPlayer {
            rate,
            voices: VoicePool::new(VOICES),
            queue: Vec::with_capacity(QUEUE),
            echo,
            send_l: vec![0.0; max_block],
            send_r: vec![0.0; max_block],
            clock: 0,
            last: [0; KINDS],
            counter: 0,
        }
    }

    pub fn play(&mut self, sfx: Sfx) {
        let k = sfx as usize % KINDS;
        // Ignore rapid repeats of the same effect (stacked copies only get louder).
        let gap = match sfx {
            Sfx::TextBlip => 0.03,
            Sfx::Step => 0.06,
            _ => 0.045,
        };
        let gap = (gap * self.rate) as u64;
        if self.last[k] != 0 && self.clock < self.last[k] + gap {
            return;
        }
        self.last[k] = self.clock.max(1);
        self.counter = self.counter.wrapping_add(1);
        let mut w = Writer {
            q: &mut self.queue,
            now: self.clock,
            rate: self.rate,
            counter: self.counter,
        };
        define(sfx, &mut w);
    }

    pub fn render(&mut self, out_l: &mut [f32], out_r: &mut [f32]) {
        let n = out_l.len().min(out_r.len()).min(self.send_l.len());
        self.send_l[..n].fill(0.0);
        self.send_r[..n].fill(0.0);
        let end = self.clock + n as u64;
        let mut start = 0usize;
        // Render up to each due trigger, start it, continue.
        loop {
            let mut next_at = end;
            for p in self.queue.iter() {
                if p.at < next_at {
                    next_at = p.at;
                }
            }
            let stop = (next_at.max(self.clock + start as u64) - self.clock) as usize;
            let stop = stop.min(n);
            if stop > start {
                self.voices.render(
                    self.rate,
                    &mut out_l[start..stop],
                    &mut out_r[start..stop],
                    &mut self.send_l[start..stop],
                    &mut self.send_r[start..stop],
                );
                start = stop;
            }
            if start >= n {
                break;
            }
            let now = self.clock + start as u64;
            let mut i = 0;
            while i < self.queue.len() {
                if self.queue[i].at <= now {
                    let p = self.queue.swap_remove(i);
                    self.voices.start(&p.patch, &p.on, self.rate);
                } else {
                    i += 1;
                }
            }
        }
        self.echo.process(
            &self.send_l[..n],
            &self.send_r[..n],
            &mut out_l[..n],
            &mut out_r[..n],
        );
        self.clock = end;
    }
}

struct Writer<'a> {
    q: &'a mut Vec<Pending>,
    now: u64,
    rate: f32,
    counter: u32,
}

impl Writer<'_> {
    /// Schedules a note `t` seconds from now, lasting `dur` seconds before release.
    #[allow(clippy::too_many_arguments)]
    fn note(&mut self, t: f32, p: &Patch, note: f32, dur: f32, vel: f32, pan: f32, send: f32) {
        if self.q.len() >= self.q.capacity().min(QUEUE) {
            return;
        }
        self.q.push(Pending {
            at: self.now + (t.max(0.0) * self.rate) as u64,
            patch: *p,
            on: NoteOn {
                note,
                vel,
                hold: (dur * self.rate).max(1.0) as u32,
                pan,
                send,
            },
        });
    }
    fn n(&mut self, t: f32, p: &Patch, note: f32, dur: f32, vel: f32) {
        self.note(t, p, note, dur, vel, 0.0, 0.0);
    }
}

// ---------------------------------------------------------------- effect patches

const BLIP: Patch = Patch {
    wave: Wave::Pulse,
    duty: 0.25,
    attack: 0.001,
    decay: 0.09,
    sustain: 0.0,
    release: 0.02,
    cutoff: 6000.0,
    gain: 0.5,
    ..BASE
};

const SQUARE_BLIP: Patch = Patch {
    duty: 0.5,
    decay: 0.16,
    ..BLIP
};

const SOFT_BLIP: Patch = Patch {
    wave: Wave::Pulse,
    duty: 0.5,
    attack: 0.002,
    decay: 0.03,
    sustain: 0.0,
    release: 0.01,
    cutoff: 2600.0,
    gain: 0.5,
    ..BASE
};

const THUMP: Patch = Patch {
    wave: Wave::Tri,
    pitch_env: 12.0,
    pitch_decay: 0.02,
    attack: 0.001,
    decay: 0.08,
    sustain: 0.0,
    release: 0.02,
    gain: 0.9,
    ..BASE
};

const SCUFF: Patch = Patch {
    wave: Wave::Noise,
    attack: 0.002,
    decay: 0.05,
    sustain: 0.0,
    release: 0.02,
    cutoff: 900.0,
    highpass: 150.0,
    gain: 0.8,
    ..BASE
};

const CREAK: Patch = Patch {
    wave: Wave::Pulse,
    duty: 0.07,
    attack: 0.04,
    decay: 0.6,
    sustain: 0.7,
    release: 0.08,
    cutoff: 1800.0,
    highpass: 250.0,
    vib_depth: 2.5,
    vib_rate: 9.0,
    sweep: 9.0,
    gain: 0.4,
    ..BASE
};

const COIN: Patch = Patch {
    wave: Wave::Pulse,
    duty: 0.5,
    attack: 0.001,
    decay: 0.45,
    sustain: 0.0,
    release: 0.05,
    cutoff: 7000.0,
    gain: 0.34,
    ..BASE
};

const SPARK: Patch = Patch {
    wave: Wave::Fm,
    fm_ratio: 3.5,
    fm_index: 1.6,
    fm_decay: 0.15,
    attack: 0.001,
    decay: 0.7,
    sustain: 0.0,
    release: 0.3,
    gain: 0.34,
    ..BASE
};

/// Sparkle that dies away quickly (combat effects).
const SPARK_SHORT: Patch = Patch {
    decay: 0.35,
    release: 0.12,
    ..SPARK
};

const WHOOSH: Patch = Patch {
    wave: Wave::Noise,
    attack: 0.05,
    decay: 0.3,
    sustain: 0.0,
    release: 0.05,
    cutoff: 500.0,
    cutoff_env: 6000.0,
    cutoff_decay: 0.3,
    highpass: 200.0,
    gain: 0.6,
    ..BASE
};

const SWISH: Patch = Patch {
    wave: Wave::Noise,
    attack: 0.03,
    decay: 0.14,
    sustain: 0.0,
    release: 0.04,
    highpass: 900.0,
    cutoff: 5000.0,
    sweep: 70.0,
    gain: 0.5,
    ..BASE
};

const ZOOM: Patch = Patch {
    wave: Wave::Saw,
    attack: 0.01,
    decay: 0.4,
    sustain: 0.0,
    release: 0.05,
    cutoff: 3000.0,
    sweep: -70.0,
    gain: 0.25,
    ..BASE
};

const IMPACT: Patch = Patch {
    wave: Wave::Noise,
    attack: 0.001,
    decay: 0.09,
    sustain: 0.0,
    release: 0.03,
    highpass: 700.0,
    cutoff: 6000.0,
    gain: 0.7,
    ..BASE
};

const BOOM: Patch = Patch {
    wave: Wave::Sine,
    pitch_env: 24.0,
    pitch_decay: 0.04,
    attack: 0.001,
    decay: 0.45,
    sustain: 0.0,
    release: 0.1,
    gain: 1.0,
    ..BASE
};

const METAL: Patch = Patch {
    wave: Wave::Fm,
    fm_ratio: 2.76,
    fm_index: 3.2,
    fm_sustain: 0.25,
    fm_decay: 0.1,
    attack: 0.001,
    decay: 0.5,
    sustain: 0.0,
    release: 0.1,
    highpass: 500.0,
    gain: 0.34,
    ..BASE
};

const SHING: Patch = Patch {
    wave: Wave::Metal,
    attack: 0.001,
    decay: 0.3,
    sustain: 0.0,
    release: 0.05,
    highpass: 3000.0,
    gain: 0.2,
    ..BASE
};

const DROOP: Patch = Patch {
    wave: Wave::Pulse,
    duty: 0.5,
    attack: 0.005,
    decay: 0.5,
    sustain: 0.3,
    release: 0.1,
    cutoff: 3000.0,
    sweep: -50.0,
    vib_depth: 0.6,
    vib_rate: 11.0,
    gain: 0.36,
    ..BASE
};

const CRUMBLE: Patch = Patch {
    wave: Wave::Noise,
    attack: 0.005,
    decay: 0.4,
    sustain: 0.0,
    release: 0.1,
    cutoff: 1500.0,
    sweep: -30.0,
    gain: 0.45,
    ..BASE
};

const RISE: Patch = Patch {
    wave: Wave::Sine,
    attack: 0.03,
    decay: 0.5,
    sustain: 0.0,
    release: 0.1,
    sweep: 60.0,
    gain: 0.35,
    ..BASE
};

const ROAR_NOISE: Patch = Patch {
    wave: Wave::Noise,
    attack: 0.08,
    decay: 0.6,
    sustain: 0.3,
    release: 0.2,
    cutoff: 700.0,
    cutoff_env: 2500.0,
    cutoff_decay: 0.25,
    sweep: -12.0,
    gain: 0.55,
    ..BASE
};

const CRACKLE: Patch = Patch {
    wave: Wave::Noise,
    attack: 0.001,
    decay: 0.025,
    sustain: 0.0,
    release: 0.01,
    highpass: 2000.0,
    gain: 0.55,
    ..BASE
};

const ZAP: Patch = Patch {
    wave: Wave::Metal,
    attack: 0.002,
    decay: 0.3,
    sustain: 0.2,
    release: 0.05,
    vib_depth: 14.0,
    vib_rate: 33.0,
    sweep: -30.0,
    highpass: 300.0,
    gain: 0.32,
    ..BASE
};

const BUZZ: Patch = Patch {
    wave: Wave::Saw,
    attack: 0.001,
    decay: 0.25,
    sustain: 0.3,
    release: 0.04,
    cutoff: 4000.0,
    vib_depth: 5.0,
    vib_rate: 45.0,
    gain: 0.2,
    ..BASE
};

const DARK: Patch = Patch {
    wave: Wave::Saw,
    detune: 16.0,
    attack: 0.25,
    decay: 0.6,
    sustain: 0.6,
    release: 0.35,
    cutoff: 500.0,
    cutoff_env: 500.0,
    cutoff_decay: 0.4,
    sweep: -3.0,
    gain: 0.4,
    ..BASE
};

const BUBBLE: Patch = Patch {
    wave: Wave::Sine,
    pitch_env: -9.0,
    pitch_decay: 0.03,
    attack: 0.002,
    decay: 0.1,
    sustain: 0.0,
    release: 0.02,
    gain: 0.45,
    ..BASE
};

const HOLY: Patch = Patch {
    attack: 0.08,
    release: 0.5,
    gain: 0.3,
    ..patches::CHOIR
};

const GROWL: Patch = Patch {
    wave: Wave::Saw,
    detune: 30.0,
    attack: 0.08,
    decay: 0.9,
    sustain: 0.5,
    release: 0.3,
    cutoff: 900.0,
    cutoff_env: 600.0,
    cutoff_decay: 0.5,
    vib_depth: 1.2,
    vib_rate: 27.0,
    sweep: -5.0,
    gain: 0.45,
    ..BASE
};

const BLAST: Patch = Patch {
    wave: Wave::Noise,
    attack: 0.002,
    decay: 1.0,
    sustain: 0.0,
    release: 0.2,
    cutoff: 400.0,
    cutoff_env: 7000.0,
    cutoff_decay: 0.25,
    gain: 0.75,
    ..BASE
};

// ---------------------------------------------------------------- the effects

fn define(sfx: Sfx, w: &mut Writer) {
    let bell = &patches::BELL;
    let lead = &patches::LEAD;
    match sfx {
        Sfx::MenuMove => {
            w.n(0.0, &BLIP, 86.0, 0.02, 0.4);
        }
        Sfx::MenuSelect => {
            w.n(0.0, &SQUARE_BLIP, 81.0, 0.04, 0.4);
            w.n(0.055, &SQUARE_BLIP, 88.0, 0.08, 0.45);
            w.note(0.055, &SPARK, 100.0, 0.05, 0.1, 0.2, 0.0);
        }
        Sfx::MenuBack => {
            w.n(0.0, &SQUARE_BLIP, 83.0, 0.04, 0.35);
            w.n(0.05, &SQUARE_BLIP, 76.0, 0.06, 0.32);
        }
        Sfx::Denied => {
            let p = Patch {
                wave: Wave::Saw,
                cutoff: 1800.0,
                decay: 0.12,
                sustain: 0.5,
                ..BLIP
            };
            w.n(0.0, &p, 45.0, 0.07, 0.5);
            w.n(0.0, &p, 46.0, 0.07, 0.35);
            w.n(0.11, &p, 45.0, 0.1, 0.5);
            w.n(0.11, &p, 46.0, 0.1, 0.35);
        }
        Sfx::TextBlip => {
            let wobble = [0.0, 2.0, -1.0, 3.0, 1.0][(w.counter % 5) as usize];
            w.n(0.0, &SOFT_BLIP, 79.0 + wobble, 0.012, 0.13);
        }
        Sfx::Step => {
            let pan = if w.counter.is_multiple_of(2) {
                -0.25
            } else {
                0.25
            };
            w.note(0.0, &SCUFF, 90.0, 0.02, 0.1, pan, 0.0);
            w.note(0.0, &THUMP, 40.0, 0.02, 0.1, pan, 0.0);
        }
        Sfx::DoorOpen => {
            w.note(0.0, &CREAK, 46.0, 0.42, 0.55, -0.2, 0.2);
            w.note(0.05, &CREAK, 53.0, 0.3, 0.2, 0.2, 0.2);
            w.note(0.46, &THUMP, 38.0, 0.05, 0.6, 0.0, 0.3);
            w.note(0.46, &SCUFF, 80.0, 0.03, 0.35, 0.0, 0.2);
        }
        Sfx::ChestOpen => {
            w.note(0.0, &CREAK, 50.0, 0.18, 0.45, 0.0, 0.1);
            w.n(0.2, &THUMP, 43.0, 0.03, 0.4);
            for (i, n) in [84.0, 88.0, 91.0, 96.0].iter().enumerate() {
                w.note(
                    0.22 + i as f32 * 0.045,
                    &SPARK,
                    *n,
                    0.1,
                    0.3,
                    -0.2 + i as f32 * 0.15,
                    0.4,
                );
            }
        }
        Sfx::Coin => {
            w.n(0.0, &COIN, 83.0, 0.06, 0.45);
            w.n(0.065, &COIN, 88.0, 0.3, 0.45);
            w.note(0.065, &SPARK, 100.0, 0.05, 0.12, 0.3, 0.2);
        }
        Sfx::ItemGet => {
            for (i, n) in [72.0, 76.0, 79.0].iter().enumerate() {
                w.note(i as f32 * 0.06, lead, *n, 0.05, 0.5, 0.0, 0.2);
            }
            w.note(0.18, lead, 84.0, 0.32, 0.55, 0.0, 0.3);
            w.note(0.18, &SPARK, 84.0, 0.2, 0.3, -0.2, 0.25);
            w.note(0.18, &SPARK, 91.0, 0.2, 0.18, 0.2, 0.25);
        }
        Sfx::Encounter => {
            w.note(0.0, &WHOOSH, 105.0, 0.25, 0.8, -0.3, 0.2);
            w.note(0.0, &ZOOM, 88.0, 0.3, 0.5, 0.3, 0.2);
            let stab = Patch {
                cutoff_env: 4000.0,
                ..patches::BRASS
            };
            for n in [48.0, 54.0, 59.0, 63.0] {
                w.note(0.22, &stab, n, 0.28, 0.55, 0.0, 0.3);
            }
            w.note(0.22, &BOOM, 31.0, 0.1, 0.7, 0.0, 0.0);
            w.note(0.22, &IMPACT, 100.0, 0.05, 0.4, 0.0, 0.2);
        }
        Sfx::Stairs => {
            for i in 0..4 {
                let t = i as f32 * 0.1;
                let pan = if i % 2 == 0 { -0.2 } else { 0.2 };
                w.note(
                    t,
                    &THUMP,
                    45.0 - i as f32 * 2.0,
                    0.03,
                    0.45 - i as f32 * 0.06,
                    pan,
                    0.15,
                );
                w.note(t, &SCUFF, 85.0, 0.02, 0.25, pan, 0.0);
            }
        }
        Sfx::Waystone => {
            let notes = [84.0, 86.0, 88.0, 91.0, 93.0, 96.0, 98.0, 100.0];
            for (i, n) in notes.iter().enumerate() {
                let pan = ((i as f32) * 0.9).sin() * 0.6;
                w.note(i as f32 * 0.04, &SPARK, *n, 0.1, 0.28, pan, 0.6);
            }
            for n in [60.0, 67.0, 72.0, 76.0] {
                w.note(0.0, &patches::GLASS_PAD, n, 0.6, 0.4, 0.0, 0.5);
            }
        }
        Sfx::Save => {
            w.note(0.0, bell, 79.0, 0.3, 0.3, -0.2, 0.4);
            w.note(0.0, bell, 84.0, 0.3, 0.25, 0.2, 0.4);
            w.note(0.14, bell, 88.0, 0.4, 0.3, 0.0, 0.4);
        }
        Sfx::Hit => {
            w.n(0.0, &IMPACT, 100.0, 0.04, 0.7);
            w.n(0.0, &THUMP, 45.0, 0.05, 0.8);
            w.note(0.0, &SWISH, 90.0, 0.03, 0.3, 0.2, 0.0);
        }
        Sfx::CritHit => {
            w.note(0.0, &SWISH, 92.0, 0.05, 0.5, -0.3, 0.0);
            w.n(0.04, &IMPACT, 100.0, 0.06, 0.75);
            w.n(0.04, &BOOM, 36.0, 0.1, 0.65);
            w.note(0.04, &SHING, 96.0, 0.1, 0.6, 0.3, 0.4);
            w.note(0.04, &METAL, 88.0, 0.1, 0.35, -0.2, 0.3);
            w.n(0.12, &IMPACT, 96.0, 0.04, 0.45);
        }
        Sfx::Miss => {
            w.note(0.0, &SWISH, 84.0, 0.08, 0.5, -0.2, 0.0);
            w.note(0.07, &SWISH, 90.0, 0.06, 0.25, 0.3, 0.0);
        }
        Sfx::Block => {
            w.note(0.0, &METAL, 81.0, 0.1, 0.7, 0.0, 0.25);
            w.n(0.0, &IMPACT, 96.0, 0.03, 0.45);
            w.n(0.0, &THUMP, 50.0, 0.03, 0.4);
        }
        Sfx::EnemyDie => {
            w.note(0.0, &DROOP, 74.0, 0.28, 0.5, 0.0, 0.15);
            w.note(0.02, &CRUMBLE, 96.0, 0.25, 0.55, 0.0, 0.15);
            w.note(0.08, &DROOP, 67.0, 0.22, 0.25, 0.2, 0.15);
        }
        Sfx::PartyDown => {
            let p = Patch {
                decay: 0.4,
                sustain: 0.5,
                ..patches::SQUARE
            };
            w.note(0.0, &p, 76.0, 0.12, 0.5, 0.0, 0.3);
            w.note(0.14, &p, 72.0, 0.12, 0.5, 0.0, 0.3);
            let last = Patch { sweep: -2.0, ..p };
            w.note(0.28, &last, 69.0, 0.3, 0.55, 0.0, 0.2);
            w.n(0.28, &THUMP, 36.0, 0.05, 0.6);
        }
        Sfx::Heal => {
            w.note(0.0, &RISE, 72.0, 0.3, 0.35, 0.0, 0.2);
            for (i, n) in [84.0, 88.0, 91.0, 96.0, 100.0].iter().enumerate() {
                let pan = if i % 2 == 0 { -0.3 } else { 0.3 };
                w.note(
                    0.04 + i as f32 * 0.05,
                    &SPARK_SHORT,
                    *n,
                    0.08,
                    0.3,
                    pan,
                    0.3,
                );
            }
        }
        Sfx::Fire => {
            let roar = Patch {
                sweep: 20.0,
                ..WHOOSH
            };
            w.note(0.0, &roar, 88.0, 0.3, 0.75, 0.0, 0.2);
            let pops = [0.03, 0.09, 0.13, 0.2, 0.26, 0.31, 0.38, 0.44];
            for (i, t) in pops.iter().enumerate() {
                let pan = ((i as f32) * 2.3).sin() * 0.6;
                w.note(*t, &CRACKLE, 100.0, 0.01, 0.5 - i as f32 * 0.04, pan, 0.1);
            }
            w.n(0.0, &BOOM, 40.0, 0.1, 0.35);
        }
        Sfx::Frost => {
            let hiss = Patch {
                cutoff: 12000.0,
                highpass: 5000.0,
                cutoff_env: 0.0,
                decay: 0.5,
                ..WHOOSH
            };
            w.note(0.0, &hiss, 105.0, 0.3, 0.45, 0.0, 0.3);
            let glass = Patch {
                fm_ratio: 5.19,
                decay: 0.45,
                release: 0.15,
                ..SPARK
            };
            let notes = [103.0, 98.0, 101.0, 94.0, 97.0, 90.0];
            for (i, n) in notes.iter().enumerate() {
                let pan = if i % 2 == 0 { -0.4 } else { 0.4 };
                w.note(i as f32 * 0.035, &glass, *n, 0.05, 0.28, pan, 0.35);
            }
        }
        Sfx::Shock => {
            w.note(0.0, &ZAP, 76.0, 0.3, 0.7, -0.2, 0.2);
            w.note(0.0, &BUZZ, 57.0, 0.3, 0.6, 0.2, 0.1);
            w.note(0.05, &CRACKLE, 100.0, 0.01, 0.6, 0.3, 0.0);
            w.note(0.17, &CRACKLE, 100.0, 0.01, 0.5, -0.3, 0.0);
            w.n(0.0, &IMPACT, 100.0, 0.03, 0.4);
        }
        Sfx::Light => {
            for (i, n) in [84.0, 88.0, 91.0, 96.0].iter().enumerate() {
                w.note(
                    i as f32 * 0.025,
                    &SPARK,
                    *n,
                    0.15,
                    0.3,
                    -0.3 + i as f32 * 0.2,
                    0.4,
                );
            }
            for n in [72.0, 76.0, 79.0] {
                w.note(0.0, &HOLY, n, 0.32, 0.35, 0.0, 0.3);
            }
        }
        Sfx::Shadow => {
            for n in [43.0, 48.0, 49.0] {
                w.note(0.0, &DARK, n, 0.38, 0.45, 0.0, 0.2);
            }
            let swell = Patch {
                attack: 0.4,
                decay: 0.2,
                cutoff: 1200.0,
                highpass: 200.0,
                ..BLAST
            };
            w.note(0.0, &swell, 80.0, 0.4, 0.4, 0.0, 0.4);
            w.note(0.42, &BOOM, 30.0, 0.1, 0.45, 0.0, 0.2);
        }
        Sfx::Poison => {
            let notes = [62.0, 69.0, 65.0, 72.0, 67.0, 74.0];
            let times = [0.0, 0.06, 0.13, 0.18, 0.27, 0.33];
            for i in 0..6 {
                let pan = if i % 2 == 0 { -0.35 } else { 0.35 };
                w.note(times[i], &BUBBLE, notes[i], 0.04, 0.5, pan, 0.2);
            }
            let hiss = Patch {
                cutoff: 2500.0,
                cutoff_env: 0.0,
                highpass: 800.0,
                decay: 0.4,
                ..WHOOSH
            };
            w.note(0.0, &hiss, 90.0, 0.2, 0.15, 0.0, 0.0);
        }
        Sfx::Buff => {
            let sweep = Patch {
                wave: Wave::Pulse,
                duty: 0.3,
                pwm_depth: 0.15,
                pwm_rate: 12.0,
                sweep: 65.0,
                decay: 0.4,
                sustain: 0.0,
                cutoff: 5000.0,
                ..RISE
            };
            w.note(0.0, &sweep, 60.0, 0.28, 0.35, 0.0, 0.1);
            for (i, n) in [72.0, 76.0, 79.0, 84.0].iter().enumerate() {
                w.note(0.05 + i as f32 * 0.06, lead, *n, 0.05, 0.35, 0.0, 0.15);
            }
        }
        Sfx::Debuff => {
            let sweep = Patch {
                wave: Wave::Pulse,
                duty: 0.3,
                sweep: -55.0,
                decay: 0.4,
                sustain: 0.0,
                cutoff: 3000.0,
                ..RISE
            };
            w.note(0.0, &sweep, 79.0, 0.28, 0.35, 0.0, 0.1);
            for (i, n) in [84.0, 81.0, 78.0, 75.0].iter().enumerate() {
                w.note(
                    0.05 + i as f32 * 0.06,
                    &patches::SQUARE,
                    *n,
                    0.05,
                    0.35,
                    0.0,
                    0.15,
                );
            }
        }
        Sfx::LevelUp => {
            let horn = Patch {
                cutoff_env: 3500.0,
                ..patches::BRASS
            };
            let seq = [(0.0, 67.0), (0.09, 72.0), (0.18, 76.0), (0.27, 79.0)];
            for (t, n) in seq {
                w.note(t, lead, n, 0.07, 0.5, 0.0, 0.2);
            }
            w.note(0.38, lead, 84.0, 0.5, 0.6, 0.0, 0.3);
            for n in [64.0, 67.0, 72.0] {
                w.note(0.38, &horn, n, 0.5, 0.45, 0.0, 0.2);
            }
            w.note(0.38, bell, 96.0, 0.3, 0.25, 0.3, 0.5);
            w.note(0.5, bell, 91.0, 0.3, 0.2, -0.3, 0.5);
            w.n(0.38, &BOOM, 36.0, 0.1, 0.4);
        }
        Sfx::Flee => {
            for i in 0..5 {
                let t = i as f32 * 0.055;
                let pan = -0.6 + i as f32 * 0.3;
                w.note(t, &SCUFF, 92.0, 0.02, 0.4 - i as f32 * 0.05, pan, 0.0);
                w.note(t, &THUMP, 48.0, 0.02, 0.3 - i as f32 * 0.04, pan, 0.0);
            }
            w.note(0.05, &SWISH, 80.0, 0.2, 0.4, 0.4, 0.2);
        }
        Sfx::Shard => {
            // The first notes of Lira's Lullaby, far away.
            let seq = [(0.0, 90.0), (0.3, 93.0), (0.45, 98.0), (0.75, 97.0)];
            for (i, (t, n)) in seq.iter().enumerate() {
                let pan = if i % 2 == 0 { -0.3 } else { 0.3 };
                w.note(*t, bell, *n, 0.4, 0.32, pan, 0.7);
            }
            for n in [62.0, 69.0, 74.0, 78.0] {
                w.note(0.0, &patches::GLASS_PAD, n, 0.9, 0.35, 0.0, 0.6);
            }
            w.note(0.0, &RISE, 86.0, 0.3, 0.12, 0.0, 0.5);
        }
        Sfx::BossRoar => {
            for n in [31.0, 32.0, 38.0] {
                w.note(0.0, &GROWL, n, 0.75, 0.5, 0.0, 0.2);
            }
            w.note(0.0, &ROAR_NOISE, 76.0, 0.7, 0.6, 0.0, 0.2);
            w.n(0.0, &BOOM, 28.0, 0.2, 0.6);
        }
        Sfx::Explosion => {
            w.note(0.0, &BLAST, 76.0, 0.3, 0.75, 0.0, 0.3);
            w.n(0.0, &BOOM, 28.0, 0.2, 0.75);
            let pops = [0.12, 0.2, 0.27, 0.36, 0.5];
            for (i, t) in pops.iter().enumerate() {
                let pan = ((i as f32) * 1.7).sin() * 0.7;
                w.note(*t, &CRACKLE, 100.0, 0.01, 0.35, pan, 0.2);
            }
        }
    }
}
