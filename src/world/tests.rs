use std::collections::BTreeSet;

use super::floor::{self, LAST_FLOOR};
use super::*;

/// Walking distances from the arrival point, treating every permanent
/// obstacle (chests, braziers, people, waystones) as solid. Monsters and
/// bosses aren't: they can be fought through.
fn distances_around_things(world: &World) -> Vec<i32> {
    let mut blocked = world.clone();
    for e in &world.entities {
        let fightable = matches!(e.kind, EntityKind::Monster { .. } | EntityKind::Boss { .. });
        if e.blocks() && !fightable && e.pos != world.arrival {
            blocked.set(e.pos, Tile::Wall);
        }
    }
    // Doors and walkable tiles only.
    blocked.distances_from(world.arrival, 100_000)
}

fn reachable(world: &World, p: (i32, i32)) -> bool {
    let dist = distances_around_things(world);
    // Entities that block stand on reachable tiles if a neighbour is reachable.
    Dir::ALL.iter().any(|d| {
        let (dx, dy) = d.delta();
        let q = (p.0 + dx, p.1 + dy);
        world.in_bounds(q) && dist[world.idx(q)] != i32::MAX
    }) || dist[world.idx(p)] != i32::MAX
}

#[test]
fn the_town_is_well_formed() {
    let town = town::build(1, true);
    assert_eq!(town.tile(town::HOME), Tile::Wood);
    for e in &town.entities {
        if let EntityKind::Npc { id, .. } = &e.kind {
            let mut w = town.clone();
            w.arrival = town::HOME;
            assert!(reachable(&w, e.pos), "{id} can't be reached");
        }
    }
    let gate = town.tiles.iter().filter(|t| **t == Tile::VaultGate).count();
    assert_eq!(gate, 3);
}

#[test]
fn every_floor_is_connected_and_populated() {
    let flags = BTreeSet::new();
    for n in 1..=LAST_FLOOR {
        for seed in [1u64, 77, 2024, 31337] {
            let world = floor::build(n, seed, &flags);
            let monsters = world
                .entities
                .iter()
                .filter(|e| matches!(e.kind, EntityKind::Monster { .. }))
                .count();
            assert!(monsters >= 6, "floor {n}: only {monsters} monsters");
            for e in &world.entities {
                if matches!(e.kind, EntityKind::Decor { .. }) {
                    continue;
                }
                assert!(
                    reachable(&world, e.pos),
                    "floor {n}: {:?} unreachable",
                    e.kind
                );
            }
            if n < LAST_FLOOR {
                let stairs = (0..world.h)
                    .flat_map(|y| (0..world.w).map(move |x| (x, y)))
                    .find(|&p| world.tile(p) == Tile::Stairs)
                    .expect("stairs");
                assert!(reachable(&world, stairs), "floor {n}: stairs unreachable");
            }
            let bosses = world
                .entities
                .iter()
                .filter(|e| matches!(e.kind, EntityKind::Boss { .. }))
                .count();
            let expected = floor::boss_of(n).is_some() as usize + (n == 3) as usize;
            assert_eq!(bosses, expected, "floor {n}");
        }
    }
}

#[test]
fn collected_uniques_do_not_return() {
    let mut flags = BTreeSet::new();
    let shard = |w: &World| {
        w.entities
            .iter()
            .any(|e| matches!(e.kind, EntityKind::Pickup(Pickup::Shard(1))))
    };
    assert!(shard(&floor::build(2, 5, &flags)));
    flags.insert("shard_1".to_string());
    assert!(!shard(&floor::build(2, 5, &flags)));
}

#[test]
fn walking_opens_doors_and_monsters_chase() {
    let mut world = World::new(Place::Floor(1), Biome::Undercroft, 7, 3, Tile::Floor);
    world.set((2, 1), Tile::Door);
    world.player = (1, 1);
    assert_eq!(world.try_move(Dir::Right), Move::OpenedDoor);
    assert_eq!(world.try_move(Dir::Right), Move::Moved);
    world.entities.push(Entity {
        pos: (6, 1),
        kind: EntityKind::Monster {
            group: vec![(crate::data::enemies::EnemyId::SewerRat, 1)],
            awake: false,
            sleep: 0,
        },
    });
    world.update_fov();
    let mut rng = rand::rngs::StdRng::seed_from_u64(1);
    use rand::SeedableRng;
    let mut touched = None;
    for _ in 0..5 {
        touched = world.creatures_act(&mut rng);
        if touched.is_some() {
            break;
        }
    }
    assert_eq!(touched, Some(0));
}
