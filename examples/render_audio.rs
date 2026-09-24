//! Renders every music track and sound effect to WAV files with the game's
//! own synthesizer, so the soundtrack can be reviewed offline (no audio
//! device needed):
//!
//! ```text
//! cargo run --release --example render_audio -- <out_dir> [filter]
//! ```
//!
//! Looping tracks are rendered for one full pass plus a few seconds of the
//! repeat (so the loop seam can be heard), fading out at the very end;
//! one-shots are rendered completely. `filter` keeps only names containing it.

#[allow(dead_code)]
#[path = "../src/audio/mod.rs"]
mod audio;

use audio::{Sfx, Track, offline};
use std::io::Write;
use std::path::Path;

const RATE: u32 = 44_100;
/// Seconds of the repeat rendered after the first pass of a looping track.
const SEAM: f32 = 6.0;

fn main() {
    let mut args = std::env::args().skip(1);
    let dir = args.next().unwrap_or_else(|| "audio_render".to_string());
    let filter = args.next().unwrap_or_default().to_lowercase();
    let dir = Path::new(&dir);
    if let Err(e) = std::fs::create_dir_all(dir) {
        eprintln!("cannot create {}: {e}", dir.display());
        std::process::exit(1);
    }

    for track in Track::ALL {
        let name = format!("music_{}", snake(&format!("{track:?}")));
        if !name.contains(&filter) {
            continue;
        }
        let mut r = offline::render_track(track, RATE, if track.loops() { SEAM } else { 0.5 });
        if r.looping {
            fade_out(&mut r.samples, RATE, 4.0);
        }
        let path = dir.join(format!("{name}.wav"));
        write_wav(&path, &r.samples, RATE);
        let (peak, rms) = levels(&r.samples);
        let kind = if r.looping {
            format!("loop {:.1}-{:.1} s", r.loop_start, r.first_pass)
        } else {
            format!("one-shot {:.1} s", r.first_pass)
        };
        println!(
            "{:<28} {:>5.0} bpm  {:<20} peak {:.2} (pre-limiter {:.2})  rms {:.3}",
            path.display(),
            r.bpm,
            kind,
            peak,
            r.peak_before_limiter,
            rms
        );
    }

    for sfx in Sfx::ALL {
        let name = format!("sfx_{}", snake(&format!("{sfx:?}")));
        if !name.contains(&filter) {
            continue;
        }
        let samples = offline::render_sfx(sfx, RATE);
        let path = dir.join(format!("{name}.wav"));
        write_wav(&path, &samples, RATE);
        let (peak, rms) = levels(&samples);
        let secs = samples.len() as f32 / 2.0 / RATE as f32;
        println!(
            "{:<28} {:>5.2} s  peak {:.2}  rms {:.3}",
            path.display(),
            secs,
            peak,
            rms
        );
    }
}

fn snake(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

fn levels(samples: &[f32]) -> (f32, f32) {
    let peak = samples.iter().fold(0.0f32, |m, x| m.max(x.abs()));
    let sum: f64 = samples.iter().map(|x| (*x as f64) * (*x as f64)).sum();
    let rms = (sum / samples.len().max(1) as f64).sqrt() as f32;
    (peak, rms)
}

fn fade_out(samples: &mut [f32], rate: u32, secs: f32) {
    let frames = samples.len() / 2;
    let n = ((secs * rate as f32) as usize).min(frames);
    for i in 0..n {
        let g = 1.0 - i as f32 / n as f32;
        let f = frames - n + i;
        samples[f * 2] *= g;
        samples[f * 2 + 1] *= g;
    }
}

/// 16-bit PCM stereo WAV.
fn write_wav(path: &Path, samples: &[f32], rate: u32) {
    let data_len = (samples.len() * 2) as u32;
    let mut bytes = Vec::with_capacity(44 + data_len as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
    bytes.extend_from_slice(&2u16.to_le_bytes()); // stereo
    bytes.extend_from_slice(&rate.to_le_bytes());
    bytes.extend_from_slice(&(rate * 4).to_le_bytes());
    bytes.extend_from_slice(&4u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    let written = std::fs::File::create(path).and_then(|mut f| f.write_all(&bytes));
    if let Err(e) = written {
        eprintln!("cannot write {}: {e}", path.display());
    }
}
