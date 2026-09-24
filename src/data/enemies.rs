//! Every monster and boss, their stat shapes and behaviour, and the fixed
//! story battles.

use rand::Rng;
use rand::RngExt;
use serde::{Deserialize, Serialize};

use super::items::ItemId;
use super::skills::SkillId;
use super::{Affinity, Element, Stats};
use crate::gfx::sprites::Sprite;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, PartialOrd, Ord)]
pub enum EnemyId {
    // Act I
    SewerRat,
    CaveBat,
    GiantCentipede,
    Smuggler,
    Cutthroat,
    Ghoul,
    KoboldScavenger,
    // Act II
    DrownedScholar,
    InkSlime,
    HauntedTome,
    ArchiveSentinel,
    Wisp,
    MireEel,
    Phantom,
    // Act III
    Sporeling,
    CaveSpider,
    Weaver,
    GiantSlug,
    HollowOne,
    Toadstool,
    RotWalker,
    // Act IV
    FireImp,
    Salamander,
    EmberGolem,
    AshThrall,
    CinderHound,
    FireElemental,
    Efreet,
    // Act V
    FrostWraith,
    IceBeast,
    PaleShade,
    WinterWolf,
    PaleKnight,
    FrostGiant,
    SoulEater,
    RimeDrake,
    // Bosses
    VexHarlan,
    Gristlemaw,
    Curator,
    MotherOfSpores,
    IronWarden,
    IlsaPale,
    AurelianPale,
    AurelianTrue,
}

/// Multipliers applied to the level curve.
#[derive(Clone, Copy)]
pub struct Shape {
    pub hp: f32,
    pub atk: f32,
    pub def: f32,
    pub mag: f32,
    pub res: f32,
    pub spd: f32,
}

const fn shape(hp: f32, atk: f32, def: f32, mag: f32, res: f32, spd: f32) -> Shape {
    Shape {
        hp,
        atk,
        def,
        mag,
        res,
        spd,
    }
}

pub struct EnemyDef {
    pub name: &'static str,
    pub sprite: Sprite,
    pub shape: Shape,
    pub affinities: &'static [(Element, Affinity)],
    /// (skill, weight)
    pub skills: &'static [(SkillId, u8)],
    /// (item, chance in percent)
    pub drops: &'static [(ItemId, u8)],
    pub reward: f32,
    /// Floors where it roams (inclusive). Bosses: (0, 0).
    pub floors: (u32, u32),
    /// Drawn size in battle, in tiles.
    pub size: f32,
    pub boss: bool,
}

use Affinity::*;
use Element::*;
use SkillId as K;

const W_FIRE: &[(Element, Affinity)] = &[(Fire, Weak)];

