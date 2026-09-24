//! The game itself, independent of any UI: one level at a time, the player
//! acts, then every monster acts, then the field of view is recomputed.

pub mod entities;
mod fov;

use rand::rngs::StdRng;
use rand::seq::IndexedRandom;
use rand::{RngExt, SeedableRng};

use crate::dungeon::carvers::Algorithm;
use crate::dungeon::grid::{Cell, Grid, neighbours};
use crate::dungeon::{self, Dungeon};
use entities::{Item, ItemKind, Monster, MonsterKind, Player};

/// Taking the stairs on this depth wins the game.
pub const FINAL_DEPTH: u32 = 10;
/// How far the player sees, in squares.
const SIGHT_RADIUS: usize = 7;
/// Awake monsters farther than this (in steps) lose track of the player.
const CHASE_DISTANCE: usize = 24;
const POTION_HEAL: i32 = 8;
const LOG_LENGTH: usize = 50;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn offset(self) -> (isize, isize) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    /// Walk, open a door, or attack whatever stands there.
    Move(Direction),
    Wait,
    /// Take the stairs down, when standing on them.
    Descend,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    Playing,
    Dead,
    Won,
}

pub struct Game {
    algorithm: Algorithm,
    rng: StdRng,
    pub depth: u32,
    pub grid: Grid,
    pub player: Player,
    pub monsters: Vec<Monster>,
    pub items: Vec<Item>,
    /// Per square: in sight right now / ever seen on this level.
    pub visible: Vec<bool>,
    pub explored: Vec<bool>,
    pub log: Vec<String>,
    pub turns: u32,
    pub state: State,
}

impl Game {
    pub fn new(algorithm: Algorithm, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let dungeon = dungeon::generate(algorithm, level_size(1).0, level_size(1).1, &mut rng);
        let mut game = Self {
            algorithm,
            rng,
            depth: 1,
            grid: Grid::new(0, 0),
            player: Player::new(dungeon.start),
            monsters: Vec::new(),
            items: Vec::new(),
            visible: Vec::new(),
            explored: Vec::new(),
            log: Vec::new(),
            turns: 0,
            state: State::Playing,
        };
        game.enter(dungeon);
        game.say("You enter the dungeon. Find the stairs down.");
        game
    }

    /// One player action followed, if it took time, by the monsters' turn.
    pub fn act(&mut self, action: Action) {
        if self.state != State::Playing {
            return;
        }
        let took_time = match action {
            Action::Move(direction) => self.player_move(direction),
            Action::Wait => true,
            Action::Descend => {
                self.descend();
                return;
            }
        };
        if !took_time {
            return;
        }
        self.turns += 1;
        self.monsters_act();
        self.update_fov();
    }

    pub fn is_visible(&self, x: usize, y: usize) -> bool {
        self.visible[self.grid.index(x, y)]
    }

    pub fn is_explored(&self, x: usize, y: usize) -> bool {
        self.explored[self.grid.index(x, y)]
    }

    pub fn on_stairs(&self) -> bool {
        self.grid.get(self.player.pos.0, self.player.pos.1) == Cell::Stairs
    }

    /// Puts the player on a freshly generated level and populates it.
    fn enter(&mut self, dungeon: Dungeon) {
        let cells = dungeon.grid.width() * dungeon.grid.height();
        self.grid = dungeon.grid;
        self.player.pos = dungeon.start;
        self.monsters.clear();
        self.items.clear();
        self.explored = vec![false; cells];
        self.populate(&dungeon.rooms, dungeon.start);
        self.update_fov();
    }

    /// Monsters and items go into every room but the first, where the
    /// player starts.
    fn populate(&mut self, rooms: &[dungeon::Room], start: (usize, usize)) {
        let weights = MonsterKind::weights(self.depth);
        let max_monsters = (1 + self.depth as usize / 2).min(4);
        for room in rooms.iter().skip(1) {
            let mut free: Vec<(usize, usize)> = room
                .squares()
                .into_iter()
                .filter(|&(x, y)| self.grid.get(x, y) == Cell::Floor && (x, y) != start)
                .collect();

            let monster_count = self.rng.random_range(0..=max_monsters);
            for _ in 0..monster_count {
                let Some(pos) = take_random(&mut free, &mut self.rng) else {
                    break;
                };
                let kind = weights
                    .choose_weighted(&mut self.rng, |&(_, w)| w)
                    .map(|&(kind, _)| kind)
                    .unwrap_or(MonsterKind::Rat);
                self.monsters.push(Monster::new(kind, pos));
            }

            if self.rng.random_bool(0.35)
                && let Some(pos) = take_random(&mut free, &mut self.rng)
            {
                self.items.push(Item {
                    kind: ItemKind::Potion,
                    pos,
                });
            }
            if self.rng.random_bool(0.5)
                && let Some(pos) = take_random(&mut free, &mut self.rng)
            {
                let amount = self.rng.random_range(5..=10 * self.depth + 5);
                self.items.push(Item {
                    kind: ItemKind::Gold(amount),
                    pos,
                });
            }
        }
    }

