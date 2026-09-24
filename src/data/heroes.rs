//! The four playable characters: growth curves, gear and skills.

use serde::{Deserialize, Serialize};

use super::Stats;
use super::items::{ArmorKind, ItemId, WeaponKind};
use super::skills::SkillId;
use crate::gfx::sprites::Sprite;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum HeroId {
    Wren,
    Brannoc,
    Maelis,
    Pip,
}

pub struct HeroDef {
    pub name: &'static str,
    pub title: &'static str,
    pub blurb: &'static str,
    pub sprite: Sprite,
    /// Stats at level 1.
    pub base: Stats,
    /// Gain per level, in tenths (so 23 means +2.3 per level).
    pub growth: Stats,
    pub weapon: WeaponKind,
    pub armors: &'static [ArmorKind],
    /// (level, skill) — a skill is known from that level on.
    pub learnset: &'static [(u32, SkillId)],
    pub start: [ItemId; 2],
}

const fn st(hp: i32, mp: i32, atk: i32, def: i32, mag: i32, res: i32, spd: i32) -> Stats {
    Stats::new(hp, mp, atk, def, mag, res, spd)
}

impl HeroId {
    pub const ALL: [HeroId; 4] = [HeroId::Wren, HeroId::Brannoc, HeroId::Maelis, HeroId::Pip];

    pub fn script_id(self) -> &'static str {
        match self {
            HeroId::Wren => "wren",
            HeroId::Brannoc => "brannoc",
            HeroId::Maelis => "maelis",
            HeroId::Pip => "pip",
        }
    }

    pub fn from_script_id(id: &str) -> Option<HeroId> {
        HeroId::ALL.into_iter().find(|h| h.script_id() == id)
    }

    pub fn def(self) -> &'static HeroDef {
        use SkillId as K;
        match self {
            HeroId::Wren => {
                &const {
                    HeroDef {
                        name: "Wren",
                        title: "Lamplighter",
                        blurb: "Hollowmere's lamplighter. Balanced fighter who wields light.",
                        sprite: Sprite::Wren,
                        base: st(72, 18, 14, 10, 10, 9, 12),
                        growth: st(110, 25, 23, 16, 18, 15, 9),
                        weapon: WeaponKind::Sword,
                        armors: &[ArmorKind::Light, ArmorKind::Heavy],
                        learnset: &[
                            (1, K::LanternStrike),
                            (1, K::Kindle),
                            (6, K::GuardianFlame),
                            (9, K::Flare),
                            (13, K::Beacon),
                            (18, K::Sunburst),
                            (24, K::LanternsMercy),
                            (30, K::LastLight),
                        ],
                        start: [ItemId::RustySword, ItemId::PaddedVest],
                    }
                }
            }
            HeroId::Brannoc => {
                &const {
                    HeroDef {
                        name: "Brannoc",
                        title: "Vanguard",
                        blurb: "Dwarf miner, last of the Ironvein crew. Takes the hits.",
                        sprite: Sprite::Brannoc,
                        base: st(96, 10, 16, 14, 5, 8, 8),
                        growth: st(150, 14, 26, 22, 8, 13, 6),
                        weapon: WeaponKind::Axe,
                        armors: &[ArmorKind::Heavy],
                        learnset: &[
                            (1, K::Cleave),
                            (1, K::StandFast),
                            (8, K::Sunder),
                            (12, K::Earthshaker),
                            (16, K::IronWill),
                            (20, K::Warcry),
                            (25, K::Skullsplitter),
                            (31, K::MountainsWrath),
                        ],
                        start: [ItemId::MinersPick, ItemId::ChainShirt],
                    }
                }
            }
            HeroId::Maelis => {
                &const {
                    HeroDef {
                        name: "Maelis",
                        title: "Archivist",
                        blurb: "Scholar of the Lantern Order. Devastating elemental magic.",
                        sprite: Sprite::Maelis,
                        base: st(55, 40, 7, 7, 18, 14, 11),
                        growth: st(80, 45, 10, 11, 29, 20, 9),
                        weapon: WeaponKind::Staff,
                        armors: &[ArmorKind::Robe],
                        learnset: &[
                            (1, K::Firebolt),
                            (1, K::FrostLance),
                            (10, K::ChainLightning),
                            (12, K::Hex),
                            (14, K::Haste),
                            (17, K::Fireball),
                            (19, K::Siphon),
                            (21, K::Blizzard),
                            (26, K::Thunderstorm),
                            (32, K::Meteor),
                        ],
                        start: [ItemId::OakStaff, ItemId::ApprenticeRobe],
                    }
                }
            }
            HeroId::Pip => {
                &const {
                    HeroDef {
                        name: "Pip",
                        title: "Grovetender",
                        blurb: "Spriggan of the Glowroot Grove. Healer and poisoner.",
                        sprite: Sprite::Pip,
                        base: st(60, 35, 9, 8, 15, 16, 14),
                        growth: st(90, 40, 12, 13, 24, 22, 11),
                        weapon: WeaponKind::Wand,
                        armors: &[ArmorKind::Robe, ArmorKind::Light],
                        learnset: &[
                            (1, K::Mend),
                            (1, K::Bramble),
                            (14, K::Cleanse),
                            (15, K::Bloom),
                            (17, K::Rekindle),
                            (20, K::Wildgrowth),
                            (24, K::ThornStorm),
                            (30, K::Lifebloom),
                        ],
                        start: [ItemId::WillowWand, ItemId::LeafTunic],
                    }
                }
            }
        }
    }
}

impl HeroDef {
    /// Base stats at a level, before equipment.
    pub fn stats_at(&self, level: u32) -> Stats {
        let l = level.saturating_sub(1) as i32;
        let g = |base: i32, growth: i32| base + growth * l / 10;
        Stats {
            hp: g(self.base.hp, self.growth.hp),
            mp: g(self.base.mp, self.growth.mp),
            atk: g(self.base.atk, self.growth.atk),
            def: g(self.base.def, self.growth.def),
            mag: g(self.base.mag, self.growth.mag),
            res: g(self.base.res, self.growth.res),
            spd: g(self.base.spd, self.growth.spd),
        }
    }

    pub fn skills_at(&self, level: u32) -> Vec<SkillId> {
        self.learnset
            .iter()
            .filter(|&&(l, _)| l <= level)
            .map(|&(_, s)| s)
            .collect()
    }

    pub fn can_wear(&self, kind: ArmorKind) -> bool {
        self.armors.contains(&kind)
    }
}

pub const MAX_LEVEL: u32 = 50;

/// Experience needed to go from `level` to `level + 1`.
pub fn xp_to_next(level: u32) -> u32 {
    let l = level as f32;
    (30.0 + 25.0 * l + 2.0 * l * l) as u32
}