impl EnemyId {
    pub fn def(self) -> &'static EnemyDef {
        use EnemyId as E;
        type D = EnemyDef;
        match self {
            E::SewerRat => &const { D { name: "Sewer Rat", sprite: Sprite::SewerRat, shape: shape(0.55, 0.8, 0.6, 0.5, 0.6, 1.2), affinities: W_FIRE, skills: &[(K::Attack, 3), (K::Bite, 3), (K::VenomBite, 1)], drops: &[(ItemId::Tonic, 8)], reward: 0.7, floors: (1, 4), size: 2.2, boss: false } },
            E::CaveBat => &const { D { name: "Cave Bat", sprite: Sprite::CaveBat, shape: shape(0.5, 0.75, 0.6, 0.6, 0.8, 1.5), affinities: &[(Light, Weak), (Shock, Weak)], skills: &[(K::Swoop, 3), (K::Screech, 1)], drops: &[(ItemId::Tonic, 6)], reward: 0.7, floors: (1, 4), size: 2.2, boss: false } },
            E::GiantCentipede => &const { D { name: "Giant Centipede", sprite: Sprite::GiantCentipede, shape: shape(0.9, 0.9, 1.2, 0.5, 0.7, 0.9), affinities: &[(Frost, Weak), (Nature, Resist)], skills: &[(K::VenomBite, 3), (K::Attack, 2)], drops: &[(ItemId::Panacea, 12)], reward: 0.9, floors: (1, 4), size: 2.6, boss: false } },
            E::Smuggler => &const { D { name: "Smuggler", sprite: Sprite::Smuggler, shape: shape(1.0, 1.0, 0.9, 0.6, 0.8, 1.0), affinities: &[], skills: &[(K::Stab, 3), (K::Attack, 2), (K::ThrowKnives, 1)], drops: &[(ItemId::Tonic, 15), (ItemId::SmokeBomb, 6)], reward: 1.0, floors: (2, 4), size: 2.8, boss: false } },
            E::Cutthroat => &const { D { name: "Cutthroat", sprite: Sprite::Cutthroat, shape: shape(1.1, 1.2, 0.9, 0.6, 0.8, 1.1), affinities: &[], skills: &[(K::Backstab, 2), (K::Stab, 2), (K::DirtyTrick, 1)], drops: &[(ItemId::Tonic, 15), (ItemId::Ether, 6)], reward: 1.1, floors: (3, 4), size: 2.8, boss: false } },
            E::Ghoul => &const { D { name: "Ghoul", sprite: Sprite::Ghoul, shape: shape(1.3, 1.0, 0.9, 0.6, 0.9, 0.7), affinities: &[(Light, Weak), (Fire, Weak), (Shadow, Resist), (Nature, Immune)], skills: &[(K::GhoulClaw, 3), (K::Attack, 1)], drops: &[(ItemId::Panacea, 10)], reward: 1.1, floors: (3, 5), size: 2.8, boss: false } },
            E::KoboldScavenger => &const { D { name: "Kobold Scavenger", sprite: Sprite::KoboldScavenger, shape: shape(0.8, 0.9, 0.8, 1.0, 0.8, 1.1), affinities: &[(Frost, Weak)], skills: &[(K::Firepot, 2), (K::Stab, 2)], drops: &[(ItemId::FireFlask, 8), (ItemId::Tonic, 10)], reward: 0.9, floors: (2, 4), size: 2.4, boss: false } },

            E::DrownedScholar => &const { D { name: "Drowned Scholar", sprite: Sprite::DrownedScholar, shape: shape(1.2, 1.0, 0.9, 1.0, 1.0, 0.7), affinities: &[(Fire, Weak), (Light, Weak), (Frost, Resist)], skills: &[(K::Drown, 2), (K::Attack, 2)], drops: &[(ItemId::Ether, 10)], reward: 1.0, floors: (5, 8), size: 2.8, boss: false } },
            E::InkSlime => &const { D { name: "Ink Slime", sprite: Sprite::InkSlime, shape: shape(1.1, 0.9, 1.4, 0.9, 0.6, 0.8), affinities: &[(Physical, Resist), (Shock, Weak), (Fire, Weak)], skills: &[(K::InkSpray, 3), (K::SlimeCoat, 1)], drops: &[(ItemId::Panacea, 12)], reward: 1.0, floors: (5, 8), size: 2.4, boss: false } },
            E::HauntedTome => &const { D { name: "Haunted Tome", sprite: Sprite::HauntedTome, shape: shape(0.7, 0.6, 0.8, 1.3, 1.3, 1.2), affinities: &[(Fire, Weak), (Shadow, Resist)], skills: &[(K::PageStorm, 2), (K::Firepot, 1), (K::Zap, 1)], drops: &[(ItemId::Ether, 12)], reward: 1.0, floors: (5, 8), size: 2.2, boss: false } },
            E::ArchiveSentinel => &const { D { name: "Archive Sentinel", sprite: Sprite::ArchiveSentinel, shape: shape(1.3, 1.2, 1.5, 0.5, 0.7, 0.8), affinities: &[(Light, Weak), (Shadow, Resist), (Frost, Resist), (Nature, Immune)], skills: &[(K::BoneSlash, 3), (K::Harden, 1)], drops: &[(ItemId::Draught, 8)], reward: 1.2, floors: (6, 8), size: 3.0, boss: false } },
            E::Wisp => &const { D { name: "Wisp", sprite: Sprite::Wisp, shape: shape(0.6, 0.5, 2.0, 1.1, 1.0, 1.4), affinities: &[(Physical, Resist), (Frost, Weak), (Light, Absorb)], skills: &[(K::WispLight, 3)], drops: &[(ItemId::Ether, 15)], reward: 1.0, floors: (5, 8), size: 2.0, boss: false } },
            E::MireEel => &const { D { name: "Mire Eel", sprite: Sprite::MireEel, shape: shape(1.0, 1.0, 1.0, 1.2, 0.9, 1.1), affinities: &[(Shock, Absorb), (Frost, Weak), (Nature, Weak)], skills: &[(K::Zap, 3), (K::Bite, 1)], drops: &[(ItemId::Tonic, 15)], reward: 1.0, floors: (5, 7), size: 2.6, boss: false } },
            E::Phantom => &const { D { name: "Phantom", sprite: Sprite::Phantom, shape: shape(0.9, 0.7, 1.35, 1.1, 1.2, 1.05), affinities: &[(Physical, Resist), (Light, Weak), (Shadow, Immune)], skills: &[(K::SoulTouch, 3), (K::Wail, 1)], drops: &[(ItemId::Ether, 12)], reward: 1.2, floors: (6, 9), size: 2.8, boss: false } },

            E::Sporeling => &const { D { name: "Sporeling", sprite: Sprite::Sporeling, shape: shape(0.9, 0.8, 0.9, 0.9, 1.0, 0.9), affinities: &[(Fire, Weak), (Nature, Absorb)], skills: &[(K::SporeCloud, 2), (K::Attack, 1)], drops: &[(ItemId::Panacea, 12), (ItemId::Glowcap, 10)], reward: 0.9, floors: (9, 12), size: 2.4, boss: false } },
            E::CaveSpider => &const { D { name: "Cave Spider", sprite: Sprite::CaveSpider, shape: shape(0.8, 1.0, 0.9, 0.6, 0.8, 1.3), affinities: &[(Fire, Weak)], skills: &[(K::VenomFang, 3), (K::Web, 1)], drops: &[(ItemId::Panacea, 10)], reward: 0.9, floors: (9, 12), size: 2.6, boss: false } },
            E::Weaver => &const { D { name: "Weaver", sprite: Sprite::Weaver, shape: shape(1.3, 1.2, 1.0, 0.8, 0.9, 1.1), affinities: &[(Fire, Weak), (Nature, Resist)], skills: &[(K::VenomFang, 2), (K::Web, 2), (K::Bite, 1)], drops: &[(ItemId::Draught, 8)], reward: 1.2, floors: (10, 12), size: 3.0, boss: false } },
            E::GiantSlug => &const { D { name: "Giant Slug", sprite: Sprite::GiantSlug, shape: shape(1.6, 0.9, 0.7, 0.9, 1.3, 0.6), affinities: &[(Physical, Resist), (Fire, Weak), (Shock, Weak)], skills: &[(K::SlimeCoat, 3), (K::Attack, 1)], drops: &[(ItemId::Draught, 10)], reward: 1.1, floors: (9, 12), size: 2.8, boss: false } },
            E::HollowOne => &const { D { name: "Hollow One", sprite: Sprite::HollowOne, shape: shape(1.1, 0.9, 0.9, 1.0, 1.0, 0.9), affinities: &[(Light, Weak), (Fire, Weak), (Frost, Resist)], skills: &[(K::HollowGrasp, 3), (K::Attack, 1)], drops: &[(ItemId::Ether, 10), (ItemId::HearthBread, 4)], reward: 1.0, floors: (9, 12), size: 2.8, boss: false } },
            E::Toadstool => &const { D { name: "Toadstool", sprite: Sprite::Toadstool, shape: shape(1.0, 0.6, 1.0, 1.1, 1.0, 0.5), affinities: &[(Fire, Weak), (Nature, Absorb)], skills: &[(K::SporeCloud, 3), (K::SlimeCoat, 1)], drops: &[(ItemId::Glowcap, 15), (ItemId::Panacea, 10)], reward: 0.9, floors: (9, 11), size: 2.4, boss: false } },
            E::RotWalker => &const { D { name: "Rot Walker", sprite: Sprite::RotWalker, shape: shape(1.7, 1.3, 1.2, 0.6, 0.9, 0.7), affinities: &[(Fire, Weak), (Nature, Resist), (Shock, Resist)], skills: &[(K::RootLash, 3), (K::Stomp, 1)], drops: &[(ItemId::Draught, 12)], reward: 1.4, floors: (10, 13), size: 3.4, boss: false } },

            E::FireImp => &const { D { name: "Fire Imp", sprite: Sprite::FireImp, shape: shape(0.8, 0.8, 0.8, 1.2, 1.0, 1.4), affinities: &[(Frost, Weak), (Fire, Absorb)], skills: &[(K::EmberSpit, 3), (K::Attack, 1)], drops: &[(ItemId::FireFlask, 10), (ItemId::Ether, 8)], reward: 0.9, floors: (13, 16), size: 2.2, boss: false } },
            E::Salamander => &const { D { name: "Salamander", sprite: Sprite::Salamander, shape: shape(1.2, 1.1, 1.0, 1.0, 0.9, 1.0), affinities: &[(Frost, Weak), (Fire, Immune)], skills: &[(K::FireBreath, 2), (K::AshClaw, 2)], drops: &[(ItemId::Draught, 10)], reward: 1.1, floors: (13, 16), size: 3.0, boss: false } },
            E::EmberGolem => &const { D { name: "Ember Golem", sprite: Sprite::EmberGolem, shape: shape(1.8, 1.3, 1.8, 0.4, 0.6, 0.5), affinities: &[(Physical, Resist), (Frost, Weak), (Shock, Weak), (Fire, Immune), (Nature, Immune)], skills: &[(K::Slam, 3), (K::Harden, 1)], drops: &[(ItemId::StarmetalOre, 45)], reward: 1.5, floors: (13, 16), size: 3.4, boss: false } },
            E::AshThrall => &const { D { name: "Ash Thrall", sprite: Sprite::AshThrall, shape: shape(1.2, 1.2, 1.0, 0.6, 0.9, 0.9), affinities: &[(Frost, Weak), (Light, Weak), (Fire, Resist)], skills: &[(K::AshClaw, 3), (K::Attack, 1)], drops: &[(ItemId::Panacea, 12)], reward: 1.1, floors: (13, 16), size: 2.8, boss: false } },
            E::CinderHound => &const { D { name: "Cinder Hound", sprite: Sprite::CinderHound, shape: shape(1.0, 1.2, 0.9, 0.9, 0.9, 1.4), affinities: &[(Frost, Weak), (Fire, Resist)], skills: &[(K::Bite, 2), (K::FireBreath, 1), (K::Howl, 1)], drops: &[(ItemId::Tonic, 12)], reward: 1.0, floors: (13, 16), size: 2.8, boss: false } },
            E::FireElemental => &const { D { name: "Fire Elemental", sprite: Sprite::FireElemental, shape: shape(1.3, 0.8, 1.3, 1.4, 1.1, 1.0), affinities: &[(Fire, Absorb), (Frost, Weak), (Physical, Resist)], skills: &[(K::Immolate, 2), (K::EmberSpit, 2)], drops: &[(ItemId::HiEther, 6)], reward: 1.3, floors: (14, 16), size: 3.0, boss: false } },
            E::Efreet => &const { D { name: "Efreet", sprite: Sprite::Efreet, shape: shape(1.6, 1.2, 1.1, 1.4, 1.1, 1.0), affinities: &[(Fire, Immune), (Frost, Weak)], skills: &[(K::Hellfire, 2), (K::Immolate, 1), (K::Attack, 1)], drops: &[(ItemId::HiEther, 10), (ItemId::StarmetalOre, 15)], reward: 1.6, floors: (15, 17), size: 3.2, boss: false } },

            E::FrostWraith => &const { D { name: "Frost Wraith", sprite: Sprite::FrostWraith, shape: shape(1.0, 0.7, 1.5, 1.3, 1.2, 1.1), affinities: &[(Frost, Absorb), (Fire, Weak), (Light, Weak), (Physical, Resist)], skills: &[(K::ChillTouch, 3), (K::SoulDrain, 1)], drops: &[(ItemId::HiEther, 8)], reward: 1.1, floors: (17, 20), size: 3.0, boss: false } },
            E::IceBeast => &const { D { name: "Ice Beast", sprite: Sprite::IceBeast, shape: shape(1.4, 1.3, 1.2, 0.8, 0.9, 0.9), affinities: &[(Frost, Absorb), (Fire, Weak)], skills: &[(K::IceClaw, 3), (K::FrostBreath, 1)], drops: &[(ItemId::Draught, 12)], reward: 1.2, floors: (17, 20), size: 3.2, boss: false } },
            E::PaleShade => &const { D { name: "Pale Shade", sprite: Sprite::PaleShade, shape: shape(0.9, 0.7, 1.2, 1.3, 1.1, 1.3), affinities: &[(Shadow, Immune), (Light, Weak), (Physical, Resist)], skills: &[(K::ShadowBolt, 2), (K::Wail, 1)], drops: &[(ItemId::Ether, 12)], reward: 1.0, floors: (17, 20), size: 2.8, boss: false } },
            E::WinterWolf => &const { D { name: "Winter Wolf", sprite: Sprite::WinterWolf, shape: shape(1.1, 1.2, 1.0, 0.7, 0.9, 1.4), affinities: &[(Fire, Weak), (Frost, Resist)], skills: &[(K::Bite, 2), (K::IceClaw, 1), (K::Howl, 1)], drops: &[(ItemId::Tonic, 15)], reward: 1.0, floors: (17, 19), size: 2.8, boss: false } },
            E::PaleKnight => &const { D { name: "Pale Knight", sprite: Sprite::PaleKnight, shape: shape(1.6, 1.4, 1.5, 0.8, 1.0, 0.9), affinities: &[(Light, Weak), (Fire, Weak), (Shadow, Resist), (Frost, Resist)], skills: &[(K::DeathStrike, 2), (K::Attack, 2), (K::Harden, 1)], drops: &[(ItemId::Elixir, 5), (ItemId::Draught, 10)], reward: 1.5, floors: (18, 20), size: 3.2, boss: false } },
            E::FrostGiant => &const { D { name: "Frost Giant", sprite: Sprite::FrostGiant, shape: shape(2.0, 1.5, 1.2, 0.7, 0.9, 0.6), affinities: &[(Fire, Weak), (Frost, Immune)], skills: &[(K::Stomp, 2), (K::Slam, 2), (K::FrostBreath, 1)], drops: &[(ItemId::Elixir, 8)], reward: 1.8, floors: (18, 20), size: 3.8, boss: false } },
            E::SoulEater => &const { D { name: "Soul Eater", sprite: Sprite::SoulEater, shape: shape(1.3, 0.8, 1.1, 1.4, 1.2, 1.0), affinities: &[(Light, Weak), (Shadow, Absorb)], skills: &[(K::SoulDrain, 3), (K::ShadowBolt, 1)], drops: &[(ItemId::HiEther, 10)], reward: 1.3, floors: (18, 20), size: 3.0, boss: false } },
            E::RimeDrake => &const { D { name: "Rime Drake", sprite: Sprite::RimeDrake, shape: shape(2.2, 1.4, 1.3, 1.4, 1.2, 1.0), affinities: &[(Fire, Weak), (Frost, Absorb)], skills: &[(K::FrostBreath, 2), (K::IceClaw, 2), (K::Bite, 1)], drops: &[(ItemId::Elixir, 12)], reward: 2.2, floors: (19, 20), size: 4.0, boss: false } },

            E::VexHarlan => &const { D { name: "Vex Harlan", sprite: Sprite::VexHarlan, shape: shape(5.5, 1.15, 1.0, 0.8, 0.9, 1.1), affinities: &[], skills: &[(K::Stab, 3), (K::Backstab, 2), (K::ThrowKnives, 2), (K::DirtyTrick, 1), (K::Rally, 1)], drops: &[(ItemId::Ether, 100)], reward: 6.0, floors: (0, 0), size: 3.4, boss: true } },
            E::Gristlemaw => &const { D { name: "Gristlemaw, the Rat King", sprite: Sprite::Gristlemaw, shape: shape(5.0, 1.1, 1.1, 0.8, 0.9, 0.9), affinities: &[(Fire, Weak)], skills: &[(K::Bite, 3), (K::Gnash, 2), (K::PlagueSqueal, 2), (K::CallTheSwarm, 1)], drops: &[(ItemId::EmberDown, 100)], reward: 8.0, floors: (0, 0), size: 5.5, boss: true } },
            E::Curator => &const { D { name: "The Curator", sprite: Sprite::Curator, shape: shape(7.0, 0.8, 1.1, 1.08, 1.2, 1.0), affinities: &[(Light, Weak), (Fire, Weak), (Shadow, Resist), (Frost, Resist)], skills: &[(K::ShadowBolt, 3), (K::ForbiddenWord, 2), (K::InkTide, 2), (K::Erase, 1), (K::CallTheShelves, 1)], drops: &[(ItemId::HiEther, 100)], reward: 8.0, floors: (0, 0), size: 5.0, boss: true } },
            E::MotherOfSpores => &const { D { name: "Mother of Spores", sprite: Sprite::MotherOfSpores, shape: shape(16.0, 1.35, 1.15, 1.45, 1.15, 0.85), affinities: &[(Fire, Weak), (Nature, Absorb)], skills: &[(K::RotBloom, 2), (K::MycelialGrasp, 3), (K::SporeCloud, 2), (K::SporeBrood, 1)], drops: &[(ItemId::Elixir, 100)], reward: 9.0, floors: (0, 0), size: 6.0, boss: true } },
            E::IronWarden => &const { D { name: "The Iron Warden", sprite: Sprite::IronWarden, shape: shape(16.5, 1.4, 1.5, 1.15, 1.0, 0.85), affinities: &[(Frost, Weak), (Shock, Weak), (Fire, Immune), (Nature, Immune)], skills: &[(K::Slam, 3), (K::Forgefire, 2), (K::Hammerfall, 2), (K::IronBulwark, 1)], drops: &[(ItemId::Elixir, 100)], reward: 10.0, floors: (0, 0), size: 6.0, boss: true } },
            E::IlsaPale => &const { D { name: "Ilsa, the Pale-Crowned", sprite: Sprite::IlsaPale, shape: shape(19.0, 1.5, 1.3, 1.5, 1.3, 1.3), affinities: &[(Frost, Absorb), (Light, Weak)], skills: &[(K::PaleBlade, 3), (K::CrownOfFrost, 2), (K::WintersEmbrace, 2), (K::Hesitation, 1), (K::Harden, 1)], drops: &[(ItemId::Elixir, 100)], reward: 11.0, floors: (0, 0), size: 4.5, boss: true } },
            E::AurelianPale => &const { D { name: "The Pale", sprite: Sprite::Aurelian, shape: shape(10.0, 1.05, 1.25, 1.2, 1.25, 1.0), affinities: &[(Light, Weak), (Fire, Weak), (Shadow, Absorb), (Frost, Resist)], skills: &[(K::GriefTide, 3), (K::EndlessWinter, 2), (K::Hollowing, 2), (K::ShadowBolt, 2), (K::CallThePale, 1)], drops: &[], reward: 12.0, floors: (0, 0), size: 7.0, boss: true } },
            E::AurelianTrue => &const { D { name: "Aurelian, the Grieving", sprite: Sprite::AurelianTrue, shape: shape(7.5, 1.15, 1.15, 1.45, 1.25, 1.05), affinities: &[(Light, Weak), (Shadow, Resist)], skills: &[(K::LullabyOfAsh, 2), (K::FrozenTears, 2), (K::LastEmbrace, 2), (K::Hollowing, 1)], drops: &[], reward: 14.0, floors: (0, 0), size: 5.0, boss: true } },
        }
    }

    pub const ROAMING: [EnemyId; 36] = {
        use EnemyId::*;
        [
            SewerRat, CaveBat, GiantCentipede, Smuggler, Cutthroat, Ghoul, KoboldScavenger, DrownedScholar, InkSlime,
            HauntedTome, ArchiveSentinel, Wisp, MireEel, Phantom, Sporeling, CaveSpider, Weaver, GiantSlug, HollowOne,
            Toadstool, RotWalker, FireImp, Salamander, EmberGolem, AshThrall, CinderHound, FireElemental, Efreet,
            FrostWraith, IceBeast, PaleShade, WinterWolf, PaleKnight, FrostGiant, SoulEater, RimeDrake,
        ]
    };
}