    /// Returns whether the move took a turn: bumping into a wall does not.
    fn player_move(&mut self, direction: Direction) -> bool {
        let Some((x, y)) = self.step(self.player.pos, direction) else {
            return false;
        };

        if let Some(index) = self.monster_at((x, y)) {
            self.player_attacks(index);
            return true;
        }

        match self.grid.get(x, y) {
            Cell::Wall => false,
            Cell::Door => {
                self.grid.set(x, y, Cell::OpenDoor);
                self.say("You open the door.");
                true
            }
            Cell::Floor | Cell::OpenDoor | Cell::Stairs => {
                self.player.pos = (x, y);
                self.pick_up();
                if self.on_stairs() {
                    self.say("Stairs lead down. Press Enter to descend.");
                }
                true
            }
        }
    }

    fn player_attacks(&mut self, index: usize) {
        let damage = self.roll_damage(self.player.attack);
        let monster = &mut self.monsters[index];
        monster.hp -= damage;
        monster.awake = true;
        let name = monster.kind.name();
        if monster.hp <= 0 {
            self.monsters.remove(index);
            self.player.kills += 1;
            self.say(format!("You kill the {name}."));
        } else {
            self.say(format!("You hit the {name} for {damage}."));
        }
    }

    fn pick_up(&mut self) {
        let Some(index) = self
            .items
            .iter()
            .position(|item| item.pos == self.player.pos)
        else {
            return;
        };
        let item = self.items.remove(index);
        match item.kind {
            ItemKind::Potion => {
                let healed = POTION_HEAL.min(self.player.max_hp - self.player.hp);
                self.player.hp += healed;
                self.say(format!("You drink a potion and recover {healed} HP."));
            }
            ItemKind::Gold(amount) => {
                self.player.gold += amount;
                self.say(format!("You pick up {amount} gold."));
            }
        }
    }

    fn descend(&mut self) {
        if !self.on_stairs() {
            self.say("There are no stairs here.");
            return;
        }
        if self.depth == FINAL_DEPTH {
            self.state = State::Won;
            self.say("You escape the dungeon!");
            return;
        }
        self.depth += 1;
        self.player.max_hp += 3;
        self.player.hp = (self.player.hp + 5).min(self.player.max_hp);
        if self.depth % 2 == 1 {
            self.player.attack += 1;
        }
        let (cx, cy) = level_size(self.depth);
        let dungeon = dungeon::generate(self.algorithm, cx, cy, &mut self.rng);
        self.enter(dungeon);
        self.say(format!(
            "You descend to depth {}. You feel stronger.",
            self.depth
        ));
    }

    /// Monsters that see the player wake up; awake ones walk towards the
    /// player along the shortest path and attack when next to them.
    fn monsters_act(&mut self) {
        let distance = self.grid.distances_from(self.player.pos);
        for i in 0..self.monsters.len() {
            let pos = self.monsters[i].pos;
            let to_player = distance[self.grid.index(pos.0, pos.1)];
            if self.is_visible(pos.0, pos.1) {
                self.monsters[i].awake = true;
            } else if to_player > CHASE_DISTANCE {
                self.monsters[i].awake = false;
            }
            if !self.monsters[i].awake {
                continue;
            }

            if to_player == 1 {
                self.monster_attacks(i);
                if self.state == State::Dead {
                    return;
                }
                continue;
            }

            // Closest free neighbour; ties keep the first found.
            let next = neighbours(pos.0, pos.1, self.grid.width(), self.grid.height())
                .into_iter()
                .filter(|&(x, y)| {
                    self.grid.walkable(x, y)
                        && distance[self.grid.index(x, y)] < to_player
                        && self.monster_at((x, y)).is_none()
                })
                .min_by_key(|&(x, y)| distance[self.grid.index(x, y)]);
            if let Some((x, y)) = next {
                if self.grid.get(x, y) == Cell::Door {
                    self.grid.set(x, y, Cell::OpenDoor);
                } else {
                    self.monsters[i].pos = (x, y);
                }
            }
        }
    }

    fn monster_attacks(&mut self, index: usize) {
        let kind = self.monsters[index].kind;
        let damage = self.roll_damage(kind.attack());
        self.player.hp -= damage;
        self.say(format!("The {} hits you for {damage}.", kind.name()));
        if self.player.hp <= 0 {
            self.player.hp = 0;
            self.state = State::Dead;
            self.say(format!("You were killed by a {}.", kind.name()));
        }
    }

