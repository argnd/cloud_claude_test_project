//! The soundtrack: sixteen pieces written in the tracker notation of `seq`.
//!
//! One leitmotif ties the game together: "Lira's Lullaby", the song a grieving
//! father sang to his daughter three hundred years ago. It is the Title theme,
//! comes back as a bittersweet Ending and a triumphant Dawn, hides in the Story
//! underscore and the Game Over sting, and is twisted into a minor-key battle
//! theme for the final fight.

use super::Track;
use super::patches::*;
use super::seq::{Builder, Song};
use super::synth::Patch;

// ------------------------------------------------------------------ the leitmotif

/// "Lira's Lullaby", first strain (3/4, written in C; songs transpose with
/// `Builder::key`). Bar 6 is the hopeful natural sixth...
const LULLABY_A: &str =
    "e4. g8 >c4 | <b2 g4 | a4. g8 f4 | e2. | f4. a8 >d4 | c2 <a4 | g4. e8 d4 | c2. |";
/// ...or the grieving flat sixth.
const LULLABY_A_SAD: &str =
    "e4. g8 >c4 | <b2 g4 | a4. g8 f4 | e2. | f4. a8 >d4 | c2 <a-4 | g4. e8 d4 | c2. |";
/// The second strain: climbs to the peak, turns towards the relative minor.
const LULLABY_B: &str =
    "a4. b8 >c4 | e2 d4 | c4. <b8 a4 | g+2. | f4. g8 a4 | g2 e4 | f4. e8 d4 | d2. |";
/// Chords for the strains (one per bar of 3/4).
const LULL_CHORDS: &str = "C Em F C Dm F C/G:2 G7:1 C";
const LULL_CHORDS_SAD: &str = "C Em F Am Dm Fm C/G:2 G7:1 C";
const LULL_B_CHORDS: &str = "Am Em7 F E Dm C/E Dm7 G7sus4:2 G7:1";

/// The lullaby broadened to 4/4 (Dawn).
const LULLABY_A4: &str =
    "e4. g8 >c2 | <b2. g4 | a4. g8 f2 | e1 | f4. a8 >d2 | c2. <a4 | g4. e8 d2 | c1 |";
const LULLABY_B4: &str =
    "a4. b8 >c2 | e2. d4 | c4. <b8 a2 | g+1 | f4. g8 a2 | g2. e4 | f4. e8 d2 | d1 |";
const LULL4_CHORDS: &str = "C Em F C/E Dm F C/G:2 G7:2 C";
const LULL4_B_CHORDS: &str = "Am Em7 F E Dm C/E Dm7 G7sus4:2 G7:2";

/// The lullaby in harmonic minor at battle speed (Final Boss; C-minor reference).
const LULLABY_MINOR: &str =
    "e-4. g8 >c2 | <b2. g4 | a-4. g8 f2 | e-1 | f4. a-8 >d2 | c2. <a-4 | g4. e-8 d2 | c1 |";
const LULL_MINOR_CHORDS: &str = "Cm G/B Fm Cm/Eb Dm7b5 Ab Cm/G:2 G7:2 Cm";
const LULLABY_MINOR_B: &str =
    "a-4. b-8 >c2 | e-2. d4 | c4. <b-8 a-2 | g1 | f4. g8 a-2 | g2. e-4 | f4. e-8 d2 | d1 |";
const LULL_MINOR_B_CHORDS: &str = "Ab Cm/Eb Fm G Fm Cm/G Dm7b5 G";

pub fn build(track: Track) -> Song {
    match track {
        Track::Title => title(),
        Track::Town => town(),
        Track::TownSorrow => town_sorrow(),
        Track::Undercroft => undercroft(),
        Track::Archive => archive(),
        Track::Hollows => hollows(),
        Track::Forge => forge(),
        Track::PaleReach => pale_reach(),
        Track::Battle => battle(),
        Track::Boss => boss(),
        Track::FinalBoss => final_boss(),
        Track::Victory => victory(),
        Track::GameOver => game_over(),
        Track::Story => story(),
        Track::Ending => ending(),
        Track::Dawn => dawn(),
    }
}

/// Length of a one-shot track in seconds (for replay bookkeeping).
pub fn one_shot_seconds(track: Track) -> f32 {
    build(track).length_seconds() as f32 + 0.5
}

/// A copy of `p` with a different overall gain.
const fn with_gain(p: Patch, gain: f32) -> Patch {
    Patch { gain, ..p }
}

// ------------------------------------------------------------------ Title

fn title() -> Song {
    let mut s = Builder::new("title", 76.0, 3, 4, kit);
    s.key(2); // D major
    s.echo(0.75, 1.0, 0.35, 0.4, 3000.0, 0.3, true);
    let bell = s.ch(BELL, 0.8, -0.25, 0.45);
    let lead = s.ch(STRINGS, 0.8, 0.1, 0.3);
    let counter = s.ch(FLUTE, 0.45, -0.45, 0.35);
    let harp = s.ch(HARP, 0.45, 0.5, 0.3);
    let horn = s.ch(BRASS, 0.3, -0.2, 0.25);
    let pad = s.ch(PAD, 0.5, 0.0, 0.3);
    let bass = s.ch(BASS, 0.45, 0.0, 0.0);
    let timp = s.drums(0.4, 0.2);
    let cym = s.drums(0.45, 0.4);
    s.gate(harp, 1.2);
    s.gate(bass, 0.97);
    s.gate(horn, 1.0);
    let harp_pat = "1 5 8 3' 8 5";

    // Intro: the harp alone with a hint of the lullaby.
    let intro = "C Am F G";
    s.pad(pad, intro, 55);
    s.bass(harp, intro, "8", harp_pat, 43);
    s.bass(bass, intro, "4", "B - -", 36);
    s.mml(bell, "o5 v55 r2. | r2. | e4. g8 >c4 | <b2. |");
    s.beat(timp, "8", "P..... | ...... | p..... | p..p.p |");
    s.sync();

    // A1: the lullaby on the bell.
    s.pad(pad, LULL_CHORDS, 55);
    s.bass(harp, LULL_CHORDS, "8", harp_pat, 43);
    s.bass(bass, LULL_CHORDS, "4", "B - -", 36);
    s.mml(bell, &format!("o5 v75 {LULLABY_A}"));
    s.beat(
        timp,
        "8",
        "P..... | ...... | ...... | ...... | p..... | ...... | p..... | ...... |",
    );
    s.sync();

    // A2: strings take the tune, the flute answers; the grieving flat six.
    s.pad(pad, LULL_CHORDS_SAD, 55);
    s.bass(harp, LULL_CHORDS_SAD, "8", harp_pat, 43);
    s.bass(bass, LULL_CHORDS_SAD, "4", "B - -", 36);
    s.mml(lead, &format!("o5 v75 {LULLABY_A_SAD}"));
    s.mml(
        counter,
        "o4 v70 g2. | r4 g8 a8 b4 | a2. | r4 a8 b8 >c4< | a2. | r4 a-8 g8 f4 | g2 f4 | e2. |",
    );
    s.beat(
        timp,
        "8",
        "P..... | ...... | p..... | ...... | p..... | p..... | p..p.. | P..... |",
    );
    s.sync();

    // B: the second strain rises; horns and timpani build.
    s.pad(pad, LULL_B_CHORDS, 55);
    s.vel(horn, 0.55);
    s.pad(horn, LULL_B_CHORDS, 50);
    s.bass(harp, LULL_B_CHORDS, "8", harp_pat, 43);
    s.bass(bass, LULL_B_CHORDS, "4", "B - -", 36);
    s.mml(lead, &format!("o5 v82 {LULLABY_B}"));
    s.mml(
        counter,
        "o4 v65 e2. | r4 d8 e8 g4 | a2. | g+2 b4 | a2. | g2. | f2 a4 | g2 f4 |",
    );
    s.beat(
        timp,
        "8",
        "P..... | ...... | p..... | p.p.pp | P..... | ...... | p..... | pppppp |",
    );
    s.beat(
        cym,
        "8",
        "...... | ...... | ...... | ...... | ...... | ...... | ...... | ..w... |",
    );
    s.sync();

    // A3: everything together, then back to the quiet harp.
    s.vel(pad, 0.9);
    s.pad(pad, LULL_CHORDS_SAD, 55);
    s.vel(horn, 0.5);
    s.pad(horn, LULL_CHORDS_SAD, 50);
    s.bass(harp, LULL_CHORDS_SAD, "8", harp_pat, 43);
    s.bass(bass, LULL_CHORDS_SAD, "4", "B - -", 36);
    s.mml(lead, &format!("o5 v90 {LULLABY_A_SAD}"));
    s.mml(bell, &format!("o6 v55 {LULLABY_A_SAD}"));
    s.mml(
        counter,
        "o4 v75 g2. | r4 g8 a8 b4 | a2. | r4 a8 b8 >c4< | a2. | r4 a-8 g8 f4 | g2 f4 | e2. |",
    );
    s.beat(
        timp,
        "8",
        "P..... | ...... | p..... | ...... | P..... | p..... | p..... | ...... |",
    );
    s.beat(
        cym,
        "8",
        "C..... | ...... | ...... | ...... | c..... | ...... | ...... | ...... |",
    );
    s.build()
}