impl EnemyDef {
    /// Stats at a level: a shared curve bent by the enemy's shape.
    pub fn stats_at(&self, level: u32) -> Stats {
        let l = level as f32;
        let s = &self.shape;
        let r = |v: f32| v.round().max(1.0) as i32;
        Stats {
            hp: r((15.0 + 9.0 * l + 0.1 * l * l) * s.hp),
            mp: 999,
            atk: r((10.0 + 3.5 * l) * s.atk),
            def: r((4.0 + 2.0 * l) * s.def),
            mag: r((9.0 + 3.4 * l) * s.mag),
            res: r((4.0 + 2.0 * l) * s.res),
            spd: r((8.0 + 0.8 * l) * s.spd),
        }
    }

    pub fn affinity(&self, element: Element) -> Affinity {
        self.affinities
            .iter()
            .find(|(e, _)| *e == element)
            .map(|&(_, a)| a)
            .unwrap_or(Affinity::Normal)
    }

    pub fn xp_at(&self, level: u32) -> u32 {
        let l = level as f32;
        ((4.0 + 3.0 * l + 0.12 * l * l) * self.reward) as u32
    }

    pub fn gold_at(&self, level: u32) -> u32 {
        ((3.0 + 2.2 * level as f32) * self.reward) as u32
    }
}

