//! The music tracker: a small MML-style notation for melodies, chord
//! progressions that drive generated bass/arpeggio/pad parts, drum step
//! patterns, and the real-time player that turns a `Song` into voices.
//!
//! Melody notation (`Builder::mml`), one token after another:
//! - `c d e f g a b` notes, with `+`/`#` (sharp) or `-` (flat), then an
//!   optional length (`4` = quarter, `8.` = dotted eighth, `12` = eighth
//!   triplet); without a length the `l` default is used;
//! - `r` rest, `^8` tie (extends the previous note or rest);
//! - `[c e g]2` chord; `o4` octave (o4 c = middle C), `>` / `<` octave up/down;
//! - `l8` default length, `v80` velocity (percent), `q6` gate in eighths
//!   of the note length (q8 legato, q16 lets notes ring twice as long),
//!   `k-2` transposes this channel;
//! - `( ... )3` repeats a group; `|` checks that a bar line falls here.
//!
//! Chord progressions are space-separated symbols with an optional length in
//! quarter notes: `Am F:2 G:2 C/E Dm7:1.5`. Without a length a chord lasts a
//! bar; `x` is silence. Patterns (`bass`, `arp`) are space-separated steps
//! that restart at every chord change: `r` rest, `-` hold, `C` whole chord,
//! `B` the chord's bass note, digits for chord tones (`bass`: degrees
//! 1 3 5 7 8 9 2 4 6 above the root; `arp`: indices into the voiced chord),
//! with `'` / `,` for an octave up/down and `!` / `?` for accents.

use super::synth::{Echo, EchoParams, NoteOn, Patch, VoicePool};

/// Ticks per quarter note.
pub const PPQ: u32 = 48;

/// One scheduled note.
#[derive(Clone, Copy, Debug)]
pub struct Event {
    pub tick: u32,
    /// Channel the note came from (for tests and offline analysis).
    #[allow(dead_code)]
    pub ch: u8,
    /// Sounding length in ticks (after the gate is applied).
    pub dur: u32,
    pub note: f32,
    pub vel: f32,
    pub patch: u16,
    pub pan: f32,
    pub send: f32,
}

/// A compiled piece of music, independent of the sample rate.
pub struct Song {
    pub bpm: f32,
    pub patches: Vec<Patch>,
    pub events: Vec<Event>,
    pub loop_start: u32,
    pub end: u32,
    pub looping: bool,
    /// Echo with times already converted to seconds.
    pub echo: EchoParams,
    pub gain: f32,
    /// Notation errors (checked by tests; ignored at runtime).
    pub errors: Vec<String>,
    /// Which channels are drum channels (for tests and offline analysis).
    #[allow(dead_code)]
    pub drum_chans: Vec<bool>,
    /// Bar length in ticks at the end of the song (for tests and offline analysis).
    #[allow(dead_code)]
    pub bar: u32,
}

impl Song {
    pub fn tick_seconds(&self) -> f64 {
        60.0 / (self.bpm as f64 * PPQ as f64)
    }
    /// Seconds from the start to the end of the first pass.
    pub fn length_seconds(&self) -> f64 {
        self.end as f64 * self.tick_seconds()
    }
    pub fn loop_start_seconds(&self) -> f64 {
        self.loop_start as f64 * self.tick_seconds()
    }
}

struct Chan {
    /// Patch index, or `None` for a drum channel.
    patch: Option<u16>,
    vol: f32,
    pan: f32,
    send: f32,
    cursor: u32,
    gate: f32,
    oct: i32,
    len: u32,
    vel: f32,
    key: i32,
}

#[derive(Clone, Copy, Debug)]
struct Chord {
    none: bool,
    root: i32,
    third: i32,
    fifth: i32,
    seventh: Option<i32>,
    ninth: Option<i32>,
    bass: i32,
    dur: u32,
}

impl Chord {
    /// Pitch classes of the chord, without duplicates.
    fn tones(&self) -> ([i32; 5], usize) {
        let mut t = [0; 5];
        let mut n = 0;
        let mut push = |x: i32| {
            let pc = x.rem_euclid(12);
            if !t[..n].contains(&pc) {
                t[n] = pc;
                n += 1;
            }
        };
        push(self.root);
        push(self.root + self.third);
        push(self.root + self.fifth);
        if let Some(s) = self.seventh {
            push(self.root + s);
        }
        if let Some(s) = self.ninth {
            push(self.root + s);
        }
        (t, n)
    }