// ------------------------------------------------------------------ Town

const TOWN_A1: &str =
    "d4 g4 b4 | a2 f+4 | g4 e4 <b4 | >d2. | e4 g4 >c4 | <b4. a8 g4 | a4 g4 e4 | f+2. |";
const TOWN_A2: &str =
    "d4 g4 b4 | a4 b8 a8 f+4 | g8 f+8 e4 <b4 | >d2 f4 | e4 g4 >c4 | c4. <b8 a4 | b4 g4 a4 | g2. |";
const TOWN_B: &str = "e8 f+8 g8 a8 b4 | >c4 <b4 g4 | d8 e8 g8 a8 b4 | a2. | e8 f+8 g8 a8 b4 | >e4 d4 c4 | <b4 a4 g4 | f+2 a4 | \
                      g2 e4 | d2 g4 | >c2 <a4 | b2. | >e4 d4 c4 | <b4 a4 g4 | a4 >c4 <f+4 | g2. |";
const TOWN_A1_CHORDS: &str = "G D/F# Em G/D C G/B Am7 D7";
const TOWN_A2_CHORDS: &str = "G D/F# Em G7/D C D7 G/D:2 D7:1 G";
const TOWN_B_CHORDS: &str = "Em C G D Em C Am7 D7 C G/B Am Em/G C G/D D7 G";

fn town() -> Song {
    let mut s = Builder::new("town", 132.0, 3, 4, kit);
    s.echo(0.5, 0.75, 0.25, 0.28, 3500.0, 0.3, false);
    let flute = s.ch(FLUTE, 0.62, 0.0, 0.25);
    let reed = s.ch(REED, 0.55, -0.35, 0.2);
    let pluck = s.ch(PLUCK, 0.5, 0.45, 0.15);
    let harp = s.ch(HARP, 0.4, -0.5, 0.25);
    let pad = s.ch(WARM_PAD, 0.32, 0.0, 0.2);
    let bass = s.ch(BASS, 0.55, 0.0, 0.0);
    let d1 = s.drums(0.4, 0.05);
    let d2 = s.drums(0.35, 0.1);
    s.gate(pluck, 0.8);
    s.gate(harp, 1.2);
    s.gate(bass, 0.8);
    let comp = "r C C";
    let a_all = format!("{TOWN_A1_CHORDS} {TOWN_A2_CHORDS}");

    // Intro (played once).
    let intro = "G Em C D7";
    s.bass(pluck, intro, "4", comp, 55);
    s.bass(bass, intro, "4", "B r r", 38);
    s.beat(d1, "4", &"k.j".repeat(4));
    s.loop_here();

    // A: the flute tune over oom-pah-pah.
    s.bass(pluck, &a_all, "4", comp, 55);
    s.bass(bass, &a_all, "4", "B r r", 38);
    s.mml(flute, &format!("o5 v78 {TOWN_A1} {TOWN_A2}"));
    s.beat(d1, "4", &"kjj".repeat(16));
    s.sync();

    // B: flowing strain; harp and a warm pad join.
    s.bass(pluck, TOWN_B_CHORDS, "4", comp, 55);
    s.bass(harp, TOWN_B_CHORDS, "8", "1 5 8 5 3' 5", 43);
    s.pad(pad, TOWN_B_CHORDS, 55);
    s.bass(bass, TOWN_B_CHORDS, "4", "B r 5", 38);
    s.mml(flute, &format!("o5 v80 {TOWN_B}"));
    s.beat(d1, "8", &"k.....".repeat(16));
    s.beat(
        d2,
        "8",
        &("..n.n.".repeat(7) + "..n.nn" + &"..n.n.".repeat(7) + "..j.jj"),
    );
    s.sync();

    // A': the oboe sings the tune low while the flute descends above it,
    // then the flute takes over with the oboe in thirds.
    s.bass(pluck, &a_all, "4", comp, 55);
    s.bass(harp, &a_all, "8", "1 5 8 5 3' 5", 43);
    s.bass(bass, &a_all, "4", "B r 5", 38);
    s.mml(reed, &format!("o4 v80 {TOWN_A1}"));
    s.mml(
        flute,
        "o5 v60 b2. | a2. | g2. | f+2. | e2. | d2. | c2. | d4 e4 f+4 |",
    );
    s.mml(flute, &format!("o5 v80 {TOWN_A2}"));
    s.mml(reed, "o4 v62 b4 >d4 g4 | f+4 g8 f+8 d4 | e8 d8 <b4 g4 | b2 >d4 | c4 e4 g4 | a4 g4 f+4 | g4 d4 f+4 | <b2. |");
    s.beat(d1, "8", &"k.x.x.".repeat(16));
    s.beat(d2, "8", &("..n.n.".repeat(15) + "..n.jj"));
    s.build()
}

fn town_sorrow() -> Song {
    const A1: &str =
        "d4 g4 b-4 | a2 f+4 | g4 e-4 <b-4 | >d2. | e-4 g4 >c4 | <b-4. a8 g4 | a4 g4 e-4 | f+2. |";
    const A2: &str = "d4 g4 b-4 | a4 b-8 a8 f+4 | g8 f8 e-4 <b-4 | >d2 f4 | e-4 g4 >c4 | c4. <b-8 a4 | b-4 g4 a4 | g2. |";
    const B: &str = "e-8 f8 g8 a8 b-4 | >c4 <b-4 g4 | d8 e-8 g8 a8 b-4 | a2. | e-8 f8 g8 a8 b-4 | >e-4 d4 c4 | <b-4 a4 g4 | f+2 a4 | \
                     g2 e-4 | d2 g4 | >c2 <a4 | b-2. | >e-4 d4 c4 | <b-4 a4 g4 | a4 >c4 <f+4 | g2. |";
    const A1C: &str = "Gm D/F# Eb Gm/D Cm Gm/Bb Am7b5 D7";
    const A2C: &str = "Gm D/F# Eb Gm7/D Cm D7 Gm/D:2 D7:1 Gm";
    const BC: &str = "Ebmaj7 Cm7 Gm D Ebmaj7 Cm Gm/Bb D7 Cm Gm/Bb Am7b5 Eb/G Cm Gm/D D7 Gm";

    let mut s = Builder::new("town_sorrow", 84.0, 3, 4, kit);
    s.echo(0.75, 1.0, 0.4, 0.4, 2500.0, 0.35, true);
    let music_box = s.ch(CELESTA, 1.0, 0.2, 0.45);
    let flute = s.ch(FLUTE, 0.55, -0.1, 0.35);
    let reed = s.ch(REED, 0.5, 0.05, 0.35);
    let harp = s.ch(HARP, 0.42, -0.45, 0.35);
    let pad = s.ch(GLASS_PAD, 0.42, 0.0, 0.35);
    let strings = s.ch(PAD, 0.4, 0.2, 0.3);
    let bass = s.ch(BASS, 0.35, 0.0, 0.0);
    let timp = s.drums(0.4, 0.3);
    s.gate(harp, 1.4);
    s.gate(bass, 0.97);

    // Two bars of harp to breathe between repeats.
    s.bass(harp, "Gm Gm", "8", "1 5 8 r 3' r", 43);
    s.bass(bass, "Gm Gm", "4", "B - -", 36);
    s.sync();

    s.bass(harp, A1C, "8", "1 5 8 r r r", 43);
    s.pad(pad, A1C, 55);
    s.bass(bass, A1C, "4", "B - -", 36);
    s.mml(music_box, &format!("o5 v70 {A1}"));
    s.sync();

    s.bass(harp, A2C, "8", "1 5 8 r 3' r", 43);
    s.pad(strings, A2C, 55);
    s.bass(bass, A2C, "4", "B - -", 36);
    s.mml(flute, &format!("o5 v72 {A2}"));
    s.beat(timp, "8", &("......".repeat(7) + "p....."));
    s.sync();

    s.bass(harp, BC, "8", "1 5 8 r 3' 5", 43);
    s.pad(pad, BC, 55);
    s.bass(bass, BC, "4", "B - -", 36);
    s.mml(reed, &format!("o4 v75 {B}"));
    s.mml(music_box, "o6 v40 r2. | r2. | r2. | r4 <a4 f+4 | r2. | r2. | r2. | r4 >c4 <a4 | r2. | r2. | r2. | r4 g4 b-4 | r2. | r2. | r2. | r2. |");
    s.beat(timp, "8", &("......".repeat(15) + "p.p..."));
    s.build()
}