    /// Between half the attack (at least 1) and the full attack.
    fn roll_damage(&mut self, attack: i32) -> i32 {
        self.rng.random_range((attack / 2).max(1)..=attack.max(1))
    }

    fn update_fov(&mut self) {
        self.visible = fov::compute(&self.grid, self.player.pos, SIGHT_RADIUS);
        for (explored, &visible) in self.explored.iter_mut().zip(&self.visible) {
            *explored |= visible;
        }
    }

    fn step(&self, (x, y): (usize, usize), direction: Direction) -> Option<(usize, usize)> {
        let (dx, dy) = direction.offset();
        let nx = x.checked_add_signed(dx)?;
        let ny = y.checked_add_signed(dy)?;
        (nx < self.grid.width() && ny < self.grid.height()).then_some((nx, ny))
    }

    fn monster_at(&self, pos: (usize, usize)) -> Option<usize> {
        self.monsters.iter().position(|m| m.pos == pos)
    }

    fn say(&mut self, message: impl Into<String>) {
        self.log.push(message.into());
        if self.log.len() > LOG_LENGTH {
            self.log.remove(0);
        }
    }
}

/// Levels grow with depth, in corridors (the grid is 2n+1 squares).
fn level_size(depth: u32) -> (usize, usize) {
    let d = depth as usize - 1;
    ((16 + 3 * d).min(40), (12 + 2 * d).min(28))
}

fn take_random(squares: &mut Vec<(usize, usize)>, rng: &mut StdRng) -> Option<(usize, usize)> {
    if squares.is_empty() {
        return None;
    }
    let i = rng.random_range(0..squares.len());
    Some(squares.swap_remove(i))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> Game {
        Game::new(Algorithm::Backtracker, 42)
    }

    /// A walkable, free neighbour of the player and the direction to it.
    fn open_direction(game: &Game) -> (Direction, (usize, usize)) {
        for direction in [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ] {
            let (x, y) = game.step(game.player.pos, direction).unwrap();
            if game.grid.get(x, y) == Cell::Floor && game.monster_at((x, y)).is_none() {
                return (direction, (x, y));
            }
        }
        panic!("player is boxed in");
    }

    #[test]
    fn player_sees_their_own_square() {
        let game = game();
        assert!(game.is_visible(game.player.pos.0, game.player.pos.1));
    }

    #[test]
    fn same_seed_same_game() {
        let (a, b) = (game(), game());
        assert_eq!(a.player.pos, b.player.pos);
        assert_eq!(a.monsters.len(), b.monsters.len());
        assert_eq!(a.items.len(), b.items.len());
    }

    #[test]
    fn moving_takes_a_turn_and_walls_do_not() {
        let mut game = game();
        let (direction, target) = open_direction(&game);
        game.act(Action::Move(direction));
        assert_eq!(game.player.pos, target);
        assert_eq!(game.turns, 1);

        // Walk into the outer wall: nothing happens.
        game.player.pos = (1, 1);
        game.grid.set(1, 1, Cell::Floor);
        game.act(Action::Move(Direction::Up));
        assert_eq!(game.player.pos, (1, 1));
        assert_eq!(game.turns, 1);
    }

    #[test]
    fn bumping_a_monster_attacks_it() {
        let mut game = game();
        game.monsters.clear();
        let (direction, target) = open_direction(&game);
        game.monsters.push(Monster::new(MonsterKind::Orc, target));
        game.act(Action::Move(direction));
        assert_ne!(game.player.pos, target);
        assert!(game.monsters.is_empty() || game.monsters[0].hp < MonsterKind::Orc.max_hp());
    }

    #[test]
    fn adjacent_awake_monsters_attack() {
        let mut game = game();
        game.monsters.clear();
        let (_, target) = open_direction(&game);
        let mut rat = Monster::new(MonsterKind::Rat, target);
        rat.awake = true;
        game.monsters.push(rat);
        game.act(Action::Wait);
        assert!(game.player.hp < game.player.max_hp);
    }

    #[test]
    fn stairs_lead_deeper_and_the_last_ones_win() {
        let mut game = game();
        game.act(Action::Descend);
        assert_eq!(game.depth, 1, "not on the stairs yet");

        for depth in 2..=FINAL_DEPTH {
            let stairs = find(&game.grid, Cell::Stairs);
            game.player.pos = stairs;
            game.act(Action::Descend);
            assert_eq!(game.depth, depth);
            assert_eq!(game.state, State::Playing);
        }
        game.player.pos = find(&game.grid, Cell::Stairs);
        game.act(Action::Descend);
        assert_eq!(game.state, State::Won);
    }

    fn find(grid: &Grid, wanted: Cell) -> (usize, usize) {
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                if grid.get(x, y) == wanted {
                    return (x, y);
                }
            }
        }
        panic!("no {wanted:?}");
    }
}
