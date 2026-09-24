//! Turn-based battles, independent of any UI. Turn order is a timeline (like
//! Final Fantasy X): every combatant has a next-turn time that its speed and
//! the weight of its last action push forward. The UI drives the battle one
//! step at a time and animates the events each step returns.

pub mod ai;

use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

use crate::audio::Sfx;
use crate::data::enemies::{BattleId, EnemyId};
use crate::data::heroes::HeroId;
use crate::data::items::{ItemId, ItemKind, Special, Use};
use crate::data::skills::{Power, SkillDef, SkillId, SkillKind, Target};
use crate::data::{Affinity, Element, StatusKind, Stats};
use crate::gfx::sprites::Sprite;
use crate::rpg::Hero;

pub use ai::choose_action;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Heroes,
    Enemies,
}

impl Side {
    pub fn other(self) -> Side {
        match self {
            Side::Heroes => Side::Enemies,
            Side::Enemies => Side::Heroes,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Unit {
    pub side: Side,
    pub name: String,
    pub sprite: Sprite,
    /// Index into the party, for heroes.
    pub hero: Option<(usize, HeroId)>,
    pub enemy: Option<EnemyId>,
    pub level: u32,
    pub stats: Stats,
    pub hp: i32,
    pub mp: i32,
    pub max_hp: i32,
    pub max_mp: i32,
    pub statuses: Vec<(StatusKind, u8)>,
    pub affinities: [Affinity; 7],
    pub skills: Vec<SkillId>,
    /// AI weights, parallel to `skills` (enemies only).
    pub weights: Vec<u8>,
    pub specials: Vec<Special>,
    pub next_time: f32,
    pub guarding: bool,
    /// Drawn size in tiles.
    pub size: f32,
    pub boss: bool,
}

impl Unit {
    pub fn alive(&self) -> bool {
        self.hp > 0
    }

    pub fn has(&self, status: StatusKind) -> bool {
        self.statuses.iter().any(|&(s, _)| s == status)
    }

    fn eff(&self, base: i32, up: StatusKind, down: StatusKind, up_f: f32, down_f: f32) -> f32 {
        let mut v = base.max(1) as f32;
        if self.has(up) {
            v *= up_f;
        }
        if self.has(down) {
            v *= down_f;
        }
        v
    }

    pub fn atk(&self) -> f32 {
        self.eff(self.stats.atk, StatusKind::Might, StatusKind::Weak, 1.3, 0.7)
    }
    pub fn mag(&self) -> f32 {
        self.eff(self.stats.mag, StatusKind::Focus, StatusKind::Weak, 1.3, 0.7)
    }
    pub fn def(&self) -> f32 {
        self.eff(self.stats.def, StatusKind::Shield, StatusKind::Sunder, 1.4, 0.65)
    }
    pub fn res(&self) -> f32 {
        self.eff(self.stats.res, StatusKind::Shield, StatusKind::Sunder, 1.4, 0.65)
    }
    pub fn spd(&self) -> f32 {
        self.eff(self.stats.spd, StatusKind::Haste, StatusKind::Slow, 1.5, 0.6)
    }

    fn delay_for(&self, delay: u32) -> f32 {
        delay as f32 / (self.spd() + 15.0)
    }

    pub fn affinity(&self, element: Element) -> Affinity {
        let mut a = self.affinities[element.index()];
        if a == Affinity::Normal && self.specials.contains(&Special::Resist(element)) {
            a = Affinity::Resist;
        }
        a
    }

    fn crit_bonus(&self) -> u32 {
        self.specials
            .iter()
            .map(|s| match s {
                Special::CritUp(c) => *c as u32,
                _ => 0,
            })
            .sum()
    }

    fn attack_element(&self) -> Element {
        self.specials
            .iter()
            .find_map(|s| match s {
                Special::AttackElement(e) => Some(*e),
                _ => None,
            })
            .unwrap_or(Element::Physical)
    }

    pub fn can_afford(&self, skill: SkillId) -> bool {
        self.mp >= skill.def().mp
    }
}

/// One thing that happened, in order, for the UI to show.
#[derive(Clone, Debug)]
pub enum Event {
    TurnStart(usize),
    /// Skill or item name banner over the actor.
    Announce { actor: usize, text: String },
    /// The actor steps toward a target (physical) or raises its hands (magic).
    Lunge { actor: usize },
    Cast { actor: usize },
    Projectile { from: usize, to: usize, sprite: Sprite },
    Effect { target: usize, sprite: Sprite },
    Damage { target: usize, amount: i32, crit: bool, affinity: Affinity, element: Element },
    Heal { target: usize, amount: i32 },
    Mp { target: usize, amount: i32 },
    Miss { target: usize },
    StatusOn { target: usize, status: StatusKind },
    #[allow(dead_code)]
    StatusOff { target: usize, status: StatusKind },
    Down { target: usize },
    Revive { target: usize },
    Summoned { unit: usize },
    Guard { actor: usize },
    Message(String),
    Sfx(Sfx),
    Shake(f32),
    Flash,
    Pause(f32),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BattleKind {
    Normal,
    /// The party caught the enemies unaware: heroes act first.
    Preemptive,
    /// The enemies caught the party: they act first.
    Ambush,
    Story(BattleId),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Choice {
    Unit(usize),
    All,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Action {
    Skill(SkillId, Choice),
    Item(ItemId, Choice),
    Defend,
    Flee,
}

#[derive(Clone, Debug)]
pub enum Step {
    Events(Vec<Event>),
    /// This hero unit is waiting for the player's command.
    Command(usize),
    /// The boss fell and has a second form: play the interlude, then call
    /// `begin_second_phase`.
    PhaseChange,
    Victory,
    Defeat,
    Escaped,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Outcome {
    Victory,
    Defeat,
    Escaped,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum TurnStage {
    Ready,
}

pub struct Rewards {
    pub xp: u32,
    pub gold: u32,
    pub items: Vec<ItemId>,
}

pub struct Battle {
    pub units: Vec<Unit>,
    pub kind: BattleKind,
    pub phase: u8,
    pub turns_taken: u32,
    /// Weaknesses the party has hit during this battle.
    pub discovered: Vec<(EnemyId, Element)>,
    time: f32,
    turn: Option<(usize, TurnStage)>,
    outcome: Option<Outcome>,
    phase_change_pending: bool,
    pub(crate) rng: StdRng,
}

impl Battle {
    /// `party` is the whole party; heroes that are down still take part (they
    /// can be revived).
    pub fn new(party: &[Hero], enemies: &[(EnemyId, u32)], kind: BattleKind, seed: u64) -> Self {
        let mut battle = Self {
            units: Vec::new(),
            kind,
            phase: 1,
            turns_taken: 0,
            discovered: Vec::new(),
            time: 0.0,
            turn: None,
            outcome: None,
            phase_change_pending: false,
            rng: StdRng::seed_from_u64(seed),
        };
        for (i, hero) in party.iter().enumerate() {
            battle.units.push(hero_unit(i, hero));
        }
        for &(enemy, level) in enemies {
            let unit = enemy_unit(enemy, level);
            battle.units.push(unit);
        }
        battle.name_duplicates();
        let (hero_offset, enemy_offset) = match kind {
            BattleKind::Preemptive => (0.0, 2.5),
            BattleKind::Ambush => (2.5, 0.0),
            _ => (0.0, 0.3),
        };
        for i in 0..battle.units.len() {
            let jitter = battle.rng.random_range(0.0..1.0);
            let u = &mut battle.units[i];
            let offset = if u.side == Side::Heroes { hero_offset } else { enemy_offset };
            u.next_time = offset + u.delay_for(60) * jitter;
            if u.specials.contains(&Special::FirstStrike) {
                u.next_time = 0.0;
                u.statuses.push((StatusKind::Haste, 3));
            }
        }
        battle
    }

    fn name_duplicates(&mut self) {
        let letters = ['A', 'B', 'C', 'D', 'E', 'F'];
        for i in 0..self.units.len() {
            let Some(enemy) = self.units[i].enemy else { continue };
            let same: Vec<usize> = (0..self.units.len())
                .filter(|&j| self.units[j].enemy == Some(enemy))
                .collect();
            if same.len() > 1 {
                let k = same.iter().position(|&j| j == i).unwrap();
                self.units[i].name = format!("{} {}", enemy.def().name, letters[k.min(5)]);
            }
        }
    }

    pub fn is_boss(&self) -> bool {
        matches!(self.kind, BattleKind::Story(_))
    }

    pub fn can_flee(&self) -> bool {
        !self.is_boss()
    }

    pub fn alive(&self, side: Side) -> Vec<usize> {
        (0..self.units.len())
            .filter(|&i| self.units[i].side == side && self.units[i].alive())
            .collect()
    }

    pub fn dead(&self, side: Side) -> Vec<usize> {
        (0..self.units.len())
            .filter(|&i| self.units[i].side == side && !self.units[i].alive() && self.units[i].hero.is_some())
            .collect()
    }

    pub fn current(&self) -> Option<usize> {
        self.turn.map(|(u, _)| u)
    }

    /// The next `n` turns as they stand now (assuming normal-weight actions).
    pub fn forecast(&self, n: usize) -> Vec<usize> {
        let mut times: Vec<(f32, usize)> = self
            .units
            .iter()
            .enumerate()
            .filter(|(_, u)| u.alive())
            .map(|(i, u)| (u.next_time, i))
            .collect();
        let mut out = Vec::new();
        if let Some((u, _)) = self.turn {
            out.push(u);
        }
        while out.len() < n && !times.is_empty() {
            times.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            let (t, i) = times[0];
            if Some(i) != self.current() || out.len() > 1 {
                out.push(i);
            }
            times[0].0 = t + self.units[i].delay_for(100);
            if times.len() == 1 && out.len() >= n {
                break;
            }
        }
        out.truncate(n);
        out
    }

    /// Advances the battle by one beat.
    pub fn step(&mut self) -> Step {
        if let Some(outcome) = self.outcome {
            return match outcome {
                Outcome::Victory => Step::Victory,
                Outcome::Defeat => Step::Defeat,
                Outcome::Escaped => Step::Escaped,
            };
        }
        if self.phase_change_pending {
            return Step::PhaseChange;
        }
        match self.turn {
            None => {
                let Some(next) = self.next_unit() else {
                    self.check_end();
                    return Step::Events(vec![]);
                };
                self.time = self.units[next].next_time;
                let mut events = vec![Event::TurnStart(next)];
                let can_act = self.start_turn(next, &mut events);
                if can_act {
                    self.turn = Some((next, TurnStage::Ready));
                } else {
                    self.end_turn(next, 100);
                }
                self.check_end();
                Step::Events(events)
            }
            Some((unit, TurnStage::Ready)) => {
                if self.units[unit].side == Side::Heroes {
                    Step::Command(unit)
                } else {
                    let action = choose_action(self, unit);
                    let events = self.act(unit, action);
                    Step::Events(events)
                }
            }
        }
    }

    fn next_unit(&self) -> Option<usize> {
        self.units
            .iter()
            .enumerate()
            .filter(|(_, u)| u.alive())
            .min_by(|a, b| {
                a.1.next_time
                    .total_cmp(&b.1.next_time)
                    .then((a.1.side == Side::Enemies).cmp(&(b.1.side == Side::Enemies)))
            })
            .map(|(i, _)| i)
    }

    /// Damage/healing over time and status expiry. Returns whether the unit
    /// can act this turn.
    fn start_turn(&mut self, i: usize, events: &mut Vec<Event>) -> bool {
        self.turns_taken += 1;
        self.units[i].guarding = false;
        let (max_hp, level) = (self.units[i].max_hp, self.units[i].level as i32);
        let dot = |pct: f32, flat: i32| ((max_hp as f32 * pct) as i32).min(flat + level * 5).max(1);

        if self.units[i].has(StatusKind::Poison) {
            let amount = dot(0.07, 10);
            self.hurt(i, amount, false, Affinity::Normal, Element::Nature, events);
        }
        if self.units[i].alive() && self.units[i].has(StatusKind::Burn) {
            let amount = dot(0.06, 12);
            self.hurt(i, amount, false, Affinity::Normal, Element::Fire, events);
        }
        if !self.units[i].alive() {
            return false;
        }
        let regen_status = self.units[i].has(StatusKind::Regen);
        let regen_gear = self.units[i].specials.contains(&Special::Regen);
        if regen_status || regen_gear {
            let pct = if regen_status { 0.07 } else { 0.0 } + if regen_gear { 0.04 } else { 0.0 };
            let amount = ((max_hp as f32 * pct) as i32).max(1);
            self.restore_hp(i, amount, events);
        }
        if self.units[i].specials.contains(&Special::MpRegen) {
            let amount = (self.units[i].max_mp as f32 * 0.05).max(2.0) as i32;
            let u = &mut self.units[i];
            let before = u.mp;
            u.mp = (u.mp + amount).min(u.max_mp);
            if u.mp > before {
                events.push(Event::Mp { target: i, amount: u.mp - before });
            }
        }

        let skip = self.units[i].has(StatusKind::Stun) || self.units[i].has(StatusKind::Frozen);
        let mut expired = Vec::new();
        for (status, turns) in self.units[i].statuses.iter_mut() {
            if matches!(status, StatusKind::Stun | StatusKind::Frozen) {
                *turns = 0;
            } else {
                *turns = turns.saturating_sub(1);
            }
            if *turns == 0 {
                expired.push(*status);
            }
        }
        self.units[i].statuses.retain(|&(_, t)| t > 0);
        if skip {
            let name = self.units[i].name.clone();
            let what = if expired.contains(&StatusKind::Frozen) { "is frozen solid" } else { "is stunned" };
            events.push(Event::Message(format!("{name} {what}!")));
            events.push(Event::Pause(0.5));
        }
        for status in expired {
            events.push(Event::StatusOff { target: i, status });
        }
        !skip
    }

    fn end_turn(&mut self, i: usize, delay: u32) {
        let d = self.units[i].delay_for(delay);
        self.units[i].next_time = self.time + d;
        self.turn = None;
    }

    /// Performs `action` for `actor` (whose turn it must be).
    pub fn act(&mut self, actor: usize, action: Action) -> Vec<Event> {
        let mut events = Vec::new();
        let delay = match action {
            Action::Skill(skill, choice) => {
                self.use_skill(actor, skill, choice, &mut events);
                skill.def().delay
            }
            Action::Item(item, choice) => {
                self.use_item(actor, item, choice, &mut events);
                90
            }
            Action::Defend => {
                self.units[actor].guarding = true;
                let u = &mut self.units[actor];
                let mp = (u.max_mp as f32 * 0.06).max(2.0) as i32;
                let before = u.mp;
                u.mp = (u.mp + mp).min(u.max_mp);
                events.push(Event::Guard { actor });
                events.push(Event::Sfx(Sfx::Block));
                if u.mp > before {
                    events.push(Event::Mp { target: actor, amount: u.mp - before });
                }
                events.push(Event::Pause(0.3));
                70
            }
            Action::Flee => {
                self.try_flee(actor, &mut events);
                100
            }
        };
        if self.outcome.is_none() {
            self.end_turn(actor, delay);
        } else {
            self.turn = None;
        }
        self.check_end();
        events
    }

    fn try_flee(&mut self, actor: usize, events: &mut Vec<Event>) {
        if !self.can_flee() {
            events.push(Event::Message("There is no escape!".into()));
            events.push(Event::Sfx(Sfx::Denied));
            return;
        }
        let avg = |side: Side| {
            let a = self.alive(side);
            a.iter().map(|&i| self.units[i].spd()).sum::<f32>() / a.len().max(1) as f32
        };
        let chance = (0.5 + (avg(Side::Heroes) - avg(Side::Enemies)) * 0.03).clamp(0.25, 0.9);
        if self.rng.random_bool(chance as f64) {
            events.push(Event::Sfx(Sfx::Flee));
            events.push(Event::Message("The party slips away!".into()));
            self.outcome = Some(Outcome::Escaped);
        } else {
            let name = self.units[actor].name.clone();
            events.push(Event::Message(format!("{name} couldn't get away!")));
            events.push(Event::Sfx(Sfx::Denied));
            events.push(Event::Pause(0.4));
        }
    }

    /// Resolves who a choice actually hits.
    pub fn targets(&self, actor: usize, target: Target, choice: Choice) -> Vec<usize> {
        let side = self.units[actor].side;
        match target {
            Target::Myself => vec![actor],
            Target::AllFoes => self.alive(side.other()),
            Target::AllAllies => self.alive(side),
            Target::Foe | Target::Ally | Target::DeadAlly => {
                let wanted_side = if target == Target::Foe { side.other() } else { side };
                let want_alive = target != Target::DeadAlly;
                let ok = |i: usize| {
                    self.units.get(i).is_some_and(|u| u.side == wanted_side && u.alive() == want_alive)
                };
                match choice {
                    Choice::Unit(i) if ok(i) => vec![i],
                    _ => {
                        // Retarget: the chosen unit is gone.
                        let pool: Vec<usize> = (0..self.units.len()).filter(|&i| ok(i)).collect();
                        pool.first().copied().into_iter().collect()
                    }
                }
            }
        }
    }

    fn use_skill(&mut self, actor: usize, skill: SkillId, choice: Choice, events: &mut Vec<Event>) {
        let def = skill.def();
        if self.units[actor].mp < def.mp {
            events.push(Event::Message("Not enough MP!".into()));
            return;
        }
        self.units[actor].mp -= def.mp;
        let mut targets = self.targets(actor, def.target, choice);
        // Taunt pulls single-target enemy attacks.
        if def.target == Target::Foe && self.units[actor].side == Side::Enemies {
            let taunters: Vec<usize> = self
                .alive(Side::Heroes)
                .into_iter()
                .filter(|&i| self.units[i].has(StatusKind::Taunt))
                .collect();
            if !taunters.is_empty() && self.rng.random_bool(0.85) {
                targets = vec![taunters[self.rng.random_range(0..taunters.len())]];
            }
        }
        if skill != SkillId::Attack {
            events.push(Event::Announce { actor, text: def.name.to_string() });
        }
        let physical = matches!(def.kind, SkillKind::Damage { power: Power::Atk(_), .. });
        if physical {
            events.push(Event::Lunge { actor });
        } else {
            events.push(Event::Cast { actor });
        }

        match def.kind {
            SkillKind::Damage { power, hits } => {
                let element = if skill == SkillId::Attack { self.units[actor].attack_element() } else { def.element };
                for _ in 0..hits {
                    for &t in &targets {
                        if !self.units[t].alive() {
                            continue;
                        }
                        self.strike(actor, t, def, power, element, events);
                    }
                }
                if targets.len() > 1 || hits > 1 {
                    events.push(Event::Shake(4.0));
                }
            }
            SkillKind::Drain { power } | SkillKind::Siphon { power } => {
                for &t in &targets {
                    let dealt = self.strike(actor, t, def, power, def.element, events);
                    if dealt > 0 {
                        if matches!(def.kind, SkillKind::Drain { .. }) {
                            self.restore_hp(actor, dealt / 2, events);
                        } else {
                            let u = &mut self.units[actor];
                            let gain = (dealt / 5).max(1).min(u.max_mp - u.mp);
                            u.mp += gain;
                            events.push(Event::Mp { target: actor, amount: gain });
                        }
                    }
                }
            }
            SkillKind::Heal { power, flat } => {
                let mag = self.units[actor].mag();
                for &t in &targets {
                    // Lifebloom (group heal with revive) raises the fallen first.
                    events.push(Event::Effect { target: t, sprite: def.fx });
                    let amount = ((mag * power) as i32 + flat) as f32 * self.rng.random_range(0.95..1.05);
                    self.restore_hp(t, amount as i32, events);
                }
                if skill == SkillId::Lifebloom {
                    for t in self.dead(self.units[actor].side) {
                        let amount = self.units[t].max_hp / 2;
                        self.revive(t, amount, events);
                    }
                }
                events.push(Event::Sfx(def.sfx));
            }
            SkillKind::Revive { fraction } => {
                for &t in &targets {
                    events.push(Event::Effect { target: t, sprite: def.fx });
                    let amount = (self.units[t].max_hp as f32 * fraction) as i32;
                    self.revive(t, amount, events);
                }
                events.push(Event::Sfx(def.sfx));
            }
            SkillKind::Cleanse => {
                let mag = self.units[actor].mag();
                for &t in &targets {
                    events.push(Event::Effect { target: t, sprite: def.fx });
                    self.cleanse(t, events);
                    self.restore_hp(t, (mag * 0.6) as i32 + 20, events);
                }
                events.push(Event::Sfx(def.sfx));
            }
            SkillKind::Status => {
                events.push(Event::Sfx(def.sfx));
                for &t in &targets {
                    events.push(Event::Effect { target: t, sprite: def.fx });
                }
            }
            SkillKind::Summon(enemy, count) => {
                events.push(Event::Sfx(def.sfx));
                let level = self.units[actor].level.saturating_sub(2).max(1);
                for _ in 0..count {
                    if self.alive(Side::Enemies).len() >= 5 {
                        break;
                    }
                    let mut unit = enemy_unit(enemy, level);
                    unit.next_time = self.time + unit.delay_for(100);
                    self.units.push(unit);
                    events.push(Event::Summoned { unit: self.units.len() - 1 });
                }
                self.name_duplicates();
            }
            SkillKind::Nothing => {
                let name = self.units[actor].name.clone();
                events.push(Event::Message(format!("{name} hesitates...")));
                events.push(Event::Pause(0.6));
            }
        }

        // Statuses: damage skills only apply to targets still standing.
        if !def.statuses.is_empty() {
            for &t in &targets {
                if !self.units[t].alive() {
                    continue;
                }
                for &(status, chance, turns) in def.statuses {
                    self.try_status(t, status, chance, turns, events);
                }
            }
        }
        events.push(Event::Pause(0.25));
    }

    /// One damage roll against one target. Returns damage dealt.
    fn strike(
        &mut self,
        actor: usize,
        t: usize,
        def: &SkillDef,
        power: Power,
        element: Element,
        events: &mut Vec<Event>,
    ) -> i32 {
        if let Some(sprite) = def.projectile {
            events.push(Event::Projectile { from: actor, to: t, sprite });
        }
        let (a, d, p, physical) = match power {
            Power::Atk(p) => (self.units[actor].atk(), self.units[t].def(), p, true),
            Power::Mag(p) => (self.units[actor].mag(), self.units[t].res(), p, false),
        };
        // Blind and basic evasion only affect physical blows.
        if physical {
            let miss = if self.units[actor].has(StatusKind::Blind) { 0.45 } else { 0.03 };
            if self.rng.random_bool(miss) {
                events.push(Event::Miss { target: t });
                events.push(Event::Sfx(Sfx::Miss));
                return 0;
            }
        }
        events.push(Event::Effect { target: t, sprite: def.fx });
        let mut dmg = p * a * a / (a + d) * self.rng.random_range(0.9..1.1);
        let crit_chance = if physical { 5 + def.crit as u32 + self.units[actor].crit_bonus() } else { def.crit as u32 };
        let crit = self.rng.random_range(0..100) < crit_chance;
        if crit {
            dmg *= 1.6;
        }
        if self.units[t].guarding {
            dmg *= 0.5;
        }
        let affinity = self.units[t].affinity(element);
        dmg *= affinity.multiplier();
        if affinity == Affinity::Weak {
            if let Some(enemy) = self.units[t].enemy {
                if !self.discovered.contains(&(enemy, element)) {
                    self.discovered.push((enemy, element));
                }
            }
        }
        // Elements interact with states: fire thaws, frost puts out burns.
        if element == Element::Fire {
            self.remove_status(t, StatusKind::Frozen, events);
        }
        if element == Element::Frost {
            self.remove_status(t, StatusKind::Burn, events);
        }
        let sfx = if crit || affinity == Affinity::Weak { Sfx::CritHit } else { def.sfx };
        events.push(Event::Sfx(sfx));
        if affinity == Affinity::Absorb {
            let amount = (-dmg).max(1.0) as i32;
            self.restore_hp(t, amount, events);
            return 0;
        }
        let amount = if affinity == Affinity::Immune { 0 } else { dmg.max(1.0) as i32 };
        if crit {
            events.push(Event::Shake(6.0));
        }
        self.hurt(t, amount, crit, affinity, element, events);
        // A hard knock breaks ice.
        if physical && amount > 0 && self.units[t].alive() && self.rng.random_bool(0.5) {
            self.remove_status(t, StatusKind::Frozen, events);
        }
        amount
    }

    fn hurt(&mut self, t: usize, amount: i32, crit: bool, affinity: Affinity, element: Element, events: &mut Vec<Event>) {
        let u = &mut self.units[t];
        u.hp = (u.hp - amount).max(0);
        events.push(Event::Damage { target: t, amount, crit, affinity, element });
        if u.hp == 0 {
            u.statuses.clear();
            u.guarding = false;
            events.push(Event::Down { target: t });
            events.push(Event::Sfx(if u.side == Side::Heroes { Sfx::PartyDown } else { Sfx::EnemyDie }));
        }
    }

    fn restore_hp(&mut self, t: usize, amount: i32, events: &mut Vec<Event>) {
        let u = &mut self.units[t];
        if !u.alive() {
            return;
        }
        let gained = amount.max(0).min(u.max_hp - u.hp);
        u.hp += gained;
        events.push(Event::Heal { target: t, amount: gained });
    }

    fn revive(&mut self, t: usize, amount: i32, events: &mut Vec<Event>) {
        let u = &mut self.units[t];
        if u.alive() {
            return;
        }
        u.hp = amount.clamp(1, u.max_hp);
        u.next_time = self.time + 0.5;
        events.push(Event::Revive { target: t });
        events.push(Event::Heal { target: t, amount: u.hp });
    }

    fn cleanse(&mut self, t: usize, events: &mut Vec<Event>) {
        let harmful: Vec<StatusKind> = self.units[t]
            .statuses
            .iter()
            .map(|&(s, _)| s)
            .filter(|s| s.is_harmful())
            .collect();
        for s in harmful {
            self.remove_status(t, s, events);
        }
    }

    fn remove_status(&mut self, t: usize, status: StatusKind, events: &mut Vec<Event>) {
        let u = &mut self.units[t];
        if u.has(status) {
            u.statuses.retain(|&(s, _)| s != status);
            events.push(Event::StatusOff { target: t, status });
        }
    }

    fn try_status(&mut self, t: usize, status: StatusKind, chance: u8, turns: u8, events: &mut Vec<Event>) {
        let mut chance = chance as f32;
        let u = &self.units[t];
        if status.is_harmful() {
            if u.specials.contains(&Special::StatusGuard) {
                chance *= 0.5;
            }
            if u.boss {
                chance *= if matches!(status, StatusKind::Stun | StatusKind::Frozen) { 0.2 } else { 0.6 };
            }
            if status == StatusKind::Poison && u.affinity(Element::Nature) == Affinity::Immune {
                chance = 0.0;
            }
        }
        if self.rng.random_range(0.0..100.0) >= chance {
            return;
        }
        let u = &mut self.units[t];
        if let Some(existing) = u.statuses.iter_mut().find(|(s, _)| *s == status) {
            existing.1 = existing.1.max(turns);
        } else {
            u.statuses.push((status, turns));
        }
        // Opposites cancel.
        let opposite = match status {
            StatusKind::Haste => Some(StatusKind::Slow),
            StatusKind::Slow => Some(StatusKind::Haste),
            _ => None,
        };
        if let Some(o) = opposite {
            self.remove_status(t, o, events);
        }
        events.push(Event::StatusOn { target: t, status });
    }

    fn use_item(&mut self, actor: usize, item: ItemId, choice: Choice, events: &mut Vec<Event>) {
        let def = item.def();
        let ItemKind::Consumable(effect) = def.kind else {
            return;
        };
        events.push(Event::Announce { actor, text: def.name.to_string() });
        events.push(Event::Cast { actor });
        let side = self.units[actor].side;
        let one_ally = |b: &Battle| b.targets(actor, Target::Ally, choice);
        match effect {
            Use::Heal(amount) => {
                for t in one_ally(self) {
                    events.push(Event::Effect { target: t, sprite: Sprite::FxHeal });
                    self.restore_hp(t, amount, events);
                }
                events.push(Event::Sfx(Sfx::Heal));
            }
            Use::HealFull => {
                for t in one_ally(self) {
                    events.push(Event::Effect { target: t, sprite: Sprite::FxHeal });
                    let amount = self.units[t].max_hp;
                    self.restore_hp(t, amount, events);
                }
                events.push(Event::Sfx(Sfx::Heal));
            }
            Use::HealAll(amount) => {
                for t in self.alive(side) {
                    events.push(Event::Effect { target: t, sprite: Sprite::FxHeal });
                    self.restore_hp(t, amount, events);
                }
                events.push(Event::Sfx(Sfx::Heal));
            }
            Use::Mp(amount) => {
                for t in one_ally(self) {
                    events.push(Event::Effect { target: t, sprite: Sprite::FxBuff });
                    let u = &mut self.units[t];
                    let gain = amount.min(u.max_mp - u.mp).max(0);
                    u.mp += gain;
                    events.push(Event::Mp { target: t, amount: gain });
                }
                events.push(Event::Sfx(Sfx::Heal));
            }
            Use::Revive(fraction) => {
                for t in self.targets(actor, Target::DeadAlly, choice) {
                    events.push(Event::Effect { target: t, sprite: Sprite::FxHeal });
                    let amount = (self.units[t].max_hp as f32 * fraction) as i32;
                    self.revive(t, amount, events);
                }
                events.push(Event::Sfx(Sfx::Heal));
            }
            Use::Cure => {
                for t in one_ally(self) {
                    events.push(Event::Effect { target: t, sprite: Sprite::FxHeal });
                    self.cleanse(t, events);
                }
                events.push(Event::Sfx(Sfx::Heal));
            }
            Use::DamageAll(element, amount) => {
                for t in self.alive(side.other()) {
                    self.item_damage(t, element, amount, events);
                }
                events.push(Event::Shake(5.0));
            }
            Use::DamageOne(element, amount, status) => {
                for t in self.targets(actor, Target::Foe, choice) {
                    self.item_damage(t, element, amount, events);
                    if let Some(s) = status {
                        if self.units[t].alive() {
                            self.try_status(t, s, 50, 1, events);
                        }
                    }
                }
            }
            Use::Escape => {
                if self.can_flee() {
                    events.push(Event::Sfx(Sfx::Flee));
                    events.push(Event::Message("A cloud of smoke — the party escapes!".into()));
                    self.outcome = Some(Outcome::Escaped);
                } else {
                    events.push(Event::Message("The smoke clears. There is no escape!".into()));
                }
            }
            Use::Warp => {}
        }
        events.push(Event::Pause(0.25));
    }

    fn item_damage(&mut self, t: usize, element: Element, amount: i32, events: &mut Vec<Event>) {
        let fx = match element {
            Element::Fire => Sprite::FxFire,
            Element::Frost => Sprite::FxFrost,
            _ => Sprite::FxExplosion,
        };
        events.push(Event::Effect { target: t, sprite: fx });
        let affinity = self.units[t].affinity(element);
        let dmg = (amount as f32 * affinity.multiplier() * self.rng.random_range(0.9..1.1)) as i32;
        events.push(Event::Sfx(if element == Element::Fire { Sfx::Fire } else { Sfx::Frost }));
        if dmg < 0 {
            self.restore_hp(t, -dmg, events);
        } else {
            self.hurt(t, dmg, false, affinity, element, events);
        }
    }

    fn check_end(&mut self) {
        if self.outcome.is_some() || self.phase_change_pending {
            return;
        }
        if self.alive(Side::Heroes).is_empty() {
            self.outcome = Some(Outcome::Defeat);
            self.turn = None;
        } else if self.alive(Side::Enemies).is_empty() {
            let second = match self.kind {
                BattleKind::Story(id) if self.phase == 1 => id.second_phase(),
                _ => None,
            };
            if second.is_some() {
                self.phase_change_pending = true;
            } else {
                self.outcome = Some(Outcome::Victory);
            }
            self.turn = None;
        }
    }

    /// Replaces the fallen boss with its second form.
    pub fn begin_second_phase(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        let BattleKind::Story(id) = self.kind else {
            return events;
        };
        let Some((enemy, level)) = id.second_phase() else {
            return events;
        };
        self.phase = 2;
        self.phase_change_pending = false;
        // Clear the old side and restore a little of the party's spirit.
        self.units.retain(|u| u.side == Side::Heroes);
        let mut unit = enemy_unit(enemy, level);
        unit.next_time = self.time + 1.0;
        self.units.push(unit);
        let summoned = self.units.len() - 1;
        events.push(Event::Summoned { unit: summoned });
        events.push(Event::Flash);
        // Ilsa's voice from above: the party rallies.
        for t in self.dead(Side::Heroes) {
            let amount = self.units[t].max_hp / 3;
            self.revive(t, amount, &mut events);
        }
        for t in self.alive(Side::Heroes) {
            let amount = self.units[t].max_hp * 2 / 5;
            self.restore_hp(t, amount, &mut events);
            for status in [StatusKind::Might, StatusKind::Regen] {
                self.try_status(t, status, 100, 4, &mut events);
            }
        }
        events.push(Event::Sfx(Sfx::BossRoar));
        events
    }

    pub fn rewards(&mut self) -> Rewards {
        let mut xp = 0;
        let mut gold = 0;
        let mut items = Vec::new();
        for i in 0..self.units.len() {
            let Some(enemy) = self.units[i].enemy else { continue };
            let def = enemy.def();
            let level = self.units[i].level;
            xp += def.xp_at(level);
            gold += def.gold_at(level);
            for &(item, chance) in def.drops {
                if self.rng.random_range(0..100) < chance as u32 {
                    items.push(item);
                }
            }
        }
        Rewards { xp, gold, items }
    }

    /// Average level of the enemies fought (for experience scaling).
    pub fn enemy_level(&self) -> u32 {
        let levels: Vec<u32> = self.units.iter().filter(|u| u.enemy.is_some()).map(|u| u.level).collect();
        levels.iter().sum::<u32>() / levels.len().max(1) as u32
    }
}

/// Experience a hero actually gets: less for fights far below their level,
/// a little more when catching up.
pub fn xp_share(xp: u32, hero_level: u32, enemy_level: u32) -> u32 {
    let diff = hero_level as i32 - enemy_level as i32;
    let f = match diff {
        i32::MIN..=-3 => 1.3,
        -2..=2 => 1.0,
        3..=5 => 0.6,
        _ => 0.3,
    };
    (xp as f32 * f).round() as u32
}

fn hero_unit(index: usize, hero: &Hero) -> Unit {
    let def = hero.def();
    let stats = hero.stats();
    Unit {
        side: Side::Heroes,
        name: def.name.to_string(),
        sprite: def.sprite,
        hero: Some((index, hero.id)),
        enemy: None,
        level: hero.level,
        stats,
        hp: hero.hp.max(0),
        mp: hero.mp,
        max_hp: hero.max_hp(),
        max_mp: hero.max_mp(),
        statuses: Vec::new(),
        affinities: [Affinity::Normal; 7],
        skills: hero.skills(),
        weights: Vec::new(),
        specials: hero.specials(),
        next_time: 0.0,
        guarding: false,
        size: 3.0,
        boss: false,
    }
}

pub fn enemy_unit(enemy: EnemyId, level: u32) -> Unit {
    let def = enemy.def();
    let stats = def.stats_at(level);
    let mut affinities = [Affinity::Normal; 7];
    for e in Element::ALL {
        affinities[e.index()] = def.affinity(e);
    }
    Unit {
        side: Side::Enemies,
        name: def.name.to_string(),
        sprite: def.sprite,
        hero: None,
        enemy: Some(enemy),
        level,
        stats,
        hp: stats.hp,
        mp: stats.mp,
        max_hp: stats.hp,
        max_mp: stats.mp,
        statuses: Vec::new(),
        affinities,
        skills: def.skills.iter().map(|&(s, _)| s).collect(),
        weights: def.skills.iter().map(|&(_, w)| w).collect(),
        specials: Vec::new(),
        next_time: 0.0,
        guarding: false,
        size: def.size,
        boss: def.boss,
    }
}

#[cfg(test)]
mod tests;
