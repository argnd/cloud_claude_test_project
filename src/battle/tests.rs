use rand::SeedableRng;
use rand::rngs::StdRng;

use super::ai::auto_action;
use super::*;
use crate::data::enemies::{random_group, BattleId};
use crate::data::heroes::HeroId;
use crate::data::items::{EquipSlot, ItemKind};
use crate::rpg::{Hero, Inventory};

#[derive(Debug, PartialEq)]
enum Result {
    Won,
    Lost,
    Fled,
}

/// Runs a battle to the end with every hero on auto. Writes HP/MP back.
fn fight(party: &mut [Hero], inv: &mut Inventory, enemies: &[(EnemyId, u32)], kind: BattleKind, seed: u64) -> (Result, Battle) {
    let mut battle = Battle::new(party, enemies, kind, seed);
    let mut guard = 0;
    let result = loop {
        guard += 1;
        assert!(guard < 5000, "battle never ended: {enemies:?}");
        match battle.step() {
            Step::Events(_) => {}
            Step::Command(unit) => {
                let action = auto_action(&battle, unit, &inv.list());
                if let Action::Item(item, _) = action {
                    inv.remove(item, 1);
                }
                battle.act(unit, action);
            }
            Step::PhaseChange => {
                battle.begin_second_phase();
            }
            Step::Victory => break Result::Won,
            Step::Defeat => break Result::Lost,
            Step::Escaped => break Result::Fled,
        }
    };
    for u in &battle.units {
        if let Some((i, _)) = u.hero {
            party[i].hp = u.hp;
            party[i].mp = u.mp;
        }
    }
    (result, battle)
}

#[test]
fn a_fresh_hero_beats_a_rat() {
    let mut party = vec![Hero::new(HeroId::Wren, 1)];
    let mut inv = Inventory::default();
    let (result, _) = fight(&mut party, &mut inv, &[(EnemyId::SewerRat, 1)], BattleKind::Normal, 1);
    assert_eq!(result, Result::Won);
    assert!(party[0].hp > 0);
}

#[test]
fn weaknesses_are_discovered() {
    let mut party = vec![Hero::new(HeroId::Maelis, 20)];
    let mut battle = Battle::new(&party, &[(EnemyId::FireImp, 15)], BattleKind::Normal, 3);
    loop {
        match battle.step() {
            Step::Command(u) => {
                battle.act(u, Action::Skill(SkillId::FrostLance, Choice::Unit(1)));
            }
            Step::Victory | Step::Defeat | Step::Escaped => break,
            _ => {}
        }
    }
    assert!(battle.discovered.contains(&(EnemyId::FireImp, Element::Frost)));
    party[0].hp = 0;
}

#[test]
fn bosses_cannot_be_fled() {
    let party = vec![Hero::new(HeroId::Wren, 10)];
    let mut battle = Battle::new(&party, &BattleId::Vex.formation(), BattleKind::Story(BattleId::Vex), 5);
    loop {
        if let Step::Command(u) = battle.step() {
            battle.act(u, Action::Flee);
            break;
        }
    }
    assert!(!matches!(battle.step(), Step::Escaped));
}

#[test]
fn the_final_boss_has_two_phases() {
    let mut party: Vec<Hero> = HeroId::ALL.iter().map(|&h| Hero::new(h, 50)).collect();
    for h in &mut party {
        h.weapon = Some(best(h, EquipSlot::Weapon));
        h.armor = Some(best(h, EquipSlot::Armor));
        h.restore();
    }
    let mut inv = Inventory::default();
    inv.add(ItemId::Elixir, 20);
    inv.add(ItemId::EmberDown, 20);
    let (result, battle) = fight(&mut party, &mut inv, &BattleId::Aurelian.formation(), BattleKind::Story(BattleId::Aurelian), 9);
    assert_eq!(result, Result::Won);
    assert_eq!(battle.phase, 2);
}

/// The strongest shop item of the tier for a slot.
fn gear(hero: &Hero, slot: EquipSlot, tier: u8) -> Option<ItemId> {
    ItemId::ALL
        .into_iter()
        .filter(|&i| i.def().tier == tier && i.def().slot() == Some(slot) && hero.can_equip(i))
        .filter(|&i| !matches!(i.def().kind, ItemKind::Armor(crate::data::items::ArmorKind::Light)) || hero.id != HeroId::Wren)
        .max_by_key(|&i| {
            let s = i.def().stats;
            s.atk + s.mag + s.def + s.res
        })
}