/// The level of ordinary monsters on a floor.
pub fn floor_level(floor: u32) -> u32 {
    (1.0 + 1.7 * (floor.max(1) - 1) as f32).round() as u32
}

/// A random group of monsters for a floor: (enemy, level).
pub fn random_group(floor: u32, rng: &mut impl Rng) -> Vec<(EnemyId, u32)> {
    let pool: Vec<EnemyId> = EnemyId::ROAMING
        .into_iter()
        .filter(|e| {
            let (lo, hi) = e.def().floors;
            (lo..=hi).contains(&floor)
        })
        .collect();
    let (min, max) = match floor {
        1..=2 => (1, 2),
        3..=5 => (1, 3),
        6..=8 => (2, 3),
        _ => (2, 4),
    };
    let count = rng.random_range(min..=max);
    let base = floor_level(floor);
    (0..count)
        .map(|_| {
            let enemy = pool[rng.random_range(0..pool.len())];
            let level = (base as i32 + rng.random_range(-1..=1)).max(1) as u32;
            (enemy, level)
        })
        .collect()
}

/// The scripted fights.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum BattleId {
    Vex,
    Gristlemaw,
    Curator,
    MotherOfSpores,
    IronWarden,
    Ilsa,
    Aurelian,
}

impl BattleId {
    pub const ALL: [BattleId; 7] = [
        BattleId::Vex,
        BattleId::Gristlemaw,
        BattleId::Curator,
        BattleId::MotherOfSpores,
        BattleId::IronWarden,
        BattleId::Ilsa,
        BattleId::Aurelian,
    ];

