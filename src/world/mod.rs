//! The map the party walks on: Hollowmere or a floor of the Deep. Movement
//! is tile by tile; monsters take a step each time the player does.

pub mod floor;
pub mod town;

use std::collections::VecDeque;

use rand::Rng;
use rand::RngExt;
use serde::{Deserialize, Serialize};

use crate::audio::Track;
use crate::data::enemies::{BattleId, EnemyId};
use crate::data::items::ItemId;
use crate::gfx::sprites::Sprite;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Biome {
    Town,
    Undercroft,
    Archive,
    Hollows,
    Forge,
    Pale,
}

impl Biome {
    pub fn for_floor(floor: u32) -> Biome {
        match floor {
            0 => Biome::Town,
            1..=4 => Biome::Undercroft,
            5..=8 => Biome::Archive,
            9..=12 => Biome::Hollows,
            13..=16 => Biome::Forge,
            _ => Biome::Pale,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Biome::Town => "Hollowmere",
            Biome::Undercroft => "The Undercroft",
            Biome::Archive => "The Drowned Archive",
            Biome::Hollows => "The Mycelium Hollows",
            Biome::Forge => "The Ember Forge",
            Biome::Pale => "The Pale Reach",
        }
    }

    pub fn music(self) -> Track {
        match self {
            Biome::Town => Track::Town,
            Biome::Undercroft => Track::Undercroft,
            Biome::Archive => Track::Archive,
            Biome::Hollows => Track::Hollows,
            Biome::Forge => Track::Forge,
            Biome::Pale => Track::PaleReach,
        }
    }

    pub fn floors(self) -> [Sprite; 3] {
        match self {
            Biome::Town => [Sprite::TownGrassA, Sprite::TownGrassB, Sprite::TownGrassC],
            Biome::Undercroft => [
                Sprite::UndercroftFloorA,
                Sprite::UndercroftFloorB,
                Sprite::UndercroftFloorC,
            ],
            Biome::Archive => [
                Sprite::ArchiveFloorA,
                Sprite::ArchiveFloorB,
                Sprite::ArchiveFloorC,
            ],
            Biome::Hollows => [
                Sprite::HollowsFloorA,
                Sprite::HollowsFloorB,
                Sprite::HollowsFloorC,
            ],
            Biome::Forge => [
                Sprite::ForgeFloorA,
                Sprite::ForgeFloorB,
                Sprite::ForgeFloorC,
            ],
            Biome::Pale => [Sprite::PaleFloorA, Sprite::PaleFloorB, Sprite::PaleFloorC],
        }
    }

    pub fn walls(self) -> [Sprite; 2] {
        match self {
            Biome::Town => [Sprite::TownWall, Sprite::TownWall],
            Biome::Undercroft => [Sprite::UndercroftWallA, Sprite::UndercroftWallB],
            Biome::Archive => [Sprite::ArchiveWallA, Sprite::ArchiveWallB],
            Biome::Hollows => [Sprite::HollowsWallA, Sprite::HollowsWallB],
            Biome::Forge => [Sprite::ForgeWallA, Sprite::ForgeWallB],
            Biome::Pale => [Sprite::PaleWallA, Sprite::PaleWallB],
        }
    }

    /// Colour of torchlight and the lantern in this place.
    pub fn light_colour(self) -> [f32; 3] {
        match self {
            Biome::Town => [1.0, 0.85, 0.6],
            Biome::Undercroft => [1.0, 0.82, 0.58],
            Biome::Archive => [0.8, 0.9, 1.0],
            Biome::Hollows => [0.75, 1.0, 0.95],
            Biome::Forge => [1.0, 0.7, 0.45],
            Biome::Pale => [0.85, 0.92, 1.0],
        }
    }

