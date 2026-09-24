//! Every action a combatant can take besides using an item: hero skills,
//! enemy attacks and boss abilities, all through the same definition.

use super::enemies::EnemyId;
use super::{Element, StatusKind};
use crate::audio::Sfx;
use crate::gfx::sprites::Sprite;

/// Who a skill can be aimed at, relative to its user.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Target {
    Foe,
    AllFoes,
    Ally,
    AllAllies,
    Myself,
    DeadAlly,
}

impl Target {
    pub fn hits_foes(self) -> bool {
        matches!(self, Target::Foe | Target::AllFoes)
    }

    pub fn is_group(self) -> bool {
        matches!(self, Target::AllFoes | Target::AllAllies)
    }
}

/// Which stat drives a damage roll.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Power {
    Atk(f32),
    Mag(f32),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SkillKind {
    Damage {
        power: Power,
        hits: u8,
    },
    /// Damage; the user recovers half of what was dealt.
    Drain {
        power: Power,
    },
    /// Damage; the user recovers MP equal to a fifth of what was dealt.
    Siphon {
        power: Power,
    },
    /// Heals `mag * power + flat`.
    Heal {
        power: f32,
        flat: i32,
    },
    Revive {
        fraction: f32,
    },
    /// Removes harmful statuses (and heals a little).
    Cleanse,
    /// Only applies the skill's statuses.
    Status,
    /// Calls reinforcements (enemies only).
    Summon(EnemyId, u8),
    /// A turn that does nothing but show its name.
    Nothing,
}

pub struct SkillDef {
    pub name: &'static str,
    pub desc: &'static str,
    pub mp: i32,
    pub target: Target,
    pub kind: SkillKind,
    pub element: Element,
    /// (status, chance in percent, turns)
    pub statuses: &'static [(StatusKind, u8, u8)],
    pub icon: Sprite,
    pub fx: Sprite,
    /// Drawn flying from user to target before the impact.
    pub projectile: Option<Sprite>,
    pub sfx: Sfx,
    /// Turn-time cost: 100 is normal, lower acts again sooner.
    pub delay: u32,
    /// Extra crit chance in percent.
    pub crit: u8,
}

impl SkillDef {
    const fn new(name: &'static str, desc: &'static str) -> Self {
        Self {
            name,
            desc,
            mp: 0,
            target: Target::Foe,
            kind: SkillKind::Damage {
                power: Power::Atk(1.0),
                hits: 1,
            },
            element: Element::Physical,
            statuses: &[],
            icon: Sprite::SkillSlash,
            fx: Sprite::FxSlash,
            projectile: None,
            sfx: Sfx::Hit,
            delay: 100,
            crit: 0,
        }
    }
    const fn mp(mut self, mp: i32) -> Self {
        self.mp = mp;
        self
    }
    const fn target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }
    const fn all(self) -> Self {
        self.target(Target::AllFoes)
    }
    const fn atk(mut self, power: f32) -> Self {
        self.kind = SkillKind::Damage {
            power: Power::Atk(power),
            hits: 1,
        };
        self
    }
    const fn mag(mut self, power: f32) -> Self {
        self.kind = SkillKind::Damage {
            power: Power::Mag(power),
            hits: 1,
        };
        self
    }
    const fn hits(mut self, hits: u8) -> Self {
        if let SkillKind::Damage { power, .. } = self.kind {
            self.kind = SkillKind::Damage { power, hits };
        }
        self
    }
    const fn kind(mut self, kind: SkillKind) -> Self {
        self.kind = kind;
        self
    }
    const fn heal(mut self, power: f32, flat: i32) -> Self {
        self.kind = SkillKind::Heal { power, flat };
        self.target = Target::Ally;
        self.element = Element::Light;
        self.fx = Sprite::FxHeal;
        self.sfx = Sfx::Heal;
        self.icon = Sprite::SkillHeal;
        self
    }
    const fn status_only(mut self, target: Target) -> Self {
        self.kind = SkillKind::Status;
        self.target = target;
        self
    }
    const fn el(mut self, element: Element) -> Self {
        self.element = element;
        self
    }
    const fn inflicts(mut self, statuses: &'static [(StatusKind, u8, u8)]) -> Self {
        self.statuses = statuses;
        self
    }
    const fn look(mut self, icon: Sprite, fx: Sprite, sfx: Sfx) -> Self {
        self.icon = icon;
        self.fx = fx;
        self.sfx = sfx;
        self
    }
    const fn bolt(mut self, projectile: Sprite) -> Self {
        self.projectile = Some(projectile);
        self
    }
    const fn delay(mut self, delay: u32) -> Self {
        self.delay = delay;
        self
    }
    const fn crit(mut self, crit: u8) -> Self {
        self.crit = crit;
        self
    }

    pub fn is_heal_like(&self) -> bool {
        matches!(
            self.kind,
            SkillKind::Heal { .. } | SkillKind::Revive { .. } | SkillKind::Cleanse
        )
    }

    /// Usable from the field menu (healing between fights).
    pub fn usable_outside_battle(&self) -> bool {
        self.is_heal_like() && !self.target.hits_foes()
    }
}