// ------------------------------------------------------------------ Undercroft

fn undercroft() -> Song {
    let mut s = Builder::new("undercroft", 92.0, 4, 4, kit);
    s.gain(1.5);
    s.echo(0.75, 1.0, 0.45, 0.35, 1800.0, 0.4, true);
    let lead = s.ch(with_gain(SQUARE, 0.32), 0.62, -0.1, 0.45);
    let bell = s.ch(CELESTA, 0.6, 0.5, 0.5);
    let bass = s.ch(SAW_BASS, 0.62, 0.0, 0.1);
    let pad = s.ch(WARM_PAD, 0.28, 0.0, 0.3);
    let drips = s.drums(0.55, 0.6);
    let tick = s.drums(0.4, 0.3);
    s.gate(bass, 0.7);

    let drip_a = "....d.......D... ..........d..... ......D.....d... ................";
    let drip_b = "..d.........D... ....D.........d. ........d..D.... ..d.............";
    let ticks = "....x.......x... ....x.......x.x.";
    let bass_pat = "1 r r 5, r 1 r r";

    // A: the cellar breathes.
    let a = "Am Am F E Am Am Bb E";
    s.bass(bass, a, "8", bass_pat, 40);
    s.pad(pad, a, 52);
    s.beat(drips, "16", &format!("{drip_a} {drip_b}"));
    s.beat(tick, "16", &ticks.repeat(4));
    s.mml(
        bell,
        "o6 v45 r1 | r1 | r1 | r2 r4 g+4 | r1 | r1 | r1 | r2 r4 d4 |",
    );
    s.sync();

    // B: a creeping tune of neighbour notes.
    let b = "Am Am F E Dm Dm Bb E";
    s.bass(bass, b, "8", bass_pat, 40);
    s.pad(pad, b, 52);
    s.beat(drips, "16", &format!("{drip_b} {drip_a}"));
    s.beat(tick, "16", &ticks.repeat(4));
    s.mml(
        lead,
        "o5 v75 r4 e4 f4 e4 | r4 c4 <b4 a4 | r4 a4 b-4 a4 | g+2. r4 | r4 f4 g4 f4 | r4 d4 e4 f4 | >d2 c4 <b-4 | b2. r4 |",
    );
    s.sync();

    // C: the music box remembers something; the lead holds long, strange notes.
    let c = "F F Am Am Dm Dm E E";
    s.bass(bass, c, "8", "1 r r 5, r 1 r 8", 40);
    s.pad(pad, c, 52);
    s.arp(bell, c, "8", "0 2 1 3 2 4 3 r", 69);
    s.beat(drips, "16", &format!("{drip_a} {drip_a}"));
    s.beat(tick, "16", &ticks.repeat(4));
    s.mml(
        lead,
        "o5 v60 a2. g+4 | a1 | c2. <b4 | a1 | f2. e4 | d1 | e1 | g+1 |",
    );
    s.sync();

    // D: only water and the pulse of the bass.
    let d = "Am Am E E";
    s.bass(bass, d, "8", bass_pat, 40);
    s.beat(drips, "16", drip_b);
    s.mml(lead, "o4 v45 r4 e4 f4 e4 | r1 | r4 e4 f4 e4 | r1 |");
    s.build()
}

// ------------------------------------------------------------------ Archive

fn archive() -> Song {
    let mut s = Builder::new("archive", 70.0, 4, 4, kit);
    s.echo(0.75, 1.0, 0.5, 0.45, 1500.0, 0.4, true);
    let bells = s.ch(DEEP_BELL, 0.42, 0.45, 0.55);
    let high = s.ch(CELESTA, 0.3, -0.5, 0.6);
    let lead = s.ch(
        Patch {
            cutoff: 1400.0,
            vib_depth: 0.22,
            vib_rate: 4.2,
            ..SQUARE
        },
        0.55,
        -0.1,
        0.5,
    );
    let choir = s.ch(CHOIR, 0.5, 0.2, 0.45);
    let pad = s.ch(WOBBLE_PAD, 0.38, 0.0, 0.4);
    let bass = s.ch(SUB, 0.38, 0.0, 0.0);
    let bubbles = s.drums(0.4, 0.6);
    s.gate(bells, 1.6);
    s.gate(bass, 1.0);

    let arp = "0 2 4 6 5 3 1 3";
    let bub = "......d.........  ..........D.....  ................  ...d.......d....";

    // A: bells drifting in dark water.
    let a = "Dm9 G/D Dm9 G/D";
    s.arp(bells, a, "8", arp, 62);
    s.pad(pad, "Dm7 G/D Dm7 G/D", 50);
    s.bass(bass, a, "1", "1", 38);
    s.beat(bubbles, "16", bub);
    s.sync();

    // B: a song half-remembered by the drowned shelves.
    let b = "Dm9 G Fmaj7 C/E Dm9 G Am7 Am7";
    s.arp(bells, b, "8", arp, 62);
    s.pad(pad, "Dm7 G F C/E Dm7 G Am7 Am7", 50);
    s.bass(bass, b, "2", "B -", 38);
    s.mml(lead, "o4 v75 a2 e4 f4 | b2. a8 g8 | a2 >c4 e4 | d2. c8 <b8 | a2 e4 f4 | b2 >d4 g4 | e2. d8 c8 | <a1 |");
    s.beat(bubbles, "16", &format!("{bub} {bub}"));
    s.sync();

    // C: the choir answers higher; celesta glints.
    let c = "Fmaj7 G Em7 Am7 Fmaj7 G Am7 Dm9";
    s.arp(bells, c, "8", arp, 62);
    s.pad(pad, "F G Em7 Am7 F G Am7 Dm7", 50);
    s.bass(bass, c, "2", "B -", 38);
    s.mml(
        choir,
        "o5 v70 c4 e4 a2 | g2. d4 | e2 d4 <b4 | >c1 | c4 e4 a2 | b2 a4 g4 | e2. c4 | d1 |",
    );
    s.arp(high, c, "4", "r 4 r 7 r r 5 r", 74);
    s.mml(
        lead,
        "o4 v45 r1 | r1 | r1 | r2 e4 a4 | r1 | r1 | r1 | r2 f4 a4 |",
    );
    s.beat(bubbles, "16", &format!("{bub} {bub}"));
    s.sync();

    // D: the bells alone again.
    let d = "Dm9 G/D Dm9 G/D";
    s.arp(bells, d, "8", "0 2 4 6 5 3 1 r", 62);
    s.pad(pad, "Dm7 G/D Dm7 G/D", 50);
    s.bass(bass, d, "1", "1", 38);
    s.arp(high, d, "4", "r r r 6 r r r r", 74);
    s.beat(bubbles, "16", bub);
    s.build()
}

// ------------------------------------------------------------------ Hollows

