//! Low-level DSP for Emberdeep's built-in synthesizer: oscillators, the voice
//! model (ADSR, filter, vibrato, pitch envelopes), a voice pool, the stereo
//! echo and the master limiter.
//!
//! Everything here is plain arithmetic on preallocated buffers: no allocation
//! or locking happens while rendering, and the hot loops avoid per-sample
//! function calls so that unoptimised debug builds keep up in real time.

/// Samples between control-rate updates (pitch, filter, modulation).
const CTRL: u32 = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wave {
    /// Band-limited pulse; `duty` sets the width (0.5 = square).
    Pulse,
    /// Band-limited sawtooth.
    Saw,
    /// Triangle.
    Tri,
    /// Sine.
    Sine,
    /// Two-operator FM (sine carrier and modulator): bells, e-pianos, metal.
    Fm,
    /// White noise, sample-and-held at the note frequency (higher = brighter).
    Noise,
    /// Short-loop LFSR noise (NES "metallic" mode), pitched by the note.
    Metal,
}

/// A voice preset. All times are in seconds, frequencies in Hz, pitches in
/// semitones.
#[derive(Clone, Copy, Debug)]
pub struct Patch {
    pub wave: Wave,
    /// Pulse width 0..1 (only for `Wave::Pulse`).
    pub duty: f32,
    /// Pulse-width modulation depth and rate.
    pub pwm_depth: f32,
    pub pwm_rate: f32,
    /// FM: modulator frequency ratio, index at note start, fraction of the
    /// index that remains after `fm_decay`.
    pub fm_ratio: f32,
    pub fm_index: f32,
    pub fm_sustain: f32,
    pub fm_decay: f32,
    /// Detune of a second, identical oscillator in cents (0 = single oscillator).
    pub detune: f32,
    /// Amount of white noise mixed into the oscillator (breath, snare rattle).
    pub noise: f32,
    pub attack: f32,
    /// Time for the decay stage to (nearly) reach the sustain level.
    pub decay: f32,
    pub sustain: f32,
    /// Time for the release to fade to silence.
    pub release: f32,
    /// Low-pass cutoff (two cascaded one-pole stages); >= 20000 bypasses it.
    pub cutoff: f32,
    /// Extra cutoff at note start (scaled by velocity), decaying with `cutoff_decay`.
    pub cutoff_env: f32,
    pub cutoff_decay: f32,
    /// 0..1: how much the cutoff follows the note pitch (1 = fully).
    pub key_track: f32,
    /// One-pole high-pass cutoff; 0 disables it.
    pub highpass: f32,
    pub vib_depth: f32,
    pub vib_rate: f32,
    pub vib_delay: f32,
    /// Pitch offset at note start, decaying exponentially with `pitch_decay`.
    pub pitch_env: f32,
    pub pitch_decay: f32,
    /// Linear pitch glide in semitones per second.
    pub sweep: f32,
    pub gain: f32,
}

/// Neutral starting point for presets (`Patch { wave: Wave::Saw, ..BASE }`).
pub const BASE: Patch = Patch {
    wave: Wave::Pulse,
    duty: 0.5,
    pwm_depth: 0.0,
    pwm_rate: 0.0,
    fm_ratio: 1.0,
    fm_index: 0.0,
    fm_sustain: 0.0,
    fm_decay: 1.0,
    detune: 0.0,
    noise: 0.0,
    attack: 0.005,
    decay: 0.3,
    sustain: 0.7,
    release: 0.12,
    cutoff: 20000.0,
    cutoff_env: 0.0,
    cutoff_decay: 0.2,
    key_track: 0.0,
    highpass: 0.0,
    vib_depth: 0.0,
    vib_rate: 5.0,
    vib_delay: 0.0,
    pitch_env: 0.0,
    pitch_decay: 0.05,
    sweep: 0.0,
    gain: 1.0,
};

/// Sine of `phase * 2π` for `phase` in [0, 1); max error about 0.001.
#[inline]
pub fn sin01(phase: f32) -> f32 {
    let x = phase * 2.0 - 1.0;
    let y = 4.0 * x * (1.0 - x.abs());
    -(0.225 * (y * y.abs() - y) + y)
}

