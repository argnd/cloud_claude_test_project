//! One floor of the Deep: the maze generator's rooms and corridors, dressed
//! for the biome and populated with monsters, treasure and story.

use std::collections::BTreeSet;

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, RngExt, SeedableRng};

use super::{Biome, Entity, EntityKind, Loot, Pickup, Place, Tile, World};
use crate::data::enemies::{BattleId, floor_level, random_group};
use crate::data::items::{ItemId, ItemKind};
use crate::dungeon::carvers::Algorithm;
use crate::dungeon::grid::Cell;
use crate::dungeon::{self, Room};
use crate::gfx::sprites::Sprite;

pub const LAST_FLOOR: u32 = 20;

/// Floors holding Aurelian's memory shards, in order.
pub const SHARD_FLOORS: [u32; 12] = [2, 3, 6, 7, 9, 10, 11, 13, 14, 15, 17, 18];
pub const JOURNAL_FLOORS: [u32; 5] = [2, 7, 11, 15, 18];

/// The boss guarding a floor's stairs.
pub fn boss_of(floor: u32) -> Option<(BattleId, &'static str)> {
    Some(match floor {
        4 => (BattleId::Gristlemaw, "gristlemaw_pre"),
        8 => (BattleId::Curator, "curator_pre"),
        12 => (BattleId::MotherOfSpores, "mother_pre"),
        16 => (BattleId::IronWarden, "iron_warden_pre"),
        19 => (BattleId::Ilsa, "ilsa_pre"),
        20 => (BattleId::Aurelian, "aurelian_pre"),
        _ => return None,
    })
}

pub fn boss_flag(battle: BattleId) -> String {
    format!("boss_{}_defeated", battle.script_id())
}

fn boss_sprite(battle: BattleId) -> Sprite {
    match battle {
        BattleId::Vex => Sprite::VexHarlan,
        BattleId::Gristlemaw => Sprite::Gristlemaw,
        BattleId::Curator => Sprite::Curator,
        BattleId::MotherOfSpores => Sprite::MotherOfSpores,
        BattleId::IronWarden => Sprite::IronWarden,
        BattleId::Ilsa => Sprite::IlsaPale,
        BattleId::Aurelian => Sprite::Aurelian,
    }
}

/// Floor size in corridors: the Deep widens as it goes down.
fn size(floor: u32) -> (usize, usize) {
    let f = floor as usize;
    ((17 + f).min(34), (12 + f * 2 / 3).min(24))
}

fn algorithm(biome: Biome, floor: u32) -> Algorithm {
    match biome {
        Biome::Undercroft => Algorithm::Kruskal,
        Biome::Archive => Algorithm::Backtracker,
        Biome::Hollows => Algorithm::Prim,
        Biome::Forge => {
            if floor.is_multiple_of(2) {
                Algorithm::Kruskal
            } else {
                Algorithm::Prim
            }
        }
        _ => Algorithm::Backtracker,
    }
}