fn hollows() -> Song {
    // 7/8 grouped 2+2+3; E lydian with detours.
    let mut s = Builder::new("hollows", 105.0, 7, 8, kit);
    s.echo(0.5, 0.75, 0.4, 0.35, 2600.0, 0.4, true);
    let mallet = s.ch(MALLET, 0.5, 0.45, 0.3);
    let flute = s.ch(FLUTE, 0.65, -0.2, 0.35);
    let bell = s.ch(BELL, 0.45, -0.1, 0.5);
    let pad = s.ch(WOBBLE_PAD, 0.4, 0.0, 0.35);
    let bass = s.ch(BASS, 0.42, 0.0, 0.0);
    let perc = s.drums(0.4, 0.2);
    let pops = s.drums(0.45, 0.6);
    s.gate(bass, 0.9);
    s.gate(mallet, 1.2);

    let ost = "0 2 1 3 2 4 3";
    let groove = "k.x.m.x";
    let pop = ".......  ....D..  .......  ..d....";

    // A: the glowing ostinato.
    let a = "Eadd9 F#/E Eadd9 F#/E C#m7 F#9 Aadd9 B";
    s.arp(mallet, a, "8", ost, 64);
    s.pad(pad, a, 52);
    s.bass(bass, a, "8", "1 - 5 - 8 - 5", 40);
    s.beat(perc, "8", &groove.repeat(8));
    s.beat(pops, "8", &pop.repeat(2));
    s.sync();

    // B: the flute wanders, curious.
    s.arp(mallet, a, "8", ost, 64);
    s.pad(pad, a, 52);
    s.bass(bass, a, "8", "1 - 5 - 8 - 5", 40);
    s.mml(flute, "o4 v75 b4 g+4 a+4. | >c+4 d+4 f+4. | e4 d+4 <b4. | a+4. g+8 f+4. | g+4 b4 >c+4. | <a+4 g+4 f+4. | g+4 a4 >c+4. | <b2 r4. |");
    s.beat(perc, "8", &groove.repeat(8));
    s.beat(pops, "8", &pop.repeat(2));
    s.sync();

    // C: the bell sings over the relative minor.
    let c = "C#m7 G#m7 Aadd9 F#/A# C#m7 G#m7 Aadd9 B";
    s.arp(mallet, c, "8", ost, 64);
    s.pad(pad, c, 52);
    s.bass(bass, c, "8", "1 - 5 - 8 - 5", 40);
    s.mml(bell, "o5 v70 e4 d+4 c+4. | <b4 >d+4 f+4. | e4 c+4 <a4. | a+2 r4. | >e4 d+4 c+4. | <b4 >d+4 g+4. | f+4 e4 c+4. | d+2 r4. |");
    s.mml(
        flute,
        "o4 v50 r2 r4. | r2 r4. | r2 r4. | r4 c+4 e4. | r2 r4. | r2 r4. | r2 r4. | r4 f+4 a4. |",
    );
    s.beat(perc, "8", &("k.x.m.x".repeat(3) + "k.x.muu").repeat(2));
    s.beat(pops, "8", &pop.repeat(2));
    s.sync();

    // D: spores drifting (no drums).
    let d = "Eadd9 F#/E Eadd9 F#/E";
    s.arp(mallet, d, "8", "0 r 1 r 2 r 4", 64);
    s.pad(pad, d, 52);
    s.bass(bass, d, "8", "1 - - - - - -", 40);
    s.beat(pops, "8", ".D.....  ...d...  .....D.  ..d..d.");
    s.sync();

    // A': the tune again with a bell descant.
    s.arp(mallet, a, "8", ost, 64);
    s.pad(pad, a, 52);
    s.bass(bass, a, "8", "1 - 5 - 8 - 5", 40);
    s.mml(flute, "o4 v80 b4 g+4 a+4. | >c+4 d+4 f+4. | e4 d+4 <b4. | a+4. g+8 f+4. | g+4 b4 >c+4. | <a+4 g+4 f+4. | g+4 a4 >c+4. | <b2 r4. |");
    s.mml(
        bell,
        "o6 v45 d+2 r4. | c+2 r4. | <b2 r4. | a+2 r4. | g+2 r4. | a+2 r4. | a2 r4. | f+2 r4. |",
    );
    s.beat(perc, "8", &groove.repeat(8));
    s.beat(pops, "8", &pop.repeat(2));
    s.build()
}

// ------------------------------------------------------------------ Forge

fn forge() -> Song {
    let mut s = Builder::new("forge", 120.0, 4, 4, kit);
    s.gain(1.35);
    s.echo(0.75, 0.5, 0.3, 0.25, 2200.0, 0.3, true);
    let brass = s.ch(BRASS, 0.85, -0.15, 0.25);
    let lead = s.ch(LEAD, 0.6, 0.2, 0.25);
    let horns = s.ch(with_gain(BRASS, 0.26), 0.5, 0.45, 0.2);
    let bass = s.ch(HEAVY_BASS, 0.6, 0.0, 0.05);
    let organ = s.ch(with_gain(ORGAN, 0.25), 0.3, -0.45, 0.2);
    let d1 = s.drums(0.5, 0.1);
    let anvil = s.drums(0.5, 0.35);
    let hat = s.drums(0.35, 0.1);
    s.gate(bass, 0.85);
    s.gate(horns, 0.5);

    let riff = "1! 1 r 1 3 1 4 5";
    let kick = "k...s..kk...s... k...s..kk...s.s.";
    let clang = "A.....a...a...a. A.....a...a.aa.a";
    let hats = "h.h.h.h.h.h.h.h.";

    // Intro: hammer and bellows.
    s.bass(bass, "Dm Dm", "8", riff, 38);
    s.beat(anvil, "16", clang);
    s.sync();

    // A: the riff.
    let a = "Dm Dm Bb C Dm Dm Gm A";
    s.bass(bass, a, "8", riff, 38);
    s.pad(organ, a, 50);
    s.beat(d1, "16", &kick.repeat(4));
    s.beat(anvil, "16", &clang.repeat(4));
    s.beat(hat, "16", &hats.repeat(8));
    s.sync();

    // B: the brass hymn of the old smiths.
    s.bass(bass, a, "8", riff, 38);
    s.bass(horns, a, "8", "r C r r C r C r", 50);
    s.mml(brass, "o4 v85 d4. e8 f4 a4 | g4 f8 e8 d2 | f4. g8 a4 >d4 | c2 <a4 g4 | d4. e8 f4 a4 | >d4 c8 <b-8 a2 | b-4 a8 g8 f4 e4 | e2 c+2 |");
    s.beat(d1, "16", &kick.repeat(4));
    s.beat(anvil, "16", &clang.repeat(4));
    s.beat(hat, "16", &hats.repeat(8));
    s.sync();

    // C: sparks fly: the lead runs over a climbing progression.
    let c = "Bb C Dm Dm Bb C A A";
    s.bass(bass, c, "8", riff, 38);
    s.pad(organ, c, 50);
    s.mml(
        lead,
        "o5 v80 f8 g8 a8 b-8 a4 f4 | g8 a8 b-8 >c8 <b-4 g4 | a8 b-8 >c8 d8 c4 <a4 | >d2. r4 | \
         d8 c8 <b-8 a8 b-4 >d4 | c8 <b-8 a8 g8 a4 >c4 | c+4 <a4 e4 g4 | a2 r2 |",
    );
    s.beat(d1, "16", &kick.repeat(3));
    s.beat(d1, "16", "k...s..kk...s... k.k.s.k.s.s.ssSS");
    s.beat(anvil, "16", &clang.repeat(4));
    s.beat(hat, "16", &hats.repeat(8));
    s.sync();

    // D: the anvils speak alone, the bass keeps time.
    let d = "Dm Dm Dm A";
    s.bass(bass, d, "8", "1! r r 1 r r 1 r", 38);
    s.beat(
        anvil,
        "16",
        "A..a..a.A..a.a.. a..a..A.a..a.aa. A..a..a.A..a.a.. A.a.A.a.AaAaAAAA",
    );
    s.beat(
        d1,
        "16",
        "t.......t....... t.......t....... t.......t....... t...m...u.m.t.t.",
    );
    s.sync();

    // B': the hymn again, doubled by the lead an octave up.
    s.bass(bass, a, "8", riff, 38);
    s.bass(horns, a, "8", "r C r r C r C r", 50);
    let hymn = "d4. e8 f4 a4 | g4 f8 e8 d2 | f4. g8 a4 >d4 | c2 <a4 g4 | d4. e8 f4 a4 | >d4 c8 <b-8 a2 | b-4 a8 g8 f4 e4 | e2 c+2 |";
    s.mml(brass, &format!("o4 v90 {hymn}"));
    s.mml(lead, &format!("o5 v55 {hymn}"));
    s.beat(d1, "16", &kick.repeat(4));
    s.beat(anvil, "16", &clang.repeat(4));
    s.beat(hat, "16", &"h.h.h.hoh.h.h.ho".repeat(8));
    s.build()
}

// ------------------------------------------------------------------ Pale Reach

