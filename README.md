# Emberdeep — The Last Lantern

A story-driven, turn-based fantasy RPG in Rust. Twenty floors beneath a
mountain village, a lamplighter goes looking for the sister who went down a
year ago to tend the sacred flame — and learns what the flame has always been
fed.

- **A full story**: a prologue, five acts, three endings (one hidden behind
  finding all twelve Memory Shards), 975 lines (~16,000 words) of dialogue
  across 167 scenes, four companions with their own arcs and quests. A first
  playthrough runs 4–5 hours.
- **Timeline battles**: a turn-order forecast (in the style of Final Fantasy
  X), four heroes with 8–10 skills each, seven elements with weaknesses to
  discover, 14 status effects, items, defend, flee, and an **Auto** command.
  Bosses have scripted moments; the final one has two phases.
- **Exploration**: the hand-drawn village of Hollowmere (shops, inn, temple,
  townsfolk whose lines change as the story darkens) and twenty procedural
  floors in five biomes, lit by your lantern, with monsters that notice you
  and give chase, treasure, lore, waystones and bosses.
- **RPG depth**: levels to 50, five tiers of gear per character plus
  legendary rewards, accessories with special effects, eight side quests,
  three save slots plus autosave, three difficulty settings.
- **Everything in the box**: the release build is one executable. Art is the
  CC0 Dungeon Crawl Stone Soup tile set (composited and recoloured into a
  custom atlas); all music and sound effects are synthesized in real time by
  the game itself.

## Playing

| Key                                  | Action                                    |
| ------------------------------------ | ----------------------------------------- |
| Arrows / WASD                        | move, navigate menus                      |
| Enter / Space / E                    | talk, open, confirm, advance dialogue     |
| Esc / Backspace                      | back, cancel                              |
| Tab (or Esc while exploring)         | party menu: status, equipment, items, skills, quests, journal, save |
| M                                    | toggle the minimap                        |
| Shift (hold)                         | fast-forward dialogue and battle animations |

Walk into people to talk, into chests to open them, into monsters to fight
(catching them from behind gives you the first move). Waystones at the start
of every floor let you rest, save, or return to Hollowmere; from the
Vaultgate you can go back down to any floor you have reached.

**Chapter Select** on the title screen drops you into any act with a party
levelled and equipped for it — handy for a quick look at later content.

## Building

Needs Rust 1.95 or later (<https://rustup.rs>).

```
cargo run --release     # play
cargo test              # unit tests, balance simulation, story and map checks
cargo build --release   # target/release/emberdeep(.exe), standalone
```

On Linux the sound needs the ALSA headers (`sudo apt install libasound2-dev`
on Debian/Ubuntu); `cargo run --no-default-features` builds a silent version
without them. Windows and macOS need nothing extra.

An automated player plays the entire game through the real UI, from the
title screen to the true ending (about 2.5 hours of game time, a few
minutes of real time), and reports how long each floor took:

```
cargo test --profile bot playthrough -- --ignored --nocapture
```

## How it's built

```
src/main.rs            window setup
src/app/               screens: title, exploring, battle, dialogue, menus, endings; the test bot
src/battle/            turn-based battle rules and AI (no UI), balance simulation
src/data/              content tables: heroes, skills, items, enemies, quests
src/world/             Hollowmere, procedural floors, movement, field of view
src/dungeon/           room-and-corridor generator (maze carvers from rust_maze)
src/story/             the story-script parser, validator and runner
src/audio/             real-time synthesizer, songs and sound effects
src/game.rs            the saved game state; save.rs: slots and settings
assets/story/*.story   the whole script, in a small readable format
assets/atlas.png       every sprite (regenerate with tools/build_atlas.py)
docs/DESIGN.md         the game design document
```

The rules never touch the UI: battles return a list of events that the
battle screen animates, and the story runner executes script commands
against the game state. That split is what lets the balance simulation and
the playthrough bot exercise the real game logic.

See [assets/CREDITS.md](assets/CREDITS.md) for art, font and audio credits.