fn best(hero: &Hero, slot: EquipSlot) -> ItemId {
    gear(hero, slot, 5).unwrap()
}

/// Plays the whole game on auto, the way a player who fights most of what
/// they meet would: ~9 fights per floor, a rest between floors, shop gear at
/// each act, companions joining on schedule, every boss. Prints a table and
/// fails if normal fights wipe the party or a boss is unbeatable.
#[test]
fn balance_full_playthrough() {
    let mut rng = StdRng::seed_from_u64(2024);
    let mut party = vec![Hero::new(HeroId::Wren, 1)];
    let mut inv = Inventory::default();
    inv.add(ItemId::Tonic, 3);
    let mut gold = 0u32;
    let mut wipes = 0;
    let mut report = String::from("floor lv   party-hp-left  fights  wipes  gold\n");
    let bosses = [
        (3, BattleId::Vex),
        (4, BattleId::Gristlemaw),
        (8, BattleId::Curator),
        (12, BattleId::MotherOfSpores),
        (16, BattleId::IronWarden),
        (19, BattleId::Ilsa),
        (20, BattleId::Aurelian),
    ];
    for floor in 1..=20u32 {
        let act = ((floor - 1) / 4 + 1) as u8;
        let join = match floor {
            3 => Some(HeroId::Brannoc),
            6 => Some(HeroId::Maelis),
            10 => Some(HeroId::Pip),
            _ => None,
        };
        if let Some(id) = join {
            party.push(Hero::new(id, party[0].level));
        }
        // Shop at the start of each act: one tier behind the act, like a
        // player who can't afford everything; chests fill the gap.
        if floor % 4 == 1 || join.is_some() {
            // Buy this act's gear if the gold is there, else last act's.
            let cost = |tier: u8, party: &[Hero]| -> u32 {
                party
                    .iter()
                    .flat_map(|h| [EquipSlot::Weapon, EquipSlot::Armor].map(|s| gear(h, s, tier)))
                    .flatten()
                    .filter(|i| !party.iter().any(|h| h.equipped().any(|e| e == *i)))
                    .map(|i| i.def().price)
                    .sum()
            };
            let tier = if cost(act, &party) <= gold { act } else { act.saturating_sub(1).max(1) };
            gold = gold.saturating_sub(cost(tier, &party));
            for h in &mut party {
                for slot in [EquipSlot::Weapon, EquipSlot::Armor] {
                    if let Some(item) = gear(h, slot, tier) {
                        h.equip(slot, Some(item));
                    }
                }
            }
            inv.add(ItemId::Tonic, 4);
            inv.add(ItemId::Ether, 2);
            inv.add(ItemId::EmberDown, 1);
        }
        for h in &mut party {
            h.restore();
        }
        let mut fights = 0;
        let mut floor_wipes = 0;
        for f in 0..9 {
            let group = random_group(floor, &mut rng);
            let (result, mut battle) = fight(&mut party, &mut inv, &group, BattleKind::Normal, rng.random());
            fights += 1;
            if result == Result::Lost {
                floor_wipes += 1;
                for h in &mut party {
                    h.restore();
                }
                continue;
            }
            let r = battle.rewards();
            gold += r.gold;
            for item in r.items {
                inv.add(item, 1);
            }
            let enemy_level = battle.enemy_level();
            for h in &mut party {
                let scaled = scale_xp(r.xp, h.level, enemy_level);
                h.gain_xp(scaled);
                if h.hp <= 0 {
                    h.hp = 1;
                }
            }
            // Players rest at a waystone mid-floor.
            if f == 4 {
                for h in &mut party {
                    h.restore();
                }
            }
        }
        wipes += floor_wipes;
        let hp_left: f32 = party.iter().map(|h| h.hp as f32 / h.max_hp() as f32).sum::<f32>() / party.len() as f32;
        report += &format!(
            "{floor:>5} {:>3}   {:>12.0}%  {fights:>6}  {floor_wipes:>5}  {gold}\n",
            party[0].level,
            hp_left * 100.0
        );
        if let Some(&(_, boss)) = bosses.iter().find(|(f, _)| *f == floor) {
            for h in &mut party {
                h.restore();
            }
            let mut tries = 0;
            loop {
                tries += 1;
                let mut boss_inv = inv.clone();
                boss_inv.add(ItemId::Tonic, 3);
                let (result, battle) = fight(&mut party, &mut boss_inv, &boss.formation(), BattleKind::Story(boss), rng.random());
                let left: f32 = party.iter().map(|h| h.hp.max(0) as f32 / h.max_hp() as f32).sum::<f32>() / party.len() as f32;
                let used = inv.list().iter().map(|(_, c)| c).sum::<u32>() + 3 - boss_inv.list().iter().map(|(_, c)| c).sum::<u32>();
                report += &format!("      boss {boss:?} try {tries}: {result:?} (phase {}), {:.0}% hp left, {used} items, {} turns\n", battle.phase, left * 100.0, battle.turns_taken);
                for h in &mut party {
                    h.restore();
                }
                if result == Result::Won {
                    break;
                }
                assert!(tries < 6, "boss {boss:?} unbeatable on floor {floor}\n{report}");
            }
        }
    }
    println!("{report}");
    assert!(wipes <= 4, "too many wipes in normal fights ({wipes})\n{report}");
    assert!(party[0].level >= 28, "party under-levelled for the finale\n{report}");
}