    pub fn script_id(self) -> &'static str {
        match self {
            BattleId::Vex => "vex",
            BattleId::Gristlemaw => "gristlemaw",
            BattleId::Curator => "curator",
            BattleId::MotherOfSpores => "mother_of_spores",
            BattleId::IronWarden => "iron_warden",
            BattleId::Ilsa => "ilsa",
            BattleId::Aurelian => "aurelian",
        }
    }

    pub fn from_script_id(id: &str) -> Option<BattleId> {
        BattleId::ALL.into_iter().find(|b| b.script_id() == id)
    }

    pub fn formation(self) -> Vec<(EnemyId, u32)> {
        use EnemyId as E;
        match self {
            BattleId::Vex => vec![(E::Smuggler, 5), (E::VexHarlan, 6), (E::Smuggler, 5)],
            BattleId::Gristlemaw => vec![(E::SewerRat, 6), (E::Gristlemaw, 8), (E::SewerRat, 6)],
            BattleId::Curator => vec![(E::HauntedTome, 13), (E::Curator, 15), (E::HauntedTome, 13)],
            BattleId::MotherOfSpores => vec![(E::Sporeling, 22), (E::MotherOfSpores, 24), (E::Sporeling, 22)],
            BattleId::IronWarden => vec![(E::AshThrall, 29), (E::IronWarden, 31), (E::AshThrall, 29)],
            BattleId::Ilsa => vec![(E::IlsaPale, 37)],
            BattleId::Aurelian => vec![(E::AurelianPale, 38)],
        }
    }

    /// The scripted second form, if the boss has one.
    pub fn second_phase(self) -> Option<(EnemyId, u32)> {
        match self {
            BattleId::Aurelian => Some((EnemyId::AurelianTrue, 38)),
            _ => None,
        }
    }
}
