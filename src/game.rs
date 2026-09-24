//! Everything that is saved: the party, inventory, story flags and the map
//! the party is on. The UI mutates it through the methods here.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::audio::{Sfx, Track};
use crate::data::enemies::{BattleId, EnemyId};
use crate::data::heroes::HeroId;
use crate::data::items::{ItemId, ItemKind, Use};
use crate::data::quests::{QuestId, Requirement};
use crate::data::Element;
use crate::rpg::{Hero, Inventory};
use crate::story::{StoryContext, Visual};
use crate::world::floor::{self, boss_flag};
use crate::world::{town, EntityKind, Place, Shop, World};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Difficulty {
    Story,
    Normal,
    Hard,
}

impl Difficulty {
    pub const ALL: [Difficulty; 3] = [Difficulty::Story, Difficulty::Normal, Difficulty::Hard];

    pub fn name(self) -> &'static str {
        match self {
            Difficulty::Story => "Story",
            Difficulty::Normal => "Normal",
            Difficulty::Hard => "Hard",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            Difficulty::Story => "For the tale. Gentler foes, more experience.",
            Difficulty::Normal => "The intended balance.",
            Difficulty::Hard => "Tougher, hungrier monsters. Bring potions.",
        }
    }

    /// (enemy HP, enemy damage, experience)
    pub fn multipliers(self) -> (f32, f32, f32) {
        match self {
            Difficulty::Story => (0.75, 0.7, 1.3),
            Difficulty::Normal => (1.0, 1.0, 1.0),
            Difficulty::Hard => (1.3, 1.25, 1.0),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PlayStats {
    pub battles_won: u32,
    pub enemies_defeated: u32,
    pub steps: u32,
    pub gold_earned: u32,
    pub chests_opened: u32,
    pub defeats: u32,
}

/// Something the story or the game wants the UI to do.
#[derive(Clone, Debug, PartialEq)]
pub enum Pending {
    Music(Track),
    Sfx(Sfx),
    Visual(Visual),
    /// A short notice ("Received Tonic x3").
    Notice(String),
    Joined(HeroId),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum QuestState {
    Hidden,
    Active,
    Ready,
    Done,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Game {
    pub version: u32,
    pub party: Vec<Hero>,
    pub inventory: Inventory,
    pub gold: u32,
    pub flags: BTreeSet<String>,
    pub world: World,
    pub deepest: u32,
    pub playtime: f64,
    pub difficulty: Difficulty,
    pub seed: u64,
    pub stats: PlayStats,
    pub known_weak: BTreeSet<(EnemyId, Element)>,
    pub floor_visits: BTreeMap<u32, u32>,
    pub town_stage_seen: u8,
    #[serde(skip)]
    pub pending: Vec<Pending>,
}

pub const SAVE_VERSION: u32 = 1;

impl Game {
    pub fn new(difficulty: Difficulty, seed: u64) -> Self {
        let mut inventory = Inventory::default();
        inventory.add(ItemId::Tonic, 2);
        Self {
            version: SAVE_VERSION,
            party: vec![Hero::new(HeroId::Wren, 1)],
            inventory,
            gold: 150,
            flags: BTreeSet::new(),
            world: town::build(1, false),
            deepest: 0,
            playtime: 0.0,
            difficulty,
            seed,
            stats: PlayStats::default(),
            known_weak: BTreeSet::new(),
            floor_visits: BTreeMap::new(),
            town_stage_seen: 1,
            pending: Vec::new(),
        }
    }

    /// A game already partway down: the party levelled, equipped and joined
    /// as it would be on arriving at `floor` (for demos and testing).
    pub fn jump_start(floor: u32, difficulty: Difficulty, seed: u64) -> Self {
        use crate::data::enemies::floor_level;
        use crate::data::items::EquipSlot;
        let mut game = Game::new(difficulty, seed);
        let floor = floor.clamp(1, crate::world::floor::LAST_FLOOR);
        let level = floor_level(floor) + 2;
        game.party[0] = Hero::new(HeroId::Wren, level);
        for (f, h) in [(3, HeroId::Brannoc), (6, HeroId::Maelis), (10, HeroId::Pip)] {
            if floor > f {
                game.join(h);
            }
        }
        for (f, b) in [(3, BattleId::Vex), (4, BattleId::Gristlemaw), (8, BattleId::Curator), (12, BattleId::MotherOfSpores), (16, BattleId::IronWarden), (19, BattleId::Ilsa)] {
            if floor > f {
                game.set_flag(&boss_flag(b));
            }
        }
        for flag in ["elder_met", "seen_intro", "seen_wake", "seen_vaultgate_first", "seen_waystone_first", "mouser_found"] {
            game.set_flag(flag);
        }
        if floor > 8 {
            game.set_flag("truth_known");
        }
        let tier = ((floor - 1) / 4 + 1) as u8;
        for i in 0..game.party.len() {
            for slot in [EquipSlot::Weapon, EquipSlot::Armor] {
                let best = ItemId::ALL
                    .into_iter()
                    .filter(|it| it.def().tier == tier && it.def().slot() == Some(slot) && game.party[i].can_equip(*it))
                    .max_by_key(|it| {
                        let s = it.def().stats;
                        s.atk + s.mag + s.def + s.res
                    });
                if let Some(b) = best {
                    game.party[i].equip(slot, Some(b));
                }
            }
            game.party[i].restore();
        }
        game.pending.clear();
        game.inventory.add(ItemId::Tonic, 6);
        game.inventory.add(ItemId::Ether, 3);
        game.inventory.add(ItemId::EmberDown, 2);
        game.gold = 150 * floor;
        game.deepest = floor;
        game.town_stage_seen = game.act();
        game.enter_floor(floor);
        game
    }

    pub fn flag(&self, name: &str) -> bool {
        self.flags.contains(name)
    }

    pub fn set_flag(&mut self, name: &str) {
        self.flags.insert(name.to_string());
    }

    /// Story act 1-5, from the bosses beaten.
    pub fn act(&self) -> u8 {
        let beaten = |b: BattleId| self.flag(&boss_flag(b));
        if beaten(BattleId::IronWarden) {
            5
        } else if beaten(BattleId::MotherOfSpores) {
            4
        } else if beaten(BattleId::Curator) {
            3
        } else if beaten(BattleId::Gristlemaw) {
            2
        } else {
            1
        }
    }

    pub fn has_hero(&self, id: HeroId) -> bool {
        self.party.iter().any(|h| h.id == id)
    }

    pub fn hero_mut(&mut self, id: HeroId) -> Option<&mut Hero> {
        self.party.iter_mut().find(|h| h.id == id)
    }

    pub fn lead_level(&self) -> u32 {
        self.party[0].level
    }

    pub fn place_name(&self) -> String {
        match self.world.place {
            Place::Town => "Hollowmere".into(),
            Place::Floor(n) => format!("{} — Floor {n}", self.world.biome.name()),
        }
    }

    pub fn objective(&self) -> &'static str {
        let beaten = |b: BattleId| self.flag(&boss_flag(b));
        if !self.flag("elder_met") && self.deepest == 0 {
            "Speak with Elder Marrow at the Lantern Hall, then take the Vaultgate down."
        } else if !beaten(BattleId::Vex) && self.deepest < 3 {
            "Descend into the Undercroft beneath the Vaultgate."
        } else if !beaten(BattleId::Gristlemaw) {
            "Find what is driving the Undercroft's creatures upward (Floor 4)."
        } else if !beaten(BattleId::Curator) {
            "Search the Drowned Archive for the truth about the Wardens (Floor 8)."
        } else if !self.flag("elder_confronted") && self.world.place == Place::Town {
            "Confront Elder Marrow about the Wardens."
        } else if !beaten(BattleId::MotherOfSpores) {
            "Follow Ilsa's trail through the Mycelium Hollows (Floor 12)."
        } else if !beaten(BattleId::IronWarden) {
            "Cross the Ember Forge (Floor 16)."
        } else if !beaten(BattleId::Ilsa) {
            "Find Ilsa at the edge of the Pale (Floor 19)."
        } else {
            "Descend to the Hearthflame and face the Pale (Floor 20)."
        }
    }

    // ---- places

    pub fn enter_town(&mut self, at_gate: bool) {
        let stage = self.act();
        self.world = town::build(stage, self.flag("quest_lost_mouser_done"));
        if at_gate {
            self.world.player = town::GATE_ARRIVAL;
            self.world.facing = crate::world::Dir::Up;
            self.world.update_fov();
        }
    }

    pub fn enter_floor(&mut self, n: u32) {
        let visits = self.floor_visits.entry(n).or_insert(0);
        *visits += 1;
        let seed = self.seed.wrapping_add(n as u64 * 7919).wrapping_add(*visits as u64 * 104_729);
        self.world = floor::build(n, seed, &self.flags);
        self.deepest = self.deepest.max(n);
    }

    // ---- party upkeep

    pub fn heal_all(&mut self) {
        for h in &mut self.party {
            h.restore();
        }
    }

    /// KO'd heroes get back on their feet after a won battle.
    pub fn wake_fallen(&mut self) {
        for h in &mut self.party {
            if h.hp <= 0 {
                h.hp = 1;
            }
        }
    }

    pub fn join(&mut self, id: HeroId) {
        if self.has_hero(id) {
            return;
        }
        let level = self.lead_level().max(1);
        self.party.push(Hero::new(id, level));
        self.set_flag(&format!("has_{}", id.script_id()));
        self.world.remove_npc(id.script_id());
        self.pending.push(Pending::Joined(id));
    }

    /// Uses a consumable outside battle on a hero. Returns a message.
    pub fn use_item_on(&mut self, item: ItemId, hero: usize) -> Result<String, String> {
        let ItemKind::Consumable(effect) = item.def().kind else {
            return Err("That can't be used.".into());
        };
        if !self.inventory.has(item) {
            return Err("You don't have any.".into());
        }
        let h = &mut self.party[hero];
        let name = h.name();
        let msg = match effect {
            Use::Heal(n) if h.hp > 0 && h.hp < h.max_hp() => {
                let before = h.hp;
                h.hp = (h.hp + n).min(h.max_hp());
                format!("{name} recovers {} HP.", h.hp - before)
            }
            Use::HealFull if h.hp > 0 && h.hp < h.max_hp() => {
                h.hp = h.max_hp();
                format!("{name} is fully healed.")
            }
            Use::HealAll(n) => {
                for h in &mut self.party {
                    if h.hp > 0 {
                        h.hp = (h.hp + n).min(h.max_hp());
                    }
                }
                "The party is refreshed.".into()
            }
            Use::Mp(n) if h.hp > 0 && h.mp < h.max_mp() => {
                let before = h.mp;
                h.mp = (h.mp + n).min(h.max_mp());
                format!("{name} recovers {} MP.", h.mp - before)
            }
            Use::Revive(f) if h.hp <= 0 => {
                h.hp = ((h.max_hp() as f32 * f) as i32).max(1);
                format!("{name} is back on their feet!")
            }
            Use::Cure => format!("{name} feels clear-headed."),
            Use::Warp => {
                if self.world.place == Place::Town {
                    return Err("You are already in Hollowmere.".into());
                }
                self.inventory.remove(item, 1);
                self.enter_town(true);
                return Ok("The shard crumbles. Wind, light — Hollowmere.".into());
            }
            _ => return Err("It would have no effect.".into()),
        };
        self.inventory.remove(item, 1);
        Ok(msg)
    }

    // ---- quests

    pub fn quest_state(&self, q: QuestId) -> QuestState {
        if self.flag(&q.done_flag()) {
            QuestState::Done
        } else if self.flag(&q.active_flag()) {
            if self.requirement_met(q) { QuestState::Ready } else { QuestState::Active }
        } else {
            QuestState::Hidden
        }
    }

    pub fn requirement_met(&self, q: QuestId) -> bool {
        match q.def().requirement {
            Requirement::Items(item, n) => self.inventory.count(item) >= n,
            Requirement::Flag(f) => self.flag(f),
        }
    }

    /// Companion quests finish on their own; returns the scene to play.
    pub fn companion_quest_ready(&self) -> Option<QuestId> {
        QuestId::ALL
            .into_iter()
            .find(|&q| q.def().giver.is_none() && self.quest_state(q) == QuestState::Ready)
    }

    /// Which scene a town NPC plays when spoken to.
    pub fn npc_scene(&self, npc: &str) -> String {
        for q in QuestId::ALL {
            let def = q.def();
            if def.giver != Some(npc) {
                continue;
            }
            match self.quest_state(q) {
                QuestState::Ready => return format!("quest_{}_done", q.script_id()),
                QuestState::Hidden if self.act() >= def.from_act => return format!("quest_{}_offer", q.script_id()),
                _ => {}
            }
        }
        if npc == "elder" && !self.flag("elder_met") {
            return "elder_first".into();
        }
        if npc == "elder" && self.flag("truth_known") && !self.flag("elder_confronted") {
            return "elder_confront".into();
        }
        for q in QuestId::ALL {
            if q.def().giver == Some(npc) && self.quest_state(q) == QuestState::Active {
                // Alternate reminders with ordinary chatter.
                if self.stats.steps % 2 == 0 {
                    return format!("quest_{}_remind", q.script_id());
                }
            }
        }
        format!("npc_{npc}_{}", self.act())
    }

    pub fn shards(&self) -> Vec<u8> {
        (1..=12).filter(|n| self.flag(&format!("shard_{n}"))).collect()
    }

    pub fn journals(&self) -> Vec<u8> {
        (1..=5).filter(|n| self.flag(&format!("journal_{n}"))).collect()
    }

    pub fn update_shard_flag(&mut self) {
        if self.shards().len() == 12 {
            self.set_flag("shards_all");
        }
    }

    // ---- shops

    pub fn stock(&self, shop: Shop) -> Vec<ItemId> {
        let act = self.act();
        ItemId::ALL
            .into_iter()
            .filter(|i| {
                let d = i.def();
                if d.tier == 0 || d.tier > act {
                    return false;
                }
                match shop {
                    Shop::Smith => d.slot().is_some() && (d.tier + 1 >= act || matches!(d.kind, ItemKind::Accessory)),
                    Shop::Apothecary => matches!(d.kind, ItemKind::Consumable(_)) && *i != ItemId::HearthBread,
                    Shop::Inn => *i == ItemId::HearthBread || *i == ItemId::Tonic,
                    Shop::Temple => matches!(i, ItemId::EmberDown | ItemId::Panacea | ItemId::WaystoneShard),
                }
            })
            .collect()
    }

    pub fn buy(&mut self, item: ItemId) -> Result<(), String> {
        let price = item.def().price;
        if self.gold < price {
            return Err("Not enough gold.".into());
        }
        self.gold -= price;
        self.inventory.add(item, 1);
        Ok(())
    }

    pub fn sell(&mut self, item: ItemId) -> Result<u32, String> {
        let def = item.def();
        if matches!(def.kind, ItemKind::Quest) || def.price == 0 {
            return Err("You can't part with that.".into());
        }
        if self.inventory.remove(item, 1) == 0 {
            return Err("You don't have one.".into());
        }
        let price = def.sell_price();
        self.gold += price;
        Ok(price)
    }

    /// Equips `item` from the inventory on hero `h`.
    pub fn equip(&mut self, h: usize, item: ItemId) -> Result<(), String> {
        let Some(slot) = item.def().slot() else {
            return Err("That isn't equipment.".into());
        };
        if !self.party[h].can_equip(item) {
            return Err(format!("{} can't use that.", self.party[h].name()));
        }
        if self.inventory.remove(item, 1) == 0 {
            return Err("You don't have one.".into());
        }
        if let Some(old) = self.party[h].equip(slot, Some(item)) {
            self.inventory.add(old, 1);
        }
        Ok(())
    }

    pub fn unequip(&mut self, h: usize, slot: crate::data::items::EquipSlot) {
        if let Some(old) = self.party[h].equip(slot, None) {
            self.inventory.add(old, 1);
        }
    }

    /// Wandering floor NPC scenes that no longer apply (e.g. after joining).
    pub fn tidy_world(&mut self) {
        let joined: Vec<&str> = self.party.iter().map(|h| h.id.script_id()).collect();
        self.world
            .entities
            .retain(|e| !matches!(&e.kind, EntityKind::Npc { id, scene: Some(_), .. } if joined.contains(&id.as_str())));
    }
}

impl StoryContext for Game {
    fn flag(&self, name: &str) -> bool {
        Game::flag(self, name)
    }
    fn set_flag(&mut self, name: &str, on: bool) {
        if on {
            self.flags.insert(name.to_string());
        } else {
            self.flags.remove(name);
        }
    }
    fn give(&mut self, item: ItemId, count: u32) {
        self.inventory.add(item, count);
        let name = item.def().name;
        let text = if count > 1 { format!("Received {name} ×{count}") } else { format!("Received {name}") };
        self.pending.push(Pending::Notice(text));
        self.pending.push(Pending::Sfx(Sfx::ItemGet));
    }
    fn take(&mut self, item: ItemId, count: u32) {
        self.inventory.remove(item, count);
    }
    fn gold(&mut self, amount: i32) {
        if amount >= 0 {
            self.gold += amount as u32;
            self.pending.push(Pending::Notice(format!("Received {amount} gold")));
            self.pending.push(Pending::Sfx(Sfx::Coin));
        } else {
            self.gold = self.gold.saturating_sub((-amount) as u32);
        }
    }
    fn join(&mut self, hero: HeroId) {
        Game::join(self, hero);
    }
    fn quest_start(&mut self, quest: QuestId) {
        if !self.flag(&quest.done_flag()) && !self.flag(&quest.active_flag()) {
            self.flags.insert(quest.active_flag());
            self.pending.push(Pending::Notice(format!("New quest: {}", quest.def().name)));
        }
    }
    fn quest_done(&mut self, quest: QuestId) {
        if !self.flag(&quest.done_flag()) {
            self.flags.remove(&quest.active_flag());
            self.flags.insert(quest.done_flag());
            self.pending.push(Pending::Notice(format!("Quest complete: {}", quest.def().name)));
            self.pending.push(Pending::Sfx(Sfx::LevelUp));
        }
    }
    fn heal_party(&mut self) {
        self.heal_all();
    }
    fn music(&mut self, track: Track) {
        self.pending.push(Pending::Music(track));
    }
    fn sfx(&mut self, sfx: Sfx) {
        self.pending.push(Pending::Sfx(sfx));
    }
    fn visual(&mut self, visual: Visual) {
        self.pending.push(Pending::Visual(visual));
    }
}
