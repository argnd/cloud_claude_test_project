//! The battle screen: animates the events the battle logic produces and
//! collects the player's commands.

use std::collections::VecDeque;

use eframe::egui::{self, Align2, Color32, CornerRadius, Pos2, Rect, Stroke, pos2, vec2};

use super::input::Input;
use super::widgets::{self, ListState, Row};
use crate::audio::{Audio, Sfx, Track};
use crate::battle::ai::{auto_action, is_auto_item};
use crate::battle::{Action, Battle, BattleKind, Choice, Event, Side, Step, xp_share};
use crate::data::enemies::BattleId;
use crate::data::items::ItemId;
use crate::data::skills::{SkillId, SkillKind, Target};
use crate::data::{Affinity, Element, StatusKind};
use crate::game::Game;
use crate::gfx::sprites::Sprite;
use crate::gfx::{self, GOLD, Gfx, PARCHMENT};
use crate::rpg::LevelUp;
use crate::world::Biome;

#[derive(Clone, Default)]
struct UnitVis {
    shown_hp: i32,
    shown_mp: i32,
    dead: bool,
    death_t: f32,
    lunge: f32,
    cast: f32,
    hit: f32,
    revealed: bool,
    appear: f32,
}

struct Popup {
    unit: usize,
    text: String,
    colour: Color32,
    age: f32,
    size: f32,
    offset: f32,
}

struct Fx {
    sprite: Sprite,
    from: Option<usize>,
    to: usize,
    age: f32,
    dur: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Intro,
    Running,
    Command(usize),
    Victory,
    Defeat,
    Done,
}

#[derive(Clone, PartialEq)]
enum Menu {
    Root,
    Skills,
    Items,
    Target {
        action: PendingAction,
        target: Target,
        cursor: usize,
    },
}

#[derive(Clone, Copy, PartialEq)]
enum PendingAction {
    Skill(SkillId),
    Item(ItemId),
}

pub enum BattleSignal {
    PhaseChange,
    Victory,
    Defeat,
    Escaped,
}

pub struct RewardsView {
    xp: u32,
    gold: u32,
    items: Vec<ItemId>,
    level_ups: Vec<LevelUp>,
    age: f32,
}

pub struct BattleView {
    pub battle: Battle,
    pub story: Option<BattleId>,
    /// The map monster that started this fight.
    pub entity: Option<usize>,
    biome: Biome,
    phase: Phase,
    queue: VecDeque<Event>,
    wait: f32,
    vis: Vec<UnitVis>,
    popups: Vec<Popup>,
    fx: Vec<Fx>,
    banner: Option<(String, f32)>,
    message: Option<(String, f32)>,
    shake: f32,
    flash: f32,
    pub auto: bool,
    menu: Menu,
    root: ListState,
    skills: ListState,
    items: ListState,
    intro_t: f32,
    rewards: Option<RewardsView>,
    active: Option<usize>,
    t: f32,
    defeat_t: f32,
}

const ROOT: [&str; 6] = ["Attack", "Skills", "Items", "Defend", "Auto", "Flee"];

impl BattleView {
    pub fn new(
        battle: Battle,
        story: Option<BattleId>,
        entity: Option<usize>,
        biome: Biome,
    ) -> Self {
        let vis = battle
            .units
            .iter()
            .map(|u| UnitVis {
                shown_hp: u.hp,
                shown_mp: u.mp,
                dead: !u.alive(),
                revealed: true,
                appear: 1.0,
                ..Default::default()
            })
            .collect();
        let banner = match battle.kind {
            BattleKind::Preemptive => Some(("Preemptive strike!".to_string(), 1.8)),
            BattleKind::Ambush => Some(("Ambush!".to_string(), 1.8)),
            BattleKind::Story(_) => {
                let boss = battle
                    .units
                    .iter()
                    .find(|u| u.boss)
                    .map(|u| u.name.clone())
                    .unwrap_or_default();
                Some((boss, 2.4))
            }
            BattleKind::Normal => None,
        };
        Self {
            battle,
            story,
            entity,
            biome,
            phase: Phase::Intro,
            queue: VecDeque::new(),
            wait: 0.0,
            vis,
            popups: Vec::new(),
            fx: Vec::new(),
            banner,
            message: None,
            shake: 0.0,
            flash: 1.0,
            auto: false,
            menu: Menu::Root,
            root: ListState::default(),
            skills: ListState::default(),
            items: ListState::default(),
            intro_t: 0.0,
            rewards: None,
            active: None,
            t: 0.0,
            defeat_t: 0.0,
        }
    }

    /// A one-off hint shown under the banner.
    pub fn tip(&mut self, text: &str) {
        self.message = Some((text.to_string(), 6.0));
    }

    pub fn music(&self) -> Track {
        match self.story {
            Some(BattleId::Aurelian) => Track::FinalBoss,
            Some(_) => Track::Boss,
            None => Track::Battle,
        }
    }

    /// After the phase-two interlude.
    pub fn begin_second_phase(&mut self) {
        let events = self.battle.begin_second_phase();
        self.vis.truncate(
            self.battle.alive(Side::Heroes).len().max(
                self.battle
                    .units
                    .iter()
                    .filter(|u| u.side == Side::Heroes)
                    .count(),
            ),
        );
        while self.vis.len() < self.battle.units.len() {
            let u = &self.battle.units[self.vis.len()];
            self.vis.push(UnitVis {
                shown_hp: u.hp,
                shown_mp: u.mp,
                revealed: false,
                ..Default::default()
            });
        }
        self.queue.extend(events);
        self.banner = Some((
            self.battle
                .units
                .last()
                .map(|u| u.name.clone())
                .unwrap_or_default(),
            2.4,
        ));
        self.phase = Phase::Running;
    }

