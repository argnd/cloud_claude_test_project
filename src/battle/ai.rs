//! Decision making: enemies pick weighted skills; heroes can be put on
//! "auto", which heals, revives and otherwise picks the hardest-hitting
//! affordable skill. The same hero logic drives the balance simulations.

use rand::RngExt;

use super::{Action, Battle, Choice, Side};
use crate::data::enemies::EnemyId;
use crate::data::items::{ItemId, ItemKind, Use};
use crate::data::skills::{Power, SkillId, SkillKind, Target};
use crate::data::{Affinity, StatusKind};

/// Enemy turn.
pub fn choose_action(battle: &mut Battle, unit: usize) -> Action {
    let u = &battle.units[unit];
    let hp_ratio = u.hp as f32 / u.max_hp.max(1) as f32;

    // Scripted boss moments.
    if u.enemy == Some(EnemyId::IronWarden) && hp_ratio < 0.5 && battle.rng.random_bool(0.35) {
        return Action::Skill(SkillId::MoltenRain, Choice::All);
    }
    if u.enemy == Some(EnemyId::AurelianTrue) && hp_ratio < 0.35 && battle.rng.random_bool(0.3) {
        return Action::Skill(SkillId::LastEmbrace, pick_foe(battle, unit));
    }

    let allies_alive = battle.alive(Side::Enemies).len();
    let mut options: Vec<(SkillId, u32)> = Vec::new();
    for (i, &skill) in u.skills.iter().enumerate() {
        let def = skill.def();
        let weight = u.weights.get(i).copied().unwrap_or(1) as u32;
        let sensible = match def.kind {
            SkillKind::Summon(..) => allies_alive < 4,
            SkillKind::Status if !def.target.hits_foes() => {
                // Don't re-buff what's already up.
                def.statuses.iter().any(|&(s, _, _)| !u.has(s))
            }
            _ => true,
        };
        if sensible {
            options.push((skill, weight));
        }
    }
    if options.is_empty() {
        options.push((SkillId::Attack, 1));
    }
    let total: u32 = options.iter().map(|&(_, w)| w).sum();
    let mut roll = battle.rng.random_range(0..total.max(1));
    let mut chosen = options[0].0;
    for &(skill, w) in &options {
        if roll < w {
            chosen = skill;
            break;
        }
        roll -= w;
    }
    let def = chosen.def();
    let choice = match def.target {
        Target::Foe => pick_foe(battle, unit),
        Target::Ally => Choice::Unit(unit),
        _ => Choice::All,
    };
    Action::Skill(chosen, choice)
}

fn pick_foe(battle: &mut Battle, unit: usize) -> Choice {
    let side = battle.units[unit].side;
    let foes = battle.alive(side.other());
    if foes.is_empty() {
        return Choice::All;
    }
    // Sometimes go for the weakest-looking target.
    if battle.rng.random_bool(0.35) {
        let weakest = foes
            .iter()
            .copied()
            .min_by(|&a, &b| {
                let ra = battle.units[a].hp as f32 / battle.units[a].max_hp as f32;
                let rb = battle.units[b].hp as f32 / battle.units[b].max_hp as f32;
                ra.total_cmp(&rb)
            })
            .unwrap();
        return Choice::Unit(weakest);
    }
    Choice::Unit(foes[battle.rng.random_range(0..foes.len())])
}