use Element::*;
use Sprite as Sp;
use StatusKind::*;
use Target::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, serde::Serialize, serde::Deserialize)]
pub enum SkillId {
    Attack,
    // Wren — Lanternbearer
    LanternStrike,
    Kindle,
    GuardianFlame,
    Flare,
    Beacon,
    Sunburst,
    LanternsMercy,
    LastLight,
    // Brannoc — Vanguard
    Cleave,
    StandFast,
    Sunder,
    Earthshaker,
    IronWill,
    Warcry,
    Skullsplitter,
    MountainsWrath,
    // Maelis — Archivist
    Firebolt,
    FrostLance,
    ChainLightning,
    Hex,
    Haste,
    Fireball,
    Siphon,
    Blizzard,
    Thunderstorm,
    Meteor,
    // Pip — Grovetender
    Mend,
    Bramble,
    Cleanse,
    Bloom,
    Rekindle,
    Wildgrowth,
    ThornStorm,
    Lifebloom,
    // Enemies
    Bite,
    VenomBite,
    Screech,
    Swoop,
    Stab,
    Backstab,
    ThrowKnives,
    Rally,
    DirtyTrick,
    GhoulClaw,
    Firepot,
    Drown,
    InkSpray,
    PageStorm,
    BoneSlash,
    WispLight,
    Zap,
    Wail,
    SoulTouch,
    SporeCloud,
    Web,
    VenomFang,
    SlimeCoat,
    HollowGrasp,
    RootLash,
    EmberSpit,
    Slam,
    AshClaw,
    FireBreath,
    Immolate,
    Hellfire,
    FrostBreath,
    IceClaw,
    ShadowBolt,
    DeathStrike,
    Stomp,
    SoulDrain,
    Howl,
    ChillTouch,
    Harden,
    // Bosses
    PlagueSqueal,
    Gnash,
    CallTheSwarm,
    ForbiddenWord,
    InkTide,
    Erase,
    CallTheShelves,
    RotBloom,
    MycelialGrasp,
    SporeBrood,
    Forgefire,
    IronBulwark,
    Hammerfall,
    MoltenRain,
    PaleBlade,
    CrownOfFrost,
    Hesitation,
    WintersEmbrace,
    GriefTide,
    EndlessWinter,
    Hollowing,
    CallThePale,
    LullabyOfAsh,
    FrozenTears,
    LastEmbrace,
}