/// MIDI note number to frequency.
#[inline]
pub fn note_hz(note: f32) -> f32 {
    440.0 * ((note - 69.0) * (1.0 / 12.0)).exp2()
}

/// Soft saturation that leaves |x| < 0.9 untouched and never exceeds 1.0.
#[inline]
pub fn soft_clip(x: f32) -> f32 {
    let a = x.abs();
    if a <= 0.9 {
        x
    } else {
        let over = (a - 0.9) * 10.0;
        let y = 0.9 + 0.1 * (over / (1.0 + over));
        y.copysign(x)
    }
}

/// Coefficient for an exponential approach that covers ~60 dB in `time` seconds.
fn exp_coef(time: f32, rate: f32) -> f32 {
    if time <= 0.0 {
        0.0
    } else {
        (-6.9 / (time * rate)).exp()
    }
}

/// One-pole coefficient for cutoff `hz` at `rate` samples per second.
fn onepole(hz: f32, rate: f32) -> f32 {
    if hz >= 0.45 * rate {
        1.0
    } else {
        1.0 - (-std::f32::consts::TAU * hz.max(1.0) / rate).exp()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Stage {
    Off,
    Attack,
    Decay,
    Release,
}

/// Parameters for starting a note.
#[derive(Clone, Copy, Debug)]
pub struct NoteOn {
    pub note: f32,
    pub vel: f32,
    /// Samples until the release starts.
    pub hold: u32,
    /// -1 (left) .. 1 (right).
    pub pan: f32,
    /// Echo send 0..1.
    pub send: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Voice {
    p: Patch,
    stage: Stage,
    note: f32,
    amp: f32,
    gain_l: f32,
    gain_r: f32,
    send: f32,
    hold: u32,
    age: u32,
    ctrl: u32,
    env: f32,
    attack_step: f32,
    decay_coef: f32,
    release_coef: f32,
    phase: f32,
    phase2: f32,
    mphase: f32,
    inc: f32,
    inc2: f32,
    minc: f32,
    duty: f32,
    fm_index: f32,
    fm_coef: f32,
    lfo: f32,
    pwm_lfo: f32,
    pitch_off: f32,
    pitch_coef: f32,
    sweep_off: f32,
    cut_env: f32,
    cut_coef: f32,
    lp_a: f32,
    lp1: f32,
    lp2: f32,
    lp3: f32,
    lp4: f32,
    hp_a: f32,
    hp: f32,
    hp2: f32,
    rng: u32,
    nval: f32,
    nphase: f32,
    lfsr: u32,
}

impl Voice {
    pub const fn silent() -> Voice {
        Voice {
            p: BASE,
            stage: Stage::Off,
            note: 60.0,
            amp: 0.0,
            gain_l: 0.0,
            gain_r: 0.0,
            send: 0.0,
            hold: 0,
            age: 0,
            ctrl: 0,
            env: 0.0,
            attack_step: 1.0,
            decay_coef: 0.0,
            release_coef: 0.0,
            phase: 0.0,
            phase2: 0.0,
            mphase: 0.0,
            inc: 0.0,
            inc2: 0.0,
            minc: 0.0,
            duty: 0.5,
            fm_index: 0.0,
            fm_coef: 0.0,
            lfo: 0.0,
            pwm_lfo: 0.0,
            pitch_off: 0.0,
            pitch_coef: 0.0,
            sweep_off: 0.0,
            cut_env: 0.0,
            cut_coef: 0.0,
            lp_a: 1.0,
            lp1: 0.0,
            lp2: 0.0,
            lp3: 0.0,
            lp4: 0.0,
            hp_a: 0.0,
            hp: 0.0,
            hp2: 0.0,
            rng: 0x1234_5678,
            nval: 0.0,
            nphase: 1.0,
            lfsr: 1,
        }
    }

    pub fn is_active(&self) -> bool {
        self.stage != Stage::Off
    }

    fn start(&mut self, p: &Patch, n: &NoteOn, rate: f32, seed: u32) {
        let pan = n.pan.clamp(-1.0, 1.0);
        let angle = (pan + 1.0) * std::f32::consts::FRAC_PI_4;
        // Keep a little of each side so hard pans do not sound detached.
        self.gain_l = angle.cos() * 1.1;
        self.gain_r = angle.sin() * 1.1;
        self.p = *p;
        self.stage = Stage::Attack;
        self.note = n.note;
        self.amp = n.vel * p.gain;
        self.send = n.send;
        self.hold = n.hold.max(1);
        self.age = 0;
        self.ctrl = 0;
        self.env = 0.0;
        self.attack_step = 1.0 / (p.attack.max(0.0005) * rate);
        self.decay_coef = exp_coef(p.decay, rate);
        self.release_coef = exp_coef(p.release.max(0.004), rate);
        self.phase = 0.0;
        // Detuned partner starts out of phase so the pair does not cancel.
        self.phase2 = 0.37;
        self.mphase = 0.0;
        self.fm_index = p.fm_index;
        self.fm_coef = exp_coef(p.fm_decay, rate / CTRL as f32);
        self.lfo = 0.0;
        self.pwm_lfo = 0.0;
        self.pitch_off = p.pitch_env;
        self.pitch_coef = exp_coef(p.pitch_decay, rate / CTRL as f32);
        self.sweep_off = 0.0;
        self.cut_env = p.cutoff_env * (0.5 + 0.5 * n.vel.min(1.0));
        self.cut_coef = exp_coef(p.cutoff_decay, rate / CTRL as f32);
        self.lp1 = 0.0;
        self.lp2 = 0.0;
        self.lp3 = 0.0;
        self.lp4 = 0.0;
        self.hp = 0.0;
        self.hp2 = 0.0;
        self.hp_a = if p.highpass > 0.0 {
            onepole(p.highpass, rate)
        } else {
            0.0
        };
        self.rng = seed | 1;
        self.nval = 0.0;
        self.nphase = 1.0;
        self.lfsr = 1;
        self.update_control(rate);
        self.ctrl = CTRL;
    }

    /// Recomputes pitch, filter and modulation (called every `CTRL` samples).
    fn update_control(&mut self, rate: f32) {
        let p = &self.p;
        let dt = CTRL as f32 / rate;
        let t = self.age as f32 / rate;
        let mut pitch = self.note + self.pitch_off + self.sweep_off;
        if p.vib_depth > 0.0 {
            let ramp = ((t - p.vib_delay) * 4.0).clamp(0.0, 1.0);
            pitch += p.vib_depth * ramp * sin01(self.lfo);
            self.lfo += p.vib_rate * dt;
            self.lfo -= self.lfo.floor();
        }
        let hz = note_hz(pitch);
        self.inc = (hz / rate).min(0.49);
        if p.detune != 0.0 {
            self.inc2 = (self.inc * (p.detune * (1.0 / 1200.0)).exp2()).min(0.49);
        }
        match p.wave {
            Wave::Fm => {
                self.minc = self.inc * p.fm_ratio;
                let floor = p.fm_index * p.fm_sustain;
                self.fm_index = floor + (self.fm_index - floor) * self.fm_coef;
            }
            Wave::Pulse => {
                let mut d = p.duty;
                if p.pwm_depth > 0.0 {
                    d += p.pwm_depth * sin01(self.pwm_lfo);
                    self.pwm_lfo += p.pwm_rate * dt;
                    self.pwm_lfo -= self.pwm_lfo.floor();
                }
                self.duty = d.clamp(0.05, 0.95);
            }
            Wave::Noise | Wave::Metal => {
                // Sample-and-hold rate: the note frequency times 16 so that
                // MIDI ~100-110 gives full-bandwidth noise.
                self.minc = (hz * 16.0 / rate).min(1.0);
            }
            _ => {}
        }
        if p.cutoff < 20000.0 || p.cutoff_env > 0.0 {
            let track = if p.key_track > 0.0 {
                ((self.note - 60.0) * p.key_track * (1.0 / 12.0)).exp2()
            } else {
                1.0
            };
            let fc = p.cutoff * track + self.cut_env;
            self.lp_a = onepole(fc, rate);
            self.cut_env *= self.cut_coef;
        } else {
            self.lp_a = 1.0;
        }
        self.pitch_off *= self.pitch_coef;
        self.sweep_off += p.sweep * dt;
    }

    /// Adds this voice into the output and echo-send buffers (as many
    /// samples as the shortest buffer holds).
    pub fn render(
        &mut self,
        rate: f32,
        out_l: &mut [f32],
        out_r: &mut [f32],
        send_l: &mut [f32],
        send_r: &mut [f32],
    ) {
        let n = out_l
            .len()
            .min(out_r.len())
            .min(send_l.len())
            .min(send_r.len());
        let wave = self.p.wave;
        let two_osc = self.p.detune != 0.0;
        let noise_mix = self.p.noise;
        let use_hp = self.hp_a > 0.0;
        let mut i = 0;
        while i < n {
            if self.stage == Stage::Off {
                return;
            }
            if self.ctrl == 0 {
                self.update_control(rate);
                self.ctrl = CTRL;
            }
            self.ctrl -= 1;

            // --- oscillator -------------------------------------------------
            let dt = self.inc;
            let ph = self.phase;
            let mut s = match wave {
                Wave::Pulse => pulse(ph, dt, self.duty),
                Wave::Saw => saw(ph, dt),
                Wave::Tri => 1.0 - 4.0 * (ph - 0.5).abs(),
                Wave::Sine => sin01(ph),
                Wave::Fm => {
                    let m = sin01(self.mphase) * self.fm_index * (1.0 / std::f32::consts::TAU);
                    self.mphase += self.minc;
                    self.mphase -= self.mphase.floor();
                    let q = ph + m;
                    sin01(q - q.floor())
                }
                Wave::Noise => {
                    self.nphase += self.minc;
                    if self.nphase >= 1.0 {
                        self.nphase -= 1.0;
                        self.rng ^= self.rng << 13;
                        self.rng ^= self.rng >> 17;
                        self.rng ^= self.rng << 5;
                        self.nval = (self.rng >> 8) as f32 * (2.0 / 16_777_216.0) - 1.0;
                    }
                    self.nval
                }
                Wave::Metal => {
                    self.nphase += self.minc;
                    if self.nphase >= 1.0 {
                        self.nphase -= 1.0;
                        let bit = (self.lfsr ^ (self.lfsr >> 6)) & 1;
                        self.lfsr = (self.lfsr >> 1) | (bit << 14);
                        self.nval = if self.lfsr & 1 == 1 { 0.8 } else { -0.8 };
                    }
                    self.nval
                }
            };
            let mut next = ph + dt;
            if next >= 1.0 {
                next -= 1.0;
            }
            self.phase = next;
            let mut noise = 0.0;
            if noise_mix > 0.0 {
                self.rng ^= self.rng << 13;
                self.rng ^= self.rng >> 17;
                self.rng ^= self.rng << 5;
                noise = (self.rng >> 8) as f32 * (2.0 / 16_777_216.0) - 1.0;
            }

            // --- filters (a detuned pair is spread across the stereo field) --
            let a = self.lp_a;
            let (mut yl, mut yr);
            if two_osc {
                let dt2 = self.inc2;
                let ph2 = self.phase2;
                let s2 = match wave {
                    Wave::Pulse => pulse(ph2, dt2, self.duty),
                    Wave::Saw => saw(ph2, dt2),
                    Wave::Tri => 1.0 - 4.0 * (ph2 - 0.5).abs(),
                    _ => sin01(ph2),
                };
                let mut next2 = ph2 + dt2;
                if next2 >= 1.0 {
                    next2 -= 1.0;
                }
                self.phase2 = next2;
                let mut in_l = s * 0.8 + s2 * 0.4;
                let mut in_r = s2 * 0.8 + s * 0.4;
                if noise_mix > 0.0 {
                    in_l += (noise - in_l) * noise_mix;
                    in_r += (noise - in_r) * noise_mix;
                }
                self.lp1 += a * (in_l - self.lp1);
                self.lp2 += a * (self.lp1 - self.lp2);
                self.lp3 += a * (in_r - self.lp3);
                self.lp4 += a * (self.lp3 - self.lp4);
                yl = self.lp2;
                yr = self.lp4;
                if use_hp {
                    self.hp += self.hp_a * (yl - self.hp);
                    yl -= self.hp;
                    self.hp2 += self.hp_a * (yr - self.hp2);
                    yr -= self.hp2;
                }
            } else {
                s += (noise - s) * noise_mix;
                self.lp1 += a * (s - self.lp1);
                self.lp2 += a * (self.lp1 - self.lp2);
                yl = self.lp2;
                if use_hp {
                    self.hp += self.hp_a * (yl - self.hp);
                    yl -= self.hp;
                }
                yr = yl;
            }

            // --- envelope ---------------------------------------------------
            match self.stage {
                Stage::Attack => {
                    self.env += self.attack_step;
                    if self.env >= 1.0 {
                        self.env = 1.0;
                        self.stage = Stage::Decay;
                    }
                }
                Stage::Decay => {
                    let sus = self.p.sustain;
                    self.env = sus + (self.env - sus) * self.decay_coef;
                    if sus <= 0.0 && self.env < 0.0005 {
                        self.stage = Stage::Off;
                    }
                }
                Stage::Release => {
                    self.env *= self.release_coef;
                    if self.env < 0.0005 {
                        self.stage = Stage::Off;
                    }
                }
                Stage::Off => {}
            }
            self.age += 1;
            if self.hold > 0 {
                self.hold -= 1;
                if self.hold == 0 && self.stage != Stage::Off {
                    self.stage = Stage::Release;
                }
            }

            let g = self.env * self.amp;
            let l = yl * g * self.gain_l;
            let r = yr * g * self.gain_r;
            out_l[i] += l;
            out_r[i] += r;
            send_l[i] += l * self.send;
            send_r[i] += r * self.send;
            i += 1;
        }
    }

    /// Current loudness estimate used for voice stealing.
    fn level(&self) -> f32 {
        if self.stage == Stage::Off {
            0.0
        } else {
            let rel = if self.stage == Stage::Release {
                0.25
            } else {
                1.0
            };
            self.env * self.amp * rel
        }
    }
}

#[inline]
fn poly_blep(t: f32, dt: f32) -> f32 {
    if t < dt {
        let x = t / dt;
        x + x - x * x - 1.0
    } else if t > 1.0 - dt {
        let x = (t - 1.0) / dt;
        x * x + x + x + 1.0
    } else {
        0.0
    }
}

#[inline]
fn saw(ph: f32, dt: f32) -> f32 {
    2.0 * ph - 1.0 - poly_blep(ph, dt)
}

#[inline]
fn pulse(ph: f32, dt: f32, duty: f32) -> f32 {
    let mut v = if ph < duty { 1.0 } else { -1.0 };
    v += poly_blep(ph, dt);
    let mut t2 = ph - duty;
    if t2 < 0.0 {
        t2 += 1.0;
    }
    v -= poly_blep(t2, dt);
    // Remove the DC offset of narrow pulses.
    v - (2.0 * duty - 1.0)
}

/// A fixed set of voices with oldest/quietest stealing.
pub struct VoicePool {
    voices: Vec<Voice>,
    seed: u32,
}

impl VoicePool {
    pub fn new(count: usize) -> VoicePool {
        VoicePool {
            voices: vec![Voice::silent(); count],
            seed: 0x9E37_79B9,
        }
    }

    pub fn start(&mut self, patch: &Patch, n: &NoteOn, rate: f32) {
        let mut best = 0;
        let mut best_score = f32::MAX;
        for (i, v) in self.voices.iter().enumerate() {
            if !v.is_active() {
                best = i;
                break;
            }
            // Prefer quiet voices, then old ones.
            let score = v.level() - v.age as f32 * 1e-7;
            if score < best_score {
                best_score = score;
                best = i;
            }
        }
        self.seed = self
            .seed
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        self.voices[best].start(patch, n, rate, self.seed);
    }

    pub fn render(
        &mut self,
        rate: f32,
        out_l: &mut [f32],
        out_r: &mut [f32],
        send_l: &mut [f32],
        send_r: &mut [f32],
    ) {
        for v in self.voices.iter_mut() {
            if v.is_active() {
                v.render(rate, out_l, out_r, send_l, send_r);
            }
        }
    }

    pub fn active(&self) -> usize {
        self.voices.iter().filter(|v| v.is_active()).count()
    }

    pub fn clear(&mut self) {
        for v in self.voices.iter_mut() {
            v.stage = Stage::Off;
        }
    }
}

/// Echo settings. Delay times are in seconds.
#[derive(Clone, Copy, Debug)]
pub struct EchoParams {
    pub time_l: f32,
    pub time_r: f32,
    /// Feedback 0..0.9.
    pub feedback: f32,
    /// How much of the feedback crosses to the other side (ping-pong).
    pub cross: f32,
    /// Low-pass in the feedback path (darker repeats).
    pub damp_hz: f32,
    /// Wet return level.
    pub level: f32,
    /// Smears the input with two all-pass filters (cavernous, reverb-like).
    pub diffuse: bool,
}

pub const NO_ECHO: EchoParams = EchoParams {
    time_l: 0.25,
    time_r: 0.25,
    feedback: 0.0,
    cross: 0.0,
    damp_hz: 5000.0,
    level: 0.0,
    diffuse: false,
};

/// Stereo feedback delay with damping and optional diffusion.
pub struct Echo {
    buf_l: Vec<f32>,
    buf_r: Vec<f32>,
    pos: usize,
    dl: usize,
    dr: usize,
    fb: f32,
    cross: f32,
    damp: f32,
    lp_l: f32,
    lp_r: f32,
    level: f32,
    diffuse: bool,
    ap: [AllPass; 4],
}

struct AllPass {
    buf: Vec<f32>,
    pos: usize,
    len: usize,
}

impl AllPass {
    fn new(max: usize) -> AllPass {
        AllPass {
            buf: vec![0.0; max.max(1)],
            pos: 0,
            len: max.max(1),
        }
    }
    #[inline]
    fn process(&mut self, x: f32) -> f32 {
        let d = self.buf[self.pos];
        let y = d - 0.6 * x;
        self.buf[self.pos] = x + 0.6 * y;
        self.pos += 1;
        if self.pos >= self.len {
            self.pos = 0;
        }
        y
    }
}

impl Echo {
    /// `max_time` is the longest delay (seconds) this echo will ever be set to.
    pub fn new(rate: f32, max_time: f32) -> Echo {
        let len = (rate * max_time) as usize + 2;
        let ms = |x: f32| (rate * x * 0.001) as usize;
        Echo {
            buf_l: vec![0.0; len],
            buf_r: vec![0.0; len],
            pos: 0,
            dl: 1,
            dr: 1,
            fb: 0.0,
            cross: 0.0,
            damp: 1.0,
            lp_l: 0.0,
            lp_r: 0.0,
            level: 0.0,
            diffuse: false,
            ap: [
                AllPass::new(ms(7.3)),
                AllPass::new(ms(11.9)),
                AllPass::new(ms(8.7)),
                AllPass::new(ms(13.1)),
            ],
        }
    }

    pub fn set(&mut self, p: &EchoParams, rate: f32) {
        let max = self.buf_l.len() - 1;
        self.dl = ((p.time_l * rate) as usize).clamp(1, max);
        self.dr = ((p.time_r * rate) as usize).clamp(1, max);
        self.fb = p.feedback.clamp(0.0, 0.92);
        self.cross = p.cross.clamp(0.0, 1.0);
        self.damp = onepole(p.damp_hz, rate);
        self.level = p.level;
        self.diffuse = p.diffuse;
    }

    pub fn clear(&mut self) {
        self.buf_l.fill(0.0);
        self.buf_r.fill(0.0);
        for ap in self.ap.iter_mut() {
            ap.buf.fill(0.0);
        }
        self.lp_l = 0.0;
        self.lp_r = 0.0;
    }

    /// Reads the sends in `in_l/in_r` and adds the wet signal into `out_l/out_r`.
    pub fn process(&mut self, in_l: &[f32], in_r: &[f32], out_l: &mut [f32], out_r: &mut [f32]) {
        if self.level <= 0.0 {
            return;
        }
        let n = in_l.len().min(in_r.len()).min(out_l.len()).min(out_r.len());
        let len = self.buf_l.len();
        let (fb, cross, damp, level) = (self.fb, self.cross, self.damp, self.level);
        let mut i = 0;
        while i < n {
            let rl = if self.pos >= self.dl {
                self.pos - self.dl
            } else {
                self.pos + len - self.dl
            };
            let rr = if self.pos >= self.dr {
                self.pos - self.dr
            } else {
                self.pos + len - self.dr
            };
            let tap_l = self.buf_l[rl];
            let tap_r = self.buf_r[rr];
            self.lp_l += damp * (tap_l - self.lp_l);
            self.lp_r += damp * (tap_r - self.lp_r);
            let mut x_l = in_l[i];
            let mut x_r = in_r[i];
            if self.diffuse {
                let a = self.ap[0].process(x_l);
                x_l = self.ap[1].process(a);
                let b = self.ap[2].process(x_r);
                x_r = self.ap[3].process(b);
            }
            let fl = self.lp_l * (1.0 - cross) + self.lp_r * cross;
            let fr = self.lp_r * (1.0 - cross) + self.lp_l * cross;
            self.buf_l[self.pos] = x_l + fl * fb;
            self.buf_r[self.pos] = x_r + fr * fb;
            out_l[i] += tap_l * level;
            out_r[i] += tap_r * level;
            self.pos += 1;
            if self.pos >= len {
                self.pos = 0;
            }
            i += 1;
        }
    }
}

/// Look-ahead peak limiter followed by a soft clipper: the signal is delayed
/// by ~1.5 ms so the gain can ramp down smoothly before a peak arrives.
pub struct Limiter {
    env: f32,
    rel: f32,
    gain: f32,
    smooth: f32,
    threshold: f32,
    buf_l: [f32; LOOKAHEAD_MAX],
    buf_r: [f32; LOOKAHEAD_MAX],
    pos: usize,
    delay: usize,
    /// Loudest input seen and number of samples that needed gain reduction.
    pub peak_in: f32,
    pub limited: u64,
}

const LOOKAHEAD_MAX: usize = 512;

impl Limiter {
    pub fn new(rate: f32) -> Limiter {
        Limiter {
            env: 0.0,
            rel: (-1.0 / (0.2 * rate)).exp(),
            gain: 1.0,
            smooth: 1.0 - (-1.0 / (0.0003 * rate)).exp(),
            threshold: 0.8,
            buf_l: [0.0; LOOKAHEAD_MAX],
            buf_r: [0.0; LOOKAHEAD_MAX],
            pos: 0,
            delay: ((0.0015 * rate) as usize).clamp(1, LOOKAHEAD_MAX),
            peak_in: 0.0,
            limited: 0,
        }
    }

    pub fn process(&mut self, l: &mut [f32], r: &mut [f32]) {
        let n = l.len().min(r.len());
        let mut i = 0;
        while i < n {
            // Never let a non-finite sample reach the device.
            let xl = if l[i].is_finite() { l[i] } else { 0.0 };
            let xr = if r[i].is_finite() { r[i] } else { 0.0 };
            let peak = xl.abs().max(xr.abs());
            if peak > self.peak_in {
                self.peak_in = peak;
            }
            if peak > self.env {
                self.env = peak;
            } else {
                self.env *= self.rel;
            }
            let target = if self.env > self.threshold {
                self.limited += 1;
                self.threshold / self.env
            } else {
                1.0
            };
            self.gain += (target - self.gain) * self.smooth;
            let dl = self.buf_l[self.pos];
            let dr = self.buf_r[self.pos];
            self.buf_l[self.pos] = xl;
            self.buf_r[self.pos] = xr;
            self.pos += 1;
            if self.pos >= self.delay {
                self.pos = 0;
            }
            l[i] = soft_clip(dl * self.gain);
            r[i] = soft_clip(dr * self.gain);
            i += 1;
        }
    }
}