/// Builds floor `floor`. `flags` is the story state: unique things already
/// taken or bosses already beaten don't come back.
pub fn build(floor: u32, seed: u64, flags: &BTreeSet<String>) -> World {
    let mut rng = StdRng::seed_from_u64(seed ^ (floor as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let biome = Biome::for_floor(floor);
    let (cx, cy) = size(floor);
    let dungeon = dungeon::generate(algorithm(biome, floor), cx, cy, &mut rng);
    let grid = &dungeon.grid;
    let (w, h) = (grid.width() as i32, grid.height() as i32);
    let mut world = World::new(Place::Floor(floor), biome, w, h, Tile::Wall);
    let mut stairs = (0, 0);
    for y in 0..h {
        for x in 0..w {
            let tile = match grid.get(x as usize, y as usize) {
                Cell::Wall => Tile::Wall,
                Cell::Floor => Tile::Floor,
                Cell::Door => Tile::Door,
                Cell::Stairs => {
                    stairs = (x, y);
                    Tile::Stairs
                }
            };
            world.set((x, y), tile);
            let i = world.idx((x, y));
            world.variant[i] = rng.random_range(0..6);
        }
    }
    let start = (dungeon.start.0 as i32, dungeon.start.1 as i32);
    world.player = start;
    world.arrival = start;
    let has = |f: &str| flags.contains(f);

    // Rooms ordered from the start outward (for placing important things far away).
    let dist = world.distances_from(start, 10_000);
    let mut rooms: Vec<(i32, Room)> = dungeon
        .rooms
        .iter()
        .skip(1)
        .map(|r| {
            let c = r.centre();
            (dist[world.idx((c.0 as i32, c.1 as i32))], *r)
        })
        .collect();
    rooms.sort_by_key(|(d, _)| *d);
    let room_tiles = |r: &Room| -> Vec<(i32, i32)> {
        r.squares()
            .into_iter()
            .map(|(x, y)| (x as i32, y as i32))
            .collect()
    };
    // Tiles strictly inside a room: things placed there can never plug a
    // doorway or a corridor.
    let mut in_room = vec![false; (w * h) as usize];
    for r in &dungeon.rooms {
        for (x, y) in r.squares() {
            in_room[world.idx((x as i32, y as i32))] = true;
        }
    }
    let interior = move |world: &World, p: (i32, i32)| {
        (-1..=1).all(|dy| {
            (-1..=1).all(|dx| {
                let q = (p.0 + dx, p.1 + dy);
                world.in_bounds(q) && in_room[world.idx(q)]
            })
        })
    };

    // A free floor tile in a room, preferring far rooms when `far`.
    let take_spot = |world: &World, rng: &mut StdRng, far: bool| -> Option<(i32, i32)> {
        if rooms.is_empty() {
            return None;
        }
        let n = rooms.len();
        for _ in 0..80 {
            let k = if far {
                rng.random_range(n / 2..n)
            } else {
                rng.random_range(0..n)
            };
            let tiles = room_tiles(&rooms[k].1);
            let p = tiles[rng.random_range(0..tiles.len())];
            if world.tile(p) == Tile::Floor
                && world.entity_at(p).is_none()
                && p != start
                && p != stairs
                && interior(world, p)
                && safe_to_block(world, start, p)
            {
                return Some(p);
            }
        }
        None
    };

    dress(&mut world, biome, &dungeon.rooms, start, &mut rng);

    // Waystone beside the arrival point, never in anyone's way.
    let ring = [
        (1, 1),
        (-1, -1),
        (1, -1),
        (-1, 1),
        (1, 0),
        (-1, 0),
        (0, 1),
        (0, -1),
    ];
    if let Some(p) = ring
        .iter()
        .map(|&(dx, dy)| (start.0 + dx, start.1 + dy))
        .find(|&q| {
            world.tile(q) == Tile::Floor
                && world.entity_at(q).is_none()
                && safe_to_block(&world, start, q)
        })
    {
        world.entities.push(Entity {
            pos: p,
            kind: EntityKind::Waystone,
        });
    }

    // The boss stands on the stairs (floor 20 has no way further down).
    if let Some((battle, scene)) = boss_of(floor)
        && !has(&boss_flag(battle))
    {
        world.entities.push(Entity {
            pos: stairs,
            kind: EntityKind::Boss {
                battle,
                scene: scene.to_string(),
                sprite: boss_sprite(battle),
            },
        });
    }
    if floor == LAST_FLOOR {
        world.set(stairs, Tile::Floor);
        world.entities.push(Entity {
            pos: (stairs.0 + 1, stairs.1),
            kind: EntityKind::Decor {
                sprite: Sprite::Brazier,
                light: 5.0,
                blocks: false,
            },
        });
    }

    // Story encounters.
    let story_npc = |world: &mut World, rng: &mut StdRng, id: &str, sprite: Sprite, scene: &str| {
        if let Some(p) = take_spot(world, rng, true) {
            world.entities.push(Entity {
                pos: p,
                kind: EntityKind::Npc {
                    id: id.into(),
                    sprite,
                    scene: Some(scene.into()),
                    home: p,
                    wander: false,
                },
            });
            Some(p)
        } else {
            None
        }
    };
    if floor == 2 && !has("mouser_found") {
        story_npc(&mut world, &mut rng, "mouser", Sprite::Cat, "mouser_found");
    }
    if floor == 3
        && !has("has_brannoc")
        && let Some(p) = story_npc(
            &mut world,
            &mut rng,
            "brannoc",
            Sprite::Brannoc,
            "brannoc_join",
        )
        && !has(&boss_flag(BattleId::Vex))
        && let Some(q) = world.free_neighbour(p)
    {
        world.entities.push(Entity {
            pos: q,
            kind: EntityKind::Boss {
                battle: BattleId::Vex,
                scene: "vex_pre".into(),
                sprite: Sprite::VexHarlan,
            },
        });
    }
    if floor == 6 && !has("has_maelis") {
        story_npc(
            &mut world,
            &mut rng,
            "maelis",
            Sprite::Maelis,
            "maelis_meet",
        );
    }
    if floor == 10 && !has("has_pip") {
        story_npc(&mut world, &mut rng, "pip", Sprite::Pip, "pip_meet");
    }

    // Unique pickups.
    let pickup = |world: &mut World, rng: &mut StdRng, p: Pickup| {
        if let Some(pos) = take_spot(world, rng, true) {
            world.entities.push(Entity {
                pos,
                kind: EntityKind::Pickup(p),
            });
        }
    };
    if let Some(n) = SHARD_FLOORS.iter().position(|&f| f == floor) {
        let n = n as u8 + 1;
        if !has(&format!("shard_{n}")) {
            pickup(&mut world, &mut rng, Pickup::Shard(n));
        }
    }
    if let Some(n) = JOURNAL_FLOORS.iter().position(|&f| f == floor) {
        let n = n as u8 + 1;
        if !has(&format!("journal_{n}")) {
            pickup(&mut world, &mut rng, Pickup::Journal(n));
        }
    }
    let unique_items: &[(u32, ItemId, &str)] = &[
        (5, ItemId::LostPage, "page_5"),
        (6, ItemId::LostPage, "page_6"),
        (7, ItemId::LostPage, "page_7"),
        (8, ItemId::LostPage, "page_8"),
        (6, ItemId::HolyRelic, "relic_6"),
        (10, ItemId::HolyRelic, "relic_10"),
        (14, ItemId::HolyRelic, "relic_14"),
        (13, ItemId::CrewTag, "tag_13"),
        (14, ItemId::CrewTag, "tag_14"),
        (15, ItemId::CrewTag, "tag_15"),
        (9, ItemId::Glowcap, "glowcap_9"),
    ];
    for &(f, item, flag) in unique_items {
        if f == floor && !has(flag) {
            pickup(
                &mut world,
                &mut rng,
                Pickup::Item {
                    item,
                    flag: flag.into(),
                },
            );
        }
    }
    if floor == 18 && !has("ilsa_lantern_found") {
        pickup(
            &mut world,
            &mut rng,
            Pickup::Scene {
                scene: "ilsa_lantern".into(),
                flag: "ilsa_lantern_found".into(),
                sprite: Sprite::ItemRelic,
            },
        );
    }

    // Treasure.
    let act = (floor - 1) / 4 + 1;
    for _ in 0..(3 + act / 2) {
        if let Some(p) = take_spot(&world, &mut rng, false) {
            let loot = roll_loot(floor, &mut rng);
            world.entities.push(Entity {
                pos: p,
                kind: EntityKind::Chest {
                    loot,
                    opened: false,
                },
            });
        }
    }

    // Monsters, kept away from the arrival point.
    let count = 8 + act as usize;
    let mut placed = 0;
    for _ in 0..count * 6 {
        if placed >= count {
            break;
        }
        let Some(p) = take_spot(&world, &mut rng, false) else {
            break;
        };
        if dist[world.idx(p)] < 9 {
            continue;
        }
        let group = random_group(floor, &mut rng);
        world.entities.push(Entity {
            pos: p,
            kind: EntityKind::Monster {
                group,
                awake: false,
                sleep: 0,
            },
        });
        placed += 1;
    }

    world.update_fov();
    world
}

/// How many tiles can be walked to from `start`, treating permanent
/// obstacles (anything blocking that isn't a monster or boss) as walls.
fn reachable_count(world: &World, start: (i32, i32), extra_wall: Option<(i32, i32)>) -> usize {
    let mut walls = vec![false; (world.w * world.h) as usize];
    for e in &world.entities {
        if e.blocks() && !matches!(e.kind, EntityKind::Monster { .. } | EntityKind::Boss { .. }) {
            walls[world.idx(e.pos)] = true;
        }
    }
    if let Some(p) = extra_wall {
        walls[world.idx(p)] = true;
    }
    let mut seen = vec![false; walls.len()];
    let mut stack = vec![start];
    seen[world.idx(start)] = true;
    let mut count = 0;
    while let Some(p) = stack.pop() {
        count += 1;
        for d in super::Dir::ALL {
            let (dx, dy) = d.delta();
            let q = (p.0 + dx, p.1 + dy);
            if !world.in_bounds(q) || seen[world.idx(q)] || walls[world.idx(q)] {
                continue;
            }
            let t = world.tile(q);
            if t.walkable() || t == Tile::Door {
                seen[world.idx(q)] = true;
                stack.push(q);
            }
        }
    }
    count
}

/// Putting something solid on `p` leaves every other tile reachable.
pub fn safe_to_block(world: &World, start: (i32, i32), p: (i32, i32)) -> bool {
    p != start && reachable_count(world, start, Some(p)) + 1 == reachable_count(world, start, None)
}

/// Biome decoration: torches, puddles, bookshelves, glowing fungus, lava, ice.
fn dress(world: &mut World, biome: Biome, rooms: &[Room], start: (i32, i32), rng: &mut StdRng) {
    let (w, h) = (world.w, world.h);
    let is_room: Vec<bool> = {
        let mut v = vec![false; (w * h) as usize];
        for r in rooms {
            for (x, y) in r.squares() {
                v[world.idx((x as i32, y as i32))] = true;
            }
        }
        v
    };
    let room_at = |world: &World, p: (i32, i32)| world.in_bounds(p) && is_room[world.idx(p)];

    // Wall torches: walls directly above a room tile, now and then.
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let p = (x, y);
            if world.tile(p) != Tile::Wall
                || !room_at(world, (x, y + 1))
                || rng.random_range(0..100) >= 9
            {
                continue;
            }
            match biome {
                Biome::Archive => {
                    if rng.random_bool(0.6) {
                        world.set(p, Tile::Bookshelf);
                    } else {
                        world.entities.push(Entity {
                            pos: p,
                            kind: EntityKind::Decor {
                                sprite: Sprite::WallTorch,
                                light: 3.5,
                                blocks: true,
                            },
                        });
                    }
                }
                Biome::Hollows => {}
                _ => world.entities.push(Entity {
                    pos: p,
                    kind: EntityKind::Decor {
                        sprite: Sprite::WallTorch,
                        light: 3.5,
                        blocks: true,
                    },
                }),
            }
        }
    }
    // Floor scatter inside rooms.
    for r in rooms.iter().skip(1) {
        let tiles: Vec<(i32, i32)> = r
            .squares()
            .into_iter()
            .map(|(x, y)| (x as i32, y as i32))
            .collect();
        let n = rng.random_range(0..3);
        for _ in 0..n {
            let p = tiles[rng.random_range(0..tiles.len())];
            if world.entity_at(p).is_some() {
                continue;
            }
            let (sprite, light) = match biome {
                Biome::Undercroft => (
                    if rng.random_bool(0.5) {
                        Sprite::Bones
                    } else {
                        Sprite::Rubble
                    },
                    0.0,
                ),
                Biome::Archive => (Sprite::Rubble, 0.0),
                Biome::Hollows => (Sprite::FungusDecor, 2.5),
                Biome::Forge => (Sprite::Brazier, 3.5),
                Biome::Pale => (Sprite::IceCrystal, 2.0),
                Biome::Town => continue,
            };
            let blocks = matches!(sprite, Sprite::Brazier);
            if blocks && !safe_to_block(world, start, p) {
                continue;
            }
            world.entities.push(Entity {
                pos: p,
                kind: EntityKind::Decor {
                    sprite,
                    light,
                    blocks,
                },
            });
        }
        // Pools: shallow water in the Archive, isolated lava in the Forge.
        let pool = match biome {
            Biome::Archive => Some(Tile::Shallow),
            Biome::Forge => Some(Tile::Lava),
            _ => None,
        };
        if let Some(pool) = pool {
            let mut spots = tiles.clone();
            spots.shuffle(rng);
            let wanted = if pool == Tile::Lava { 2 } else { 5 };
            let mut done = 0;
            for p in spots {
                if done >= wanted {
                    break;
                }
                // Lava only where all eight neighbours are open room floor,
                // and never next to other lava: it can't cut a path.
                let ok = if pool == Tile::Lava {
                    (-1..=1).all(|dy| {
                        (-1..=1).all(|dx| {
                            let q = (p.0 + dx, p.1 + dy);
                            room_at(world, q) && world.tile(q) == Tile::Floor
                        })
                    }) && world.entity_at(p).is_none()
                } else {
                    world.entity_at(p).is_none() && world.tile(p) == Tile::Floor
                };
                if ok {
                    world.set(p, pool);
                    done += 1;
                }
            }
        }
    }
    // Hollows: glowing mushrooms along corridors too.
    if biome == Biome::Hollows {
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                if world.tile((x, y)) == Tile::Floor
                    && !room_at(world, (x, y))
                    && rng.random_range(0..100) < 4
                {
                    world.entities.push(Entity {
                        pos: (x, y),
                        kind: EntityKind::Decor {
                            sprite: Sprite::FungusDecor,
                            light: 2.2,
                            blocks: false,
                        },
                    });
                }
            }
        }
    }
}