    /// Chord tones voiced upwards from `base`, ascending: close position,
    /// except that a tone a semitone below its neighbour moves up an octave
    /// (so 7th and 9th chords open up instead of forming clusters).
    fn voiced(&self, base: i32) -> ([i32; 5], usize) {
        let (pcs, n) = self.tones();
        let mut v = [0; 5];
        for i in 0..n {
            v[i] = lowest_at_or_above(pcs[i], base);
        }
        v[..n].sort_unstable();
        for _ in 0..n {
            let mut moved = false;
            for i in 1..n {
                if v[i] - v[i - 1] == 1 {
                    v[i - 1] += 12;
                    moved = true;
                    break;
                }
            }
            if !moved {
                break;
            }
            v[..n].sort_unstable();
        }
        (v, n)
    }

    /// Semitones above the root for a degree token.
    fn degree(&self, d: i32) -> i32 {
        match d {
            1 => 0,
            2 => self.ninth.map(|n| n - 12).unwrap_or(2),
            3 => self.third,
            4 => 5,
            5 => self.fifth,
            6 => 9,
            7 => self.seventh.unwrap_or(12),
            8 => 12,
            _ => self.ninth.unwrap_or(14),
        }
    }
}

fn lowest_at_or_above(pc: i32, base: i32) -> i32 {
    base + (pc - base).rem_euclid(12)
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Step {
    Rest,
    Hold,
    /// Whole chord, velocity factor.
    Chord(f32),
    /// Bass note, octave shift, velocity factor.
    Bass(i32, f32),
    /// Degree or index, octave shift, velocity factor.
    Tone(i32, i32, f32),
}

/// A drum sound for `Builder::beat`.
pub struct DrumHit {
    pub patch: Patch,
    pub note: f32,
    pub vel: f32,
    pub pan: f32,
    /// Note length in seconds before release.
    pub len: f32,
}

/// Builds a `Song` channel by channel.
pub struct Builder {
    bpm: f32,
    bar: u32,
    bar_origin: u32,
    chans: Vec<Chan>,
    patches: Vec<Patch>,
    events: Vec<Event>,
    key: i32,
    loop_start: u32,
    looping: bool,
    echo: EchoParams,
    gain: f32,
    errors: Vec<String>,
    rng: u32,
    name: &'static str,
    humanize: f32,
    kit: fn(char) -> Option<DrumHit>,
    /// Kit sounds already added to `patches`, by character.
    kit_idx: Vec<(char, u16)>,
}

impl Builder {
    /// `beats` per bar of `unit` notes (3, 4 = 3/4; 7, 8 = 7/8).
    pub fn new(
        name: &'static str,
        bpm: f32,
        beats: u32,
        unit: u32,
        kit: fn(char) -> Option<DrumHit>,
    ) -> Builder {
        Builder {
            bpm,
            bar: beats * 4 * PPQ / unit,
            bar_origin: 0,
            chans: Vec::new(),
            patches: Vec::new(),
            events: Vec::with_capacity(4096),
            key: 0,
            loop_start: 0,
            looping: true,
            echo: super::synth::NO_ECHO,
            gain: 1.0,
            errors: Vec::new(),
            rng: 0x2545_F491,
            name,
            humanize: 0.06,
            kit,
            kit_idx: Vec::new(),
        }
    }

    /// Echo with delay times in quarter notes.
    #[allow(clippy::too_many_arguments)]
    pub fn echo(
        &mut self,
        left_beats: f32,
        right_beats: f32,
        feedback: f32,
        level: f32,
        damp_hz: f32,
        cross: f32,
        diffuse: bool,
    ) {
        let beat = 60.0 / self.bpm;
        self.echo = EchoParams {
            time_l: (left_beats * beat).min(1.45),
            time_r: (right_beats * beat).min(1.45),
            feedback,
            cross,
            damp_hz,
            level,
            diffuse,
        };
    }

    pub fn gain(&mut self, g: f32) {
        self.gain = g;
    }

    /// Plays once instead of looping.
    pub fn once(&mut self) {
        self.looping = false;
    }

    /// Global transposition (semitones) for everything written afterwards.
    pub fn key(&mut self, semis: i32) {
        self.key = semis;
    }

    /// Adds a melodic channel and returns its index.
    pub fn ch(&mut self, patch: Patch, vol: f32, pan: f32, send: f32) -> usize {
        self.patches.push(patch);
        let idx = (self.patches.len() - 1) as u16;
        self.chans.push(Chan {
            patch: Some(idx),
            vol,
            pan,
            send,
            cursor: 0,
            gate: 0.9,
            oct: 4,
            len: PPQ,
            vel: 0.8,
            key: 0,
        });
        self.chans.len() - 1
    }

    /// Adds a drum channel (sounds picked per step from the kit).
    pub fn drums(&mut self, vol: f32, send: f32) -> usize {
        self.chans.push(Chan {
            patch: None,
            vol,
            pan: 0.0,
            send,
            cursor: 0,
            gate: 1.0,
            oct: 4,
            len: PPQ,
            vel: 0.8,
            key: 0,
        });
        self.chans.len() - 1
    }

    /// Sets a channel's gate (fraction of each step that sounds; >1 rings on).
    pub fn gate(&mut self, ch: usize, gate: f32) {
        self.chans[ch].gate = gate;
    }

    /// Sets a channel's velocity (0..1) for generated parts.
    pub fn vel(&mut self, ch: usize, vel: f32) {
        self.chans[ch].vel = vel;
    }

    fn err(&mut self, msg: String) {
        self.errors.push(format!("{}: {}", self.name, msg));
    }

    fn rand(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng >> 8) as f32 / 16_777_216.0
    }

    /// Adds a note with a raw (ungated) length; returns the event index.
    fn emit(&mut self, ch: usize, tick: u32, raw: u32, note: i32, vel: f32) -> usize {
        let h = 1.0 + (self.rand() - 0.5) * 2.0 * self.humanize;
        let c = &self.chans[ch];
        let ev = Event {
            tick,
            ch: ch as u8,
            dur: ((raw as f32 * c.gate) as u32).max(1),
            note: note as f32,
            vel: (vel * c.vol * h).max(0.0),
            patch: c.patch.unwrap_or(0),
            pan: c.pan,
            send: c.send,
        };
        self.events.push(ev);
        self.events.len() - 1
    }

    /// Re-gates events `a..b` to a new raw length.
    fn stretch(&mut self, ch: usize, a: usize, b: usize, raw: u32) {
        let gate = self.chans[ch].gate;
        for e in a..b {
            self.events[e].dur = ((raw as f32 * gate) as u32).max(1);
        }
    }

    /// Moves every channel to the latest cursor (silent channels rest).
    pub fn sync(&mut self) -> u32 {
        let end = self.chans.iter().map(|c| c.cursor).max().unwrap_or(0);
        for c in self.chans.iter_mut() {
            c.cursor = end;
        }
        end
    }

    /// Everything before this point is an intro that is not repeated.
    pub fn loop_here(&mut self) {
        self.loop_start = self.sync();
    }

    /// Rests `bars` bars on one channel.
    pub fn rest_bars(&mut self, ch: usize, bars: u32) {
        self.chans[ch].cursor += bars * self.bar;
    }

    pub fn build(mut self) -> Song {
        let end = self.sync();
        if !(end - self.bar_origin).is_multiple_of(self.bar) {
            let msg = format!("song ends mid-bar at tick {end}");
            self.err(msg);
        }
        self.events.sort_by_key(|e| e.tick);
        Song {
            bpm: self.bpm,
            patches: self.patches,
            events: self.events,
            loop_start: self.loop_start,
            end,
            looping: self.looping,
            echo: self.echo,
            gain: self.gain,
            errors: self.errors,
            drum_chans: self.chans.iter().map(|c| c.patch.is_none()).collect(),
            bar: self.bar,
        }
    }

    // ------------------------------------------------------------------
    // Melodies
    // ------------------------------------------------------------------

    /// Appends MML to a channel (see module docs).
    pub fn mml(&mut self, ch: usize, text: &str) {
        let expanded = match expand_repeats(text) {
            Ok(s) => s,
            Err(e) => {
                self.err(format!("ch{ch}: {e}"));
                return;
            }
        };
        let s = expanded.as_bytes();
        let mut i = 0;
        // The last note (event range and raw length) that `^` extends.
        let mut last: Option<(usize, usize, u32)> = None;
        let mut after_rest = false;
        while i < s.len() {
            let c = s[i] as char;
            i += 1;
            match c {
                ' ' | '\n' | '\t' | '\r' => {}
                '|' => {
                    let cur = self.chans[ch].cursor;
                    if cur < self.bar_origin || !(cur - self.bar_origin).is_multiple_of(self.bar) {
                        let bars = cur as f32 / self.bar as f32;
                        self.err(format!(
                            "ch{ch}: bar line at {bars:.3} bars near `{}`",
                            context(s, i)
                        ));
                    }
                }
                'o' => {
                    let (n, j) = read_int(s, i);
                    i = j;
                    self.chans[ch].oct = n.unwrap_or(4);
                }
                '>' => self.chans[ch].oct += 1,
                '<' => self.chans[ch].oct -= 1,
                'l' => {
                    let (len, j) = read_len(s, i, self.chans[ch].len);
                    i = j;
                    self.chans[ch].len = len;
                }
                'v' => {
                    let (n, j) = read_int(s, i);
                    i = j;
                    self.chans[ch].vel = n.unwrap_or(80) as f32 / 100.0;
                }
                'q' => {
                    let (n, j) = read_int(s, i);
                    i = j;
                    self.chans[ch].gate = n.unwrap_or(8) as f32 / 8.0;
                }
                'k' => {
                    let (n, j) = read_int(s, i);
                    i = j;
                    self.chans[ch].key = n.unwrap_or(0);
                }
                'r' => {
                    let (len, j) = read_len(s, i, self.chans[ch].len);
                    i = j;
                    last = None;
                    after_rest = true;
                    self.chans[ch].cursor += len;
                }
                '^' | '&' => {
                    let (len, j) = read_len(s, i, self.chans[ch].len);
                    i = j;
                    if let Some((a, b, raw)) = last {
                        self.stretch(ch, a, b, raw + len);
                        last = Some((a, b, raw + len));
                    } else if !after_rest {
                        self.err(format!(
                            "ch{ch}: tie without a note near `{}`",
                            context(s, i)
                        ));
                    }
                    self.chans[ch].cursor += len;
                }
                'a'..='g' => {
                    let (pc, j) = read_pitch(c, s, i);
                    let (len, j) = read_len(s, j, self.chans[ch].len);
                    i = j;
                    let note = self.pitch(ch, pc, 0);
                    let tick = self.chans[ch].cursor;
                    let vel = self.chans[ch].vel;
                    let e = self.emit(ch, tick, len, note, vel);
                    last = Some((e, e + 1, len));
                    after_rest = false;
                    self.chans[ch].cursor += len;
                }
                '[' => {
                    let tick = self.chans[ch].cursor;
                    let mut oct = 0;
                    let mut notes = [0i32; 8];
                    let mut n = 0;
                    while i < s.len() && s[i] != b']' {
                        let d = s[i] as char;
                        i += 1;
                        match d {
                            '>' => oct += 1,
                            '<' => oct -= 1,
                            'a'..='g' => {
                                let (pc, j) = read_pitch(d, s, i);
                                i = j;
                                if n < notes.len() {
                                    notes[n] = self.pitch(ch, pc, oct);
                                    n += 1;
                                }
                            }
                            ' ' => {}
                            _ => self.err(format!("ch{ch}: bad chord char `{d}`")),
                        }
                    }
                    i += 1; // ']'
                    let (len, j) = read_len(s, i, self.chans[ch].len);
                    i = j;
                    let vel = self.chans[ch].vel * 0.8;
                    let first = self.events.len();
                    for &note in &notes[..n] {
                        self.emit(ch, tick, len, note, vel);
                    }
                    last = Some((first, self.events.len(), len));
                    after_rest = false;
                    self.chans[ch].cursor += len;
                }
                _ => {
                    self.err(format!("ch{ch}: unexpected `{c}` near `{}`", context(s, i)));
                }
            }
        }
        let cur = self.chans[ch].cursor;
        if cur < self.bar_origin || !(cur - self.bar_origin).is_multiple_of(self.bar) {
            let bars = cur as f32 / self.bar as f32;
            self.err(format!(
                "ch{ch}: line ends mid-bar ({bars:.3} bars): `{}`",
                context(s, s.len())
            ));
        }
    }

    fn pitch(&self, ch: usize, pc: i32, oct_off: i32) -> i32 {
        let c = &self.chans[ch];
        (c.oct + oct_off + 1) * 12 + pc + c.key + self.key
    }

    // ------------------------------------------------------------------
    // Chord-driven parts
    // ------------------------------------------------------------------

    fn parse_chords(&mut self, text: &str) -> Vec<Chord> {
        let mut out = Vec::new();
        for tok in text.split_whitespace() {
            if tok == "|" {
                continue;
            }
            let (sym, dur) = match tok.split_once(':') {
                Some((a, b)) => match b.parse::<f32>() {
                    Ok(q) => (a, (q * PPQ as f32).round() as u32),
                    Err(_) => {
                        self.err(format!("bad chord length `{tok}`"));
                        (a, self.bar)
                    }
                },
                None => (tok, self.bar),
            };
            match parse_chord(sym) {
                Some(mut c) => {
                    c.dur = dur;
                    c.root += self.key;
                    c.bass += self.key;
                    out.push(c);
                }
                None => self.err(format!("bad chord `{tok}`")),
            }
        }
        out
    }

    fn parse_pattern(&mut self, pat: &str) -> Vec<Step> {
        let mut out = Vec::new();
        for tok in pat.split_whitespace() {
            if tok == "|" {
                continue;
            }
            let b = tok.as_bytes();
            let mut oct = 0;
            let mut vel = 1.0;
            for &m in &b[1..] {
                match m {
                    b'\'' => oct += 1,
                    b',' => oct -= 1,
                    b'!' => vel *= 1.3,
                    b'?' => vel *= 0.6,
                    _ => self.err(format!("bad pattern token `{tok}`")),
                }
            }
            let step = match b[0] {
                b'r' | b'.' => Step::Rest,
                b'-' => Step::Hold,
                b'C' => Step::Chord(vel),
                b'B' => Step::Bass(oct, vel),
                d @ b'0'..=b'9' => Step::Tone((d - b'0') as i32, oct, vel),
                _ => {
                    self.err(format!("bad pattern token `{tok}`"));
                    Step::Rest
                }
            };
            out.push(step);
        }
        if out.is_empty() {
            out.push(Step::Rest);
        }
        out
    }

    fn pattern(
        &mut self,
        ch: usize,
        chords: &str,
        step: &str,
        pat: &str,
        base: i32,
        degrees: bool,
    ) {
        let chords = self.parse_chords(chords);
        let steps = self.parse_pattern(pat);
        let (step_len, _) = read_len(step.as_bytes(), 0, PPQ);
        let base_vel = self.chans[ch].vel;
        let mut tick = self.chans[ch].cursor;
        for c in chords {
            let end = tick + c.dur;
            let mut t = tick;
            let mut k = 0;
            let mut last: Option<(usize, usize, u32)> = None;
            if c.none {
                tick = end;
                continue;
            }
            let (voiced, nv) = c.voiced(base);
            let root = lowest_at_or_above(c.root, base);
            let bass = lowest_at_or_above(c.bass, base);
            while t < end {
                let len = step_len.min(end - t);
                match steps[k % steps.len()] {
                    Step::Rest => last = None,
                    Step::Hold => {
                        if let Some((a, b, raw)) = last {
                            self.stretch(ch, a, b, raw + len);
                            last = Some((a, b, raw + len));
                        }
                    }
                    Step::Chord(v) => {
                        let first = self.events.len();
                        for &n in &voiced[..nv] {
                            self.emit(ch, t, len, n, base_vel * v * 0.8);
                        }
                        last = Some((first, self.events.len(), len));
                    }
                    Step::Bass(o, v) => {
                        let e = self.emit(ch, t, len, bass + 12 * o, base_vel * v);
                        last = Some((e, e + 1, len));
                    }
                    Step::Tone(d, o, v) => {
                        let note = if degrees {
                            root + c.degree(d)
                        } else {
                            let d = d as usize;
                            voiced[d % nv] + 12 * (d / nv) as i32
                        };
                        let e = self.emit(ch, t, len, note + 12 * o, base_vel * v);
                        last = Some((e, e + 1, len));
                    }
                }
                t += len;
                k += 1;
            }
            tick = end;
        }
        self.chans[ch].cursor = tick;
    }

    /// Chord-tone pattern with degree digits (bass lines, counter lines).
    /// `base` is the lowest MIDI note the root may take.
    pub fn bass(&mut self, ch: usize, chords: &str, step: &str, pat: &str, base: i32) {
        self.pattern(ch, chords, step, pat, base, true);
    }

    /// Arpeggio pattern: digits index the chord voiced upwards from `base`.
    pub fn arp(&mut self, ch: usize, chords: &str, step: &str, pat: &str, base: i32) {
        self.pattern(ch, chords, step, pat, base, false);
    }

    /// Sustained chords voiced upwards from `base`, one per chord symbol.
    pub fn pad(&mut self, ch: usize, chords: &str, base: i32) {
        let chords = self.parse_chords(chords);
        let vel = self.chans[ch].vel * 0.7;
        let mut tick = self.chans[ch].cursor;
        for c in chords {
            if !c.none {
                let (voiced, nv) = c.voiced(base);
                for &n in &voiced[..nv] {
                    self.emit(ch, tick, c.dur, n, vel);
                }
            }
            tick += c.dur;
        }
        self.chans[ch].cursor = tick;
    }

    /// Drum steps: one character per step of length `step` (see the kit).
    pub fn beat(&mut self, ch: usize, step: &str, pat: &str) {
        let (step_len, _) = read_len(step.as_bytes(), 0, PPQ);
        let vol = self.chans[ch].vol;
        let send = self.chans[ch].send;
        let spt = 60.0 / (self.bpm * PPQ as f32);
        let mut tick = self.chans[ch].cursor;
        for c in pat.chars() {
            match c {
                ' ' | '\n' => continue,
                '|' => {
                    if !(tick - self.bar_origin.min(tick)).is_multiple_of(self.bar) {
                        let bars = tick as f32 / self.bar as f32;
                        self.err(format!("ch{ch}: drum bar line at {bars:.3} bars"));
                    }
                    continue;
                }
                '.' | '-' => {}
                _ => match (self.kit)(c) {
                    Some(hit) => {
                        let idx = match self.kit_idx.iter().find(|(k, _)| *k == c) {
                            Some(&(_, i)) => i,
                            None => {
                                self.patches.push(hit.patch);
                                let i = (self.patches.len() - 1) as u16;
                                self.kit_idx.push((c, i));
                                i
                            }
                        };
                        let h = 1.0 + (self.rand() - 0.5) * 0.12;
                        self.events.push(Event {
                            tick,
                            ch: ch as u8,
                            dur: ((hit.len / spt) as u32).max(1),
                            note: hit.note,
                            vel: hit.vel * vol * h,
                            patch: idx,
                            pan: hit.pan,
                            send,
                        });
                    }
                    None => self.err(format!("unknown drum `{c}`")),
                },
            }
            tick += step_len;
        }
        self.chans[ch].cursor = tick;
    }
}

