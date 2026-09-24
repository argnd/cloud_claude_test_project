//! The party as it exists between battles: levels, equipment, HP and MP,
//! and the shared inventory.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::data::Stats;
use crate::data::heroes::{HeroDef, HeroId, MAX_LEVEL, xp_to_next};
use crate::data::items::{EquipSlot, ItemId, ItemKind, Special};
use crate::data::skills::SkillId;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Hero {
    pub id: HeroId,
    pub level: u32,
    pub xp: u32,
    pub hp: i32,
    pub mp: i32,
    pub weapon: Option<ItemId>,
    pub armor: Option<ItemId>,
    pub accessory: Option<ItemId>,
}

/// What changed when a hero gained a level.
#[derive(Clone, Debug)]
pub struct LevelUp {
    pub hero: HeroId,
    pub level: u32,
    pub gains: Stats,
    pub new_skills: Vec<SkillId>,
}

impl Hero {
    pub fn new(id: HeroId, level: u32) -> Self {
        let def = id.def();
        let mut hero = Self {
            id,
            level: level.clamp(1, MAX_LEVEL),
            xp: 0,
            hp: 1,
            mp: 0,
            weapon: Some(def.start[0]),
            armor: Some(def.start[1]),
            accessory: None,
        };
        hero.restore();
        hero
    }

    pub fn def(&self) -> &'static HeroDef {
        self.id.def()
    }

    pub fn name(&self) -> &'static str {
        self.def().name
    }

    pub fn equipped(&self) -> impl Iterator<Item = ItemId> + '_ {
        [self.weapon, self.armor, self.accessory].into_iter().flatten()
    }

    /// Level stats plus everything equipped.
    pub fn stats(&self) -> Stats {
        self.equipped()
            .fold(self.def().stats_at(self.level), |s, item| s.add(item.def().stats))
    }

    pub fn max_hp(&self) -> i32 {
        self.stats().hp.max(1)
    }

    pub fn max_mp(&self) -> i32 {
        self.stats().mp.max(0)
    }

    pub fn specials(&self) -> Vec<Special> {
        self.equipped()
            .map(|item| item.def().special)
            .filter(|s| *s != Special::None)
            .collect()
    }

    pub fn skills(&self) -> Vec<SkillId> {
        self.def().skills_at(self.level)
    }

    pub fn is_down(&self) -> bool {
        self.hp <= 0
    }

    pub fn restore(&mut self) {
        self.hp = self.max_hp();
        self.mp = self.max_mp();
    }

    /// Keeps HP/MP within the (possibly changed) maxima.
    pub fn clamp(&mut self) {
        self.hp = self.hp.min(self.max_hp());
        self.mp = self.mp.clamp(0, self.max_mp());
    }

    pub fn can_equip(&self, item: ItemId) -> bool {
        match item.def().kind {
            ItemKind::Weapon(kind) => kind == self.def().weapon,
            ItemKind::Armor(kind) => self.def().can_wear(kind),
            ItemKind::Accessory => true,
            _ => false,
        }
    }

    pub fn slot(&self, slot: EquipSlot) -> Option<ItemId> {
        match slot {
            EquipSlot::Weapon => self.weapon,
            EquipSlot::Armor => self.armor,
            EquipSlot::Accessory => self.accessory,
        }
    }

    /// Puts `item` (or nothing) in its slot and returns what was there.
    pub fn equip(&mut self, slot: EquipSlot, item: Option<ItemId>) -> Option<ItemId> {
        let place = match slot {
            EquipSlot::Weapon => &mut self.weapon,
            EquipSlot::Armor => &mut self.armor,
            EquipSlot::Accessory => &mut self.accessory,
        };
        let old = std::mem::replace(place, item);
        self.clamp();
        old
    }

    pub fn xp_to_next(&self) -> u32 {
        xp_to_next(self.level)
    }

    /// Adds experience; returns one entry per level gained. HP and MP grow
    /// by the same amount as their maxima.
    pub fn gain_xp(&mut self, amount: u32) -> Vec<LevelUp> {
        let mut ups = Vec::new();
        if self.level >= MAX_LEVEL {
            return ups;
        }
        self.xp += amount;
        while self.level < MAX_LEVEL && self.xp >= xp_to_next(self.level) {
            self.xp -= xp_to_next(self.level);
            let before = self.def().stats_at(self.level);
            let known = self.skills();
            self.level += 1;
            let after = self.def().stats_at(self.level);
            let gains = after.add(before.scale(-1.0));
            self.hp += gains.hp;
            self.mp += gains.mp;
            let new_skills = self
                .skills()
                .into_iter()
                .filter(|s| !known.contains(s))
                .collect();
            ups.push(LevelUp {
                hero: self.id,
                level: self.level,
                gains,
                new_skills,
            });
        }
        if self.level >= MAX_LEVEL {
            self.xp = 0;
        }
        self.clamp();
        ups
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Inventory {
    items: BTreeMap<ItemId, u32>,
}

impl Inventory {
    pub fn add(&mut self, item: ItemId, count: u32) {
        if count > 0 {
            *self.items.entry(item).or_insert(0) += count;
        }
    }

    /// Removes up to `count`; returns how many were removed.
    pub fn remove(&mut self, item: ItemId, count: u32) -> u32 {
        let Some(have) = self.items.get_mut(&item) else {
            return 0;
        };
        let taken = count.min(*have);
        *have -= taken;
        if *have == 0 {
            self.items.remove(&item);
        }
        taken
    }

    pub fn count(&self, item: ItemId) -> u32 {
        self.items.get(&item).copied().unwrap_or(0)
    }

    pub fn has(&self, item: ItemId) -> bool {
        self.count(item) > 0
    }

    /// Everything carried, in item order.
    pub fn list(&self) -> Vec<(ItemId, u32)> {
        self.items.iter().map(|(&i, &c)| (i, c)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_ups_raise_stats_and_teach_skills() {
        let mut wren = Hero::new(HeroId::Wren, 1);
        let hp1 = wren.max_hp();
        let ups = wren.gain_xp(10_000);
        assert!(ups.len() > 3);
        assert!(wren.max_hp() > hp1);
        assert!(wren.skills().contains(&SkillId::Flare));
        assert!(ups.iter().any(|u| u.new_skills.contains(&SkillId::GuardianFlame)));
    }

    #[test]
    fn equipment_is_restricted_by_kind() {
        let wren = Hero::new(HeroId::Wren, 1);
        assert!(wren.can_equip(ItemId::IronSword));
        assert!(!wren.can_equip(ItemId::BattleAxe));
        assert!(!wren.can_equip(ItemId::ScholarRobe));
        assert!(wren.can_equip(ItemId::ScaleMail));
        let pip = Hero::new(HeroId::Pip, 1);
        assert!(!pip.can_equip(ItemId::ScaleMail));
        assert!(pip.can_equip(ItemId::HestaCharm));
    }

    #[test]
    fn inventory_counts() {
        let mut inv = Inventory::default();
        inv.add(ItemId::Tonic, 3);
        assert_eq!(inv.remove(ItemId::Tonic, 5), 3);
        assert!(!inv.has(ItemId::Tonic));
    }
}
