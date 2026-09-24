//! Static game content: every item, skill, hero, enemy and quest, as plain
//! tables. Runtime state (levels, inventories, battles) lives elsewhere and
//! only refers to these by id.

pub mod enemies;
pub mod heroes;
pub mod items;
pub mod quests;
pub mod skills;

use serde::{Deserialize, Serialize};

/// Damage types. Enemies can be weak to, resist, be immune to or absorb each.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Element {
    Physical,
    Fire,
    Frost,
    Shock,
    Light,
    Shadow,
    Nature,
}

impl Element {
    pub const ALL: [Element; 7] = [
        Element::Physical,
        Element::Fire,
        Element::Frost,
        Element::Shock,
        Element::Light,
        Element::Shadow,
        Element::Nature,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Element::Physical => "Physical",
            Element::Fire => "Fire",
            Element::Frost => "Frost",
            Element::Shock => "Shock",
            Element::Light => "Light",
            Element::Shadow => "Shadow",
            Element::Nature => "Nature",
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }

    /// RGB used for damage numbers and effect tints.
    pub fn colour(self) -> [u8; 3] {
        match self {
            Element::Physical => [240, 240, 240],
            Element::Fire => [255, 130, 50],
            Element::Frost => [140, 210, 255],
            Element::Shock => [255, 240, 90],
            Element::Light => [255, 235, 170],
            Element::Shadow => [190, 120, 255],
            Element::Nature => [120, 230, 110],
        }
    }
}

/// How a combatant takes damage of one element.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Affinity {
    Normal,
    Weak,
    Resist,
    Immune,
    Absorb,
}

impl Affinity {
    pub fn multiplier(self) -> f32 {
        match self {
            Affinity::Normal => 1.0,
            Affinity::Weak => 1.5,
            Affinity::Resist => 0.5,
            Affinity::Immune => 0.0,
            Affinity::Absorb => -1.0,
        }
    }
}

/// The seven numbers every combatant has.
#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Stats {
    pub hp: i32,
    pub mp: i32,
    pub atk: i32,
    pub def: i32,
    pub mag: i32,
    pub res: i32,
    pub spd: i32,
}

impl Stats {
    pub const fn new(hp: i32, mp: i32, atk: i32, def: i32, mag: i32, res: i32, spd: i32) -> Self {
        Self {
            hp,
            mp,
            atk,
            def,
            mag,
            res,
            spd,
        }
    }

    pub fn add(self, o: Stats) -> Stats {
        Stats {
            hp: self.hp + o.hp,
            mp: self.mp + o.mp,
            atk: self.atk + o.atk,
            def: self.def + o.def,
            mag: self.mag + o.mag,
            res: self.res + o.res,
            spd: self.spd + o.spd,
        }
    }

    pub fn scale(self, f: f32) -> Stats {
        let s = |v: i32| (v as f32 * f).round() as i32;
        Stats {
            hp: s(self.hp),
            mp: s(self.mp),
            atk: s(self.atk),
            def: s(self.def),
            mag: s(self.mag),
            res: s(self.res),
            spd: s(self.spd),
        }
    }
}

/// Timed conditions on a combatant.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum StatusKind {
    Poison,
    Burn,
    Frozen,
    Stun,
    Blind,
    Weak,
    Sunder,
    Haste,
    Slow,
    Regen,
    Shield,
    Taunt,
    /// Magic up.
    Focus,
    /// Physical up.
    Might,
}

impl StatusKind {
    pub fn name(self) -> &'static str {
        match self {
            StatusKind::Poison => "Poison",
            StatusKind::Burn => "Burn",
            StatusKind::Frozen => "Frozen",
            StatusKind::Stun => "Stun",
            StatusKind::Blind => "Blind",
            StatusKind::Weak => "Weak",
            StatusKind::Sunder => "Sunder",
            StatusKind::Haste => "Haste",
            StatusKind::Slow => "Slow",
            StatusKind::Regen => "Regen",
            StatusKind::Shield => "Shield",
            StatusKind::Taunt => "Taunt",
            StatusKind::Focus => "Focus",
            StatusKind::Might => "Might",
        }
    }

    pub fn is_harmful(self) -> bool {
        matches!(
            self,
            StatusKind::Poison
                | StatusKind::Burn
                | StatusKind::Frozen
                | StatusKind::Stun
                | StatusKind::Blind
                | StatusKind::Weak
                | StatusKind::Sunder
                | StatusKind::Slow
        )
    }

    pub fn describe(self) -> &'static str {
        match self {
            StatusKind::Poison => "Loses HP every turn.",
            StatusKind::Burn => "Loses HP every turn; Frost puts it out.",
            StatusKind::Frozen => "Cannot act. Fire or a hit thaws it.",
            StatusKind::Stun => "Loses the next turn.",
            StatusKind::Blind => "Physical attacks often miss.",
            StatusKind::Weak => "Attack and Magic lowered.",
            StatusKind::Sunder => "Defense and Resistance lowered.",
            StatusKind::Haste => "Acts more often.",
            StatusKind::Slow => "Acts less often.",
            StatusKind::Regen => "Recovers HP every turn.",
            StatusKind::Shield => "Defense and Resistance raised.",
            StatusKind::Taunt => "Draws enemy attacks.",
            StatusKind::Focus => "Magic raised.",
            StatusKind::Might => "Attack raised.",
        }
    }
}
