//! The mixer that the audio thread (or the offline renderer) runs: up to
//! three songs crossfading, the sound-effect voices, volumes and the limiter.

use super::Sfx;
use super::seq::{Player, Song};
use super::sfx::SfxPlayer;
use super::synth::Limiter;

/// Frames rendered per internal block.
pub const BLOCK: usize = 256;
const SLOTS: usize = 3;
const MUSIC_VOICES: usize = 40;
/// Overall music level before the volume setting (leaves headroom for effects).
const MUSIC_GAIN: f32 = 0.7;

/// Messages from the game thread to the engine.
pub enum Cmd {
    /// Start `song` (or fade to silence with `None`).
    Music {
        song: Option<Box<Song>>,
        fade_in: f32,
        fade_out: f32,
    },
    Sfx(Sfx),
    MusicVolume(f32),
    SfxVolume(f32),
}

struct Slot {
    player: Player,
    /// Linear fade position 0..1 and where it is heading.
    fade: f32,
    target: f32,
    step: f32,
    active: bool,
}

pub struct Engine {
    rate: f32,
    slots: Vec<Slot>,
    current: Option<usize>,
    sfx: SfxPlayer,
    music_vol: f32,
    music_target: f32,
    sfx_vol: f32,
    sfx_target: f32,
    vol_smooth: f32,
    limiter: Limiter,
    tmp_l: Vec<f32>,
    tmp_r: Vec<f32>,
    mus_l: Vec<f32>,
    mus_r: Vec<f32>,
    fx_l: Vec<f32>,
    fx_r: Vec<f32>,
}

impl Engine {
    pub fn new(rate: u32) -> Engine {
        let rate = rate.clamp(8000, 192_000) as f32;
        let slots = (0..SLOTS)
            .map(|_| Slot {
                player: Player::new(rate, MUSIC_VOICES, BLOCK),
                fade: 0.0,
                target: 0.0,
                step: 0.0,
                active: false,
            })
            .collect();
        Engine {
            rate,
            slots,
            current: None,
            sfx: SfxPlayer::new(rate, BLOCK),
            music_vol: 1.0,
            music_target: 1.0,
            sfx_vol: 1.0,
            sfx_target: 1.0,
            vol_smooth: 1.0 - (-1.0 / (0.03 * rate)).exp(),
            limiter: Limiter::new(rate),
            tmp_l: vec![0.0; BLOCK],
            tmp_r: vec![0.0; BLOCK],
            mus_l: vec![0.0; BLOCK],
            mus_r: vec![0.0; BLOCK],
            fx_l: vec![0.0; BLOCK],
            fx_r: vec![0.0; BLOCK],
        }
    }

    /// Loudest pre-limiter sample and how many samples the limiter reduced.
    pub fn limiter_stats(&self) -> (f32, u64) {
        (self.limiter.peak_in, self.limiter.limited)
    }

    /// True while any music is playing or fading out.
    pub fn music_playing(&self) -> bool {
        self.slots.iter().any(|s| s.active)
    }

    /// Applies a command. Returns a replaced song so the caller may drop it
    /// outside a time-critical section if it wants to.
    pub fn command(&mut self, cmd: Cmd) -> Option<Box<Song>> {
        match cmd {
            Cmd::Music {
                song,
                fade_in,
                fade_out,
            } => {
                let out_step = 1.0 / (fade_out.max(0.005) * self.rate);
                if let Some(cur) = self.current.take() {
                    let s = &mut self.slots[cur];
                    s.target = 0.0;
                    s.step = out_step;
                }
                let song = song?;
                // Use an idle slot, or else the one that has faded out furthest.
                let mut best = 0;
                let mut best_fade = f32::MAX;
                for (i, s) in self.slots.iter().enumerate() {
                    let f = if s.active { s.fade + s.target } else { -1.0 };
                    if f < best_fade {
                        best_fade = f;
                        best = i;
                    }
                }
                let s = &mut self.slots[best];
                let old = s.player.load(song);
                s.active = true;
                s.target = 1.0;
                if fade_in <= 0.0 {
                    s.fade = 1.0;
                    s.step = 0.0;
                } else {
                    s.fade = 0.0;
                    s.step = 1.0 / (fade_in * self.rate);
                }
                self.current = Some(best);
                old
            }
            Cmd::Sfx(sfx) => {
                self.sfx.play(sfx);
                None
            }
            Cmd::MusicVolume(v) => {
                self.music_target = volume_curve(v);
                None
            }
            Cmd::SfxVolume(v) => {
                self.sfx_target = volume_curve(v);
                None
            }
        }
    }

    /// Fills interleaved stereo samples.
    pub fn render_interleaved(&mut self, out: &mut [f32]) {
        let frames = out.len() / 2;
        let mut done = 0;
        while done < frames {
            let n = (frames - done).min(BLOCK);
            self.render_block(n);
            let dst = &mut out[done * 2..(done + n) * 2];
            let mut i = 0;
            while i < n {
                dst[i * 2] = self.mus_l[i];
                dst[i * 2 + 1] = self.mus_r[i];
                i += 1;
            }
            done += n;
        }
    }

    /// Renders `n <= BLOCK` frames into `mus_l/mus_r` (final mix).
    fn render_block(&mut self, n: usize) {
        self.mus_l[..n].fill(0.0);
        self.mus_r[..n].fill(0.0);
        for s in self.slots.iter_mut() {
            if !s.active {
                continue;
            }
            self.tmp_l[..n].fill(0.0);
            self.tmp_r[..n].fill(0.0);
            s.player.render(&mut self.tmp_l[..n], &mut self.tmp_r[..n]);
            let mut i = 0;
            while i < n {
                if s.fade < s.target {
                    s.fade = (s.fade + s.step).min(s.target);
                } else if s.fade > s.target {
                    s.fade = (s.fade - s.step).max(s.target);
                }
                let f = s.fade;
                let g = f * f * (3.0 - 2.0 * f);
                self.mus_l[i] += self.tmp_l[i] * g;
                self.mus_r[i] += self.tmp_r[i] * g;
                i += 1;
            }
            if (s.fade <= 0.0 && s.target <= 0.0) || s.player.finished() {
                s.active = false;
            }
        }
        self.fx_l[..n].fill(0.0);
        self.fx_r[..n].fill(0.0);
        self.sfx.render(&mut self.fx_l[..n], &mut self.fx_r[..n]);
        let a = self.vol_smooth;
        let mut i = 0;
        while i < n {
            self.music_vol += (self.music_target - self.music_vol) * a;
            self.sfx_vol += (self.sfx_target - self.sfx_vol) * a;
            let mv = self.music_vol * MUSIC_GAIN;
            self.mus_l[i] = self.mus_l[i] * mv + self.fx_l[i] * self.sfx_vol;
            self.mus_r[i] = self.mus_r[i] * mv + self.fx_r[i] * self.sfx_vol;
            i += 1;
        }
        self.limiter
            .process(&mut self.mus_l[..n], &mut self.mus_r[..n]);
    }
}

/// Maps a 0..1 slider to gain with a gentle curve (perceptually smoother).
fn volume_curve(v: f32) -> f32 {
    let v = if v.is_finite() {
        v.clamp(0.0, 1.0)
    } else {
        0.0
    };
    v * v
}