    fn sync_vis(&mut self) {
        while self.vis.len() < self.battle.units.len() {
            let u = &self.battle.units[self.vis.len()];
            self.vis.push(UnitVis {
                shown_hp: u.hp,
                shown_mp: u.mp,
                revealed: false,
                ..Default::default()
            });
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        input: &mut Input,
        game: &mut Game,
        audio: &mut Audio,
        speed: f32,
    ) -> Option<BattleSignal> {
        let dt = input.dt;
        self.t += dt;
        let speed = speed * if input.fast { 2.5 } else { 1.0 };
        let sdt = dt * speed;
        self.intro_t += dt;
        for v in &mut self.vis {
            v.lunge = (v.lunge + sdt / 0.34).min(1.0);
            v.cast = (v.cast + sdt / 0.45).min(1.0);
            v.hit = (v.hit - sdt * 4.0).max(0.0);
            if v.dead {
                v.death_t = (v.death_t + sdt / 0.6).min(1.0);
            } else {
                v.death_t = 0.0;
            }
            if v.revealed {
                v.appear = (v.appear + sdt / 0.5).min(1.0);
            }
        }
        for p in &mut self.popups {
            p.age += dt;
        }
        self.popups.retain(|p| p.age < 1.3);
        for f in &mut self.fx {
            f.age += sdt;
        }
        self.fx.retain(|f| f.age < f.dur);
        if let Some(b) = &mut self.banner {
            b.1 -= dt;
        }
        if self.banner.as_ref().is_some_and(|b| b.1 <= 0.0) {
            self.banner = None;
        }
        if let Some(m) = &mut self.message {
            m.1 -= dt;
        }
        if self.message.as_ref().is_some_and(|m| m.1 <= 0.0) {
            self.message = None;
        }

        match self.phase {
            Phase::Intro => {
                if self.intro_t > 0.9 {
                    self.phase = Phase::Running;
                }
                return None;
            }
            Phase::Victory => {
                if let Some(r) = &mut self.rewards {
                    r.age += dt;
                    if (input.confirm || input.click) && r.age > 0.6 {
                        self.phase = Phase::Done;
                        return Some(BattleSignal::Victory);
                    }
                }
                return None;
            }
            Phase::Defeat => {
                self.defeat_t += dt;
                if self.defeat_t > 2.2 {
                    self.phase = Phase::Done;
                    return Some(BattleSignal::Defeat);
                }
                return None;
            }
            Phase::Done => return None,
            _ => {}
        }

        // Play queued events.
        if self.wait > 0.0 {
            self.wait -= sdt;
            return None;
        }
        while let Some(e) = self.queue.pop_front() {
            let d = self.play(e, audio);
            if d > 0.0 {
                self.wait = d;
                return None;
            }
        }

        if let Phase::Command(unit) = self.phase {
            if self.auto {
                self.phase = Phase::Running;
                self.run_auto(unit, game);
                return None;
            }
            return None; // Commands come in through draw().
        }

        // Advance the logic.
        match self.battle.step() {
            Step::Events(events) => {
                self.sync_vis();
                self.queue.extend(events);
            }
            Step::Command(unit) => {
                self.active = Some(unit);
                if self.auto {
                    self.run_auto(unit, game);
                } else {
                    self.phase = Phase::Command(unit);
                    self.menu = Menu::Root;
                    self.root = ListState::default();
                    audio.play_sfx(Sfx::MenuMove);
                }
            }
            Step::PhaseChange => {
                return Some(BattleSignal::PhaseChange);
            }
            Step::Victory => {
                self.win(game, audio);
            }
            Step::Defeat => {
                self.phase = Phase::Defeat;
                audio.play_music(Track::GameOver);
            }
            Step::Escaped => {
                self.phase = Phase::Done;
                self.write_back(game);
                return Some(BattleSignal::Escaped);
            }
        }
        None
    }

    fn run_auto(&mut self, unit: usize, game: &mut Game) {
        let items: Vec<(ItemId, u32)> = game
            .inventory
            .list()
            .into_iter()
            .filter(|(i, _)| is_auto_item(*i))
            .collect();
        let action = auto_action(&self.battle, unit, &items);
        self.perform(unit, action, game);
    }

    fn perform(&mut self, unit: usize, action: Action, game: &mut Game) {
        if let Action::Item(item, _) = action {
            game.inventory.remove(item, 1);
        }
        let events = self.battle.act(unit, action);
        self.sync_vis();
        self.queue.extend(events);
        self.phase = Phase::Running;
        self.menu = Menu::Root;
    }

    fn write_back(&self, game: &mut Game) {
        for u in &self.battle.units {
            if let Some((i, _)) = u.hero
                && let Some(h) = game.party.get_mut(i)
            {
                h.hp = u.hp.max(0);
                h.mp = u.mp.max(0);
            }
        }
    }

    fn win(&mut self, game: &mut Game, audio: &mut Audio) {
        self.write_back(game);
        let rewards = self.battle.rewards();
        let (_, _, xp_mult) = game.difficulty.multipliers();
        let enemy_level = self.battle.enemy_level();
        let mut level_ups = Vec::new();
        let mut shown_xp = 0;
        for h in &mut game.party {
            let xp = (xp_share(rewards.xp, h.level, enemy_level) as f32 * xp_mult) as u32;
            shown_xp = shown_xp.max(xp);
            level_ups.extend(h.gain_xp(xp));
        }
        game.wake_fallen();
        game.gold += rewards.gold;
        game.stats.gold_earned += rewards.gold;
        game.stats.battles_won += 1;
        game.stats.enemies_defeated += self
            .battle
            .units
            .iter()
            .filter(|u| u.enemy.is_some())
            .count() as u32;
        for item in &rewards.items {
            game.inventory.add(*item, 1);
        }
        for &d in &self.battle.discovered {
            game.known_weak.insert(d);
        }
        audio.play_music(Track::Victory);
        if !level_ups.is_empty() {
            audio.play_sfx(Sfx::LevelUp);
        }
        self.rewards = Some(RewardsView {
            xp: shown_xp,
            gold: rewards.gold,
            items: rewards.items,
            level_ups,
            age: 0.0,
        });
        self.phase = Phase::Victory;
    }

    /// Starts showing one event; returns how long to wait before the next.
    fn play(&mut self, e: Event, audio: &mut Audio) -> f32 {
        self.sync_vis();
        match e {
            Event::TurnStart(u) => {
                self.active = Some(u);
                0.08
            }
            Event::Announce { actor, text } => {
                let _ = actor;
                self.banner = Some((text, 1.1));
                0.35
            }
            Event::Lunge { actor } => {
                self.vis[actor].lunge = 0.0;
                0.2
            }
            Event::Cast { actor } => {
                self.vis[actor].cast = 0.0;
                0.28
            }
            Event::Projectile { from, to, sprite } => {
                self.fx.push(Fx {
                    sprite,
                    from: Some(from),
                    to,
                    age: 0.0,
                    dur: 0.28,
                });
                0.28
            }
            Event::Effect { target, sprite } => {
                self.fx.push(Fx {
                    sprite,
                    from: None,
                    to: target,
                    age: 0.0,
                    dur: 0.5,
                });
                0.08
            }
            Event::Damage {
                target,
                amount,
                crit,
                affinity,
                element,
            } => {
                let v = &mut self.vis[target];
                v.shown_hp -= amount;
                v.hit = 1.0;
                let [r, g, b] = element.colour();
                let colour = if self.battle.units[target].side == Side::Heroes {
                    Color32::from_rgb(255, 120, 110)
                } else {
                    Color32::from_rgb(r, g, b)
                };
                let text = if affinity == Affinity::Immune {
                    "IMMUNE".to_string()
                } else {
                    amount.to_string()
                };
                self.popup(target, text, colour, if crit { 40.0 } else { 32.0 });
                if crit {
                    self.popup(target, "CRITICAL!".into(), GOLD, 22.0);
                }
                match affinity {
                    Affinity::Weak => self.popup(
                        target,
                        "WEAK!".into(),
                        Color32::from_rgb(255, 210, 80),
                        24.0,
                    ),
                    Affinity::Resist => {
                        self.popup(target, "resist".into(), Color32::from_gray(180), 20.0)
                    }
                    _ => {}
                }
                if crit || affinity == Affinity::Weak {
                    self.shake = self.shake.max(7.0);
                }
                0.03
            }
            Event::Heal { target, amount } => {
                self.vis[target].shown_hp += amount;
                if amount > 0 {
                    self.popup(
                        target,
                        format!("+{amount}"),
                        Color32::from_rgb(120, 255, 140),
                        30.0,
                    );
                }
                0.03
            }
            Event::Mp { target, amount } => {
                self.vis[target].shown_mp += amount;
                if amount > 0 {
                    self.popup(
                        target,
                        format!("+{amount} MP"),
                        Color32::from_rgb(130, 180, 255),
                        24.0,
                    );
                }
                0.0
            }
            Event::Miss { target } => {
                self.popup(target, "miss".into(), Color32::from_gray(200), 26.0);
                0.05
            }
            Event::StatusOn { target, status } => {
                let colour = if status.is_harmful() {
                    Color32::from_rgb(200, 130, 255)
                } else {
                    Color32::from_rgb(255, 220, 120)
                };
                self.popup(target, status.name().to_string(), colour, 21.0);
                0.1
            }
            Event::StatusOff { .. } => 0.0,
            Event::Down { target } => {
                self.vis[target].dead = true;
                self.vis[target].shown_hp = 0;
                0.3
            }
            Event::Revive { target } => {
                self.vis[target].dead = false;
                self.vis[target].shown_hp = 0;
                0.1
            }
            Event::Summoned { unit } => {
                self.sync_vis();
                let hp = self.battle.units[unit].hp;
                let v = &mut self.vis[unit];
                v.revealed = true;
                v.appear = 0.0;
                v.dead = false;
                v.shown_hp = hp;
                0.35
            }
            Event::Guard { actor } => {
                self.popup(
                    actor,
                    "Guard".into(),
                    Color32::from_rgb(180, 210, 255),
                    22.0,
                );
                0.15
            }
            Event::Message(text) => {
                self.message = Some((text, 1.6));
                0.6
            }
            Event::Sfx(s) => {
                audio.play_sfx(s);
                0.0
            }
            Event::Shake(a) => {
                self.shake = self.shake.max(a);
                0.0
            }
            Event::Flash => {
                self.flash = 1.0;
                0.0
            }
            Event::Pause(t) => t,
        }
    }

    fn popup(&mut self, unit: usize, text: String, colour: Color32, size: f32) {
        let stacked = self
            .popups
            .iter()
            .filter(|p| p.unit == unit && p.age < 0.4)
            .count();
        self.popups.push(Popup {
            unit,
            text,
            colour,
            age: 0.0,
            size,
            offset: stacked as f32 * 26.0,
        });
    }

    // ---- layout

    fn unit_anchor(&self, screen: Rect, i: usize) -> Pos2 {
        let u = &self.battle.units[i];
        let (w, h) = (screen.width(), screen.height());
        if u.side == Side::Heroes {
            let k = self
                .battle
                .units
                .iter()
                .take(i)
                .filter(|x| x.side == Side::Heroes)
                .count() as f32;
            return pos2(
                screen.left() + w * (0.70 + k * 0.035),
                screen.top() + h * (0.30 + k * 0.12),
            );
        }
        let enemies: Vec<usize> = (0..self.battle.units.len())
            .filter(|&j| self.battle.units[j].side == Side::Enemies)
            .collect();
        let k = enemies.iter().position(|&j| j == i).unwrap_or(0);
        let n = enemies.len();
        if u.boss {
            return pos2(screen.left() + w * 0.27, screen.top() + h * 0.50);
        }
        let has_boss = enemies.iter().any(|&j| self.battle.units[j].boss);
        if has_boss {
            let slots = [
                (0.43, 0.30),
                (0.43, 0.62),
                (0.12, 0.28),
                (0.12, 0.64),
                (0.40, 0.46),
            ];
            let non_boss_k = enemies
                .iter()
                .take(k)
                .filter(|&&j| !self.battle.units[j].boss)
                .count();
            let (x, y) = slots[non_boss_k % slots.len()];
            return pos2(screen.left() + w * x, screen.top() + h * y);
        }
        let t = (k as f32 + 0.5) / n as f32;
        let x =
            0.30 - (t * std::f32::consts::PI).sin() * 0.1 + if k % 2 == 1 { -0.07 } else { 0.0 };
        pos2(screen.left() + w * x, screen.top() + h * (0.24 + t * 0.46))
    }

    fn unit_rect(&self, screen: Rect, i: usize) -> Rect {
        let u = &self.battle.units[i];
        let s = screen.height() / 800.0;
        let size = u.size * 32.0 * s;
        let anchor = self.unit_anchor(screen, i);
        let v = &self.vis[i.min(self.vis.len() - 1)];
        // Lunge toward the other side and back.
        let dir = if u.side == Side::Heroes { -1.0 } else { 1.0 };
        let lunge = (v.lunge * std::f32::consts::PI).sin() * 50.0 * s * dir;
        let bob = ((self.t * 2.2 + i as f32).sin()) * 3.0 * s;
        Rect::from_center_size(anchor + vec2(lunge, bob - size * 0.5), vec2(size, size))
    }

    // ---- drawing and command input

    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        painter: &egui::Painter,
        gfx: &Gfx,
        screen: Rect,
        input: &mut Input,
        game: &mut Game,
        audio: &mut Audio,
        time: f64,
    ) {
        // Screen effects fade even while a scene plays over the battle.
        self.flash = (self.flash - input.dt * 2.5).max(0.0);
        self.shake = (self.shake - input.dt * 30.0).max(0.0);
        let shake = if self.shake > 0.0 {
            vec2(
                ((time * 90.0).sin() as f32) * self.shake,
                ((time * 70.0).cos() as f32) * self.shake * 0.5,
            )
        } else {
            vec2(0.0, 0.0)
        };
        let field = screen.translate(shake);
        self.draw_background(painter, gfx, screen, time);

        // Intro: combatants slide in.
        let intro = ease((self.intro_t / 0.8).min(1.0));

        // Units.
        let mut order: Vec<usize> = (0..self.battle.units.len().min(self.vis.len())).collect();
        order.sort_by(|&a, &b| {
            self.unit_anchor(field, a)
                .y
                .total_cmp(&self.unit_anchor(field, b).y)
        });
        let targets = self.current_targets();
        for &i in &order {
            let u = &self.battle.units[i];
            let v = &self.vis[i];
            if !v.revealed {
                continue;
            }
            let mut r = self.unit_rect(field, i);
            let slide = (1.0 - intro)
                * screen.width()
                * 0.5
                * if u.side == Side::Heroes { 1.0 } else { -1.0 };
            r = r.translate(vec2(slide, 0.0));
            let mut alpha = v.appear;
            let mut tint = Color32::WHITE;
            if v.dead {
                if u.side == Side::Enemies {
                    alpha *= 1.0 - v.death_t;
                    r = r.translate(vec2(0.0, v.death_t * 20.0));
                    tint = Color32::from_rgb(255, 120, 120);
                } else {
                    tint = Color32::from_rgb(90, 90, 110);
                }
            }
            if alpha <= 0.01 {
                continue;
            }
            // Shadow and selection ring.
            let feet = r.center_bottom();
            painter.add(egui::Shape::ellipse_filled(
                feet,
                vec2(r.width() * 0.34, r.width() * 0.08),
                Color32::from_black_alpha((110.0 * alpha) as u8),
            ));
            if Some(i) == self.active
                && matches!(self.phase, Phase::Command(_) | Phase::Running)
                && u.side == Side::Heroes
                && !v.dead
            {
                let pulse = 0.6 + 0.4 * (time * 5.0).sin() as f32;
                painter.add(egui::Shape::ellipse_stroke(
                    feet,
                    vec2(r.width() * 0.42, r.width() * 0.12),
                    Stroke::new(2.5, GOLD.gamma_multiply(pulse)),
                ));
            }
            if v.cast < 1.0 {
                let c = 1.0 - v.cast;
                painter.circle_filled(
                    r.center(),
                    r.width() * (0.4 + v.cast * 0.5),
                    Color32::from_rgba_unmultiplied(255, 230, 150, (90.0 * c) as u8),
                );
            }
            let hit_tint = if v.hit > 0.0 {
                gfx::lerp_colour(tint, Color32::from_rgb(255, 60, 60), v.hit)
            } else {
                tint
            };
            let final_tint = hit_tint.gamma_multiply(alpha);
            let jitter = if v.hit > 0.5 {
                vec2(((time * 80.0).sin() as f32) * 4.0, 0.0)
            } else {
                vec2(0.0, 0.0)
            };
            if u.side == Side::Heroes {
                gfx.draw_flipped(painter, u.sprite, r.translate(jitter), final_tint);
            } else {
                gfx.draw(painter, u.sprite, r.translate(jitter), final_tint);
            }
            if v.hit > 0.6 {
                gfx.draw(
                    painter,
                    u.sprite,
                    r.translate(jitter),
                    Color32::from_white_alpha(((v.hit - 0.6) * 300.0) as u8),
                );
            }
            // Enemy HP bar.
            if u.side == Side::Enemies && !v.dead && !u.boss {
                let br = Rect::from_center_size(
                    r.center_bottom() + vec2(0.0, 12.0),
                    vec2(r.width().max(70.0) * 0.8, 7.0),
                );
                gfx::bar(
                    painter,
                    br,
                    v.shown_hp as f32 / u.max_hp as f32,
                    gfx::HP_RED,
                );
            }
            // Status icons.
            let icons: Vec<Sprite> = u.statuses.iter().map(|&(s, _)| status_icon(s)).collect();
            for (k, icon) in icons.iter().enumerate().take(6) {
                let ir = Rect::from_min_size(
                    r.left_top() + vec2(k as f32 * 22.0, -18.0),
                    vec2(22.0, 22.0),
                );
                gfx.draw(painter, *icon, ir, Color32::WHITE.gamma_multiply(alpha));
            }
            // Target arrow.
            if targets.contains(&i) {
                let y = r.top() - 30.0 + ((time * 8.0).sin() as f32) * 5.0;
                painter.add(egui::Shape::convex_polygon(
                    vec![
                        pos2(r.center().x - 12.0, y),
                        pos2(r.center().x + 12.0, y),
                        pos2(r.center().x, y + 16.0),
                    ],
                    GOLD,
                    Stroke::new(1.5, Color32::BLACK),
                ));
            }
        }

        // Effects.
        for f in &self.fx {
            let t = (f.age / f.dur).clamp(0.0, 1.0);
            let to = self
                .unit_rect(field, f.to.min(self.battle.units.len() - 1))
                .center();
            let s = screen.height() / 800.0;
            if let Some(from) = f.from {
                let a = self.unit_rect(field, from).center();
                let p = a + (to - a) * t - vec2(0.0, (t * std::f32::consts::PI).sin() * 40.0 * s);
                let r = Rect::from_center_size(p, vec2(56.0 * s, 56.0 * s));
                gfx.draw(painter, f.sprite, r, Color32::WHITE);
            } else {
                let size = (70.0 + t * 60.0) * s;
                let a = (1.0 - t).powf(0.7);
                let r = Rect::from_center_size(to, vec2(size, size));
                painter.circle_filled(
                    to,
                    size * 0.5,
                    Color32::from_rgba_unmultiplied(255, 240, 200, (40.0 * a) as u8),
                );
                gfx.draw(painter, f.sprite, r, Color32::WHITE.gamma_multiply(a));
            }
        }

        // Popups.
        for p in &self.popups {
            if p.unit >= self.battle.units.len() {
                continue;
            }
            let r = self.unit_rect(field, p.unit);
            let rise = p.age * 50.0 + p.offset;
            let a = (1.0 - (p.age - 0.8).max(0.0) / 0.5).clamp(0.0, 1.0);
            let pop = 1.0 + (0.15 - p.age).max(0.0) * 3.0;
            gfx::text(
                painter,
                pos2(r.center().x, r.top() + r.height() * 0.3 - rise),
                Align2::CENTER_CENTER,
                &p.text,
                gfx::heading_font(p.size * pop),
                p.colour.gamma_multiply(a),
            );
        }

        self.draw_boss_bar(painter, screen);
        self.draw_turn_order(painter, gfx, screen);
        self.draw_party_panel(painter, gfx, screen, time);

        // Banner and messages.
        if let Some((text, left)) = &self.banner {
            let a = (left / 0.3).min(1.0);
            let r = Rect::from_center_size(
                pos2(screen.center().x, screen.top() + 118.0),
                vec2(text.len() as f32 * 15.0 + 120.0, 50.0),
            );
            painter.rect_filled(
                r,
                CornerRadius::same(6),
                Color32::from_black_alpha((190.0 * a) as u8),
            );
            painter.rect_stroke(
                r,
                CornerRadius::same(6),
                Stroke::new(1.5, GOLD.gamma_multiply(a)),
                egui::StrokeKind::Inside,
            );
            gfx::text(
                painter,
                r.center(),
                Align2::CENTER_CENTER,
                text,
                gfx::heading_font(26.0),
                GOLD.gamma_multiply(a),
            );
        }
        if let Some((text, left)) = &self.message {
            let a = (left / 0.3).min(1.0);
            let r = Rect::from_center_size(
                pos2(screen.center().x, screen.top() + 176.0),
                vec2(text.len() as f32 * 11.0 + 80.0, 40.0),
            );
            painter.rect_filled(
                r,
                CornerRadius::same(6),
                Color32::from_black_alpha((170.0 * a) as u8),
            );
            gfx::text(
                painter,
                r.center(),
                Align2::CENTER_CENTER,
                text,
                gfx::body_font(22.0),
                PARCHMENT.gamma_multiply(a),
            );
        }

        if let Phase::Command(unit) = self.phase {
            self.command_ui(painter, gfx, screen, input, game, audio, unit, time);
        }

        if self.auto && self.phase != Phase::Victory {
            let r = Rect::from_min_size(
                pos2(screen.right() - 170.0, screen.top() + 70.0),
                vec2(150.0, 32.0),
            );
            painter.rect_filled(r, CornerRadius::same(6), Color32::from_black_alpha(160));
            gfx::text(
                painter,
                r.center(),
                Align2::CENTER_CENTER,
                "AUTO  (Esc stops)",
                gfx::body_font(17.0),
                GOLD,
            );
            if input.cancel {
                self.auto = false;
                input.cancel = false;
            }
        }

        if self.phase == Phase::Victory {
            self.draw_rewards(painter, gfx, screen, game);
        }
        if self.phase == Phase::Defeat {
            let a = (self.defeat_t / 1.5).min(1.0);
            painter.rect_filled(
                screen,
                CornerRadius::ZERO,
                Color32::from_rgba_unmultiplied(20, 0, 0, (220.0 * a) as u8),
            );
            gfx::text(
                painter,
                screen.center(),
                Align2::CENTER_CENTER,
                "The light goes out...",
                gfx::title_font(44.0),
                Color32::from_rgb(220, 90, 80).gamma_multiply(a),
            );
        }
        // Intro flash.
        if self.flash > 0.0 {
            painter.rect_filled(
                screen,
                CornerRadius::ZERO,
                Color32::from_white_alpha((self.flash * 200.0) as u8),
            );
        }
    }