fn pale_reach() -> Song {
    let mut s = Builder::new("pale_reach", 54.0, 4, 4, kit);
    s.echo(1.0, 1.5, 0.55, 0.45, 2500.0, 0.5, true);
    let lead = s.ch(
        Patch {
            noise: 0.03,
            vib_depth: 0.1,
            ..FLUTE
        },
        0.5,
        -0.15,
        0.6,
    );
    let glass = s.ch(GLASS_PAD, 0.4, 0.0, 0.5);
    let ice = s.ch(CELESTA, 0.3, 0.55, 0.7);
    let bell = s.ch(BELL, 0.3, -0.5, 0.7);
    let drone = s.ch(SUB, 0.36, 0.0, 0.0);
    let wind = s.drums(0.35, 0.4);
    s.gate(drone, 1.0);

    let twinkle = "r r 4 r r r 2 r r 5 r r r r r r";
    let gusts = "................ ........w....... ................ ................";

    // A: cold air.
    let a = "Bmadd9 Gmaj7 Bmadd9 Gmaj7 Em9 F#sus4";
    s.pad(glass, a, 54);
    s.bass(drone, a, "1", "1", 35);
    s.arp(ice, a, "8", twinkle, 78);
    s.beat(wind, "16", gusts);
    s.beat(wind, "16", "................ ............w...");
    s.mml(bell, "o5 r1 | r1 | r1 | r2 r4 f+4 | r1 | r1 |");
    s.sync();

    // B: a lone voice.
    let b = "Bmadd9 Gmaj7 Dmaj7 Aadd9 Em9 F#sus4:2 F#:2";
    s.pad(glass, "Bmadd9 Gmaj7 Dmaj7 Aadd9 Em7 F#sus4:2 F#:2", 54);
    s.bass(drone, b, "1", "1", 35);
    s.arp(ice, b, "8", twinkle, 78);
    s.mml(
        lead,
        "o5 v75 f+2. e4 | d2 c+4 <b4 | a2 >f+2 | e1 | g2 f+4 e4 | f+1 |",
    );
    s.beat(wind, "16", gusts);
    s.beat(wind, "16", "................ ................");
    s.sync();

    // C: the voice climbs and is answered by a far bell.
    let c = "Gmaj7 Aadd9 Bm Bmadd9 Em9 F#sus4:2 F#:2";
    s.pad(glass, "Gmaj7 Aadd9 Bm Bmadd9 Em7 F#sus4:2 F#:2", 54);
    s.bass(drone, c, "1", "1", 35);
    s.arp(ice, c, "8", "r r r r 4 r r r r r 6 r r r r r", 78);
    s.mml(
        lead,
        "o5 v70 b2. a4 | f+2 e4 d4 | d2 c+4 <b4 | b1 | r1 | r1 |",
    );
    s.mml(bell, "o5 r1 | r1 | r1 | r2 b4 f+4 | e2. d4 | r2 c+2 |");
    s.beat(wind, "16", gusts);
    s.beat(wind, "16", "................ ........w.......");
    s.build()
}

// ------------------------------------------------------------------ Battle

fn battle() -> Song {
    let mut s = Builder::new("battle", 150.0, 4, 4, kit);
    s.echo(0.75, 0.5, 0.2, 0.18, 3500.0, 0.3, false);
    let lead = s.ch(LEAD, 0.62, 0.1, 0.2);
    let second = s.ch(with_gain(SQUARE, 0.3), 0.5, -0.45, 0.2);
    let brass = s.ch(BRASS, 0.5, -0.2, 0.15);
    let arp = s.ch(with_gain(PLUCK, 0.28), 0.5, 0.5, 0.2);
    let bass = s.ch(PULSE_BASS, 0.62, 0.0, 0.0);
    let d1 = s.drums(0.62, 0.05);
    let hat = s.drums(0.38, 0.05);
    let cym = s.drums(0.45, 0.15);
    s.gate(bass, 0.75);
    s.gate(arp, 0.7);
    s.gate(brass, 0.6);

    let pump = "1! 8 1 8 1 8 5 8";
    let rock = "k...s...k.k.s... k...s...k.k.s.k.";
    let fill = "k...s...k.k.s... k.s.s.ssS.S.SSSS";
    let hats = "h.h.h.h.h.h.h.h.";

    // Intro (once): the bass runs up, snare roll.
    s.mml(
        bass,
        "o2 e8 e8 >e8 <e8 d8 d8 >d8 <d8 | c8 c8 >c8 <c8 b8 >b8 d+8 f+8 |",
    );
    s.beat(d1, "16", "k.......k....... k...k...s.s.ssss");
    s.beat(cym, "16", "................ ............w...");
    s.loop_here();

    // A: the theme.
    let a = "Em C D Em Em C Am B7";
    let theme = "b4. >e4. d8 e8 | g4. e4. d8 e8 | f+4. d4. <a8 >d8 | e2 r8 e8 f+8 g8 | b4. a4. g8 f+8 | g4. e4. c8 e8 | a4. >c4. <b8 a8 | b2 a4 f+4 |";
    s.bass(bass, a, "8", pump, 40);
    s.arp(arp, a, "16", "0 1 2 3 2 1 0 1", 64);
    s.mml(lead, &format!("o4 v85 {theme}"));
    s.beat(d1, "16", &format!("{}{}", rock.repeat(3), fill));
    s.beat(hat, "16", &hats.repeat(8));
    s.beat(
        cym,
        "16",
        &format!("c...............{}", ".".repeat(16 * 7)),
    );
    s.sync();

    // A': the theme with a harmony a third below and brass stabs.
    s.bass(bass, a, "8", pump, 40);
    s.arp(arp, a, "16", "0 1 2 3 2 1 0 1", 64);
    s.mml(lead, &format!("o4 v90 {theme}"));
    s.mml(second, "o4 v75 g4. b4. a8 b8 | e4. c4. <b8 >c8 | d4. <a4. f+8 a8 | b2 r8 b8 >d8 e8 | g4. f+4. e8 d+8 | e4. c4. <a8 >c8 | e4. a4. g8 e8 | f+2 d+4 <b4 |");
    s.bass(brass, a, "8", "r r r C r r C r", 52);
    s.beat(d1, "16", &format!("{}{}", rock.repeat(3), fill));
    s.beat(hat, "16", &hats.repeat(8));
    s.beat(
        cym,
        "16",
        &format!("c...............{}", ".".repeat(16 * 7)),
    );
    s.sync();

    // B: a brighter, singing strain in the relative major.
    let b = "Cmaj7 D Bm7 Em Am7 D Cmaj7 B7";
    s.bass(bass, b, "8", "1! 5 8 5 1 5 8 5", 40);
    s.pad(brass, b, 55);
    s.arp(arp, b, "16", "0 2 1 3 2 4 3 5", 64);
    s.mml(lead, "o5 v85 e4. g4. b4 | a2. f+4 | d4. f+4. a4 | g2. e4 | c4. e4. g4 | f+2. d4 | e4. g4. b4 | a2 d+2 |");
    s.mml(
        second,
        "o4 v60 g1 | f+1 | f+1 | e1 | e1 | d1 | e1 | f+2 d+2 |",
    );
    s.beat(
        d1,
        "16",
        &format!(
            "{}{}",
            "k.......s.......k.k.....s.......".repeat(3),
            "k.......s.......k.k.s.s.s.ssssss"
        ),
    );
    s.beat(hat, "16", &"h.h.h.hoh.h.h.ho".repeat(8));
    s.beat(
        cym,
        "16",
        &format!("c...............{}", ".".repeat(16 * 7)),
    );
    s.sync();

    // C: stabs and a riff: the drums drive, the lead answers the brass.
    let c = "Em Em D D C C B7 B7";
    s.bass(bass, c, "8", "1! 1 8 1 1 8 1 8", 40);
    s.bass(brass, c, "8", "C! r r C r r C r", 55);
    s.mml(lead, "o5 r2 r8 e8 g8 b8 | >e4 d8 <b8 g4 e4 | r2 r8 d8 f+8 a8 | >d4 c8 <a8 f+4 d4 | r2 r8 c8 e8 g8 | >c4 <b8 g8 e4 c4 | d+4 f+4 b4 >d+4 | f+4 e4 d+4 <b4 |");
    s.beat(
        d1,
        "16",
        &format!("{}{}", "k..k..s.k..k..s.".repeat(7), "s.s.s.s.S.S.SSSS"),
    );
    s.beat(hat, "16", &hats.repeat(8));
    s.beat(
        cym,
        "16",
        &format!(
            "c...............{}c...............{}",
            ".".repeat(16 * 3),
            ".".repeat(16 * 3)
        ),
    );
    s.sync();

    // D: breakdown and build - the bass climbs, the lead holds on, snare rises.
    let d = "Am Bm C D C D B7sus4 B7";
    s.bass(bass, d, "8", "1! 8 1 8 1 8 1 8", 40);
    s.arp(arp, d, "16", "0 2 4 2 1 3 5 3", 64);
    s.mml(
        lead,
        "o4 v80 a1 | b1 | >c1 | d1 | e2 c2 | f+2 d2 | e1 | d+1 |",
    );
    s.mml(
        second,
        "o4 v55 r1 | r1 | r1 | r1 | g2 e2 | a2 f+2 | b1 | b1 |",
    );
    s.rest_bars(brass, 4);
    s.bass(brass, "C D B7sus4 B7", "8", "C! r r C r r C r", 55);
    s.beat(
        d1,
        "16",
        &format!(
            "{}{}k.s.k.s.k.s.s.s.s.s.s.s.ssssSSSS",
            "k.......k.......".repeat(4),
            "k...s...k.k.s...".repeat(2)
        ),
    );
    s.beat(
        hat,
        "16",
        &format!(
            "{}{}",
            "h.h.h.h.h.h.h.h.".repeat(4),
            "hhhhhhhhhhhhhhhh".repeat(4)
        ),
    );
    s.beat(
        cym,
        "16",
        &format!("{}w{}", ".".repeat(116), ".".repeat(11)),
    );
    s.build()
}

