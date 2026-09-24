//! Hollowmere, drawn by hand.

use super::{Biome, Entity, EntityKind, Place, Shop, Tile, World};
use crate::gfx::sprites::Sprite;

/// Legend: T tree, . grass, : path, # wall, = wooden floor, + door, ~ water,
/// F fountain, S statue, L lamppost, A altar, V the Vaultgate,
/// i/w/p/t shop doors (inn, smith, apothecary, temple), @ Wren's bed,
/// digits and g: townsfolk (see `NPCS`).
const MAP: &str = "\
TTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTT
T....T..~~~~...........T...........T......TT
T.#######.~~~..###########...#######.......T
T.#=====#..~...#=========#...#==A==#.......T
T.#==@==#......#====1====#...#==5==#..TT...T
T.#=====#......#=========#...#=====#..TT...T
T.###+###......#####+#####...###t###.......T
T....:....L.........:....L......:....L.....T
T....:::::::::::::::::::::::::::::::::.....T
T..........:.....................:.........T
T..L.......:.....:::::::::.......:....L....T
T..........:.....:.......:.......:.........T
T...8......:.....:...F...:.......:...9.....T
T..........:::::::.......:::::::::.........T
T..TT......:.....:...S...:.......:....TT...T
T..TT......:.....:.......:.......:....TT...T
T..........:.....:::::::::.......:.........T
T..L.......:........:............:....L....T
T....:::::::::::::::::::::::::::::::::.....T
T....:.......:..........:.........:........T
T.#######..#######...#######...#######.....T
T.#=====#..#=====#...#=====#...#=====#.....T
T.#==2==#..#==3==#...#==4==#...#==7==#..0..T
T.#=====#..#=====#...#=====#...#=====#.....T
T.###i###..###w###...###p###...###+###.....T
T.......................:..................T
T..T.....L.........g....:....L.......T.....T
T...................:::::::................T
T...................:#VVV#:....6..........TT
TTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTT";

/// Map marker -> (speaker id, sprite, wanders).
const NPCS: &[(char, &str, Sprite, bool)] = &[
    ('1', "elder", Sprite::Elder, false),
    ('2', "bess", Sprite::Innkeeper, false),
    ('3', "dagna", Sprite::Smith, false),
    ('4', "fen", Sprite::Apothecary, false),
    ('5', "oriel", Sprite::Priestess, false),
    ('6', "rennick", Sprite::Captain, false),
    ('7', "hesta", Sprite::OldWoman, false),
    ('8', "tobin", Sprite::Child, true),
    ('9', "villager_a", Sprite::VillagerA, true),
    ('0', "villager_b", Sprite::VillagerB, true),
    ('g', "guard", Sprite::GuardNpc, false),
];

/// Where the party appears when coming back up from the Deep.
pub const GATE_ARRIVAL: (i32, i32) = (22, 27);
/// Wren's bed, where a new game starts.
pub const HOME: (i32, i32) = (5, 4);

/// Builds the town for a story stage (1-5): fewer lanterns burn as the
/// Pale rises. `mouser_home` puts Hesta's cat back by her fire.
pub fn build(stage: u8, mouser_home: bool) -> World {
    let rows: Vec<&str> = MAP.lines().collect();
    let (w, h) = (rows[0].len() as i32, rows.len() as i32);
    let mut world = World::new(Place::Town, Biome::Town, w, h, Tile::Grass);
    let mut lamp_index = 0;
    for (y, row) in rows.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            let p = (x as i32, y as i32);
            let tile = match c {
                'T' => Tile::Tree,
                ':' => Tile::Path,
                '#' => Tile::TownWall,
                '=' | '@' => Tile::Wood,
                '+' => Tile::OpenDoor,
                '~' => Tile::Water,
                'F' => Tile::Fountain,
                'S' => Tile::Statue,
                'A' => Tile::Altar,
                'V' => Tile::VaultGate,
                'i' => Tile::ShopDoor(Shop::Inn),
                'w' => Tile::ShopDoor(Shop::Smith),
                'p' => Tile::ShopDoor(Shop::Apothecary),
                't' => Tile::ShopDoor(Shop::Temple),
                _ => Tile::Grass,
            };
            world.set(p, tile);
            let i = world.idx(p);
            world.variant[i] = ((x * 7 + y * 13 + x * y) % 3) as u8;
            if c == 'L' {
                // Lanterns go dark as the story darkens.
                let lit = match stage {
                    1 => lamp_index % 2 == 0,
                    2 => lamp_index % 4 != 3,
                    3 => lamp_index % 3 == 0,
                    4 => lamp_index % 4 == 0,
                    _ => lamp_index == 0,
                };
                lamp_index += 1;
                let (sprite, light) = if lit { (Sprite::LamppostLit, 4.5) } else { (Sprite::LamppostOut, 0.0) };
                world.entities.push(Entity { pos: p, kind: EntityKind::Decor { sprite, light, blocks: true } });
            }
            if let Some(&(_, id, sprite, wander)) = NPCS.iter().find(|n| n.0 == c) {
                world.entities.push(Entity {
                    pos: p,
                    kind: EntityKind::Npc { id: id.to_string(), sprite, scene: None, home: p, wander },
                });
            }
        }
    }
    if mouser_home {
        world.entities.push(Entity {
            pos: (33, 23),
            kind: EntityKind::Npc { id: "mouser".into(), sprite: Sprite::Cat, scene: Some("npc_mouser".into()), home: (33, 23), wander: false },
        });
    }
    // Hearth fires in the houses.
    for &p in &[(3, 3), (7, 5), (16, 3), (24, 5), (3, 21), (12, 21), (26, 21), (36, 21)] {
        world.entities.push(Entity { pos: p, kind: EntityKind::Decor { sprite: Sprite::Brazier, light: 3.0, blocks: true } });
    }
    world.explored = vec![true; (w * h) as usize];
    world.player = HOME;
    world.arrival = GATE_ARRIVAL;
    world.update_fov();
    world
}
