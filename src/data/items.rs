//! Consumables, quest items and equipment.

use serde::{Deserialize, Serialize};

use super::{Element, Stats, StatusKind};
use crate::gfx::sprites::Sprite;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WeaponKind {
    Sword,
    Axe,
    Staff,
    Wand,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArmorKind {
    Light,
    Heavy,
    Robe,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EquipSlot {
    Weapon,
    Armor,
    Accessory,
}

/// What an accessory (or legendary weapon) does beyond its stats.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Special {
    None,
    /// Halves damage of one element.
    Resist(Element),
    /// Extra critical chance in percent.
    CritUp(u8),
    /// Recovers a little HP at the start of each of the wearer's turns.
    Regen,
    /// Recovers a little MP at the start of each of the wearer's turns.
    MpRegen,
    /// Harmful statuses land half as often.
    StatusGuard,
    /// Starts every battle with Haste.
    FirstStrike,
    /// Adds an element to normal attacks.
    AttackElement(Element),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Use {
    Heal(i32),
    HealAll(i32),
    HealFull,
    Mp(i32),
    Revive(f32),
    Cure,
    DamageAll(Element, i32),
    DamageOne(Element, i32, Option<StatusKind>),
    Escape,
    Warp,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ItemKind {
    Consumable(Use),
    Weapon(WeaponKind),
    Armor(ArmorKind),
    Accessory,
    Quest,
}

pub struct ItemDef {
    pub name: &'static str,
    pub desc: &'static str,
    pub sprite: Sprite,
    pub price: u32,
    pub kind: ItemKind,
    pub stats: Stats,
    pub special: Special,
    /// Shop tier: sold once the story reaches this act (0 = never sold).
    pub tier: u8,
}

impl ItemDef {
    const fn new(
        name: &'static str,
        desc: &'static str,
        sprite: Sprite,
        price: u32,
        kind: ItemKind,
    ) -> Self {
        Self {
            name,
            desc,
            sprite,
            price,
            kind,
            stats: Stats::new(0, 0, 0, 0, 0, 0, 0),
            special: Special::None,
            tier: 0,
        }
    }
    const fn stats(mut self, stats: Stats) -> Self {
        self.stats = stats;
        self
    }
    const fn special(mut self, special: Special) -> Self {
        self.special = special;
        self
    }
    const fn tier(mut self, tier: u8) -> Self {
        self.tier = tier;
        self
    }

    pub fn slot(&self) -> Option<EquipSlot> {
        match self.kind {
            ItemKind::Weapon(_) => Some(EquipSlot::Weapon),
            ItemKind::Armor(_) => Some(EquipSlot::Armor),
            ItemKind::Accessory => Some(EquipSlot::Accessory),
            _ => None,
        }
    }

    pub fn usable_in_battle(&self) -> bool {
        matches!(self.kind, ItemKind::Consumable(u) if !matches!(u, Use::Warp))
    }

    pub fn usable_in_field(&self) -> bool {
        matches!(
            self.kind,
            ItemKind::Consumable(
                Use::Heal(_)
                    | Use::HealAll(_)
                    | Use::HealFull
                    | Use::Mp(_)
                    | Use::Revive(_)
                    | Use::Cure
                    | Use::Warp
            )
        )
    }

    pub fn sell_price(&self) -> u32 {
        self.price / 2
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ItemId {
    // Consumables
    Tonic,
    Draught,
    Elixir,
    Ether,
    HiEther,
    EmberDown,
    Panacea,
    HearthBread,
    FireFlask,
    FrostFlask,
    SmokeBomb,
    WaystoneShard,
    // Quest
    Glowcap,
    LostPage,
    HolyRelic,
    CrewTag,
    StarmetalOre,
    EverbloomSeed,
    IlsaLantern,
    // Swords (Wren)
    RustySword,
    IronSword,
    SteelSword,
    RunedBlade,
    MoonsteelSaber,
    DawnEdge,
    Emberbrand,
    // Axes (Brannoc)
    MinersPick,
    BattleAxe,
    DwarvenAxe,
    ForgefireAxe,
    RunicGreataxe,
    Worldsplitter,
    IronveinAxe,
    // Staves (Maelis)
    OakStaff,
    CrystalStaff,
    ArchivistRod,
    StarglassStaff,
    VoidScepter,
    SagesAstrolabe,
    CodexOfStars,
    // Wands (Pip)
    WillowWand,
    ThornwoodWand,
    BloomWand,
    SunrootWand,
    VerdantScepter,
    ElderbarkRod,
    GrovewoodStaff,
    // Light armour
    PaddedVest,
    LeatherArmor,
    StuddedLeather,
    RangerCoat,
    Shadowweave,
    WardensCoat,
    // Heavy armour
    ChainShirt,
    ScaleMail,
    DwarvenPlate,
    EmberforgedPlate,
    PaleguardPlate,
    HearthguardPlate,
    GuardMail,
    // Robes
    LeafTunic,
    ApprenticeRobe,
    ScholarRobe,
    SpidersilkRobe,
    EmberweaveRobe,
    StarlightRobe,
    AurorasMantle,
    // Accessories
    CopperRing,
    VigorRing,
    FocusRing,
    QuickstepAnklet,
    WardingCharm,
    FireWard,
    FrostWard,
    ShadowWard,
    ClarityPendant,
    SoulGem,
    HestaCharm,
    TobinCharm,
    AmuletOfDawn,
}

use Element::*;
use ItemKind::*;
use Sprite as Sp;
use WeaponKind::*;

const fn st(hp: i32, mp: i32, atk: i32, def: i32, mag: i32, res: i32, spd: i32) -> Stats {
    Stats::new(hp, mp, atk, def, mag, res, spd)
}

impl ItemId {
    pub const ALL: [ItemId; 80] = {
        use ItemId::*;
        [
            Tonic,
            Draught,
            Elixir,
            Ether,
            HiEther,
            EmberDown,
            Panacea,
            HearthBread,
            FireFlask,
            FrostFlask,
            SmokeBomb,
            WaystoneShard,
            Glowcap,
            LostPage,
            HolyRelic,
            CrewTag,
            StarmetalOre,
            EverbloomSeed,
            IlsaLantern,
            RustySword,
            IronSword,
            SteelSword,
            RunedBlade,
            MoonsteelSaber,
            DawnEdge,
            Emberbrand,
            MinersPick,
            BattleAxe,
            DwarvenAxe,
            ForgefireAxe,
            RunicGreataxe,
            Worldsplitter,
            IronveinAxe,
            OakStaff,
            CrystalStaff,
            ArchivistRod,
            StarglassStaff,
            VoidScepter,
            SagesAstrolabe,
            CodexOfStars,
            WillowWand,
            ThornwoodWand,
            BloomWand,
            SunrootWand,
            VerdantScepter,
            ElderbarkRod,
            GrovewoodStaff,
            PaddedVest,
            LeatherArmor,
            StuddedLeather,
            RangerCoat,
            Shadowweave,
            WardensCoat,
            ChainShirt,
            ScaleMail,
            DwarvenPlate,
            EmberforgedPlate,
            PaleguardPlate,
            HearthguardPlate,
            GuardMail,
            LeafTunic,
            ApprenticeRobe,
            ScholarRobe,
            SpidersilkRobe,
            EmberweaveRobe,
            StarlightRobe,
            AurorasMantle,
            CopperRing,
            VigorRing,
            FocusRing,
            QuickstepAnklet,
            WardingCharm,
            FireWard,
            FrostWard,
            ShadowWard,
            ClarityPendant,
            SoulGem,
            HestaCharm,
            TobinCharm,
            AmuletOfDawn,
        ]
    };

    /// The snake_case id used by the story scripts.
    pub fn script_id(self) -> String {
        let name = format!("{self:?}");
        let mut out = String::new();
        for (i, c) in name.chars().enumerate() {
            if c.is_ascii_uppercase() {
                if i > 0 {
                    out.push('_');
                }
                out.push(c.to_ascii_lowercase());
            } else {
                out.push(c);
            }
        }
        out
    }

    pub fn from_script_id(id: &str) -> Option<ItemId> {
        ItemId::ALL.into_iter().find(|item| item.script_id() == id)
    }

    pub fn def(self) -> &'static ItemDef {
        use ItemId as I;
        type D = ItemDef;
        match self {
            I::Tonic => {
                &const {
                    D::new(
                        "Tonic",
                        "Restores 80 HP to one ally.",
                        Sp::ItemPotionRed,
                        20,
                        Consumable(Use::Heal(80)),
                    )
                    .tier(1)
                }
            }
            I::Draught => {
                &const {
                    D::new(
                        "Draught",
                        "Restores 280 HP to one ally.",
                        Sp::ItemPotionRed,
                        80,
                        Consumable(Use::Heal(280)),
                    )
                    .tier(2)
                }
            }
            I::Elixir => {
                &const {
                    D::new(
                        "Elixir",
                        "Fully restores one ally's HP.",
                        Sp::ItemPotionGold,
                        300,
                        Consumable(Use::HealFull),
                    )
                    .tier(4)
                }
            }
            I::Ether => {
                &const {
                    D::new(
                        "Ether",
                        "Restores 30 MP to one ally.",
                        Sp::ItemPotionBlue,
                        60,
                        Consumable(Use::Mp(30)),
                    )
                    .tier(1)
                }
            }
            I::HiEther => {
                &const {
                    D::new(
                        "Hi-Ether",
                        "Restores 100 MP to one ally.",
                        Sp::ItemPotionBlue,
                        220,
                        Consumable(Use::Mp(100)),
                    )
                    .tier(3)
                }
            }
            I::EmberDown => {
                &const {
                    D::new(
                        "Ember Down",
                        "A feather still warm from the flame. Revives a fallen ally with half HP.",
                        Sp::ItemFeather,
                        150,
                        Consumable(Use::Revive(0.5)),
                    )
                    .tier(1)
                }
            }
            I::Panacea => {
                &const {
                    D::new(
                        "Panacea",
                        "Cures every ailment on one ally.",
                        Sp::ItemPotionGreen,
                        40,
                        Consumable(Use::Cure),
                    )
                    .tier(1)
                }
            }
            I::HearthBread => {
                &const {
                    D::new(
                        "Hearth Bread",
                        "Bess's bread, still warm. Restores 150 HP to the whole party.",
                        Sp::ItemMushroom,
                        120,
                        Consumable(Use::HealAll(150)),
                    )
                    .tier(2)
                }
            }
            I::FireFlask => {
                &const {
                    D::new(
                        "Fire Flask",
                        "Deals 120 Fire damage to all foes.",
                        Sp::ItemFlaskFire,
                        90,
                        Consumable(Use::DamageAll(Fire, 120)),
                    )
                    .tier(2)
                }
            }
            I::FrostFlask => {
                &const {
                    D::new(
                        "Frost Flask",
                        "Deals 250 Frost damage to one foe; may Freeze.",
                        Sp::ItemFlaskFrost,
                        110,
                        Consumable(Use::DamageOne(Frost, 250, Some(StatusKind::Frozen))),
                    )
                    .tier(3)
                }
            }
            I::SmokeBomb => {
                &const {
                    D::new(
                        "Smoke Bomb",
                        "Guarantees escape from a normal battle.",
                        Sp::ItemSmoke,
                        50,
                        Consumable(Use::Escape),
                    )
                    .tier(1)
                }
            }
            I::WaystoneShard => {
                &const {
                    D::new(
                        "Waystone Shard",
                        "Crush it to return to Hollowmere from the Deep.",
                        Sp::ItemScroll,
                        60,
                        Consumable(Use::Warp),
                    )
                    .tier(1)
                }
            }

            I::Glowcap => {
                &const {
                    D::new(
                        "Glowcap",
                        "A mushroom that glows without heat. Tobin wants one.",
                        Sp::ItemMushroom,
                        0,
                        Quest,
                    )
                }
            }
            I::LostPage => {
                &const {
                    D::new(
                        "Lost Page",
                        "A page torn from the Forbidden Index.",
                        Sp::ItemPage,
                        0,
                        Quest,
                    )
                }
            }
            I::HolyRelic => {
                &const {
                    D::new(
                        "Holy Relic",
                        "A keepsake of one of the first Wardens.",
                        Sp::ItemRelic,
                        0,
                        Quest,
                    )
                }
            }
            I::CrewTag => {
                &const {
                    D::new(
                        "Crew Tag",
                        "An iron tag stamped with the Ironvein crest and a name.",
                        Sp::ItemTag,
                        0,
                        Quest,
                    )
                }
            }
            I::StarmetalOre => {
                &const {
                    D::new(
                        "Starmetal Ore",
                        "Ore that fell from the sky ages ago. Dagna wants it.",
                        Sp::ItemOre,
                        0,
                        Quest,
                    )
                }
            }
            I::EverbloomSeed => {
                &const {
                    D::new(
                        "Everbloom Seed",
                        "The last seed of the Glowroot Grove.",
                        Sp::ItemSeed,
                        0,
                        Quest,
                    )
                }
            }
            I::IlsaLantern => {
                &const {
                    D::new(
                        "Ilsa's Lantern",
                        "Your sister's lantern. Still faintly warm.",
                        Sp::ItemRelic,
                        0,
                        Quest,
                    )
                }
            }

            I::RustySword => {
                &const {
                    D::new(
                        "Father's Sword",
                        "Father's old sword, kept oiled for twenty years.",
                        Sp::ItemSword,
                        10,
                        Weapon(Sword),
                    )
                    .stats(st(0, 0, 8, 0, 0, 0, 0))
                }
            }
            I::IronSword => {
                &const {
                    D::new(
                        "Iron Sword",
                        "Honest iron.",
                        Sp::ItemSword,
                        90,
                        Weapon(Sword),
                    )
                    .stats(st(0, 0, 11, 0, 0, 0, 0))
                    .tier(1)
                }
            }
            I::SteelSword => {
                &const {
                    D::new(
                        "Steel Sword",
                        "Dagna's steady work.",
                        Sp::ItemSword,
                        260,
                        Weapon(Sword),
                    )
                    .stats(st(0, 0, 21, 0, 2, 0, 0))
                    .tier(2)
                }
            }
            I::RunedBlade => {
                &const {
                    D::new(
                        "Runed Blade",
                        "Runes of the old Order glimmer along the edge.",
                        Sp::ItemSwordFine,
                        600,
                        Weapon(Sword),
                    )
                    .stats(st(0, 0, 34, 0, 6, 0, 0))
                    .tier(3)
                }
            }
            I::MoonsteelSaber => {
                &const {
                    D::new(
                        "Moonsteel Saber",
                        "Light as a lantern's shadow.",
                        Sp::ItemSwordFine,
                        1200,
                        Weapon(Sword),
                    )
                    .stats(st(0, 0, 49, 0, 9, 0, 3))
                    .tier(4)
                }
            }
            I::DawnEdge => {
                &const {
                    D::new(
                        "Dawn Edge",
                        "Forged for the last Wardens.",
                        Sp::ItemSwordFine,
                        2200,
                        Weapon(Sword),
                    )
                    .stats(st(0, 0, 66, 0, 12, 0, 4))
                    .tier(5)
                }
            }
            I::Emberbrand => {
                &const {
                    D::new(
                        "Emberbrand",
                        "Starmetal forged by Dagna. Its attacks carry Light.",
                        Sp::ItemSwordFine,
                        0,
                        Weapon(Sword),
                    )
                    .stats(st(0, 10, 78, 0, 18, 4, 5))
                    .special(Special::AttackElement(Light))
                }
            }

            I::MinersPick => {
                &const {
                    D::new(
                        "Miner's Pick",
                        "It has dug through worse than you.",
                        Sp::ItemHammer,
                        10,
                        Weapon(Axe),
                    )
                    .stats(st(0, 0, 6, 0, 0, 0, 0))
                }
            }
            I::BattleAxe => {
                &const {
                    D::new(
                        "Battle Axe",
                        "Heavy and dependable.",
                        Sp::ItemAxe,
                        90,
                        Weapon(Axe),
                    )
                    .stats(st(0, 0, 13, 0, 0, 0, -1))
                    .tier(1)
                }
            }
            I::DwarvenAxe => {
                &const {
                    D::new(
                        "Dwarven Axe",
                        "Balanced the way only dwarves bother to.",
                        Sp::ItemAxe,
                        260,
                        Weapon(Axe),
                    )
                    .stats(st(0, 0, 24, 2, 0, 0, 0))
                    .tier(2)
                }
            }
            I::ForgefireAxe => {
                &const {
                    D::new(
                        "Forgefire Axe",
                        "The head never quite cools.",
                        Sp::ItemAxe,
                        600,
                        Weapon(Axe),
                    )
                    .stats(st(0, 0, 38, 3, 0, 0, 0))
                    .tier(3)
                }
            }
            I::RunicGreataxe => {
                &const {
                    D::new(
                        "Runic Greataxe",
                        "Runes of breaking and holding.",
                        Sp::ItemAxe,
                        1200,
                        Weapon(Axe),
                    )
                    .stats(st(20, 0, 54, 4, 0, 0, 0))
                    .tier(4)
                }
            }
            I::Worldsplitter => {
                &const {
                    D::new(
                        "Worldsplitter",
                        "A dwarven legend, or a copy of one.",
                        Sp::ItemAxe,
                        2200,
                        Weapon(Axe),
                    )
                    .stats(st(40, 0, 72, 6, 0, 0, 0))
                    .tier(5)
                }
            }
            I::IronveinAxe => {
                &const {
                    D::new(
                        "Ironvein Axe",
                        "The crew's axe, carried home at last. Crits often.",
                        Sp::ItemAxe,
                        0,
                        Weapon(Axe),
                    )
                    .stats(st(60, 0, 84, 10, 0, 6, 2))
                    .special(Special::CritUp(15))
                }
            }

            I::OakStaff => {
                &const {
                    D::new(
                        "Oak Staff",
                        "A walking stick with ambitions.",
                        Sp::ItemStaff,
                        10,
                        Weapon(Staff),
                    )
                    .stats(st(0, 0, 2, 0, 5, 0, 0))
                }
            }
            I::CrystalStaff => {
                &const {
                    D::new(
                        "Crystal Staff",
                        "A quartz focus.",
                        Sp::ItemStaff,
                        90,
                        Weapon(Staff),
                    )
                    .stats(st(0, 5, 3, 0, 12, 2, 0))
                    .tier(1)
                }
            }
            I::ArchivistRod => {
                &const {
                    D::new(
                        "Archivist's Rod",
                        "Standard issue, footnotes included.",
                        Sp::ItemStaff,
                        260,
                        Weapon(Staff),
                    )
                    .stats(st(0, 10, 4, 0, 22, 4, 0))
                    .tier(2)
                }
            }
            I::StarglassStaff => {
                &const {
                    D::new(
                        "Starglass Staff",
                        "Holds a little night sky.",
                        Sp::ItemStaff,
                        600,
                        Weapon(Staff),
                    )
                    .stats(st(0, 15, 5, 0, 35, 6, 0))
                    .tier(3)
                }
            }
            I::VoidScepter => {
                &const {
                    D::new(
                        "Void Scepter",
                        "Drinks the light around it.",
                        Sp::ItemWand,
                        1200,
                        Weapon(Staff),
                    )
                    .stats(st(0, 20, 6, 0, 50, 8, 0))
                    .tier(4)
                }
            }
            I::SagesAstrolabe => {
                &const {
                    D::new(
                        "Sage's Astrolabe",
                        "Maps the stars, and the spells between them.",
                        Sp::ItemStaff,
                        2200,
                        Weapon(Staff),
                    )
                    .stats(st(0, 30, 8, 0, 67, 10, 2))
                    .tier(5)
                }
            }
            I::CodexOfStars => {
                &const {
                    D::new(
                        "Codex of Stars",
                        "The Forbidden Index, reassembled. Restores MP each turn.",
                        Sp::ItemBook,
                        0,
                        Weapon(Staff),
                    )
                    .stats(st(0, 50, 8, 4, 80, 14, 3))
                    .special(Special::MpRegen)
                }
            }

            I::WillowWand => {
                &const {
                    D::new(
                        "Willow Wand",
                        "Pip grew it.",
                        Sp::ItemWand,
                        10,
                        Weapon(Wand),
                    )
                    .stats(st(0, 0, 2, 0, 4, 2, 0))
                }
            }
            I::ThornwoodWand => {
                &const {
                    D::new(
                        "Thornwood Wand",
                        "Prickly, like its owner claims not to be.",
                        Sp::ItemWand,
                        90,
                        Weapon(Wand),
                    )
                    .stats(st(0, 5, 4, 0, 11, 4, 0))
                    .tier(1)
                }
            }
            I::BloomWand => {
                &const {
                    D::new(
                        "Bloom Wand",
                        "Flowers open when it is waved.",
                        Sp::ItemWand,
                        260,
                        Weapon(Wand),
                    )
                    .stats(st(0, 10, 5, 0, 20, 7, 0))
                    .tier(2)
                }
            }
            I::SunrootWand => {
                &const {
                    D::new(
                        "Sunroot Wand",
                        "Root of a plant that remembers the sun.",
                        Sp::ItemWand,
                        600,
                        Weapon(Wand),
                    )
                    .stats(st(0, 15, 6, 0, 32, 10, 0))
                    .tier(3)
                }
            }
            I::VerdantScepter => {
                &const {
                    D::new(
                        "Verdant Scepter",
                        "Moss grows on it in seconds.",
                        Sp::ItemStaff,
                        1200,
                        Weapon(Wand),
                    )
                    .stats(st(0, 20, 7, 0, 46, 13, 0))
                    .tier(4)
                }
            }
            I::ElderbarkRod => {
                &const {
                    D::new(
                        "Elderbark Rod",
                        "Bark from a tree older than the town.",
                        Sp::ItemStaff,
                        2200,
                        Weapon(Wand),
                    )
                    .stats(st(0, 30, 8, 0, 62, 17, 2))
                    .tier(5)
                }
            }
            I::GrovewoodStaff => {
                &const {
                    D::new(
                        "Grovewood Staff",
                        "Grown from the Everbloom. Its bearer regenerates.",
                        Sp::ItemStaff,
                        0,
                        Weapon(Wand),
                    )
                    .stats(st(40, 40, 10, 6, 74, 22, 4))
                    .special(Special::Regen)
                }
            }

            I::PaddedVest => {
                &const {
                    D::new(
                        "Lamplighter's Coat",
                        "Quilted, oil-stained, warm.",
                        Sp::ItemLightArmor,
                        10,
                        Armor(ArmorKind::Light),
                    )
                    .stats(st(10, 0, 0, 5, 0, 3, 0))
                }
            }
            I::LeatherArmor => {
                &const {
                    D::new(
                        "Leather Armor",
                        "Boiled and stitched.",
                        Sp::ItemLightArmor,
                        90,
                        Armor(ArmorKind::Light),
                    )
                    .stats(st(0, 0, 0, 8, 0, 4, 0))
                    .tier(1)
                }
            }
            I::StuddedLeather => {
                &const {
                    D::new(
                        "Studded Leather",
                        "Rivets where it counts.",
                        Sp::ItemLightArmor,
                        260,
                        Armor(ArmorKind::Light),
                    )
                    .stats(st(10, 0, 0, 15, 0, 8, 0))
                    .tier(2)
                }
            }
            I::RangerCoat => {
                &const {
                    D::new(
                        "Ranger Coat",
                        "Waxed against damp and worse.",
                        Sp::ItemLightArmor,
                        600,
                        Armor(ArmorKind::Light),
                    )
                    .stats(st(20, 0, 0, 24, 0, 14, 1))
                    .tier(3)
                }
            }
            I::Shadowweave => {
                &const {
                    D::new(
                        "Shadowweave",
                        "Woven in the dark, for the dark.",
                        Sp::ItemLightArmor,
                        1200,
                        Armor(ArmorKind::Light),
                    )
                    .stats(st(30, 0, 0, 35, 0, 22, 2))
                    .tier(4)
                }
            }
            I::WardensCoat => {
                &const {
                    D::new(
                        "Warden's Coat",
                        "Cut for a Warden. It fits you.",
                        Sp::ItemLightArmor,
                        2200,
                        Armor(ArmorKind::Light),
                    )
                    .stats(st(50, 10, 0, 47, 0, 32, 3))
                    .tier(5)
                }
            }

            I::ChainShirt => {
                &const {
                    D::new(
                        "Chain Shirt",
                        "Old links, mostly unbroken.",
                        Sp::ItemHeavyArmor,
                        10,
                        Armor(ArmorKind::Heavy),
                    )
                    .stats(st(0, 0, 0, 5, 0, 1, 0))
                }
            }
            I::ScaleMail => {
                &const {
                    D::new(
                        "Scale Mail",
                        "Heavy, loud, reassuring.",
                        Sp::ItemHeavyArmor,
                        90,
                        Armor(ArmorKind::Heavy),
                    )
                    .stats(st(0, 0, 0, 11, 0, 3, -1))
                    .tier(1)
                }
            }
            I::DwarvenPlate => {
                &const {
                    D::new(
                        "Dwarven Plate",
                        "Made to survive cave-ins.",
                        Sp::ItemHeavyArmor,
                        260,
                        Armor(ArmorKind::Heavy),
                    )
                    .stats(st(15, 0, 0, 20, 0, 6, -1))
                    .tier(2)
                }
            }
            I::EmberforgedPlate => {
                &const {
                    D::new(
                        "Emberforged Plate",
                        "Tempered in the Forge's last fire.",
                        Sp::ItemHeavyArmor,
                        600,
                        Armor(ArmorKind::Heavy),
                    )
                    .stats(st(30, 0, 0, 31, 0, 11, -1))
                    .tier(3)
                }
            }
            I::PaleguardPlate => {
                &const {
                    D::new(
                        "Paleguard Plate",
                        "Frost slides off it.",
                        Sp::ItemHeavyArmor,
                        1200,
                        Armor(ArmorKind::Heavy),
                    )
                    .stats(st(50, 0, 0, 45, 0, 18, -1))
                    .tier(4)
                }
            }
            I::HearthguardPlate => {
                &const {
                    D::new(
                        "Hearthguard Plate",
                        "Warm to the touch, always.",
                        Sp::ItemHeavyArmor,
                        2200,
                        Armor(ArmorKind::Heavy),
                    )
                    .stats(st(80, 0, 0, 60, 0, 26, 0))
                    .tier(5)
                }
            }
            I::GuardMail => {
                &const {
                    D::new(
                        "Guard Mail",
                        "Captain Rennick's spare. Solid.",
                        Sp::ItemHeavyArmor,
                        0,
                        Armor(ArmorKind::Heavy),
                    )
                    .stats(st(20, 0, 2, 22, 0, 8, 0))
                }
            }

            I::LeafTunic => {
                &const {
                    D::new(
                        "Leaf Tunic",
                        "Leaves, persuaded to hold together.",
                        Sp::ItemRobe,
                        10,
                        Armor(ArmorKind::Robe),
                    )
                    .stats(st(0, 5, 0, 2, 1, 3, 0))
                }
            }
            I::ApprenticeRobe => {
                &const {
                    D::new(
                        "Apprentice Robe",
                        "Ink-stained.",
                        Sp::ItemRobe,
                        10,
                        Armor(ArmorKind::Robe),
                    )
                    .stats(st(0, 5, 0, 2, 2, 3, 0))
                }
            }
            I::ScholarRobe => {
                &const {
                    D::new(
                        "Scholar's Robe",
                        "Deep pockets for deep books.",
                        Sp::ItemRobe,
                        90,
                        Armor(ArmorKind::Robe),
                    )
                    .stats(st(0, 10, 0, 5, 3, 7, 0))
                    .tier(1)
                }
            }
            I::SpidersilkRobe => {
                &const {
                    D::new(
                        "Spidersilk Robe",
                        "Light, strong, faintly sticky.",
                        Sp::ItemRobe,
                        260,
                        Armor(ArmorKind::Robe),
                    )
                    .stats(st(10, 15, 0, 10, 5, 13, 1))
                    .tier(2)
                }
            }
            I::EmberweaveRobe => {
                &const {
                    D::new(
                        "Emberweave Robe",
                        "Threads spun from forge-fire.",
                        Sp::ItemRobe,
                        600,
                        Armor(ArmorKind::Robe),
                    )
                    .stats(st(15, 20, 0, 16, 8, 21, 1))
                    .tier(3)
                }
            }
            I::StarlightRobe => {
                &const {
                    D::new(
                        "Starlight Robe",
                        "It glows faintly in the dark.",
                        Sp::ItemRobe,
                        1200,
                        Armor(ArmorKind::Robe),
                    )
                    .stats(st(25, 30, 0, 24, 12, 31, 2))
                    .tier(4)
                }
            }
            I::AurorasMantle => {
                &const {
                    D::new(
                        "Aurora's Mantle",
                        "Colours no one in Hollowmere has a name for.",
                        Sp::ItemRobe,
                        2200,
                        Armor(ArmorKind::Robe),
                    )
                    .stats(st(40, 40, 0, 33, 16, 43, 3))
                    .tier(5)
                }
            }

            I::CopperRing => {
                &const {
                    D::new("Copper Ring", "+4 Defense.", Sp::ItemRing, 60, Accessory)
                        .stats(st(0, 0, 0, 4, 0, 2, 0))
                        .tier(1)
                }
            }
            I::VigorRing => {
                &const {
                    D::new("Vigor Ring", "+40 max HP.", Sp::ItemRing, 300, Accessory)
                        .stats(st(40, 0, 0, 0, 0, 0, 0))
                        .tier(2)
                }
            }
            I::FocusRing => {
                &const {
                    D::new(
                        "Focus Ring",
                        "+8 Magic, +10 MP.",
                        Sp::ItemRing,
                        300,
                        Accessory,
                    )
                    .stats(st(0, 10, 0, 0, 8, 0, 0))
                    .tier(2)
                }
            }
            I::QuickstepAnklet => {
                &const {
                    D::new(
                        "Quickstep Anklet",
                        "+5 Speed.",
                        Sp::ItemAmulet,
                        450,
                        Accessory,
                    )
                    .stats(st(0, 0, 0, 0, 0, 0, 5))
                    .tier(3)
                }
            }
            I::WardingCharm => {
                &const {
                    D::new(
                        "Warding Charm",
                        "+12 Resistance.",
                        Sp::ItemAmulet,
                        450,
                        Accessory,
                    )
                    .stats(st(0, 0, 0, 0, 0, 12, 0))
                    .tier(3)
                }
            }
            I::FireWard => {
                &const {
                    D::new(
                        "Fire Ward",
                        "Halves Fire damage.",
                        Sp::ItemAmulet,
                        700,
                        Accessory,
                    )
                    .special(Special::Resist(Fire))
                    .tier(4)
                }
            }
            I::FrostWard => {
                &const {
                    D::new(
                        "Frost Ward",
                        "Halves Frost damage.",
                        Sp::ItemAmulet,
                        900,
                        Accessory,
                    )
                    .special(Special::Resist(Frost))
                    .tier(5)
                }
            }
            I::ShadowWard => {
                &const {
                    D::new(
                        "Shadow Ward",
                        "Halves Shadow damage.",
                        Sp::ItemAmulet,
                        900,
                        Accessory,
                    )
                    .special(Special::Resist(Shadow))
                    .tier(5)
                }
            }
            I::ClarityPendant => {
                &const {
                    D::new(
                        "Clarity Pendant",
                        "Ailments land half as often.",
                        Sp::ItemAmulet,
                        800,
                        Accessory,
                    )
                    .special(Special::StatusGuard)
                    .tier(4)
                }
            }
            I::SoulGem => {
                &const {
                    D::new(
                        "Soul Gem",
                        "+20 MP; restores MP each turn.",
                        Sp::ItemRing,
                        1000,
                        Accessory,
                    )
                    .stats(st(0, 20, 0, 0, 0, 0, 0))
                    .special(Special::MpRegen)
                    .tier(4)
                }
            }
            I::HestaCharm => {
                &const {
                    D::new(
                        "Hesta's Charm",
                        "A knot of cat hair and old thread. Oddly lucky. +12% crit.",
                        Sp::ItemAmulet,
                        0,
                        Accessory,
                    )
                    .stats(st(0, 0, 3, 0, 3, 0, 2))
                    .special(Special::CritUp(12))
                }
            }
            I::TobinCharm => {
                &const {
                    D::new(
                        "Tobin's Charm",
                        "Lantern-glass that never stops glowing. Act first in battle.",
                        Sp::ItemAmulet,
                        0,
                        Accessory,
                    )
                    .stats(st(0, 0, 0, 2, 0, 2, 3))
                    .special(Special::FirstStrike)
                }
            }
            I::AmuletOfDawn => {
                &const {
                    D::new(
                        "Amulet of Dawn",
                        "Blessed by Sister Oriel. Regenerates HP every turn.",
                        Sp::ItemAmulet,
                        0,
                        Accessory,
                    )
                    .stats(st(30, 10, 0, 5, 5, 10, 0))
                    .special(Special::Regen)
                }
            }
        }
    }
}