fn scale_xp(xp: u32, hero_level: u32, enemy_level: u32) -> u32 {
    crate::battle::xp_share(xp, hero_level, enemy_level)
}

/// Win rate of each boss against a party at the level and gear a player
/// would have, with a modest bag of items, over many seeds. Prints a table.
#[test]
fn boss_win_rates() {
    let bosses = [
        (BattleId::Vex, 5, vec![HeroId::Wren, HeroId::Brannoc]),
        (BattleId::Gristlemaw, 7, vec![HeroId::Wren, HeroId::Brannoc]),
        (BattleId::Curator, 14, vec![HeroId::Wren, HeroId::Brannoc, HeroId::Maelis]),
        (BattleId::MotherOfSpores, 23, HeroId::ALL.to_vec()),
        (BattleId::IronWarden, 30, HeroId::ALL.to_vec()),
        (BattleId::Ilsa, 35, HeroId::ALL.to_vec()),
        (BattleId::Aurelian, 37, HeroId::ALL.to_vec()),
    ];
    let mut report = String::from("boss            level  wins/20  avg hp left\n");
    let mut worst = 20;
    for (boss, level, heroes) in bosses {
        let act = match boss {
            BattleId::Vex | BattleId::Gristlemaw => 1,
            BattleId::Curator => 2,
            BattleId::MotherOfSpores => 3,
            BattleId::IronWarden => 4,
            _ => 5,
        };
        let mut wins = 0;
        let mut left = 0.0;
        for seed in 0..20u64 {
            let mut party: Vec<Hero> = heroes.iter().map(|&h| Hero::new(h, level)).collect();
            for h in &mut party {
                for slot in [EquipSlot::Weapon, EquipSlot::Armor] {
                    // One tier behind the act's best, as a thrifty player would be.
                    if let Some(i) = gear(h, slot, act.max(2) - 1).or_else(|| gear(h, slot, 1)) {
                        h.equip(slot, Some(i));
                    }
                }
                h.restore();
            }
            let mut inv = Inventory::default();
            inv.add(ItemId::Tonic, 5);
            inv.add(ItemId::Ether, 3);
            inv.add(ItemId::EmberDown, 2);
            if act >= 3 {
                inv.add(ItemId::Draught, 3);
                inv.add(ItemId::Elixir, 1);
            }
            let (result, battle) = fight(&mut party, &mut inv, &boss.formation(), BattleKind::Story(boss), seed * 7919 + 1);
            if result == Result::Lost && boss == BattleId::Aurelian {
                let enemies: Vec<String> = battle.units.iter().filter(|u| u.enemy.is_some()).map(|u| format!("{}:{}/{}", u.name, u.hp, u.max_hp)).collect();
                println!("  lost phase {} after {} turns: {enemies:?}", battle.phase, battle.turns_taken);
            }
            if result == Result::Won {
                wins += 1;
                left += party.iter().map(|h| h.hp.max(0) as f32 / h.max_hp() as f32).sum::<f32>() / party.len() as f32;
            }
        }
        worst = worst.min(wins);
        report += &format!("{:<15} {level:>5}  {wins:>7}  {:>10.0}%\n", format!("{boss:?}"), if wins > 0 { left / wins as f32 * 100.0 } else { 0.0 });
    }
    println!("{report}");
    assert!(worst >= 8, "a boss is too hard on auto\n{report}");
}
