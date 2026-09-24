# Dungeon crawler

A small turn-based, tile-based 2D dungeon crawler. Built in Rust with
[egui](https://github.com/emilk/egui); the release build is a single
`dungeon_crawler.exe` with no installation and no files next to it.

Levels are generated the same way as the *Dungeon* type of
[rust_maze](https://github.com/argnd/rust_maze): rooms are placed first, a
maze algorithm carves corridors around them, doors join every region into
one, and dead-end corridors are filled back in.

## Playing

Reach the stairs of depth 10 to escape. Each level is bigger than the last,
and monsters (rats, goblins, orcs) get tougher the deeper you go.

| Key                          | Action                                   |
| ---------------------------- | ---------------------------------------- |
| Arrows / WASD / HJKL         | move; bump a door to open it, a monster to attack it |
| Space or `.`                 | wait a turn                              |
| Enter or `>`                 | take the stairs down                     |

- You start in the first room; the stairs are on the square farthest away.
- You see 7 squares around you. Walls and closed doors block sight; places
  you have seen stay on the map, dimmed.
- Monsters wake up when they see you and chase you along the shortest path.
- Red potions heal 8 HP, gold is your score. Walk over them to pick them up.
- Every staircase gives +3 max HP and some healing, and every other one +1
  attack.

The menu lets you pick the corridor algorithm: *Recursive backtracker*
(long winding corridors), *Prim* (short branching ones) or *Kruskal*.

## Building

Needs the Rust toolchain from <https://rustup.rs> (1.95 or later; MSVC on
Windows).

```
cargo run              # debug build, opens the window, keeps a console for panics
cargo test             # dungeon connectivity and game rules
cargo build --release  # target/release/dungeon_crawler(.exe), standalone
```

The release build links the C runtime statically on Windows
(`.cargo/config.toml`) and hides the console window; debug builds keep it.

## Tiles

Every square is drawn from a 32x32 PNG in `assets/tiles/`, embedded into the
exe at build time. Repaint them with any pixel editor and rebuild. Everything
but `wall`, `floor` and `stairs` uses transparency and is drawn over the
floor. The placeholder set was made with `cargo run --example make_tiles`.

## Layout

```
src/main.rs                  window setup
src/dungeon/                 level generation (rooms, doors, pruning, tests)
src/dungeon/grid.rs          the square grid: Wall / Floor / Door / OpenDoor / Stairs
src/dungeon/carvers/         corridor carvers: backtracker, prim, kruskal
src/game/                    turns, combat, monsters, items, field of view (tests)
src/ui/                      screens, level rendering, tile textures
assets/tiles/                the tile PNGs
examples/make_tiles.rs       one-shot generator for placeholder tiles
```

The game logic in `src/game/` knows nothing about the UI: the app turns key
presses into `Action`s and calls `Game::act`, then draws the resulting state.
