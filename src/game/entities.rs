#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MonsterKind {
    Rat,
    Goblin,
    Orc,
}

impl MonsterKind {
    pub fn name(self) -> &'static str {
        match self {
            MonsterKind::Rat => "rat",
            MonsterKind::Goblin => "goblin",
            MonsterKind::Orc => "orc",
        }
    }

    pub fn max_hp(self) -> i32 {
        match self {
            MonsterKind::Rat => 4,
            MonsterKind::Goblin => 8,
            MonsterKind::Orc => 14,
        }
    }

    pub fn attack(self) -> i32 {
        match self {
            MonsterKind::Rat => 2,
            MonsterKind::Goblin => 3,
            MonsterKind::Orc => 5,
        }
    }

    /// Relative spawn weights at a given depth: rats early, orcs later.
    pub fn weights(depth: u32) -> [(MonsterKind, u32); 3] {
        let d = depth.saturating_sub(1);
        [
            (MonsterKind::Rat, 10u32.saturating_sub(d)),
            (MonsterKind::Goblin, 3 + d * 2),
            (MonsterKind::Orc, d.saturating_sub(1) * 2),
        ]
    }
}

#[derive(Clone, Debug)]
pub struct Monster {
    pub kind: MonsterKind,
    pub pos: (usize, usize),
    pub hp: i32,
    /// Set once the monster has seen the player; it then gives chase.
    pub awake: bool,
}

impl Monster {
    pub fn new(kind: MonsterKind, pos: (usize, usize)) -> Self {
        Self {
            kind,
            pos,
            hp: kind.max_hp(),
            awake: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ItemKind {
    Potion,
    Gold(u32),
}

#[derive(Clone, Debug)]
pub struct Item {
    pub kind: ItemKind,
    pub pos: (usize, usize),
}

#[derive(Clone, Debug)]
pub struct Player {
    pub pos: (usize, usize),
    pub hp: i32,
    pub max_hp: i32,
    pub attack: i32,
    pub gold: u32,
    pub kills: u32,
}

impl Player {
    pub fn new(pos: (usize, usize)) -> Self {
        Self {
            pos,
            hp: 20,
            max_hp: 20,
            attack: 4,
            gold: 0,
            kills: 0,
        }
    }
}