fn parse_chord(sym: &str) -> Option<Chord> {
    if sym == "x" {
        return Some(Chord {
            none: true,
            root: 0,
            third: 4,
            fifth: 7,
            seventh: None,
            ninth: None,
            bass: 0,
            dur: 0,
        });
    }
    let (main, slash) = match sym.split_once('/') {
        Some((a, b)) => (a, Some(b)),
        None => (sym, None),
    };
    let (root, rest) = parse_note_name(main)?;
    let (third, fifth, seventh, ninth) = match rest {
        "" => (4, 7, None, None),
        "m" => (3, 7, None, None),
        "7" => (4, 7, Some(10), None),
        "maj7" | "M7" => (4, 7, Some(11), None),
        "m7" => (3, 7, Some(10), None),
        "mM7" => (3, 7, Some(11), None),
        "dim" => (3, 6, None, None),
        "dim7" => (3, 6, Some(9), None),
        "m7b5" => (3, 6, Some(10), None),
        "aug" | "+" => (4, 8, None, None),
        "sus4" | "sus" => (5, 7, None, None),
        "sus2" => (2, 7, None, None),
        "7sus4" => (5, 7, Some(10), None),
        "6" => (4, 7, Some(9), None),
        "m6" => (3, 7, Some(9), None),
        "9" => (4, 7, Some(10), Some(14)),
        "m9" => (3, 7, Some(10), Some(14)),
        "maj9" => (4, 7, Some(11), Some(14)),
        "add9" => (4, 7, None, Some(14)),
        "madd9" => (3, 7, None, Some(14)),
        "5" => (7, 7, None, None),
        _ => return None,
    };
    let bass = match slash {
        Some(b) => parse_note_name(b)?.0,
        None => root,
    };
    Some(Chord {
        none: false,
        root,
        third,
        fifth,
        seventh,
        ninth,
        bass,
        dur: 0,
    })
}