// ------------------------------------------------------------------ Boss

fn boss() -> Song {
    let mut s = Builder::new("boss", 160.0, 4, 4, kit);
    s.echo(0.75, 0.5, 0.25, 0.2, 3000.0, 0.3, false);
    let lead = s.ch(LEAD, 0.62, 0.1, 0.2);
    let second = s.ch(with_gain(SQUARE, 0.3), 0.55, -0.45, 0.2);
    let brass = s.ch(BRASS, 0.6, -0.25, 0.15);
    let organ = s.ch(with_gain(ORGAN, 0.3), 0.45, 0.45, 0.2);
    let bass = s.ch(HEAVY_BASS, 0.72, 0.0, 0.0);
    let d1 = s.drums(0.66, 0.05);
    let hat = s.drums(0.36, 0.05);
    let tom = s.drums(0.5, 0.15);
    let cym = s.drums(0.45, 0.2);
    s.gate(bass, 0.7);
    s.gate(brass, 0.55);

    let ostinato = "c8 c8 d-8 c8 c8 d-8 <b8 >c8";
    let heavy = "k.k.s..kk.k.s.k. k.k.s..kk.k.s.ss";
    let hats = "h.h.h.h.h.h.h.h.";

    // Intro (once): gong, timpani, the ostinato wakes.
    s.mml(bass, &format!("o2 {ostinato} | {ostinato} |"));
    s.beat(tom, "16", "P...............P...P...P.P.PPPP");
    s.beat(cym, "16", "g...............................");
    s.loop_here();

    // A: chromatic menace.
    let a_melody = "c4 g4 f+4 g4 | e-4. d8 c4 <b4 | >c4 <a-4 g4 a-4 | >e-4. d-8 c4 <b-4 | >c4 g4 a-4 g4 | b-4. a-8 g4 f+4 | f4 a-4 >d-4 c4 | <b2 >d2 |";
    let a_bass = format!(
        "o2 ({ostinato} |)2 k-4 ({ostinato} |)2 k0 ({ostinato} |)2 d-8 d-8 d8 d-8 d-8 d8 c8 d-8 | <g8 g8 a-8 g8 g8 a-8 f+8 g8 |"
    );
    let a_second = "o4 v65 g1 | g1 | e-1 | a-2 g2 | e-1 | e-2 d2 | f1 | f1 |";
    s.mml(bass, &a_bass);
    s.bass(
        brass,
        "Cm Cm Ab Ab Cm Cm Db G7",
        "8",
        "C! r r C r r r r",
        55,
    );
    s.mml(lead, &format!("o5 v85 {a_melody}"));
    s.beat(d1, "16", &heavy.repeat(4));
    s.beat(hat, "16", &hats.repeat(8));
    s.beat(cym, "16", &format!("c{}", ".".repeat(127)));
    s.sync();

    // A': octave up with the organ and a second voice.
    s.mml(bass, &a_bass);
    s.pad(organ, "Cm Cm Ab Ab Cm Cm Db G7", 55);
    s.mml(lead, &format!("o5 v85 {a_melody}"));
    s.mml(second, a_second);
    s.beat(d1, "16", &heavy.repeat(4));
    s.beat(hat, "16", &hats.repeat(8));
    s.beat(cym, "16", &format!("c{}", ".".repeat(127)));
    s.sync();

    // B: the ground gives way: descending bass, brass cries.
    let b = "Ab G Fm Eb Db C Db G7";
    s.bass(bass, b, "8", "1! 1 8 1 1 8 1 5", 36);
    s.pad(organ, b, 55);
    s.mml(
        brass,
        "o5 v90 c2. e-4 | d2. <b4 | >c2 <a-2 | b-2 g2 | a-2. f4 | g2 e2 | f2 a-2 | b1 |",
    );
    s.mml(lead, "o4 v70 r1 | r2 r8 g8 a-8 b8 | >c1 | r2 r8 e-8 f8 g8 | a-1 | r2 r8 g8 a-8 b-8 | >d-2 c2 | <b1 |");
    s.beat(d1, "16", &"k...s...k.k.s...".repeat(7));
    s.beat(d1, "16", "k.s.s.s.S.S.SSSS");
    s.beat(
        tom,
        "16",
        &format!("{}t.t.m.m.u.u.m.t.", ".".repeat(16 * 7)),
    );
    s.beat(hat, "16", &"h.h.h.hoh.h.h.ho".repeat(8));
    s.sync();

    // C: half-time dread; the lead climbs chromatically.
    let c = "Cm Cm Bbm Bbm Abmaj7 Abmaj7 G7 G7";
    s.bass(bass, c, "4", "1! - 1 5,", 38);
    s.pad(organ, c, 55);
    s.mml(
        lead,
        "o5 v80 g2. a-4 | g2 f+2 | f2. g-4 | f2 e2 | e-1 | c2 e-4 g4 | b1 | >d2 <b2 |",
    );
    s.mml(
        second,
        "o4 v60 e-1 | e-1 | d-1 | d-1 | c1 | c1 | d1 | f2 d2 |",
    );
    s.beat(d1, "16", &"k.......s.......".repeat(6));
    s.beat(d1, "16", "k.......s.......k.k.s.s.s.ssSSSS");
    s.beat(hat, "16", &"h...h...h...h...".repeat(8));
    s.beat(
        cym,
        "16",
        &format!("c{}c{}", ".".repeat(63), ".".repeat(63)),
    );
    s.sync();

    // D: all voices together for the last push.
    s.mml(bass, &a_bass);
    s.bass(
        brass,
        "Cm Cm Ab Ab Cm Cm Db G7",
        "8",
        "C! r C r r C r r",
        55,
    );
    s.mml(lead, &format!("o5 v90 {a_melody}"));
    s.mml(second, a_second);
    s.beat(d1, "16", &heavy.repeat(3));
    s.beat(d1, "16", "k.k.s..kk.k.s.ss k.s.s.s.S.S.SSSS");
    s.beat(hat, "16", &hats.repeat(8));
    s.beat(
        tom,
        "16",
        &format!("{}t.t.m.m.u.u.m.t.", ".".repeat(16 * 7)),
    );
    s.beat(cym, "16", &format!("c{}", ".".repeat(127)));
    s.build()
}

// ------------------------------------------------------------------ Final Boss