    /// Light everywhere regardless of lamps (0 = pitch dark).
    pub fn ambient(self) -> f32 {
        match self {
            Biome::Town => 0.62,
            Biome::Hollows => 0.08,
            Biome::Forge => 0.07,
            _ => 0.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Tile {
    Floor,
    Wall,
    Door,
    OpenDoor,
    Stairs,
    Water,
    Shallow,
    Lava,
    Grass,
    Path,
    Wood,
    TownWall,
    Tree,
    Fountain,
    Statue,
    Altar,
    VaultGate,
    Bookshelf,
    /// A shop entrance: walkable, drawn with its shop sign.
    ShopDoor(Shop),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Shop {
    Inn,
    Smith,
    Apothecary,
    Temple,
}

impl Tile {
    pub fn walkable(self) -> bool {
        !matches!(
            self,
            Tile::Wall
                | Tile::Water
                | Tile::Lava
                | Tile::TownWall
                | Tile::Tree
                | Tile::Fountain
                | Tile::Statue
                | Tile::Altar
                | Tile::Bookshelf
        )
    }

    pub fn blocks_sight(self) -> bool {
        matches!(
            self,
            Tile::Wall | Tile::TownWall | Tile::Door | Tile::Bookshelf | Tile::Tree
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    pub fn delta(self) -> (i32, i32) {
        match self {
            Dir::Up => (0, -1),
            Dir::Down => (0, 1),
            Dir::Left => (-1, 0),
            Dir::Right => (1, 0),
        }
    }

    pub const ALL: [Dir; 4] = [Dir::Up, Dir::Down, Dir::Left, Dir::Right];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Loot {
    Gold(u32),
    Item(ItemId, u32),
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Pickup {
    Shard(u8),
    Journal(u8),
    /// A unique quest item; `flag` records that it was taken.
    Item {
        item: ItemId,
        flag: String,
    },
    /// A scene that plays when stepped on (e.g. Ilsa's lantern).
    Scene {
        scene: String,
        flag: String,
        sprite: Sprite,
    },
}

impl Pickup {
    pub fn sprite(&self) -> Sprite {
        match self {
            Pickup::Shard(_) => Sprite::MemoryShard,
            Pickup::Journal(_) => Sprite::NoteScroll,
            Pickup::Item { item, .. } => item.def().sprite,
            Pickup::Scene { sprite, .. } => *sprite,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EntityKind {
    /// Someone to talk to. Town NPCs pick their scene from the story state;
    /// others carry a fixed scene.
    Npc {
        id: String,
        sprite: Sprite,
        scene: Option<String>,
        home: (i32, i32),
        wander: bool,
    },
    Monster {
        group: Vec<(EnemyId, u32)>,
        awake: bool,
        sleep: u8,
    },
    Boss {
        battle: BattleId,
        scene: String,
        sprite: Sprite,
    },
    Chest {
        loot: Loot,
        opened: bool,
    },
    Pickup(Pickup),
    Waystone,
    /// Scenery; `light` is a light radius in tiles (0 for none).
    Decor {
        sprite: Sprite,
        light: f32,
        blocks: bool,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entity {
    pub pos: (i32, i32),
    pub kind: EntityKind,
}

impl Entity {
    pub fn blocks(&self) -> bool {
        match &self.kind {
            EntityKind::Pickup(_) => false,
            EntityKind::Decor { blocks, .. } => *blocks,
            _ => true,
        }
    }

    pub fn sprite(&self) -> Sprite {
        match &self.kind {
            EntityKind::Npc { sprite, .. } => *sprite,
            EntityKind::Monster { group, .. } => group[0].0.def().sprite,
            EntityKind::Boss { sprite, .. } => *sprite,
            EntityKind::Chest { opened, .. } => {
                if *opened {
                    Sprite::ChestOpen
                } else {
                    Sprite::ChestClosed
                }
            }
            EntityKind::Pickup(p) => p.sprite(),
            EntityKind::Waystone => Sprite::Waystone,
            EntityKind::Decor { sprite, .. } => *sprite,
        }
    }

    pub fn light(&self) -> f32 {
        match &self.kind {
            EntityKind::Decor { light, .. } => *light,
            EntityKind::Waystone => 3.5,
            EntityKind::Pickup(Pickup::Shard(_)) => 2.5,
            EntityKind::Boss { .. } => 2.0,
            _ => 0.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Place {
    Town,
    Floor(u32),
}

/// What happened when the player tried to step.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Move {
    Moved,
    Blocked,
    OpenedDoor,
    /// Walked into an entity (talk, open, fight...).
    Bump(usize),
    /// Stepped onto a pickup.
    Pickup(usize),
    Stairs,
    VaultGate,
    Shop(Shop),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct World {
    pub place: Place,
    pub biome: Biome,
    pub w: i32,
    pub h: i32,
    pub tiles: Vec<Tile>,
    /// Visual variant per tile.
    pub variant: Vec<u8>,
    pub entities: Vec<Entity>,
    pub player: (i32, i32),
    pub facing: Dir,
    pub explored: Vec<bool>,
    pub arrival: (i32, i32),
    /// The party rested at this floor's waystone.
    pub rested: bool,
    #[serde(skip)]
    pub visible: Vec<bool>,
}

pub const SIGHT: i32 = 8;

impl World {
    pub fn new(place: Place, biome: Biome, w: i32, h: i32, fill: Tile) -> Self {
        let n = (w * h) as usize;
        Self {
            place,
            biome,
            w,
            h,
            tiles: vec![fill; n],
            variant: vec![0; n],
            entities: Vec::new(),
            player: (1, 1),
            facing: Dir::Down,
            explored: vec![false; n],
            arrival: (1, 1),
            rested: false,
            visible: vec![false; n],
        }
    }

    pub fn floor_number(&self) -> Option<u32> {
        match self.place {
            Place::Floor(n) => Some(n),
            Place::Town => None,
        }
    }

    pub fn in_bounds(&self, (x, y): (i32, i32)) -> bool {
        x >= 0 && y >= 0 && x < self.w && y < self.h
    }

    pub fn idx(&self, (x, y): (i32, i32)) -> usize {
        (y * self.w + x) as usize
    }

    pub fn tile(&self, p: (i32, i32)) -> Tile {
        if self.in_bounds(p) {
            self.tiles[self.idx(p)]
        } else {
            Tile::Wall
        }
    }

    pub fn set(&mut self, p: (i32, i32), t: Tile) {
        let i = self.idx(p);
        self.tiles[i] = t;
    }

    pub fn entity_at(&self, p: (i32, i32)) -> Option<usize> {
        // Prefer blocking entities (a monster standing on a pickup).
        self.entities
            .iter()
            .position(|e| e.pos == p && e.blocks())
            .or_else(|| self.entities.iter().position(|e| e.pos == p))
    }

    fn blocking_entity_at(&self, p: (i32, i32)) -> Option<usize> {
        self.entities.iter().position(|e| e.pos == p && e.blocks())
    }

    pub fn is_visible(&self, p: (i32, i32)) -> bool {
        self.in_bounds(p) && self.visible.get(self.idx(p)).copied().unwrap_or(false)
    }

    pub fn is_explored(&self, p: (i32, i32)) -> bool {
        self.in_bounds(p) && self.explored[self.idx(p)]
    }

    /// Tries to step the player one tile.
    pub fn try_move(&mut self, dir: Dir) -> Move {
        self.facing = dir;
        let (dx, dy) = dir.delta();
        let to = (self.player.0 + dx, self.player.1 + dy);
        if !self.in_bounds(to) {
            return Move::Blocked;
        }
        if let Some(i) = self.blocking_entity_at(to) {
            return Move::Bump(i);
        }
        match self.tile(to) {
            Tile::Door => {
                self.set(to, Tile::OpenDoor);
                self.update_fov();
                return Move::OpenedDoor;
            }
            t if !t.walkable() => return Move::Blocked,
            _ => {}
        }
        self.player = to;
        self.update_fov();
        match self.tile(to) {
            Tile::Stairs => return Move::Stairs,
            Tile::VaultGate => return Move::VaultGate,
            Tile::ShopDoor(shop) => return Move::Shop(shop),
            _ => {}
        }
        if let Some(i) = self
            .entities
            .iter()
            .position(|e| e.pos == to && matches!(e.kind, EntityKind::Pickup(_)))
        {
            return Move::Pickup(i);
        }
        Move::Moved
    }

    /// Recomputes what the player can see.
    pub fn update_fov(&mut self) {
        let n = (self.w * self.h) as usize;
        self.visible = vec![false; n];
        let (px, py) = self.player;
        let r = if self.biome == Biome::Town { 14 } else { SIGHT };
        for ty in (py - r).max(0)..=(py + r).min(self.h - 1) {
            for tx in (px - r).max(0)..=(px + r).min(self.w - 1) {
                let (dx, dy) = (tx - px, ty - py);
                if dx * dx + dy * dy > r * r + r {
                    continue;
                }
                if self.line_clear((px, py), (tx, ty)) {
                    let i = self.idx((tx, ty));
                    self.visible[i] = true;
                    self.explored[i] = true;
                }
            }
        }
    }

    fn line_clear(&self, from: (i32, i32), to: (i32, i32)) -> bool {
        let (mut x, mut y) = from;
        let dx = (to.0 - x).abs();
        let dy = -(to.1 - y).abs();
        let sx = if x < to.0 { 1 } else { -1 };
        let sy = if y < to.1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            if (x, y) == to {
                return true;
            }
            if (x, y) != from && self.tile((x, y)).blocks_sight() {
                return false;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Steps along walkable tiles from the player: distance per tile.
    pub fn distances_from(&self, start: (i32, i32), limit: i32) -> Vec<i32> {
        let mut dist = vec![i32::MAX; (self.w * self.h) as usize];
        let mut queue = VecDeque::new();
        dist[self.idx(start)] = 0;
        queue.push_back(start);
        while let Some(p) = queue.pop_front() {
            let d = dist[self.idx(p)];
            if d >= limit {
                continue;
            }
            for dir in Dir::ALL {
                let (dx, dy) = dir.delta();
                let q = (p.0 + dx, p.1 + dy);
                if !self.in_bounds(q) {
                    continue;
                }
                let t = self.tile(q);
                if !(t.walkable() || t == Tile::Door) {
                    continue;
                }
                let i = self.idx(q);
                if dist[i] == i32::MAX {
                    dist[i] = d + 1;
                    queue.push_back(q);
                }
            }
        }
        dist
    }

    /// Monsters (and wandering townsfolk) take their step. Returns the index
    /// of a monster that walked into the player, if any.
    pub fn creatures_act(&mut self, rng: &mut impl Rng) -> Option<usize> {
        let dist = self.distances_from(self.player, 14);
        for i in 0..self.entities.len() {
            let pos = self.entities[i].pos;
            let d = dist[self.idx(pos)];
            let sees = self.is_visible(pos) && d <= 7;
            let chasing = match &mut self.entities[i].kind {
                EntityKind::Monster { awake, sleep, .. } => {
                    if *sleep > 0 {
                        *sleep -= 1;
                        continue;
                    }
                    if sees {
                        *awake = true;
                    } else if d > 12 {
                        *awake = false;
                    }
                    *awake
                }
                EntityKind::Npc {
                    wander: true, home, ..
                } => {
                    let home = *home;
                    if rng.random_bool(0.15) {
                        self.wander(i, rng, Some((home, 3)));
                    }
                    continue;
                }
                _ => continue,
            };
            if !chasing {
                if rng.random_bool(0.3) {
                    self.wander(i, rng, None);
                }
                continue;
            }
            if d == 1 {
                return Some(i);
            }
            // Step to the neighbour closest to the player.
            let mut best: Option<(i32, (i32, i32))> = None;
            for dir in Dir::ALL {
                let (dx, dy) = dir.delta();
                let q = (pos.0 + dx, pos.1 + dy);
                if !self.in_bounds(q)
                    || !self.tile(q).walkable()
                    || self.blocking_entity_at(q).is_some()
                {
                    continue;
                }
                let dq = dist[self.idx(q)];
                if dq < d && best.is_none_or(|(bd, _)| dq < bd) {
                    best = Some((dq, q));
                }
            }
            if let Some((_, q)) = best {
                if q == self.player {
                    return Some(i);
                }
                self.entities[i].pos = q;
            }
        }
        None
    }

    fn wander(&mut self, i: usize, rng: &mut impl Rng, leash: Option<((i32, i32), i32)>) {
        let dir = Dir::ALL[rng.random_range(0..4)];
        let (dx, dy) = dir.delta();
        let pos = self.entities[i].pos;
        let q = (pos.0 + dx, pos.1 + dy);
        if !self.in_bounds(q)
            || q == self.player
            || !self.tile(q).walkable()
            || self.entity_at(q).is_some()
        {
            return;
        }
        if matches!(
            self.tile(q),
            Tile::Stairs | Tile::VaultGate | Tile::ShopDoor(_) | Tile::OpenDoor
        ) {
            return;
        }
        if let Some((home, r)) = leash
            && (q.0 - home.0).abs() + (q.1 - home.1).abs() > r
        {
            return;
        }
        self.entities[i].pos = q;
    }

    /// A free walkable tile next to `p` (for placing things).
    pub fn free_neighbour(&self, p: (i32, i32)) -> Option<(i32, i32)> {
        let ring = [
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (-1, -1),
            (1, -1),
            (-1, 1),
        ];
        ring.iter()
            .map(|&(dx, dy)| (p.0 + dx, p.1 + dy))
            .find(|&q| {
                self.in_bounds(q)
                    && self.tile(q).walkable()
                    && self.entity_at(q).is_none()
                    && q != self.player
                    && matches!(
                        self.tile(q),
                        Tile::Floor | Tile::Grass | Tile::Path | Tile::Wood
                    )
            })
    }

    pub fn remove_entity(&mut self, i: usize) {
        if i < self.entities.len() {
            self.entities.remove(i);
        }
    }

    pub fn remove_npc(&mut self, id: &str) {
        self.entities
            .retain(|e| !matches!(&e.kind, EntityKind::Npc { id: n, .. } if n == id));
    }

    /// After a load: rebuild what isn't saved.
    pub fn restore(&mut self) {
        self.update_fov();
    }
}

#[cfg(test)]
mod tests;