fn parse_note_name(s: &str) -> Option<(i32, &str)> {
    let b = s.as_bytes();
    let mut pc = match b.first()? {
        b'C' => 0,
        b'D' => 2,
        b'E' => 4,
        b'F' => 5,
        b'G' => 7,
        b'A' => 9,
        b'B' => 11,
        _ => return None,
    };
    let mut i = 1;
    while i < b.len() && (b[i] == b'#' || b[i] == b'b') {
        pc += if b[i] == b'#' { 1 } else { -1 };
        i += 1;
    }
    Some((pc, &s[i..]))
}

fn context(s: &[u8], i: usize) -> String {
    let a = i.saturating_sub(24);
    let b = (i + 8).min(s.len());
    String::from_utf8_lossy(&s[a..b]).into_owned()
}

fn read_int(s: &[u8], mut i: usize) -> (Option<i32>, usize) {
    let mut neg = false;
    if i + 1 < s.len() && s[i] == b'-' && s[i + 1].is_ascii_digit() {
        neg = true;
        i += 1;
    }
    let start = i;
    let mut n: i32 = 0;
    while i < s.len() && s[i].is_ascii_digit() {
        n = n.saturating_mul(10).saturating_add((s[i] - b'0') as i32);
        i += 1;
    }
    if i == start {
        (None, i)
    } else {
        (Some(if neg { -n } else { n }), i)
    }
}