fn final_boss() -> Song {
    let mut s = Builder::new("final_boss", 170.0, 4, 4, kit);
    s.gain(1.1);
    s.key(2); // D minor
    s.echo(0.75, 0.5, 0.25, 0.22, 3000.0, 0.3, true);
    let lead = s.ch(LEAD, 0.6, 0.1, 0.2);
    let brass = s.ch(BRASS, 0.65, -0.25, 0.2);
    let choir = s.ch(with_gain(CHOIR, 0.34), 0.5, 0.0, 0.35);
    let bells = s.ch(BELL, 0.6, 0.3, 0.4);
    let arp = s.ch(with_gain(PLUCK, 0.26), 0.5, 0.5, 0.2);
    let bass = s.ch(HEAVY_BASS, 0.7, 0.0, 0.0);
    let d1 = s.drums(0.55, 0.05);
    let hat = s.drums(0.36, 0.05);
    let tom = s.drums(0.52, 0.2);
    let cym = s.drums(0.45, 0.2);
    s.gate(bass, 0.72);
    s.gate(arp, 0.6);
    s.gate(choir, 1.0);

    let drive = "1! 1 8 1 1 8 1 8";
    let beat = "k.k.s..kk.k.s.k. k.k.s..kk.ks.s.s";
    let hats = "h.h.h.h.h.h.h.h.";

    // Intro (once): the lullaby's first notes, sung by a choir in ruins.
    s.pad(choir, "Cm Cm G/B G/B", 50);
    s.mml(bells, "o5 v70 e-4. g8 >c2 | r1 | <b2. g4 | r1 |");
    s.beat(
        tom,
        "16",
        "P............... ................ P............... P...P...P.P.PPPP",
    );
    s.beat(
        cym,
        "16",
        &format!("g{}w{}", ".".repeat(50), ".".repeat(12)),
    );
    s.loop_here();

    // A: the storm - riff, stabs, arpeggios.
    let a = "Cm Cm Ab Bb Cm Cm Ab G";
    s.bass(bass, a, "8", drive, 36);
    s.bass(brass, a, "8", "C! r r C r r C r", 55);
    s.arp(arp, a, "16", "0 1 2 3 4 3 2 1", 60);
    s.mml(lead, "o5 v75 r1 | r2 r8 c8 e-8 g8 | a-4. g8 f4 e-4 | d2 f2 | e-4. d8 c4 <b4 | >c2 e-4 g4 | a-4 g4 f4 e-4 | d2 <b2 |");
    s.beat(d1, "16", &beat.repeat(4));
    s.beat(hat, "16", &hats.repeat(8));
    s.beat(cym, "16", &format!("C{}", ".".repeat(127)));
    s.sync();

    // B: the lullaby, twisted into minor, on lead and brass.
    s.bass(bass, LULL_MINOR_CHORDS, "8", drive, 36);
    s.arp(arp, LULL_MINOR_CHORDS, "16", "0 1 2 3 4 3 2 1", 60);
    s.mml(lead, &format!("o5 v90 {LULLABY_MINOR}"));
    s.mml(brass, &format!("o4 v70 {LULLABY_MINOR}"));
    s.pad(choir, LULL_MINOR_CHORDS, 55);
    s.beat(d1, "16", &beat.repeat(4));
    s.beat(hat, "16", &hats.repeat(8));
    s.beat(cym, "16", &format!("C{}", ".".repeat(127)));
    s.sync();

    // C: the second strain, higher and wilder.
    s.bass(bass, LULL_MINOR_B_CHORDS, "8", drive, 36);
    s.arp(arp, LULL_MINOR_B_CHORDS, "16", "0 2 4 5 4 2 1 3", 60);
    s.mml(lead, &format!("o5 v90 {LULLABY_MINOR_B}"));
    s.bass(brass, LULL_MINOR_B_CHORDS, "8", "C! r r C r r C r", 55);
    s.beat(d1, "16", &beat.repeat(3));
    s.beat(d1, "16", "k.k.s..kk.k.s.s. s.s.s.s.S.S.SSSS");
    s.beat(hat, "16", &hats.repeat(8));
    s.beat(
        tom,
        "16",
        &format!("{}t.t.m.m.u.u.m.t.", ".".repeat(16 * 7)),
    );
    s.sync();

    // D: half time - for eight bars the old tune shines through in major.
    let d = "Ab Cm/G Db Ab/C Bbm Db Ab/Eb:2 Eb7:2 Ab";
    s.bass(bass, d, "4", "1! - 5 -", 36);
    s.mml(bells, "o5 v70 k-4 e4. g8 >c2 | <b2. g4 | a4. g8 f2 | e1 | f4. a8 >d2 | c2. <a4 | g4. e8 d2 | c1 |");
    s.mml(
        choir,
        "o4 v65 k-4 g1 | g1 | a1 | g1 | a1 | a1 | g2 f2 | e1 |",
    );
    s.pad(brass, d, 50);
    s.beat(d1, "16", &"k.......s.......".repeat(7));
    s.beat(d1, "16", "k.s.s.s.S.S.SSSS");
    s.beat(hat, "16", &"h...h...h...h...".repeat(8));
    s.beat(
        cym,
        "16",
        &format!("c{}w{}", ".".repeat(115), ".".repeat(11)),
    );
    s.sync();

    // E: the storm returns; the lullaby in the brass, the lead in counterpoint.
    s.bass(bass, LULL_MINOR_CHORDS, "8", drive, 36);
    s.arp(arp, LULL_MINOR_CHORDS, "16", "0 1 2 3 4 3 2 1", 60);
    s.mml(brass, &format!("o4 v90 {LULLABY_MINOR}"));
    s.mml(
        lead,
        "o5 v70 g2 e-2 | d2. f4 | e-2 c2 | e-1 | f2 a-2 | g2 e-2 | e-2 d2 | e-2 c2 |",
    );
    s.pad(choir, LULL_MINOR_CHORDS, 55);
    s.beat(d1, "16", &beat.repeat(3));
    s.beat(d1, "16", "k.k.s..kk.k.s.s. s.s.s.s.S.S.SSSS");
    s.beat(hat, "16", &hats.repeat(8));
    s.beat(cym, "16", &format!("C{}", ".".repeat(127)));
    s.build()
}

// ------------------------------------------------------------------ Victory / Game Over

fn victory() -> Song {
    let mut s = Builder::new("victory", 132.0, 4, 4, kit);
    s.once();
    s.key(2); // D major
    s.echo(0.5, 0.75, 0.3, 0.3, 4000.0, 0.3, false);
    let ring = |p: Patch| Patch { release: 0.7, ..p };
    let lead = s.ch(ring(LEAD), 0.62, 0.1, 0.25);
    let brass = s.ch(ring(BRASS), 0.7, -0.3, 0.2);
    let bell = s.ch(BELL, 0.3, 0.3, 0.4);
    let bass = s.ch(ring(BASS), 0.45, 0.0, 0.0);
    let d1 = s.drums(0.35, 0.2);
    let cym = s.drums(0.5, 0.3);
    // A rising call, then the home chord held and left to ring.
    s.mml(lead, "o4 v85 q7 l12 g >c e g4 e8. g16 a8 b8 | q7 >c1 |");
    s.mml(
        brass,
        "o4 v75 q7 [c e]4 [c e]4 [c f]4 [d g]4 | q7 [e g >c]1 |",
    );
    s.mml(bell, "o6 v70 r1 | c2 <g2 |");
    s.mml(bass, "o2 q7 c4 c4 f4 g4 | c1 |");
    s.beat(d1, "16", "p...p...p...pppp P...............");
    s.beat(cym, "16", "................ C...............");
    s.build()
}

fn game_over() -> Song {
    let mut s = Builder::new("game_over", 56.0, 3, 4, kit);
    s.once();
    s.echo(0.75, 1.0, 0.45, 0.4, 2000.0, 0.4, true);
    let bell = s.ch(BELL, 0.7, 0.0, 0.5);
    let strings = s.ch(STRINGS, 0.45, -0.1, 0.35);
    let pad = s.ch(PAD, 0.5, 0.0, 0.4);
    let bass = s.ch(BASS, 0.35, 0.0, 0.0);
    let timp = s.drums(0.4, 0.3);
    s.gate(pad, 1.1);
    // The lullaby's last phrase, in minor, falling to rest.
    s.mml(bell, "o5 v75 g4. e-8 d4 | c2. |");
    s.mml(strings, "o4 v60 c4. c8 <b4 | g2. |");
    s.pad(pad, "Abmaj7:2 G7:1 Cm", 50);
    s.mml(bass, "o2 a-2 g4 | c2. |");
    s.beat(timp, "8", "...... P.....");
    s.build()
}

// ------------------------------------------------------------------ Story

fn story() -> Song {
    let mut s = Builder::new("story", 72.0, 4, 4, kit);
    s.gain(0.85);
    s.echo(0.75, 1.0, 0.4, 0.35, 2500.0, 0.35, true);
    let piano = s.ch(EPIANO, 0.6, 0.3, 0.3);
    let pad = s.ch(WARM_PAD, 0.42, 0.0, 0.3);
    let flute = s.ch(FLUTE, 0.45, -0.2, 0.4);
    let celesta = s.ch(CELESTA, 0.6, 0.3, 0.5);
    let harp = s.ch(HARP, 0.4, -0.5, 0.3);
    let bass = s.ch(BASS, 0.35, 0.0, 0.0);
    s.gate(piano, 1.3);
    s.gate(bass, 1.0);

    // A: quiet questions.
    let a = "Dm9 Bbmaj7 Gm9 A7sus4 Dm9 Bbmaj7 Gm9 A7";
    s.pad(pad, a, 53);
    s.arp(piano, a, "4", "0 2 1 3", 57);
    s.bass(bass, a, "1", "1", 38);
    s.mml(
        flute,
        "o5 r1 | r1 | r2 a4 g4 | e1 | r1 | r1 | r2 d4 e4 | c+1 |",
    );
    s.sync();

    // B: something half-remembered.
    let b = "Bbmaj7 F/A Gm7 Dm/F Ebmaj7 Bbmaj7/D A7sus4 A7";
    s.pad(pad, b, 53);
    s.arp(harp, b, "8", "0 2 4 r 5 r 3 r", 50);
    s.bass(bass, b, "2", "B -", 38);
    s.mml(
        flute,
        "o5 d2. c4 | c2 <a2 | b-2. a4 | a1 | g2. b-4 | a2 f2 | e1 | e2 c+2 |",
    );
    s.sync();

    // C: the lullaby's first notes on the celesta, as if from another room.
    let c = "Fadd9 C/E Dm9 A7sus4";
    s.pad(pad, c, 53);
    s.arp(piano, c, "4", "0 2 1 3", 57);
    s.bass(bass, c, "1", "B", 38);
    s.mml(celesta, "o4 v65 a4. >c8 f2 | e1 | r1 | r1 |");
    s.build()
}