    fn current_targets(&self) -> Vec<usize> {
        let Phase::Command(unit) = self.phase else {
            return vec![];
        };
        let Menu::Target { target, cursor, .. } = &self.menu else {
            return vec![];
        };
        let candidates = self.target_candidates(unit, *target);
        if target.is_group() {
            candidates
        } else {
            candidates.get(*cursor).copied().into_iter().collect()
        }
    }

    fn target_candidates(&self, unit: usize, target: Target) -> Vec<usize> {
        let side = self.battle.units[unit].side;
        match target {
            Target::Foe | Target::AllFoes => self.battle.alive(side.other()),
            Target::Ally | Target::AllAllies => self.battle.alive(side),
            Target::DeadAlly => self.battle.dead(side),
            Target::Myself => vec![unit],
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn command_ui(
        &mut self,
        painter: &egui::Painter,
        gfx: &Gfx,
        screen: Rect,
        input: &mut Input,
        game: &mut Game,
        audio: &mut Audio,
        unit: usize,
        time: f64,
    ) {
        let s = screen.height() / 800.0;
        let panel_h = 262.0 * s.max(1.0);
        let area = Rect::from_min_size(
            pos2(screen.left() + 16.0, screen.bottom() - panel_h - 12.0),
            vec2(360.0 * s.max(1.0), panel_h),
        );
        let u = self.battle.units[unit].clone();
        match self.menu.clone() {
            Menu::Root => {
                gfx::panel(painter, area);
                gfx::text(
                    painter,
                    area.left_top() + vec2(16.0, 10.0),
                    Align2::LEFT_TOP,
                    &u.name,
                    gfx::heading_font(20.0),
                    GOLD,
                );
                let rows: Vec<Row> = ROOT
                    .iter()
                    .map(|&n| {
                        let enabled = match n {
                            "Flee" => self.battle.can_flee(),
                            "Items" => game
                                .inventory
                                .list()
                                .iter()
                                .any(|(i, _)| i.def().usable_in_battle()),
                            _ => true,
                        };
                        Row::new(n).enabled(enabled)
                    })
                    .collect();
                let inner = Rect::from_min_max(
                    area.left_top() + vec2(10.0, 38.0),
                    area.right_bottom() - vec2(10.0, 8.0),
                );
                let r = widgets::list(
                    painter,
                    gfx,
                    inner,
                    &rows,
                    &mut self.root,
                    input,
                    true,
                    time,
                );
                if r.moved {
                    audio.play_sfx(Sfx::MenuMove);
                }
                if r.denied {
                    audio.play_sfx(Sfx::Denied);
                }
                if let Some(i) = r.picked {
                    audio.play_sfx(Sfx::MenuSelect);
                    match ROOT[i] {
                        "Attack" => {
                            self.menu = Menu::Target {
                                action: PendingAction::Skill(SkillId::Attack),
                                target: Target::Foe,
                                cursor: 0,
                            }
                        }
                        "Skills" => self.menu = Menu::Skills,
                        "Items" => self.menu = Menu::Items,
                        "Defend" => self.perform(unit, Action::Defend, game),
                        "Auto" => {
                            self.auto = true;
                            self.phase = Phase::Running;
                            self.run_auto(unit, game);
                        }
                        "Flee" => self.perform(unit, Action::Flee, game),
                        _ => {}
                    }
                }
            }
            Menu::Skills => {
                let skills: Vec<SkillId> = u.skills.clone();
                let rows: Vec<Row> = skills
                    .iter()
                    .map(|&s| {
                        let d = s.def();
                        let usable = u.mp >= d.mp
                            && !(matches!(d.kind, SkillKind::Revive { .. })
                                && self.battle.dead(Side::Heroes).is_empty());
                        Row::new(d.name)
                            .right(format!("{} MP", d.mp))
                            .icon(d.icon)
                            .enabled(usable)
                    })
                    .collect();
                let wide =
                    Rect::from_min_size(area.left_top(), vec2(area.width() * 1.9, area.height()));
                gfx::panel(painter, wide);
                gfx::text(
                    painter,
                    wide.left_top() + vec2(16.0, 10.0),
                    Align2::LEFT_TOP,
                    &format!("Skills — {} MP", u.mp),
                    gfx::heading_font(18.0),
                    GOLD,
                );
                let list_rect = Rect::from_min_size(
                    wide.left_top() + vec2(10.0, 38.0),
                    vec2(wide.width() * 0.55, wide.height() - 46.0),
                );
                let r = widgets::list(
                    painter,
                    gfx,
                    list_rect,
                    &rows,
                    &mut self.skills,
                    input,
                    true,
                    time,
                );
                if let Some(&sk) = skills.get(self.skills.cursor) {
                    let d = sk.def();
                    let desc = Rect::from_min_max(
                        pos2(list_rect.right() + 14.0, wide.top() + 40.0),
                        wide.right_bottom() - vec2(14.0, 10.0),
                    );
                    gfx::wrapped(
                        painter,
                        desc.min,
                        desc.width(),
                        d.desc,
                        gfx::italic_font(19.0),
                        PARCHMENT,
                    );
                    let el = if d.element == Element::Physical {
                        String::new()
                    } else {
                        format!("{} · ", d.element.name())
                    };
                    let tgt = match d.target {
                        Target::Foe => "one foe",
                        Target::AllFoes => "all foes",
                        Target::Ally => "one ally",
                        Target::AllAllies => "party",
                        Target::Myself => "self",
                        Target::DeadAlly => "fallen ally",
                    };
                    gfx::text(
                        painter,
                        pos2(desc.left(), desc.bottom() - 4.0),
                        Align2::LEFT_BOTTOM,
                        &format!("{el}{tgt}"),
                        gfx::body_font(17.0),
                        gfx::DIM,
                    );
                }
                if r.moved {
                    audio.play_sfx(Sfx::MenuMove);
                }
                if r.denied {
                    audio.play_sfx(Sfx::Denied);
                }
                if let Some(i) = r.picked {
                    audio.play_sfx(Sfx::MenuSelect);
                    let skill = skills[i];
                    self.choose_target(unit, PendingAction::Skill(skill), skill.def().target, game);
                } else if input.cancel {
                    audio.play_sfx(Sfx::MenuBack);
                    self.menu = Menu::Root;
                }
            }
            Menu::Items => {
                let items: Vec<(ItemId, u32)> = game
                    .inventory
                    .list()
                    .into_iter()
                    .filter(|(i, _)| i.def().usable_in_battle())
                    .collect();
                let rows: Vec<Row> = items
                    .iter()
                    .map(|&(i, c)| {
                        Row::new(i.def().name)
                            .right(format!("×{c}"))
                            .icon(i.def().sprite)
                    })
                    .collect();
                let wide =
                    Rect::from_min_size(area.left_top(), vec2(area.width() * 1.9, area.height()));
                gfx::panel(painter, wide);
                gfx::text(
                    painter,
                    wide.left_top() + vec2(16.0, 10.0),
                    Align2::LEFT_TOP,
                    "Items",
                    gfx::heading_font(18.0),
                    GOLD,
                );
                let list_rect = Rect::from_min_size(
                    wide.left_top() + vec2(10.0, 38.0),
                    vec2(wide.width() * 0.55, wide.height() - 46.0),
                );
                let r = widgets::list(
                    painter,
                    gfx,
                    list_rect,
                    &rows,
                    &mut self.items,
                    input,
                    true,
                    time,
                );
                if let Some(&(it, _)) = items.get(self.items.cursor) {
                    let desc = Rect::from_min_max(
                        pos2(list_rect.right() + 14.0, wide.top() + 40.0),
                        wide.right_bottom() - vec2(14.0, 10.0),
                    );
                    gfx::wrapped(
                        painter,
                        desc.min,
                        desc.width(),
                        it.def().desc,
                        gfx::italic_font(19.0),
                        PARCHMENT,
                    );
                }
                if r.moved {
                    audio.play_sfx(Sfx::MenuMove);
                }
                if let Some(i) = r.picked {
                    audio.play_sfx(Sfx::MenuSelect);
                    let item = items[i].0;
                    let target = item_target(item);
                    self.choose_target(unit, PendingAction::Item(item), target, game);
                } else if input.cancel {
                    audio.play_sfx(Sfx::MenuBack);
                    self.menu = Menu::Root;
                }
            }
            Menu::Target {
                action,
                target,
                mut cursor,
            } => {
                let candidates = self.target_candidates(unit, target);
                if candidates.is_empty() {
                    self.menu = Menu::Root;
                    return;
                }
                cursor = cursor.min(candidates.len() - 1);
                if !target.is_group() {
                    if input.up || input.left {
                        cursor = (cursor + candidates.len() - 1) % candidates.len();
                        audio.play_sfx(Sfx::MenuMove);
                    }
                    if input.down || input.right {
                        cursor = (cursor + 1) % candidates.len();
                        audio.play_sfx(Sfx::MenuMove);
                    }
                    if let Some(p) = input.pointer {
                        for (k, &c) in candidates.iter().enumerate() {
                            if self.unit_rect(screen, c).contains(p) {
                                cursor = k;
                            }
                        }
                    }
                }
                self.menu = Menu::Target {
                    action,
                    target,
                    cursor,
                };
                // Info about the target.
                let focus = candidates[cursor];
                self.draw_target_info(painter, gfx, screen, game, focus, target.is_group());
                let clicked_unit = input.click
                    && input.pointer.is_some_and(|p| {
                        candidates
                            .iter()
                            .any(|&c| self.unit_rect(screen, c).contains(p))
                    });
                if input.confirm || clicked_unit {
                    audio.play_sfx(Sfx::MenuSelect);
                    let choice = if target.is_group() {
                        Choice::All
                    } else {
                        Choice::Unit(focus)
                    };
                    let action = match action {
                        PendingAction::Skill(s) => Action::Skill(s, choice),
                        PendingAction::Item(i) => Action::Item(i, choice),
                    };
                    self.perform(unit, action, game);
                } else if input.cancel {
                    audio.play_sfx(Sfx::MenuBack);
                    self.menu = match action {
                        PendingAction::Skill(SkillId::Attack) => Menu::Root,
                        PendingAction::Skill(_) => Menu::Skills,
                        PendingAction::Item(_) => Menu::Items,
                    };
                }
            }
        }
    }

    fn choose_target(
        &mut self,
        unit: usize,
        action: PendingAction,
        target: Target,
        game: &mut Game,
    ) {
        match target {
            Target::Myself => {
                let a = match action {
                    PendingAction::Skill(s) => Action::Skill(s, Choice::Unit(unit)),
                    PendingAction::Item(i) => Action::Item(i, Choice::Unit(unit)),
                };
                self.perform(unit, a, game);
            }
            _ => {
                // Start on the weakest-looking foe or the most hurt ally.
                let cands = self.target_candidates(unit, target);
                let cursor = if matches!(target, Target::Ally) {
                    cands
                        .iter()
                        .enumerate()
                        .min_by(|a, b| {
                            let ra = self.battle.units[*a.1].hp as f32
                                / self.battle.units[*a.1].max_hp as f32;
                            let rb = self.battle.units[*b.1].hp as f32
                                / self.battle.units[*b.1].max_hp as f32;
                            ra.total_cmp(&rb)
                        })
                        .map(|(k, _)| k)
                        .unwrap_or(0)
                } else {
                    0
                };
                self.menu = Menu::Target {
                    action,
                    target,
                    cursor,
                };
            }
        }
    }

    fn draw_target_info(
        &self,
        painter: &egui::Painter,
        _gfx: &Gfx,
        screen: Rect,
        game: &Game,
        unit: usize,
        group: bool,
    ) {
        let u = &self.battle.units[unit];
        let r = Rect::from_min_size(
            pos2(screen.left() + 16.0, screen.bottom() - 120.0),
            vec2(460.0, 104.0),
        );
        gfx::panel(painter, r);
        let title = if group {
            "All targets".to_string()
        } else {
            format!("{}  ·  Lv {}", u.name, u.level)
        };
        gfx::text(
            painter,
            r.left_top() + vec2(16.0, 10.0),
            Align2::LEFT_TOP,
            &title,
            gfx::heading_font(19.0),
            GOLD,
        );
        if group {
            return;
        }
        gfx::bar(
            painter,
            Rect::from_min_size(r.left_top() + vec2(16.0, 42.0), vec2(260.0, 12.0)),
            self.vis[unit].shown_hp.max(0) as f32 / u.max_hp as f32,
            gfx::HP_RED,
        );
        gfx::text(
            painter,
            r.left_top() + vec2(286.0, 48.0),
            Align2::LEFT_CENTER,
            &format!("{} / {}", self.vis[unit].shown_hp.max(0), u.max_hp),
            gfx::body_font(17.0),
            PARCHMENT,
        );
        if let Some(enemy) = u.enemy {
            let weak: Vec<&str> = Element::ALL
                .iter()
                .filter(|e| game.known_weak.contains(&(enemy, **e)))
                .map(|e| e.name())
                .collect();
            let text = if weak.is_empty() {
                "Weakness: ???".to_string()
            } else {
                format!("Weak to: {}", weak.join(", "))
            };
            gfx::text(
                painter,
                r.left_top() + vec2(16.0, 72.0),
                Align2::LEFT_TOP,
                &text,
                gfx::body_font(18.0),
                Color32::from_rgb(255, 210, 120),
            );
        }
        // What is ailing (or helping) the target.
        for (k, &(status, turns)) in u.statuses.iter().take(4).enumerate() {
            let y = r.top() - 30.0 - k as f32 * 26.0;
            let colour = if status.is_harmful() {
                Color32::from_rgb(210, 150, 255)
            } else {
                Color32::from_rgb(255, 225, 140)
            };
            gfx::text(
                painter,
                pos2(r.left() + 8.0, y),
                Align2::LEFT_CENTER,
                &format!("{} ({turns}) — {}", status.name(), status.describe()),
                gfx::body_font(17.0),
                colour,
            );
        }
    }

    fn draw_background(&self, painter: &egui::Painter, gfx: &Gfx, screen: Rect, time: f64) {
        let (top, bottom) = match self.biome {
            Biome::Town => (Color32::from_rgb(20, 24, 44), Color32::from_rgb(40, 34, 30)),
            Biome::Undercroft => (Color32::from_rgb(18, 14, 12), Color32::from_rgb(46, 36, 28)),
            Biome::Archive => (Color32::from_rgb(8, 16, 30), Color32::from_rgb(24, 40, 60)),
            Biome::Hollows => (Color32::from_rgb(16, 8, 26), Color32::from_rgb(20, 46, 50)),
            Biome::Forge => (Color32::from_rgb(28, 6, 4), Color32::from_rgb(70, 24, 10)),
            Biome::Pale => (
                Color32::from_rgb(30, 40, 60),
                Color32::from_rgb(170, 190, 215),
            ),
        };
        gfx::gradient(painter, screen, top, bottom);
        let s = screen.height() / 800.0;
        let ts = 64.0 * s;
        // Back wall.
        let walls = self.biome.walls();
        let wall_y = screen.top() + screen.height() * 0.12;
        let mut x = screen.left();
        let mut k = 0;
        while x < screen.right() {
            for row in 0..3 {
                let r = Rect::from_min_size(pos2(x, wall_y + row as f32 * ts), vec2(ts, ts));
                gfx.draw(
                    painter,
                    walls[(k + row) % 2],
                    r,
                    Color32::from_gray(70 - row as u8 * 10),
                );
            }
            x += ts;
            k += 1;
        }
        // Floor in rough perspective: rows get taller toward the viewer.
        let floors = self.biome.floors();
        let mut y = wall_y + 3.0 * ts;
        let mut row = 0;
        while y < screen.bottom() {
            let h = ts * (0.55 + row as f32 * 0.22);
            let w = h * 1.4;
            let shade = (55.0 + row as f32 * 22.0).min(170.0) as u8;
            let mut x = screen.left() - (row as f32 * 13.0) % w;
            let mut k = row;
            while x < screen.right() {
                gfx.draw(
                    painter,
                    floors[k % 3],
                    Rect::from_min_size(pos2(x, y), vec2(w, h)),
                    Color32::from_gray(shade),
                );
                x += w;
                k += 1;
            }
            y += h;
            row += 1;
        }
        // Warm light pools under both lines.
        for (cx, colour) in [
            (0.28, Color32::from_rgba_unmultiplied(255, 180, 110, 26)),
            (0.76, Color32::from_rgba_unmultiplied(255, 220, 160, 30)),
        ] {
            let c = pos2(
                screen.left() + screen.width() * cx,
                screen.top() + screen.height() * 0.55,
            );
            for k in 0..5 {
                painter.add(egui::Shape::ellipse_filled(
                    c,
                    vec2(260.0 - k as f32 * 40.0, 120.0 - k as f32 * 18.0) * s,
                    colour,
                ));
            }
        }
        // Drifting motes.
        for i in 0..40 {
            let f = i as f32 * 7.13;
            let x = screen.left() + ((f.sin() * 9999.0).fract().abs()) * screen.width();
            let y = screen.bottom()
                - ((time as f32 * (8.0 + (i % 5) as f32 * 4.0) + i as f32 * 53.0)
                    % screen.height());
            let c = match self.biome {
                Biome::Forge => Color32::from_rgba_unmultiplied(255, 140, 60, 120),
                Biome::Pale => Color32::from_rgba_unmultiplied(240, 245, 255, 150),
                Biome::Hollows => Color32::from_rgba_unmultiplied(120, 255, 210, 110),
                _ => Color32::from_rgba_unmultiplied(220, 200, 170, 70),
            };
            painter.circle_filled(pos2(x, y), 1.5 + (i % 3) as f32 * 0.7, c);
        }
        gfx::vignette(painter, screen, 0.65);
    }

    fn draw_boss_bar(&self, painter: &egui::Painter, screen: Rect) {
        let Some(i) = self.battle.units.iter().position(|u| u.boss && u.alive()) else {
            return;
        };
        if i >= self.vis.len() || !self.vis[i].revealed {
            return;
        }
        let u = &self.battle.units[i];
        let r = Rect::from_min_size(
            pos2(screen.left() + 24.0, screen.top() + 36.0),
            vec2(screen.width() * 0.5 - 90.0, 16.0),
        );
        gfx::text(
            painter,
            r.left_top() - vec2(0.0, 4.0),
            Align2::LEFT_BOTTOM,
            &u.name,
            gfx::heading_font(18.0),
            Color32::from_rgb(255, 150, 130),
        );
        gfx::bar(
            painter,
            r.translate(vec2(0.0, 8.0)),
            self.vis[i].shown_hp.max(0) as f32 / u.max_hp as f32,
            Color32::from_rgb(190, 40, 50),
        );
    }

    fn draw_turn_order(&self, painter: &egui::Painter, gfx: &Gfx, screen: Rect) {
        if self.phase == Phase::Victory {
            return;
        }
        let order = self.battle.forecast(8);
        let size = 40.0;
        let x0 = screen.right() - 20.0 - order.len() as f32 * (size + 6.0);
        let y = screen.top() + 16.0;
        gfx::text(
            painter,
            pos2(x0 - 10.0, y + size / 2.0),
            Align2::RIGHT_CENTER,
            "Next",
            gfx::heading_font(15.0),
            gfx::DIM,
        );
        for (k, &i) in order.iter().enumerate() {
            let Some(u) = self.battle.units.get(i) else {
                continue;
            };
            let r = Rect::from_min_size(pos2(x0 + k as f32 * (size + 6.0), y), vec2(size, size));
            let border = if u.side == Side::Heroes {
                Color32::from_rgb(110, 170, 255)
            } else {
                Color32::from_rgb(230, 90, 80)
            };
            painter.rect_filled(r, CornerRadius::same(5), Color32::from_black_alpha(180));
            gfx.draw(painter, u.sprite, r.shrink(3.0), Color32::WHITE);
            painter.rect_stroke(
                r,
                CornerRadius::same(5),
                Stroke::new(
                    if k == 0 { 2.5 } else { 1.2 },
                    if k == 0 { GOLD } else { border },
                ),
                egui::StrokeKind::Inside,
            );
        }
    }

    fn draw_party_panel(&self, painter: &egui::Painter, gfx: &Gfx, screen: Rect, time: f64) {
        let heroes: Vec<usize> = (0..self.battle.units.len())
            .filter(|&i| self.battle.units[i].side == Side::Heroes)
            .collect();
        let s = screen.height() / 800.0;
        let row_h = 50.0 * s;
        let w = 470.0 * s.max(1.0);
        let area = Rect::from_min_size(
            pos2(
                screen.right() - w - 16.0,
                screen.bottom() - row_h * heroes.len() as f32 - 30.0,
            ),
            vec2(w, row_h * heroes.len() as f32 + 18.0),
        );
        gfx::panel(painter, area);
        for (k, &i) in heroes.iter().enumerate() {
            let u = &self.battle.units[i];
            let v = &self.vis[i];
            let r = Rect::from_min_size(
                area.left_top() + vec2(10.0, 9.0 + k as f32 * row_h),
                vec2(w - 20.0, row_h - 4.0),
            );
            if Some(i) == self.active && !matches!(self.phase, Phase::Victory) {
                let pulse = 0.12 + 0.06 * (time * 4.0).sin() as f32;
                painter.rect_filled(r, CornerRadius::same(5), GOLD.gamma_multiply(pulse));
            }
            let pr = Rect::from_min_size(r.left_top(), vec2(r.height(), r.height()));
            gfx.draw(
                painter,
                u.sprite,
                pr,
                if v.dead {
                    Color32::from_gray(90)
                } else {
                    Color32::WHITE
                },
            );
            let name_col = if v.dead {
                Color32::from_rgb(200, 90, 90)
            } else {
                PARCHMENT
            };
            gfx::text(
                painter,
                pos2(pr.right() + 8.0, r.center().y),
                Align2::LEFT_CENTER,
                &u.name,
                gfx::heading_font(17.0),
                name_col,
            );
            let bx = r.left() + r.width() * 0.34;
            let bw = r.width() * 0.36;
            gfx::bar(
                painter,
                Rect::from_min_size(pos2(bx, r.top() + 8.0), vec2(bw, 12.0)),
                v.shown_hp.max(0) as f32 / u.max_hp as f32,
                gfx::HP_RED,
            );
            gfx::text(
                painter,
                pos2(bx + bw + 8.0, r.top() + 14.0),
                Align2::LEFT_CENTER,
                &format!("{}/{}", v.shown_hp.max(0), u.max_hp),
                gfx::body_font(17.0),
                PARCHMENT,
            );
            gfx::bar(
                painter,
                Rect::from_min_size(pos2(bx, r.top() + 26.0), vec2(bw, 8.0)),
                v.shown_mp.max(0) as f32 / u.max_mp.max(1) as f32,
                gfx::MP_BLUE,
            );
            gfx::text(
                painter,
                pos2(bx + bw + 8.0, r.top() + 31.0),
                Align2::LEFT_CENTER,
                &format!("{} MP", v.shown_mp.max(0)),
                gfx::body_font(14.0),
                Color32::from_rgb(150, 190, 255),
            );
        }
    }

    fn draw_rewards(&self, painter: &egui::Painter, _gfx: &Gfx, screen: Rect, game: &Game) {
        let Some(r) = &self.rewards else { return };
        let a = (r.age / 0.4).min(1.0);
        painter.rect_filled(
            screen,
            CornerRadius::ZERO,
            Color32::from_black_alpha((120.0 * a) as u8),
        );
        let lines = 1 + r.items.len().min(6) + r.level_ups.len() * 2;
        let rect =
            Rect::from_center_size(screen.center(), vec2(560.0, 150.0 + lines as f32 * 30.0));
        gfx::panel(painter, rect);
        gfx::text(
            painter,
            pos2(rect.center().x, rect.top() + 40.0),
            Align2::CENTER_CENTER,
            "Victory!",
            gfx::title_font(44.0),
            GOLD,
        );
        let mut y = rect.top() + 90.0;
        let mut line = |text: &str, colour: Color32| {
            gfx::text(
                painter,
                pos2(rect.center().x, y),
                Align2::CENTER_CENTER,
                text,
                gfx::body_font(22.0),
                colour,
            );
            y += 30.0;
        };
        line(
            &format!("{} experience   ·   {} gold", r.xp, r.gold),
            PARCHMENT,
        );
        for item in r.items.iter().take(6) {
            line(
                &format!("Found {}", item.def().name),
                Color32::from_rgb(170, 220, 255),
            );
        }
        for up in &r.level_ups {
            let name = up.hero.def().name;
            line(&format!("{name} reached level {}!", up.level), GOLD);
            let g = up.gains;
            let mut text = format!(
                "HP +{}  MP +{}  ATK +{}  DEF +{}  MAG +{}",
                g.hp, g.mp, g.atk, g.def, g.mag
            );
            if !up.new_skills.is_empty() {
                let names: Vec<&str> = up.new_skills.iter().map(|s| s.def().name).collect();
                text = format!("Learned {}!", names.join(", "));
            }
            line(&text, Color32::from_rgb(200, 240, 180));
        }
        let _ = game;
        if r.age > 0.6 {
            gfx::text(
                painter,
                pos2(rect.center().x, rect.bottom() - 24.0),
                Align2::CENTER_CENTER,
                "Press Enter",
                gfx::italic_font(18.0),
                gfx::DIM,
            );
        }
    }
}

fn item_target(item: ItemId) -> Target {
    use crate::data::items::{ItemKind, Use};
    match item.def().kind {
        ItemKind::Consumable(Use::Revive(_)) => Target::DeadAlly,
        ItemKind::Consumable(Use::HealAll(_)) => Target::AllAllies,
        ItemKind::Consumable(Use::DamageAll(..)) => Target::AllFoes,
        ItemKind::Consumable(Use::DamageOne(..)) => Target::Foe,
        ItemKind::Consumable(Use::Escape) => Target::Myself,
        _ => Target::Ally,
    }
}

pub fn status_icon(s: StatusKind) -> Sprite {
    match s {
        StatusKind::Poison => Sprite::StatusPoison,
        StatusKind::Burn => Sprite::StatusBurn,
        StatusKind::Frozen => Sprite::StatusFrozen,
        StatusKind::Stun => Sprite::StatusStun,
        StatusKind::Blind => Sprite::StatusBlind,
        StatusKind::Weak => Sprite::StatusWeak,
        StatusKind::Sunder => Sprite::StatusSunder,
        StatusKind::Haste => Sprite::StatusHaste,
        StatusKind::Slow => Sprite::StatusSlow,
        StatusKind::Regen => Sprite::StatusRegen,
        StatusKind::Shield => Sprite::StatusShield,
        StatusKind::Taunt => Sprite::StatusTaunt,
        StatusKind::Focus => Sprite::SkillArcane,
        StatusKind::Might => Sprite::SkillBuff,
    }
}

fn ease(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}