/// Reads `<digits><dots>`; without digits the default length is used.
fn read_len(s: &[u8], i: usize, default: u32) -> (u32, usize) {
    let (base, mut j) = match read_int(s, i) {
        (Some(n), j) if n > 0 => ((4 * PPQ) / n as u32, j),
        (_, j) => (default, j),
    };
    let mut add = base / 2;
    let mut len = base;
    while j < s.len() && s[j] == b'.' {
        len += add;
        add /= 2;
        j += 1;
    }
    (len, j)
}

fn read_pitch(c: char, s: &[u8], mut i: usize) -> (i32, usize) {
    let mut pc = match c {
        'c' => 0,
        'd' => 2,
        'e' => 4,
        'f' => 5,
        'g' => 7,
        'a' => 9,
        _ => 11,
    };
    while i < s.len() {
        match s[i] {
            b'+' | b'#' => pc += 1,
            b'-' => pc -= 1,
            _ => break,
        }
        i += 1;
    }
    (pc, i)
}

/// Expands `( ... )n` groups (nesting allowed).
fn expand_repeats(text: &str) -> Result<String, String> {
    fn inner(s: &[u8], i: &mut usize, depth: usize) -> Result<String, String> {
        let mut out = String::new();
        while *i < s.len() {
            let c = s[*i];
            *i += 1;
            match c {
                b'(' => {
                    let body = inner(s, i, depth + 1)?;
                    let (n, j) = read_int(s, *i);
                    *i = j;
                    let n = n.unwrap_or(2).max(1) as usize;
                    for _ in 0..n {
                        out.push_str(&body);
                        out.push(' ');
                    }
                }
                b')' => {
                    if depth == 0 {
                        return Err("unbalanced `)`".into());
                    }
                    return Ok(out);
                }
                _ => out.push(c as char),
            }
        }
        if depth > 0 {
            return Err("unclosed `(`".into());
        }
        Ok(out)
    }
    let mut i = 0;
    inner(text.as_bytes(), &mut i, 0)
}