/// A sensible hero turn. `items` is what the party carries; the chosen item
/// (if any) must be removed from the inventory by the caller.
pub fn auto_action(battle: &Battle, unit: usize, items: &[(ItemId, u32)]) -> Action {
    let u = &battle.units[unit];
    let side = u.side;
    let allies = battle.alive(side);
    let dead = battle.dead(side);
    let skills: Vec<SkillId> = u.skills.iter().copied().filter(|&s| u.can_afford(s)).collect();
    let has_item = |item: ItemId| items.iter().any(|&(i, c)| i == item && c > 0);

    // Revive.
    if let Some(&fallen) = dead.first() {
        if let Some(&s) = skills.iter().find(|s| matches!(s.def().kind, SkillKind::Revive { .. })) {
            return Action::Skill(s, Choice::Unit(fallen));
        }
        if skills.contains(&SkillId::Lifebloom) {
            return Action::Skill(SkillId::Lifebloom, Choice::All);
        }
        if has_item(ItemId::EmberDown) {
            return Action::Item(ItemId::EmberDown, Choice::Unit(fallen));
        }
    }

    // Heal.
    let ratio = |i: usize| battle.units[i].hp as f32 / battle.units[i].max_hp.max(1) as f32;
    let hurt: Vec<usize> = allies.iter().copied().filter(|&i| ratio(i) < 0.45).collect();
    if !hurt.is_empty() {
        let group = skills
            .iter()
            .copied()
            .find(|s| matches!(s.def().kind, SkillKind::Heal { .. }) && s.def().target == Target::AllAllies);
        if hurt.len() >= 2 {
            if let Some(s) = group {
                return Action::Skill(s, Choice::All);
            }
        }
        let worst = *hurt.iter().min_by(|&&a, &&b| ratio(a).total_cmp(&ratio(b))).unwrap();
        if let Some(&s) = skills
            .iter()
            .find(|s| matches!(s.def().kind, SkillKind::Heal { .. }) && s.def().target == Target::Ally)
        {
            return Action::Skill(s, Choice::Unit(worst));
        }
        if let Some(s) = group {
            return Action::Skill(s, Choice::All);
        }
        if ratio(worst) < 0.3 {
            for item in [ItemId::Draught, ItemId::Tonic, ItemId::Elixir] {
                if has_item(item) {
                    return Action::Item(item, Choice::Unit(worst));
                }
            }
        }
    }

    // Cure a disabling ailment.
    if skills.contains(&SkillId::Cleanse) {
        if let Some(&t) = allies.iter().find(|&&i| {
            battle.units[i]
                .statuses
                .iter()
                .any(|&(s, _)| matches!(s, StatusKind::Poison | StatusKind::Burn | StatusKind::Slow | StatusKind::Weak))
        }) {
            return Action::Skill(SkillId::Cleanse, Choice::Unit(t));
        }
    }

    // Low on MP: casters recover.
    if u.max_mp > 40 && (u.mp as f32) < u.max_mp as f32 * 0.15 {
        if skills.contains(&SkillId::Siphon) {
            if let Some(&foe) = battle.alive(side.other()).first() {
                return Action::Skill(SkillId::Siphon, Choice::Unit(foe));
            }
        }
        for ether in [ItemId::Ether, ItemId::HiEther] {
            if has_item(ether) {
                return Action::Item(ether, Choice::Unit(unit));
            }
        }
    }

    // Tank: keep the taunt up in boss fights.
    if battle.is_boss() && skills.contains(&SkillId::StandFast) && !u.has(StatusKind::Taunt) && allies.len() > 1 {
        return Action::Skill(SkillId::StandFast, Choice::Unit(unit));
    }

    // Otherwise, the most expected damage.
    let foes = battle.alive(side.other());
    if foes.is_empty() {
        return Action::Defend;
    }
    let target = *foes
        .iter()
        .min_by_key(|&&i| battle.units[i].hp)
        .unwrap();
    let mut best = (Action::Skill(SkillId::Attack, Choice::Unit(target)), expected(battle, unit, SkillId::Attack, &foes, target));
    let mp_reserve = if skills.iter().any(|s| s.def().is_heal_like()) { 0.35 } else { 0.0 };
    for &s in &skills {
        let def = s.def();
        if !matches!(def.kind, SkillKind::Damage { .. } | SkillKind::Drain { .. } | SkillKind::Siphon { .. }) {
            continue;
        }
        if def.mp > 0 && (u.mp - def.mp) < (u.max_mp as f32 * mp_reserve) as i32 && !battle.is_boss() {
            continue;
        }
        let value = expected(battle, unit, s, &foes, target);
        if value > best.1 {
            let choice = if def.target.is_group() { Choice::All } else { Choice::Unit(target) };
            best = (Action::Skill(s, choice), value);
        }
    }
    best.0
}

/// Rough expected damage of a skill against the current foes.
fn expected(battle: &Battle, unit: usize, skill: SkillId, foes: &[usize], target: usize) -> f32 {
    let def = skill.def();
    let (power, hits) = match def.kind {
        SkillKind::Damage { power, hits } => (power, hits as f32),
        SkillKind::Drain { power } | SkillKind::Siphon { power } => (power, 1.0),
        _ => return 0.0,
    };
    let u = &battle.units[unit];
    let element = if skill == SkillId::Attack { u.attack_element() } else { def.element };
    let hit = |t: usize| {
        let f = &battle.units[t];
        let (a, d, p) = match power {
            Power::Atk(p) => (u.atk(), f.def(), p),
            Power::Mag(p) => (u.mag(), f.res(), p),
        };
        let aff = match f.affinity(element) {
            Affinity::Absorb => -1.0,
            a => a.multiplier(),
        };
        (p * a * a / (a + d) * aff).min(f.hp as f32 * 1.2)
    };
    let raw = if def.target.is_group() { foes.iter().map(|&t| hit(t)).sum() } else { hit(target) };
    // Slightly prefer cheap actions.
    raw * hits - def.mp as f32 * 0.3
}

/// Consumable items the auto-battler is allowed to spend.
pub fn is_auto_item(item: ItemId) -> bool {
    matches!(item.def().kind, ItemKind::Consumable(Use::Heal(_) | Use::HealFull | Use::Revive(_) | Use::Mp(_)))
}