// ------------------------------------------------------------------ Ending

fn ending() -> Song {
    let mut s = Builder::new("ending", 66.0, 3, 4, kit);
    s.key(5); // F major
    s.echo(0.75, 1.0, 0.4, 0.4, 2800.0, 0.35, true);
    let music_box = s.ch(CELESTA, 1.0, 0.2, 0.45);
    let strings = s.ch(STRINGS, 0.75, -0.05, 0.3);
    let flute = s.ch(FLUTE, 0.5, -0.45, 0.35);
    let harp = s.ch(HARP, 0.45, 0.45, 0.3);
    let pad = s.ch(PAD, 0.45, 0.0, 0.3);
    let bass = s.ch(BASS, 0.35, 0.0, 0.0);
    let timp = s.drums(0.4, 0.3);
    s.gate(harp, 1.3);
    s.gate(bass, 0.97);
    let harp_pat = "1 5 8 3' 8 5";

    // Intro.
    s.bass(harp, "C F/C", "8", harp_pat, 43);
    s.bass(bass, "C F/C", "4", "B - -", 36);
    s.sync();

    // A1: the music box, grieving.
    s.bass(harp, LULL_CHORDS_SAD, "8", "1 5 8 r r r", 43);
    s.pad(pad, LULL_CHORDS_SAD, 55);
    s.bass(bass, LULL_CHORDS_SAD, "4", "B - -", 36);
    s.mml(music_box, &format!("o5 v75 {LULLABY_A_SAD}"));
    s.sync();

    // B: the strings carry the second strain.
    s.bass(harp, LULL_B_CHORDS, "8", harp_pat, 43);
    s.pad(pad, LULL_B_CHORDS, 55);
    s.bass(bass, LULL_B_CHORDS, "4", "B - -", 36);
    s.mml(strings, &format!("o5 v78 {LULLABY_B}"));
    s.beat(timp, "8", &("......".repeat(7) + "p....."));
    s.sync();

    // A2: strings and flute together - and this time the sixth is bright.
    s.bass(harp, LULL_CHORDS, "8", harp_pat, 43);
    s.pad(pad, LULL_CHORDS, 55);
    s.bass(bass, LULL_CHORDS, "4", "B - -", 36);
    s.mml(strings, &format!("o5 v80 {LULLABY_A}"));
    s.mml(
        flute,
        "o4 v70 g2. | r4 g8 a8 b4 | a2. | r4 a8 b8 >c4< | a2. | r4 a8 g8 f4 | g2 f4 | e2. |",
    );
    s.sync();

    // Outro: a last echo on the music box.
    s.bass(harp, "F/C C", "8", "1 5 8 3' 8 5", 43);
    s.bass(bass, "F/C C", "4", "B - -", 36);
    s.mml(music_box, "o5 v50 r4 e4. g8 | >c2. |");
    s.build()
}

// ------------------------------------------------------------------ Dawn

fn dawn() -> Song {
    let mut s = Builder::new("dawn", 96.0, 4, 4, kit);
    s.key(2); // D major, later E major
    s.echo(0.75, 1.0, 0.3, 0.3, 3500.0, 0.3, true);
    let lead = s.ch(STRINGS, 0.75, 0.05, 0.3);
    let brass = s.ch(BRASS, 0.75, -0.25, 0.25);
    let horns = s.ch(with_gain(BRASS, 0.24), 0.45, 0.25, 0.25);
    let flute = s.ch(FLUTE, 0.5, -0.45, 0.3);
    let choir = s.ch(CHOIR, 0.35, 0.1, 0.35);
    let harp = s.ch(HARP, 0.45, 0.5, 0.3);
    let bass = s.ch(BASS, 0.42, 0.0, 0.0);
    let snare = s.drums(0.5, 0.15);
    let timp = s.drums(0.4, 0.25);
    let cym = s.drums(0.45, 0.3);
    s.gate(harp, 1.2);
    s.gate(bass, 0.9);
    s.gate(horns, 0.9);

    let march = "k.......s.....s.k.....k.s.......";
    let harp_pat = "1 5 8 3' 8 5 8 5";

    // Intro: horns announce the lullaby's opening.
    s.mml(
        brass,
        "o4 v85 e4. g8 >c2 | <b2. g4 | a4 b4 >c4 d4 | e2 d4 <b4 |",
    );
    s.pad(horns, "C Em F G", 50);
    s.bass(bass, "C Em F G", "4", "1 - 5 -", 36);
    s.beat(
        timp,
        "16",
        "P...........p.p. p...............  P...P...P...P... p.p.p.p.pppppppp",
    );
    s.beat(cym, "16", &format!("C{}", ".".repeat(63)));
    s.sync();

    // A: the lullaby as a sunrise.
    s.mml(lead, &format!("o5 v85 {LULLABY_A4}"));
    s.bass(harp, LULL4_CHORDS, "8", harp_pat, 43);
    s.pad(horns, LULL4_CHORDS, 50);
    s.bass(bass, LULL4_CHORDS, "4", "1 - 5 -", 36);
    s.beat(snare, "16", &march.repeat(4));
    s.beat(cym, "16", &format!("c{}", ".".repeat(127)));
    s.sync();

    // B: the second strain, flute descant above.
    s.mml(lead, &format!("o5 v85 {LULLABY_B4}"));
    s.mml(
        flute,
        "o5 v65 e1 | g1 | a1 | b1 | a1 | g1 | a2 f2 | g2 f2 |",
    );
    s.bass(harp, LULL4_B_CHORDS, "8", harp_pat, 43);
    s.pad(horns, LULL4_B_CHORDS, 50);
    s.bass(bass, LULL4_B_CHORDS, "4", "1 - 5 -", 36);
    s.beat(snare, "16", &march.repeat(3));
    s.beat(snare, "16", "k.......s.....s.k.s.s.s.s.ssSSSS");
    s.beat(cym, "16", &format!("{}w.......", ".".repeat(120)));
    s.sync();

    // A': a step higher, everyone singing.
    s.key(4);
    s.mml(lead, &format!("o5 v92 {LULLABY_A4}"));
    s.mml(brass, &format!("o4 v70 {LULLABY_A4}"));
    s.pad(choir, LULL4_CHORDS, 55);
    s.mml(flute, "o5 v60 g1 | g1 | a1 | g1 | a1 | a1 | g2 f2 | e1 |");
    s.bass(harp, LULL4_CHORDS, "8", harp_pat, 43);
    s.bass(bass, LULL4_CHORDS, "4", "1 - 5 -", 36);
    s.beat(snare, "16", &march.repeat(4));
    s.beat(
        timp,
        "16",
        &format!("P{}P{}", ".".repeat(63), ".".repeat(63)),
    );
    s.beat(
        cym,
        "16",
        &format!("C{}c{}", ".".repeat(63), ".".repeat(63)),
    );
    s.sync();

    // Turnaround back home to D (E major -> A7 -> D).
    s.key(2);
    let turn = "D D G7sus4 G7";
    s.pad(choir, turn, 55);
    s.pad(horns, turn, 50);
    s.bass(harp, turn, "8", harp_pat, 43);
    s.bass(bass, turn, "4", "1 - 5 -", 36);
    s.mml(lead, "o5 v80 f+4. e8 d2 | a1 | g4 f4 d4 c4 | <b2 >d2 |");
    s.beat(
        timp,
        "16",
        "P............... ................ p...p...p...p... p.p.p.p.pppppppp",
    );
    s.build()
}