// ----------------------------------------------------------------------
// Playback
// ----------------------------------------------------------------------

/// Plays a `Song` in real time on its own voices and echo.
pub struct Player {
    song: Option<Box<Song>>,
    rate: f32,
    spt: f64,
    pos: f64,
    next: usize,
    loop_idx: usize,
    loop_start_s: f64,
    end_s: f64,
    done: bool,
    voices: VoicePool,
    echo: Echo,
    send_l: Vec<f32>,
    send_r: Vec<f32>,
    /// Consecutive silent samples since the sequence ended (one-shots).
    quiet: u32,
}

impl Player {
    pub fn new(rate: f32, voices: usize, max_block: usize) -> Player {
        Player {
            song: None,
            rate,
            spt: 1.0,
            pos: 0.0,
            next: 0,
            loop_idx: 0,
            loop_start_s: 0.0,
            end_s: 0.0,
            done: true,
            voices: VoicePool::new(voices),
            echo: Echo::new(rate, 1.5),
            send_l: vec![0.0; max_block],
            send_r: vec![0.0; max_block],
            quiet: u32::MAX,
        }
    }

    /// Starts a song from the top; returns the previous one for disposal.
    pub fn load(&mut self, song: Box<Song>) -> Option<Box<Song>> {
        self.spt = song.tick_seconds() * self.rate as f64;
        self.pos = 0.0;
        self.next = 0;
        self.loop_idx = song.events.partition_point(|e| e.tick < song.loop_start);
        self.loop_start_s = song.loop_start as f64 * self.spt;
        self.end_s = song.end as f64 * self.spt;
        self.done = song.events.is_empty() && !song.looping;
        self.voices.clear();
        self.echo.clear();
        self.echo.set(&song.echo, self.rate);
        self.quiet = 0;
        self.song.replace(song)
    }