impl SkillId {
    pub fn def(self) -> &'static SkillDef {
        use SkillId as K;
        type S = SkillDef;
        match self {
            K::Attack => &const { S::new("Attack", "A basic weapon strike.") },

            // ---- Wren
            K::LanternStrike => {
                &const {
                    S::new(
                        "Lantern Strike",
                        "A blow wreathed in lantern-fire. Light damage.",
                    )
                    .mp(4)
                    .atk(1.5)
                    .el(Light)
                    .look(Sp::SkillLight, Sp::FxLight, Sfx::Light)
                }
            }
            K::Kindle => {
                &const {
                    S::new("Kindle", "Warm one ally with gentle light, restoring HP.")
                        .mp(5)
                        .heal(1.4, 25)
                }
            }
            K::GuardianFlame => {
                &const {
                    S::new("Guardian Flame", "Shield the whole party for 3 turns.")
                        .mp(10)
                        .status_only(AllAllies)
                        .inflicts(&[(Shield, 100, 3)])
                        .look(Sp::SkillShield, Sp::FxBuff, Sfx::Buff)
                }
            }
            K::Flare => {
                &const {
                    S::new(
                        "Flare",
                        "A blinding burst of light at every foe. May Blind.",
                    )
                    .mp(12)
                    .mag(1.0)
                    .all()
                    .el(Light)
                    .inflicts(&[(Blind, 40, 3)])
                    .look(Sp::SkillLight, Sp::FxLight, Sfx::Light)
                }
            }
            K::Beacon => {
                &const {
                    S::new(
                        "Beacon",
                        "Raise the lantern high: the party gains Might and Regen.",
                    )
                    .mp(16)
                    .status_only(AllAllies)
                    .inflicts(&[(Might, 100, 3), (Regen, 100, 3)])
                    .look(Sp::SkillBuff, Sp::FxBuff, Sfx::Buff)
                }
            }
            K::Sunburst => {
                &const {
                    S::new(
                        "Sunburst",
                        "Pour out the lantern's heart. Heavy Light damage to all foes.",
                    )
                    .mp(24)
                    .mag(2.0)
                    .all()
                    .el(Light)
                    .look(Sp::SkillLight, Sp::FxLight, Sfx::Light)
                }
            }
            K::LanternsMercy => {
                &const {
                    S::new("Lantern's Mercy", "Restore HP to the whole party.")
                        .mp(22)
                        .heal(1.1, 60)
                        .target(AllAllies)
                }
            }
            K::LastLight => {
                &const {
                    S::new(
                        "Last Light",
                        "Everything Old Tom has left, in one strike. Massive Light damage.",
                    )
                    .mp(36)
                    .atk(3.4)
                    .el(Light)
                    .crit(20)
                    .look(Sp::SkillUltimate, Sp::FxLight, Sfx::Light)
                    .delay(120)
                }
            }

            // ---- Brannoc
            K::Cleave => {
                &const {
                    S::new("Cleave", "A wide swing that hits every foe.")
                        .mp(5)
                        .atk(0.8)
                        .all()
                        .look(Sp::SkillSlash, Sp::FxSlash, Sfx::Hit)
                }
            }
            K::StandFast => {
                &const {
                    S::new(
                        "Stand Fast",
                        "Draw every enemy's attention and brace. Taunt + Shield.",
                    )
                    .mp(4)
                    .status_only(Myself)
                    .inflicts(&[(Taunt, 100, 3), (Shield, 100, 3)])
                    .look(Sp::SkillShield, Sp::FxBuff, Sfx::Buff)
                    .delay(70)
                }
            }
            K::Sunder => {
                &const {
                    S::new("Sunder", "Crack a foe's armour. Inflicts Sunder.")
                        .mp(6)
                        .atk(1.3)
                        .inflicts(&[(StatusKind::Sunder, 85, 3)])
                        .look(Sp::SkillDebuff, Sp::FxSlash, Sfx::CritHit)
                }
            }
            K::Earthshaker => {
                &const {
                    S::new("Earthshaker", "Slam the ground. Hits all foes; may Stun.")
                        .mp(12)
                        .atk(1.0)
                        .all()
                        .inflicts(&[(Stun, 25, 1)])
                        .look(Sp::SkillEarth, Sp::FxExplosion, Sfx::Explosion)
                }
            }
            K::IronWill => {
                &const {
                    S::new("Iron Will", "Grit your teeth: Regen and Might for 4 turns.")
                        .mp(10)
                        .status_only(Myself)
                        .inflicts(&[(Regen, 100, 4), (Might, 100, 4)])
                        .look(Sp::SkillBuff, Sp::FxBuff, Sfx::Buff)
                }
            }
            K::Warcry => {
                &const {
                    S::new("Warcry", "A dwarven roar. The party gains Might.")
                        .mp(14)
                        .status_only(AllAllies)
                        .inflicts(&[(Might, 100, 3)])
                        .look(Sp::SkillBuff, Sp::FxBuff, Sfx::Buff)
                }
            }
            K::Skullsplitter => {
                &const {
                    S::new(
                        "Skullsplitter",
                        "An overhead blow with all his weight. High crit.",
                    )
                    .mp(20)
                    .atk(2.7)
                    .crit(25)
                    .look(Sp::SkillSlash, Sp::FxSlash, Sfx::CritHit)
                    .delay(120)
                }
            }
            K::MountainsWrath => {
                &const {
                    S::new(
                        "Mountain's Wrath",
                        "The deep roots answer. Heavy damage to all foes.",
                    )
                    .mp(32)
                    .atk(1.8)
                    .all()
                    .inflicts(&[(Stun, 20, 1)])
                    .look(Sp::SkillUltimate, Sp::FxExplosion, Sfx::Explosion)
                    .delay(120)
                }
            }

            // ---- Maelis
            K::Firebolt => {
                &const {
                    S::new("Firebolt", "Fire damage to one foe. May Burn.")
                        .mp(5)
                        .mag(1.4)
                        .el(Fire)
                        .inflicts(&[(Burn, 30, 3)])
                        .look(Sp::SkillFire, Sp::FxFire, Sfx::Fire)
                        .bolt(Sp::FxBoltFire)
                }
            }
            K::FrostLance => {
                &const {
                    S::new("Frost Lance", "Frost damage to one foe. May Freeze.")
                        .mp(5)
                        .mag(1.4)
                        .el(Frost)
                        .inflicts(&[(Frozen, 20, 1)])
                        .look(Sp::SkillFrost, Sp::FxFrost, Sfx::Frost)
                        .bolt(Sp::FxBoltFrost)
                }
            }
            K::ChainLightning => {
                &const {
                    S::new("Chain Lightning", "Shock damage arcs through every foe.")
                        .mp(12)
                        .mag(1.0)
                        .all()
                        .el(Shock)
                        .look(Sp::SkillShock, Sp::FxShock, Sfx::Shock)
                }
            }
            K::Hex => {
                &const {
                    S::new("Hex", "A precise curse: Weak and Sunder on one foe.")
                        .mp(8)
                        .status_only(Foe)
                        .el(Shadow)
                        .inflicts(&[(Weak, 90, 3), (StatusKind::Sunder, 90, 3)])
                        .look(Sp::SkillDebuff, Sp::FxShadow, Sfx::Debuff)
                }
            }
            K::Haste => {
                &const {
                    S::new("Haste", "Quicken one ally for 4 turns.")
                        .mp(10)
                        .status_only(Ally)
                        .inflicts(&[(StatusKind::Haste, 100, 4)])
                        .look(Sp::SkillBuff, Sp::FxBuff, Sfx::Buff)
                }
            }
            K::Fireball => {
                &const {
                    S::new("Fireball", "Fire damage to all foes. May Burn.")
                        .mp(18)
                        .mag(1.25)
                        .all()
                        .el(Fire)
                        .inflicts(&[(Burn, 30, 3)])
                        .look(Sp::SkillFire, Sp::FxExplosion, Sfx::Explosion)
                }
            }
            K::Siphon => {
                &const {
                    S::new(
                        "Siphon",
                        "Drain a foe's essence: Shadow damage that restores MP.",
                    )
                    .mp(0)
                    .kind(SkillKind::Siphon {
                        power: super::skills::Power::Mag(1.0),
                    })
                    .el(Shadow)
                    .look(Sp::SkillArcane, Sp::FxShadow, Sfx::Shadow)
                    .bolt(Sp::FxBoltArcane)
                }
            }
            K::Blizzard => {
                &const {
                    S::new("Blizzard", "Frost damage to all foes. May Freeze.")
                        .mp(20)
                        .mag(1.3)
                        .all()
                        .el(Frost)
                        .inflicts(&[(Frozen, 15, 1)])
                        .look(Sp::SkillFrost, Sp::FxFrost, Sfx::Frost)
                }
            }
            K::Thunderstorm => {
                &const {
                    S::new("Thunderstorm", "Heavy Shock damage to all foes.")
                        .mp(28)
                        .mag(1.7)
                        .all()
                        .el(Shock)
                        .inflicts(&[(Stun, 15, 1)])
                        .look(Sp::SkillShock, Sp::FxShock, Sfx::Shock)
                }
            }
            K::Meteor => {
                &const {
                    S::new(
                        "Meteor",
                        "A star from the Codex. Enormous Fire damage to all foes.",
                    )
                    .mp(45)
                    .mag(2.5)
                    .all()
                    .el(Fire)
                    .look(Sp::SkillUltimate, Sp::FxExplosion, Sfx::Explosion)
                    .delay(130)
                }
            }

            // ---- Pip
            K::Mend => {
                &const {
                    S::new("Mend", "Knit an ally's wounds with green light.")
                        .mp(4)
                        .heal(1.5, 30)
                        .el(Nature)
                }
            }
            K::Bramble => {
                &const {
                    S::new("Bramble", "Nature damage to one foe. Likely to Poison.")
                        .mp(5)
                        .mag(1.1)
                        .el(Nature)
                        .inflicts(&[(Poison, 60, 4)])
                        .look(Sp::SkillNature, Sp::FxPoison, Sfx::Poison)
                }
            }
            K::Cleanse => {
                &const {
                    S::new("Cleanse", "Wash away an ally's ailments and heal a little.")
                        .mp(6)
                        .kind(SkillKind::Cleanse)
                        .target(Ally)
                        .look(Sp::SkillHeal, Sp::FxHeal, Sfx::Heal)
                }
            }
            K::Bloom => {
                &const {
                    S::new("Bloom", "Flowers open over the whole party, restoring HP.")
                        .mp(14)
                        .heal(1.0, 40)
                        .target(AllAllies)
                        .el(Nature)
                }
            }
            K::Rekindle => {
                &const {
                    S::new("Rekindle", "Call a fallen ally back with half their HP.")
                        .mp(18)
                        .kind(SkillKind::Revive { fraction: 0.5 })
                        .target(DeadAlly)
                        .look(Sp::SkillRevive, Sp::FxHeal, Sfx::Heal)
                }
            }
            K::Wildgrowth => {
                &const {
                    S::new("Wildgrowth", "The party gains Regen for 4 turns.")
                        .mp(16)
                        .status_only(AllAllies)
                        .inflicts(&[(Regen, 100, 4)])
                        .look(Sp::SkillNature, Sp::FxHeal, Sfx::Buff)
                }
            }
            K::ThornStorm => {
                &const {
                    S::new("Thorn Storm", "Nature damage to all foes. May Poison.")
                        .mp(24)
                        .mag(1.5)
                        .all()
                        .el(Nature)
                        .inflicts(&[(Poison, 40, 4)])
                        .look(Sp::SkillNature, Sp::FxPoison, Sfx::Poison)
                }
            }
            K::Lifebloom => {
                &const {
                    S::new(
                        "Lifebloom",
                        "Revive every fallen ally, then heal the whole party.",
                    )
                    .mp(45)
                    .heal(1.2, 80)
                    .target(AllAllies)
                    .el(Nature)
                    .look(Sp::SkillUltimate, Sp::FxHeal, Sfx::Heal)
                }
            }

            // ---- Enemies
            K::Bite => &const { S::new("Bite", "").atk(1.0) },
            K::VenomBite => {
                &const {
                    S::new("Venom Bite", "")
                        .atk(0.9)
                        .inflicts(&[(Poison, 40, 3)])
                        .look(Sp::SkillNature, Sp::FxPoison, Sfx::Poison)
                }
            }
            K::Screech => {
                &const {
                    S::new("Screech", "")
                        .status_only(AllFoes)
                        .inflicts(&[(Weak, 35, 2)])
                        .look(Sp::SkillDebuff, Sp::FxCloud, Sfx::Debuff)
                }
            }
            K::Swoop => &const { S::new("Swoop", "").atk(0.9).delay(70) },
            K::Stab => &const { S::new("Stab", "").atk(1.15) },
            K::Backstab => {
                &const {
                    S::new("Backstab", "").atk(1.5).crit(20).delay(120).look(
                        Sp::SkillSlash,
                        Sp::FxSlash,
                        Sfx::CritHit,
                    )
                }
            }
            K::ThrowKnives => {
                &const {
                    S::new("Throw Knives", "")
                        .atk(0.65)
                        .all()
                        .bolt(Sp::FxBoltArcane)
                }
            }
            K::Rally => {
                &const {
                    S::new("Rally", "")
                        .status_only(AllAllies)
                        .inflicts(&[(Might, 100, 3)])
                        .look(Sp::SkillBuff, Sp::FxBuff, Sfx::Buff)
                }
            }
            K::DirtyTrick => {
                &const {
                    S::new("Dirty Trick", "")
                        .atk(0.6)
                        .inflicts(&[(Blind, 60, 3)])
                        .look(Sp::SkillDebuff, Sp::FxCloud, Sfx::Debuff)
                }
            }
            K::GhoulClaw => {
                &const {
                    S::new("Ghoul Claw", "")
                        .atk(1.05)
                        .inflicts(&[(Stun, 15, 1)])
                }
            }
            K::Firepot => {
                &const {
                    S::new("Firepot", "")
                        .mag(1.0)
                        .el(Fire)
                        .inflicts(&[(Burn, 25, 2)])
                        .look(Sp::SkillFire, Sp::FxFire, Sfx::Fire)
                        .bolt(Sp::FxBoltFire)
                }
            }
            K::Drown => {
                &const {
                    S::new("Drown", "")
                        .mag(1.1)
                        .el(Frost)
                        .inflicts(&[(Slow, 30, 2)])
                        .look(Sp::SkillFrost, Sp::FxFrost, Sfx::Frost)
                }
            }
            K::InkSpray => {
                &const {
                    S::new("Ink Spray", "")
                        .mag(0.8)
                        .el(Shadow)
                        .inflicts(&[(Blind, 50, 3)])
                        .look(Sp::SkillShadow, Sp::FxShadow, Sfx::Shadow)
                }
            }
            K::PageStorm => {
                &const {
                    S::new("Page Storm", "").mag(0.7).all().el(Shadow).look(
                        Sp::SkillArcane,
                        Sp::FxCloud,
                        Sfx::Shadow,
                    )
                }
            }
            K::BoneSlash => &const { S::new("Bone Slash", "").atk(1.2) },
            K::WispLight => {
                &const {
                    S::new("Wisp Light", "")
                        .mag(1.0)
                        .el(Light)
                        .look(Sp::SkillLight, Sp::FxLight, Sfx::Light)
                        .bolt(Sp::FxBoltArcane)
                }
            }
            K::Zap => {
                &const {
                    S::new("Zap", "")
                        .mag(1.2)
                        .el(Shock)
                        .inflicts(&[(Stun, 10, 1)])
                        .look(Sp::SkillShock, Sp::FxShock, Sfx::Shock)
                        .bolt(Sp::FxBoltShock)
                }
            }
            K::Wail => {
                &const {
                    S::new("Wail", "")
                        .status_only(AllFoes)
                        .el(Shadow)
                        .inflicts(&[(Weak, 50, 2)])
                        .look(Sp::SkillShadow, Sp::FxShadow, Sfx::Debuff)
                }
            }
            K::SoulTouch => {
                &const {
                    S::new("Soul Touch", "")
                        .kind(SkillKind::Drain {
                            power: Power::Mag(1.0),
                        })
                        .el(Shadow)
                        .look(Sp::SkillShadow, Sp::FxShadow, Sfx::Shadow)
                }
            }
            K::SporeCloud => {
                &const {
                    S::new("Spore Cloud", "")
                        .mag(0.55)
                        .all()
                        .el(Nature)
                        .inflicts(&[(Poison, 45, 3)])
                        .look(Sp::SkillNature, Sp::FxPoison, Sfx::Poison)
                }
            }
            K::Web => {
                &const {
                    S::new("Web", "")
                        .status_only(Foe)
                        .inflicts(&[(Slow, 75, 3)])
                        .look(Sp::SkillDebuff, Sp::FxCloud, Sfx::Debuff)
                }
            }
            K::VenomFang => {
                &const {
                    S::new("Venom Fang", "")
                        .atk(1.1)
                        .inflicts(&[(Poison, 50, 3)])
                        .look(Sp::SkillNature, Sp::FxPoison, Sfx::Poison)
                }
            }
            K::SlimeCoat => {
                &const {
                    S::new("Slime Coat", "")
                        .mag(0.9)
                        .el(Nature)
                        .inflicts(&[(StatusKind::Sunder, 40, 3)])
                        .look(Sp::SkillNature, Sp::FxPoison, Sfx::Poison)
                }
            }
            K::HollowGrasp => {
                &const {
                    S::new("Hollow Grasp", "")
                        .mag(1.0)
                        .el(Frost)
                        .inflicts(&[(Weak, 30, 2)])
                        .look(Sp::SkillFrost, Sp::FxFrost, Sfx::Frost)
                }
            }
            K::RootLash => &const { S::new("Root Lash", "").atk(1.1).hits(2).el(Nature) },
            K::EmberSpit => {
                &const {
                    S::new("Ember Spit", "")
                        .mag(1.1)
                        .el(Fire)
                        .inflicts(&[(Burn, 30, 3)])
                        .look(Sp::SkillFire, Sp::FxFire, Sfx::Fire)
                        .bolt(Sp::FxBoltFire)
                }
            }
            K::Slam => {
                &const {
                    S::new("Slam", "")
                        .atk(1.4)
                        .inflicts(&[(Stun, 20, 1)])
                        .delay(130)
                        .look(Sp::SkillEarth, Sp::FxExplosion, Sfx::CritHit)
                }
            }
            K::AshClaw => &const { S::new("Ash Claw", "").atk(1.1).inflicts(&[(Burn, 20, 2)]) },
            K::FireBreath => {
                &const {
                    S::new("Fire Breath", "")
                        .mag(0.9)
                        .all()
                        .el(Fire)
                        .inflicts(&[(Burn, 20, 2)])
                        .look(Sp::SkillFire, Sp::FxFire, Sfx::Fire)
                }
            }
            K::Immolate => {
                &const {
                    S::new("Immolate", "").mag(1.6).el(Fire).look(
                        Sp::SkillFire,
                        Sp::FxExplosion,
                        Sfx::Explosion,
                    )
                }
            }
            K::Hellfire => {
                &const {
                    S::new("Hellfire", "")
                        .mag(1.1)
                        .all()
                        .el(Fire)
                        .inflicts(&[(Burn, 35, 3)])
                        .look(Sp::SkillFire, Sp::FxFire, Sfx::Fire)
                }
            }
            K::FrostBreath => {
                &const {
                    S::new("Frost Breath", "")
                        .mag(1.0)
                        .all()
                        .el(Frost)
                        .inflicts(&[(Frozen, 10, 1)])
                        .look(Sp::SkillFrost, Sp::FxFrost, Sfx::Frost)
                }
            }
            K::IceClaw => {
                &const {
                    S::new("Ice Claw", "")
                        .atk(1.1)
                        .el(Frost)
                        .inflicts(&[(Slow, 20, 2)])
                }
            }
            K::ShadowBolt => {
                &const {
                    S::new("Shadow Bolt", "")
                        .mag(1.3)
                        .el(Shadow)
                        .look(Sp::SkillShadow, Sp::FxShadow, Sfx::Shadow)
                        .bolt(Sp::FxBoltArcane)
                }
            }
            K::DeathStrike => {
                &const {
                    S::new("Death Strike", "")
                        .atk(1.7)
                        .crit(10)
                        .delay(130)
                        .look(Sp::SkillSlash, Sp::FxSlash, Sfx::CritHit)
                }
            }
            K::Stomp => {
                &const {
                    S::new("Stomp", "").atk(0.95).all().look(
                        Sp::SkillEarth,
                        Sp::FxExplosion,
                        Sfx::Explosion,
                    )
                }
            }
            K::SoulDrain => {
                &const {
                    S::new("Soul Drain", "")
                        .kind(SkillKind::Drain {
                            power: Power::Mag(1.2),
                        })
                        .el(Shadow)
                        .look(Sp::SkillShadow, Sp::FxShadow, Sfx::Shadow)
                }
            }
            K::Howl => {
                &const {
                    S::new("Howl", "")
                        .status_only(AllAllies)
                        .inflicts(&[(Might, 100, 3)])
                        .look(Sp::SkillBuff, Sp::FxBuff, Sfx::Buff)
                }
            }
            K::ChillTouch => {
                &const {
                    S::new("Chill Touch", "")
                        .mag(1.0)
                        .el(Frost)
                        .inflicts(&[(Frozen, 15, 1)])
                        .look(Sp::SkillFrost, Sp::FxFrost, Sfx::Frost)
                }
            }
            K::Harden => {
                &const {
                    S::new("Harden", "")
                        .status_only(Myself)
                        .inflicts(&[(Shield, 100, 3)])
                        .look(Sp::SkillShield, Sp::FxBuff, Sfx::Buff)
                }
            }

            // ---- Bosses
            K::PlagueSqueal => {
                &const {
                    S::new("Plague Squeal", "")
                        .atk(0.55)
                        .all()
                        .inflicts(&[(Poison, 50, 3)])
                        .look(Sp::SkillNature, Sp::FxPoison, Sfx::BossRoar)
                }
            }
            K::Gnash => {
                &const {
                    S::new("Gnash", "").atk(1.8).crit(10).delay(120).look(
                        Sp::SkillSlash,
                        Sp::FxSlash,
                        Sfx::CritHit,
                    )
                }
            }
            K::CallTheSwarm => {
                &const {
                    S::new("Call the Swarm", "")
                        .kind(SkillKind::Summon(EnemyId::SewerRat, 2))
                        .target(Myself)
                        .look(Sp::SkillBuff, Sp::FxCloud, Sfx::BossRoar)
                }
            }
            K::ForbiddenWord => {
                &const {
                    S::new("Forbidden Word", "")
                        .mag(1.15)
                        .all()
                        .el(Shadow)
                        .inflicts(&[(Weak, 40, 2)])
                        .look(Sp::SkillShadow, Sp::FxShadow, Sfx::Shadow)
                }
            }
            K::InkTide => {
                &const {
                    S::new("Ink Tide", "")
                        .mag(0.9)
                        .all()
                        .el(Frost)
                        .inflicts(&[(Blind, 40, 2)])
                        .look(Sp::SkillFrost, Sp::FxCloud, Sfx::Frost)
                }
            }
            K::Erase => {
                &const {
                    S::new("Erase", "").mag(2.0).el(Shadow).delay(130).look(
                        Sp::SkillShadow,
                        Sp::FxShadow,
                        Sfx::Shadow,
                    )
                }
            }
            K::CallTheShelves => {
                &const {
                    S::new("Call the Shelves", "")
                        .kind(SkillKind::Summon(EnemyId::HauntedTome, 2))
                        .target(Myself)
                        .look(Sp::SkillArcane, Sp::FxCloud, Sfx::Shadow)
                }
            }
            K::RotBloom => {
                &const {
                    S::new("Rot Bloom", "")
                        .mag(1.2)
                        .all()
                        .el(Nature)
                        .inflicts(&[(Poison, 55, 3)])
                        .look(Sp::SkillNature, Sp::FxPoison, Sfx::Poison)
                }
            }
            K::MycelialGrasp => {
                &const {
                    S::new("Mycelial Grasp", "")
                        .atk(1.35)
                        .inflicts(&[(Slow, 50, 2)])
                        .look(Sp::SkillNature, Sp::FxPoison, Sfx::Hit)
                }
            }
            K::SporeBrood => {
                &const {
                    S::new("Spore Brood", "")
                        .kind(SkillKind::Summon(EnemyId::Sporeling, 2))
                        .target(Myself)
                        .look(Sp::SkillNature, Sp::FxCloud, Sfx::Poison)
                }
            }
            K::Forgefire => {
                &const {
                    S::new("Forgefire", "")
                        .mag(1.15)
                        .all()
                        .el(Fire)
                        .inflicts(&[(Burn, 35, 3)])
                        .look(Sp::SkillFire, Sp::FxFire, Sfx::Fire)
                }
            }
            K::IronBulwark => {
                &const {
                    S::new("Iron Bulwark", "")
                        .status_only(Myself)
                        .inflicts(&[(Shield, 100, 3), (Regen, 100, 3)])
                        .look(Sp::SkillShield, Sp::FxBuff, Sfx::Buff)
                }
            }
            K::Hammerfall => {
                &const {
                    S::new("Hammerfall", "")
                        .atk(2.2)
                        .inflicts(&[(Stun, 30, 1)])
                        .delay(140)
                        .look(Sp::SkillEarth, Sp::FxExplosion, Sfx::Explosion)
                }
            }
            K::MoltenRain => {
                &const {
                    S::new("Molten Rain", "")
                        .mag(1.5)
                        .all()
                        .el(Fire)
                        .delay(130)
                        .look(Sp::SkillFire, Sp::FxExplosion, Sfx::Explosion)
                }
            }
            K::PaleBlade => {
                &const {
                    S::new("Pale Blade", "").atk(1.5).el(Frost).look(
                        Sp::SkillFrost,
                        Sp::FxFrost,
                        Sfx::Frost,
                    )
                }
            }
            K::CrownOfFrost => {
                &const {
                    S::new("Crown of Frost", "")
                        .mag(1.15)
                        .all()
                        .el(Frost)
                        .inflicts(&[(Slow, 40, 2)])
                        .look(Sp::SkillFrost, Sp::FxFrost, Sfx::Frost)
                }
            }
            K::Hesitation => {
                &const {
                    S::new("...Wren?", "")
                        .kind(SkillKind::Nothing)
                        .target(Myself)
                        .look(Sp::SkillLight, Sp::FxBuff, Sfx::MenuBack)
                }
            }
            K::WintersEmbrace => {
                &const {
                    S::new("Winter's Embrace", "")
                        .mag(1.0)
                        .el(Frost)
                        .inflicts(&[(Frozen, 40, 1)])
                        .look(Sp::SkillFrost, Sp::FxFrost, Sfx::Frost)
                }
            }
            K::GriefTide => {
                &const {
                    S::new("Grief Tide", "")
                        .mag(1.3)
                        .all()
                        .el(Shadow)
                        .inflicts(&[(Weak, 40, 2)])
                        .look(Sp::SkillShadow, Sp::FxShadow, Sfx::Shadow)
                }
            }
            K::EndlessWinter => {
                &const {
                    S::new("Endless Winter", "")
                        .mag(1.25)
                        .all()
                        .el(Frost)
                        .inflicts(&[(Frozen, 15, 1)])
                        .look(Sp::SkillFrost, Sp::FxFrost, Sfx::Frost)
                }
            }
            K::Hollowing => {
                &const {
                    S::new("Hollowing", "").mag(2.1).el(Shadow).delay(130).look(
                        Sp::SkillShadow,
                        Sp::FxShadow,
                        Sfx::Shadow,
                    )
                }
            }
            K::CallThePale => {
                &const {
                    S::new("Call the Pale", "")
                        .kind(SkillKind::Summon(EnemyId::PaleShade, 2))
                        .target(Myself)
                        .look(Sp::SkillShadow, Sp::FxCloud, Sfx::Shadow)
                }
            }
            K::LullabyOfAsh => {
                &const {
                    S::new("Lullaby of Ash", "")
                        .mag(1.5)
                        .all()
                        .el(Shadow)
                        .inflicts(&[(Stun, 20, 1)])
                        .look(Sp::SkillShadow, Sp::FxShadow, Sfx::Shadow)
                }
            }
            K::FrozenTears => {
                &const {
                    S::new("Frozen Tears", "")
                        .mag(1.45)
                        .all()
                        .el(Frost)
                        .inflicts(&[(Slow, 40, 2)])
                        .look(Sp::SkillFrost, Sp::FxFrost, Sfx::Frost)
                }
            }
            K::LastEmbrace => {
                &const {
                    S::new("Last Embrace", "")
                        .mag(2.1)
                        .el(Shadow)
                        .delay(130)
                        .look(Sp::SkillShadow, Sp::FxShadow, Sfx::Shadow)
                }
            }
        }
    }
}