/// What a chest holds: gold, supplies, or now and then gear.
fn roll_loot(floor: u32, rng: &mut impl Rng) -> Loot {
    let act = ((floor - 1) / 4 + 1) as u8;
    let roll = rng.random_range(0..100);
    if roll < 40 {
        let level = floor_level(floor);
        return Loot::Gold(rng.random_range(12..=22) * (level + 2));
    }
    if roll < 78 {
        let supplies: Vec<ItemId> = ItemId::ALL
            .into_iter()
            .filter(|i| {
                matches!(i.def().kind, ItemKind::Consumable(_))
                    && i.def().tier >= 1
                    && i.def().tier <= act.min(4)
            })
            .collect();
        let item = supplies[rng.random_range(0..supplies.len())];
        let count = if item.def().price < 100 {
            rng.random_range(1..=3)
        } else {
            1
        };
        return Loot::Item(item, count);
    }
    // Gear: this act's tier, sometimes the next.
    let tier = if rng.random_bool(0.35) {
        (act + 1).min(5)
    } else {
        act
    };
    let gear: Vec<ItemId> = ItemId::ALL
        .into_iter()
        .filter(|i| i.def().slot().is_some() && i.def().tier == tier)
        .collect();
    Loot::Item(gear[rng.random_range(0..gear.len())], 1)
}