    /// True once a one-shot song, its voices and its echo have died away.
    pub fn finished(&self) -> bool {
        self.song.is_none() || (self.done && self.quiet > (self.rate * 0.25) as u32)
    }

    /// Adds the next `out_l.len()` samples into the output buffers.
    pub fn render(&mut self, out_l: &mut [f32], out_r: &mut [f32]) {
        let n = out_l.len().min(out_r.len()).min(self.send_l.len());
        let Some(song) = self.song.as_ref() else {
            return;
        };
        self.send_l[..n].fill(0.0);
        self.send_r[..n].fill(0.0);
        let mut i = 0;
        while i < n {
            let mut run = n - i;
            if !self.done {
                while self.next < song.events.len()
                    && song.events[self.next].tick as f64 * self.spt <= self.pos
                {
                    let e = &song.events[self.next];
                    let patch = &song.patches[e.patch as usize];
                    let on = NoteOn {
                        note: e.note,
                        vel: e.vel * song.gain,
                        hold: (e.dur as f64 * self.spt) as u32,
                        pan: e.pan,
                        send: e.send,
                    };
                    self.voices.start(patch, &on, self.rate);
                    self.next += 1;
                }
                let next_t = if self.next < song.events.len() {
                    (song.events[self.next].tick as f64 * self.spt).min(self.end_s)
                } else {
                    self.end_s
                };
                let until = (next_t - self.pos).ceil().max(1.0) as usize;
                run = run.min(until);
            }
            self.voices.render(
                self.rate,
                &mut out_l[i..i + run],
                &mut out_r[i..i + run],
                &mut self.send_l[i..i + run],
                &mut self.send_r[i..i + run],
            );
            self.pos += run as f64;
            i += run;
            if !self.done && self.pos >= self.end_s {
                if song.looping && self.end_s > self.loop_start_s {
                    self.pos -= self.end_s - self.loop_start_s;
                    self.next = self.loop_idx;
                } else {
                    self.done = true;
                }
            }
        }
        self.echo.process(
            &self.send_l[..n],
            &self.send_r[..n],
            &mut out_l[..n],
            &mut out_r[..n],
        );
        if self.done {
            let peak = out_l[..n]
                .iter()
                .chain(out_r[..n].iter())
                .fold(0.0f32, |m, x| m.max(x.abs()));
            if peak < 1e-4 && self.voices.active() == 0 {
                self.quiet = self.quiet.saturating_add(n as u32);
            } else {
                self.quiet = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::patches::{LEAD, kit};

    fn notes(song: &Song) -> Vec<(u32, u32, i32)> {
        song.events
            .iter()
            .map(|e| (e.tick, e.dur, e.note as i32))
            .collect()
    }

    #[test]
    fn mml_lengths_ties_chords_repeats() {
        let mut b = Builder::new("t", 120.0, 4, 4, kit);
        let c = b.ch(LEAD, 1.0, 0.0, 0.0);
        b.mml(
            c,
            "o4 l8 c d4. [c e g]4 r4 | e4^8 (f16)2 c12 c12 c12 r8 >g8 |",
        );
        let song = b.build();
        assert!(song.errors.is_empty(), "{:?}", song.errors);
        let g = |raw: u32| (raw as f32 * 0.9) as u32;
        assert_eq!(
            notes(&song),
            vec![
                (0, g(24), 60),
                (24, g(72), 62),
                (96, g(48), 60),
                (96, g(48), 64),
                (96, g(48), 67),
                (192, g(72), 64),
                (264, g(12), 65),
                (276, g(12), 65),
                (288, g(16), 60),
                (304, g(16), 60),
                (320, g(16), 60),
                (360, g(24), 79),
            ]
        );
        assert_eq!(song.end, 384);
    }

    #[test]
    fn bar_checks_catch_mistakes() {
        let mut b = Builder::new("t", 120.0, 3, 4, kit);
        let c = b.ch(LEAD, 1.0, 0.0, 0.0);
        b.mml(c, "c4 d4 | e2. |");
        let errors = b.build().errors;
        // Misplaced bar lines (twice), a line ending mid-bar, the song ending mid-bar.
        assert_eq!(errors.len(), 4, "{errors:?}");
        assert!(errors[0].contains("bar line at 0.667"), "{errors:?}");
    }

    #[test]
    fn chord_patterns_follow_the_progression() {
        let mut b = Builder::new("t", 120.0, 4, 4, kit);
        let c = b.ch(LEAD, 1.0, 0.0, 0.0);
        b.gate(c, 1.0);
        b.bass(c, "C F:2 G7/B:2", "4", "1 3 5 -", 48);
        let song = b.build();
        assert!(song.errors.is_empty(), "{:?}", song.errors);
        assert_eq!(
            notes(&song),
            vec![
                (0, 48, 48),
                (48, 48, 52),
                (96, 96, 55),
                (192, 48, 53),
                (240, 48, 57),
                (288, 48, 55),
                (336, 48, 59)
            ]
        );
    }

    #[test]
    fn voicings_avoid_semitone_clusters() {
        let dm9 = parse_chord("Dm9").unwrap();
        let (v, n) = dm9.voiced(50);
        assert_eq!(&v[..n], &[50, 53, 57, 60, 64]);
        let emaj7 = parse_chord("Emaj7").unwrap();
        let (v, n) = emaj7.voiced(62);
        assert!(v[..n].windows(2).all(|w| w[1] - w[0] > 1), "{:?}", &v[..n]);
        assert_eq!(parse_chord("F#/A#").unwrap().bass, 10);
        assert!(parse_chord("Hm").is_none());
    }
}
