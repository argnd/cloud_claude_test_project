//! The application: screens, the flow between exploring, talking and
//! fighting, and everything that ties the game systems to the player.

mod battle_view;
mod dialogue;
mod explore;
mod input;
mod menus;
mod title;
mod widgets;
#[cfg(test)]
mod bot;

use std::collections::VecDeque;

use eframe::egui::{self, Color32, CornerRadius};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

use crate::audio::{Audio, Sfx, Track};
use crate::battle::{Battle, BattleKind, Side};
use crate::data::enemies::{BattleId, EnemyId};
use crate::data::items::ItemId;
use crate::game::{Difficulty, Game, Pending};
use crate::gfx::{self, Gfx};
use crate::save::{self, Settings};
use crate::story::{self, Ending, Visual};
use crate::world::floor::{boss_flag, LAST_FLOOR};
use crate::world::{Dir, EntityKind, Loot, Move, Pickup, Place, Shop, Tile};
use battle_view::{BattleSignal, BattleView};
use dialogue::{Dialogue, DialogueOut};
use explore::ExploreView;
use input::Input;
use menus::{ConfirmYes, MenuOut, Overlay, PauseMenu, ShopView};
use title::{EndingView, GameOverOut, GameOverView, TitleOut, TitleView};
use widgets::ListState;

enum Screen {
    Title(Box<TitleView>),
    Playing,
    GameOver(GameOverView),
    Ending(EndingView),
}

/// Things to do once the current dialogue ends.
#[derive(Clone, Debug)]
enum After {
    Scene(String),
    Service(String),
    Waystone,
    FloorSelect,
    RemoveNpc(String),
}

pub struct App {
    gfx: Gfx,
    audio: Audio,
    settings: Settings,
    screen: Screen,
    game: Option<Game>,
    explore: ExploreView,
    dialogues: Vec<Dialogue>,
    after: VecDeque<After>,
    battle: Option<BattleView>,
    /// The phase-two interlude is playing over the battle.
    interlude: bool,
    overlay: Option<Overlay>,
    time: f64,
    rng: StdRng,
    fade: f32,
    flash: f32,
    shake: f32,
    autosave_due: bool,
    fullscreen_applied: Option<bool>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::with_context(&cc.egui_ctx)
    }

    pub fn with_context(ctx: &egui::Context) -> Self {
        gfx::setup_fonts(ctx);
        let settings = save::load_settings();
        let mut audio = Audio::new();
        audio.set_music_volume(settings.music);
        audio.set_sfx_volume(settings.sfx);
        audio.play_music(Track::Title);
        let seed = rand::rng().random();
        let jump = std::env::args().skip_while(|a| a != "--jump").nth(1).and_then(|n| n.parse::<u32>().ok());
        let mut app = Self {
            gfx: Gfx::load(ctx),
            audio,
            settings,
            screen: Screen::Title(Box::new(TitleView::new())),
            game: None,
            explore: ExploreView::new(),
            dialogues: Vec::new(),
            after: VecDeque::new(),
            battle: None,
            interlude: false,
            overlay: None,
            time: 0.0,
            rng: StdRng::seed_from_u64(seed),
            fade: 1.0,
            flash: 0.0,
            shake: 0.0,
            autosave_due: false,
            fullscreen_applied: None,
        };
        if let Some(floor) = jump {
            app.jump(floor);
            let args: Vec<String> = std::env::args().collect();
            if args.iter().any(|a| a == "--battle") {
                let group = crate::data::enemies::random_group(floor, &mut app.rng);
                app.start_battle(group, BattleKind::Normal, None, None);
            } else if args.iter().any(|a| a == "--boss") {
                if let Some((b, _)) = crate::world::floor::boss_of(floor) {
                    app.start_battle(b.formation(), BattleKind::Story(b), None, Some(b));
                }
            }
        }
        app
    }

    fn apply_settings(&mut self, ctx: &egui::Context) {
        self.audio.set_music_volume(self.settings.music);
        self.audio.set_sfx_volume(self.settings.sfx);
        if self.fullscreen_applied != Some(self.settings.fullscreen) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.settings.fullscreen));
            self.fullscreen_applied = Some(self.settings.fullscreen);
        }
        save::save_settings(&self.settings);
    }

    // ------------------------------------------------------------ flow

    fn new_game(&mut self, difficulty: Difficulty) {
        let seed = self.rng.random();
        let game = Game::new(difficulty, seed);
        self.explore = ExploreView::new();
        self.explore.reset(&game.world);
        self.game = Some(game);
        self.dialogues.clear();
        self.after.clear();
        self.battle = None;
        self.overlay = None;
        self.screen = Screen::Playing;
        self.fade = 1.0;
        self.audio.play_music(Track::Story);
        self.start_scene("intro");
        self.after.push_back(After::Scene("__arrive_town".into()));
    }

    /// `--jump N`: straight onto floor N with a suitable party.
    fn jump(&mut self, floor: u32) {
        let seed = self.rng.random();
        let game = Game::jump_start(floor, Difficulty::Normal, seed);
        self.explore = ExploreView::new();
        self.explore.reset(&game.world);
        self.explore.area_title = Some((game.world.biome.name().to_string(), format!("Floor {floor}"), 0.0));
        self.game = Some(game);
        self.screen = Screen::Playing;
        self.restore_music();
    }

    fn load(&mut self, slot: usize) {
        match save::read(slot) {
            Ok(game) => {
                self.explore = ExploreView::new();
                self.explore.reset(&game.world);
                self.game = Some(game);
                self.dialogues.clear();
                self.after.clear();
                self.battle = None;
                self.overlay = None;
                self.screen = Screen::Playing;
                self.fade = 1.0;
                self.restore_music();
                self.explore.notice("Game loaded");
            }
            Err(e) => {
                self.explore.notice(format!("Couldn't load: {e}"));
                self.audio.play_sfx(Sfx::Denied);
            }
        }
    }

    fn save(&mut self, slot: usize) {
        let Some(game) = &self.game else { return };
        match save::write(slot, game) {
            Ok(()) => {
                if slot != 0 {
                    self.explore.notice("Game saved");
                    self.audio.play_sfx(Sfx::Save);
                }
            }
            Err(e) => self.explore.notice(format!("Couldn't save: {e}")),
        }
    }

    fn to_title(&mut self) {
        self.game = None;
        self.dialogues.clear();
        self.after.clear();
        self.battle = None;
        self.overlay = None;
        self.screen = Screen::Title(Box::new(TitleView::new()));
        self.audio.play_music(Track::Title);
        self.fade = 1.0;
    }

    fn restore_music(&mut self) {
        let Some(game) = &self.game else { return };
        let track = if game.world.place == Place::Town && game.act() >= 3 { Track::TownSorrow } else { game.world.biome.music() };
        self.audio.play_music(track);
    }

    fn start_scene(&mut self, id: &str) {
        if story::script().has(id) {
            self.dialogues.push(Dialogue::new(id));
        } else if id == "npc_mouser" {
            self.explore.notice("Mouser purrs by Hesta's fire.");
        }
    }

    fn go_to_floor(&mut self, n: u32) {
        let Some(game) = &mut self.game else { return };
        game.enter_floor(n);
        self.explore.reset(&game.world);
        self.explore.area_title = Some((game.world.biome.name().to_string(), format!("Floor {n}"), 0.0));
        self.fade = 1.0;
        self.audio.play_sfx(Sfx::Stairs);
        let scene = format!("floor_{n}_enter");
        let seen = game.flag(&format!("seen_{scene}"));
        self.restore_music();
        if !seen {
            self.start_scene(&scene);
        }
        self.autosave_due = true;
    }

    fn go_to_town(&mut self, at_gate: bool) {
        let Some(game) = &mut self.game else { return };
        game.enter_town(at_gate);
        self.explore.reset(&game.world);
        self.explore.area_title = Some(("Hollowmere".into(), "A village at the top of the world".into(), 0.0));
        self.fade = 1.0;
        let act = game.act();
        let scene = format!("town_return_{act}");
        if act >= 2 && act > game.town_stage_seen {
            game.town_stage_seen = act;
            self.restore_music();
            self.start_scene(&scene);
        } else {
            self.restore_music();
        }
        self.autosave_due = true;
    }

    fn start_battle(&mut self, enemies: Vec<(EnemyId, u32)>, kind: BattleKind, entity: Option<usize>, story: Option<BattleId>) {
        let Some(game) = &mut self.game else { return };
        let mut battle = Battle::new(&game.party, &enemies, kind, self.rng.random());
        let (hp, dmg, _) = game.difficulty.multipliers();
        for u in battle.units.iter_mut().filter(|u| u.side == Side::Enemies) {
            u.max_hp = (u.max_hp as f32 * hp).max(1.0) as i32;
            u.hp = u.max_hp;
            u.stats.atk = (u.stats.atk as f32 * dmg) as i32;
            u.stats.mag = (u.stats.mag as f32 * dmg) as i32;
        }
        let view = BattleView::new(battle, story, entity, game.world.biome);
        self.audio.play_sfx(Sfx::Encounter);
        self.audio.play_music(view.music());
        self.battle = Some(view);
        self.flash = 0.8;
    }

    fn post_scene(battle: BattleId) -> Option<&'static str> {
        Some(match battle {
            BattleId::Vex => return None,
            BattleId::Gristlemaw => "gristlemaw_post",
            BattleId::Curator => "curator_post",
            BattleId::MotherOfSpores => "mother_post",
            BattleId::IronWarden => "iron_warden_post",
            BattleId::Ilsa => "ilsa_post",
            BattleId::Aurelian => "final_choice",
        })
    }

    fn battle_over(&mut self, signal: BattleSignal) {
        let Some(view) = self.battle.take() else { return };
        let Some(game) = &mut self.game else { return };
        match signal {
            BattleSignal::Victory => {
                if let Some(id) = view.story {
                    game.set_flag(&boss_flag(id));
                    if id == BattleId::Vex {
                        game.set_flag("vex_defeated");
                    }
                    game.world.entities.retain(|e| !matches!(&e.kind, EntityKind::Boss { battle, .. } if *battle == id));
                    if let Some(post) = Self::post_scene(id) {
                        self.after.push_front(After::Scene(post.to_string()));
                    }
                } else if let Some(i) = view.entity {
                    game.world.remove_entity(i);
                }
                self.explore.reset(&game.world);
                if let Some(d) = self.dialogues.last_mut() {
                    d.resume();
                }
                self.restore_music();
                // Bosses are milestones worth a save; ordinary fights are not.
                if view.story.is_some() {
                    self.autosave_due = true;
                }
            }
            BattleSignal::Escaped => {
                if let Some(i) = view.entity {
                    if let Some(EntityKind::Monster { sleep, awake, .. }) = game.world.entities.get_mut(i).map(|e| &mut e.kind) {
                        *sleep = 6;
                        *awake = false;
                    }
                }
                self.restore_music();
            }
            BattleSignal::Defeat => {
                game.stats.defeats += 1;
                self.dialogues.clear();
                self.after.clear();
                self.screen = Screen::GameOver(GameOverView::new());
            }
            BattleSignal::PhaseChange => {}
        }
    }

    /// A dialogue finished: bookkeeping, then whatever was queued.
    fn dialogue_finished(&mut self, d: Dialogue) {
        let Some(game) = &mut self.game else { return };
        for scene in &d.visited {
            game.set_flag(&format!("seen_{scene}"));
        }
        game.tidy_world();
        if d.visited.iter().any(|s| s == "mouser_found") {
            game.world.remove_npc("mouser");
        }
        if self.interlude {
            self.interlude = false;
            if let Some(b) = &mut self.battle {
                b.begin_second_phase();
            }
            return;
        }
        // Companion quests complete on their own.
        if let Some(q) = game.companion_quest_ready() {
            let scene = format!("quest_{}_done", q.script_id());
            if story::script().has(&scene) && !game.flag(&format!("seen_{scene}")) {
                self.after.push_front(After::Scene(scene));
            }
        }
        if self.dialogues.is_empty() && self.battle.is_none() {
            self.restore_music();
        }
    }

    fn run_after(&mut self) {
        while self.dialogues.is_empty() && self.overlay.is_none() && self.battle.is_none() {
            let Some(next) = self.after.pop_front() else { break };
            match next {
                After::Scene(s) if s == "__arrive_town" => {
                    self.explore.area_title = Some(("Hollowmere".into(), "A village at the top of the world".into(), 0.0));
                    self.restore_music();
                    self.autosave_due = true;
                }
                After::Scene(s) => self.start_scene(&s),
                After::Service(npc) => {
                    self.overlay = match npc.as_str() {
                        "bess" => Some(Overlay::Service { kind: Shop::Inn, list: ListState::default() }),
                        "oriel" => Some(Overlay::Service { kind: Shop::Temple, list: ListState::default() }),
                        "dagna" => Some(Overlay::Shop(ShopView::new(Shop::Smith))),
                        "fen" => Some(Overlay::Shop(ShopView::new(Shop::Apothecary))),
                        _ => None,
                    }
                }
                After::Waystone => self.overlay = Some(Overlay::Waystone(ListState::default())),
                After::FloorSelect => self.overlay = Some(Overlay::FloorSelect(ListState::default())),
                After::RemoveNpc(id) => {
                    if let Some(g) = &mut self.game {
                        g.world.remove_npc(&id);
                    }
                }
            }
        }
    }

    // ------------------------------------------------------------ exploring

    fn explore_input(&mut self, input: &mut Input) {
        let Some(game) = &mut self.game else { return };
        if input.menu || input.cancel {
            self.overlay = Some(Overlay::Pause(PauseMenu::new()));
            self.audio.play_sfx(Sfx::MenuSelect);
            return;
        }
        if input.map {
            self.explore.show_map = !self.explore.show_map;
        }
        if self.explore.moving() {
            return;
        }
        if input.confirm {
            let (dx, dy) = game.world.facing.delta();
            let target = (game.world.player.0 + dx, game.world.player.1 + dy);
            if let Some(i) = game.world.entity_at(target) {
                self.interact(i);
                return;
            }
            let here = game.world.tile(game.world.player);
            if here == Tile::Stairs {
                self.confirm_stairs();
                return;
            }
            if here == Tile::VaultGate {
                self.overlay = Some(Overlay::FloorSelect(ListState::default()));
                return;
            }
        }
        let Some(dir) = input.held else { return };
        self.explore.snapshot(&game.world);
        let result = game.world.try_move(dir);
        match result {
            Move::Blocked => {
                let (dx, dy) = dir.delta();
                if self.explore.bump.is_none() {
                    self.explore.bump = Some((dx as f32, dy as f32, 0.0));
                }
            }
            Move::Bump(i) => self.interact(i),
            Move::OpenedDoor => {
                self.audio.play_sfx(Sfx::DoorOpen);
                self.after_step();
            }
            Move::Moved | Move::Shop(_) => {
                self.explore.stepped(&self.game.as_ref().unwrap().world);
                self.after_step();
            }
            Move::Pickup(i) => {
                self.explore.stepped(&self.game.as_ref().unwrap().world);
                self.pickup(i);
                self.after_step();
            }
            Move::Stairs => {
                self.explore.stepped(&self.game.as_ref().unwrap().world);
                self.confirm_stairs();
            }
            Move::VaultGate => {
                self.explore.stepped(&self.game.as_ref().unwrap().world);
                let g = self.game.as_mut().unwrap();
                if !g.flag("seen_vaultgate_first") {
                    self.start_scene("vaultgate_first");
                    self.after.push_back(After::FloorSelect);
                } else {
                    self.overlay = Some(Overlay::FloorSelect(ListState::default()));
                }
            }
        }
    }

    fn confirm_stairs(&mut self) {
        let Some(game) = &self.game else { return };
        if let Some(n) = game.world.floor_number() {
            if n < LAST_FLOOR {
                self.overlay = Some(Overlay::Confirm {
                    text: format!("Descend to Floor {}?", n + 1),
                    yes: ConfirmYes::Descend(n + 1),
                    list: ListState::default(),
                });
            }
        }
    }

    /// Monsters move after each of the player's steps.
    fn after_step(&mut self) {
        let Some(game) = &mut self.game else { return };
        game.stats.steps += 1;
        if game.world.place == Place::Town {
            game.world.creatures_act(&mut self.rng);
            return;
        }
        if let Some(i) = game.world.creatures_act(&mut self.rng) {
            self.encounter(i, false);
        }
    }

    fn encounter(&mut self, i: usize, player_started: bool) {
        let Some(game) = &self.game else { return };
        let Some(EntityKind::Monster { group, .. }) = game.world.entities.get(i).map(|e| e.kind.clone()) else { return };
        let roll: f32 = self.rng.random();
        let kind = if player_started && roll < 0.5 {
            BattleKind::Preemptive
        } else if !player_started && roll < 0.2 {
            BattleKind::Ambush
        } else {
            BattleKind::Normal
        };
        self.start_battle(group, kind, Some(i), None);
    }

    fn interact(&mut self, i: usize) {
        let Some(game) = &mut self.game else { return };
        let Some(entity) = game.world.entities.get(i).cloned() else { return };
        match entity.kind {
            EntityKind::Npc { id, scene: None, .. } => {
                let scene = game.npc_scene(&id);
                self.start_scene(&scene);
                if matches!(id.as_str(), "bess" | "oriel" | "dagna" | "fen") {
                    self.after.push_back(After::Service(id));
                }
            }
            EntityKind::Npc { id, scene: Some(scene), .. } => {
                let vex_here = game.world.entities.iter().any(|e| matches!(e.kind, EntityKind::Boss { battle: BattleId::Vex, .. }));
                if id == "brannoc" && vex_here {
                    self.start_scene("vex_pre");
                } else {
                    self.start_scene(&scene);
                }
                if id == "mouser" {
                    self.after.push_back(After::RemoveNpc("mouser".into()));
                }
            }
            EntityKind::Monster { .. } => self.encounter(i, true),
            EntityKind::Boss { scene, .. } => self.start_scene(&scene),
            EntityKind::Chest { loot, opened: false } => {
                if let EntityKind::Chest { opened, .. } = &mut game.world.entities[i].kind {
                    *opened = true;
                }
                game.stats.chests_opened += 1;
                self.audio.play_sfx(Sfx::ChestOpen);
                match loot {
                    Loot::Gold(g) => {
                        game.gold += g;
                        game.stats.gold_earned += g;
                        self.explore.notice(format!("Found {g} gold"));
                        self.audio.play_sfx(Sfx::Coin);
                    }
                    Loot::Item(item, n) => {
                        game.inventory.add(item, n);
                        let name = item.def().name;
                        self.explore.notice(if n > 1 { format!("Found {name} ×{n}") } else { format!("Found {name}") });
                        self.audio.play_sfx(Sfx::ItemGet);
                    }
                }
            }
            EntityKind::Chest { .. } => self.explore.notice("Empty."),
            EntityKind::Waystone => {
                if !game.flag("seen_waystone_first") {
                    self.start_scene("waystone_first");
                    self.after.push_back(After::Waystone);
                } else {
                    self.overlay = Some(Overlay::Waystone(ListState::default()));
                }
                self.audio.play_sfx(Sfx::Waystone);
            }
            EntityKind::Pickup(_) => self.pickup(i),
            EntityKind::Decor { .. } => {}
        }
    }

    fn pickup(&mut self, i: usize) {
        let Some(game) = &mut self.game else { return };
        let Some(EntityKind::Pickup(p)) = game.world.entities.get(i).map(|e| e.kind.clone()) else { return };
        game.world.remove_entity(i);
        match p {
            Pickup::Shard(n) => {
                game.set_flag(&format!("shard_{n}"));
                game.update_shard_flag();
                self.audio.play_sfx(Sfx::Shard);
                let count = game.shards().len();
                self.explore.notice(format!("Memory Shard ({count}/12)"));
                self.start_scene(&format!("shard_{n}"));
            }
            Pickup::Journal(n) => {
                game.set_flag(&format!("journal_{n}"));
                self.audio.play_sfx(Sfx::ItemGet);
                self.start_scene(&format!("journal_{n}"));
            }
            Pickup::Item { item, flag } => {
                game.set_flag(&flag);
                game.inventory.add(item, 1);
                self.audio.play_sfx(Sfx::ItemGet);
                self.explore.notice(format!("Found {}", item.def().name));
                if let Some(q) = game.companion_quest_ready() {
                    let scene = format!("quest_{}_done", q.script_id());
                    if story::script().has(&scene) {
                        self.start_scene(&scene);
                    }
                }
            }
            Pickup::Scene { scene, flag, .. } => {
                game.set_flag(&flag);
                self.start_scene(&scene);
            }
        }
    }

    fn handle_pending(&mut self) {
        let Some(game) = &mut self.game else { return };
        for p in std::mem::take(&mut game.pending) {
            match p {
                Pending::Music(t) => self.audio.play_music(t),
                Pending::Sfx(s) => self.audio.play_sfx(s),
                Pending::Visual(Visual::Shake) => self.shake = 12.0,
                Pending::Visual(Visual::Flash) => self.flash = 1.0,
                Pending::Visual(Visual::Fade) => self.fade = 1.0,
                Pending::Notice(n) => self.explore.notice(n),
                Pending::Joined(h) => {
                    self.explore.notice(format!("{} joined the party!", h.def().name));
                    self.audio.play_sfx(Sfx::LevelUp);
                }
            }
        }
    }

    fn menu_out(&mut self, out: MenuOut, ctx: &egui::Context) {
        match out {
            MenuOut::None => {}
            MenuOut::Close => self.overlay = None,
            MenuOut::Descend(n) => {
                self.overlay = None;
                self.go_to_floor(n);
            }
            MenuOut::ToTown => {
                self.overlay = None;
                self.audio.play_sfx(Sfx::Waystone);
                self.go_to_town(true);
            }
            MenuOut::Rest => {
                self.overlay = None;
                let Some(game) = &mut self.game else { return };
                game.heal_all();
                self.fade = 1.0;
                self.audio.play_sfx(Sfx::Heal);
                if game.world.place == Place::Town {
                    self.start_scene("inn_rest");
                    self.explore.notice("The party is fully rested.");
                } else {
                    game.world.rested = true;
                    self.explore.notice("The party rests by the waystone.");
                    let camp = format!("camp_{}", game.act());
                    if !game.flag(&format!("seen_{camp}")) {
                        self.start_scene(&camp);
                    }
                }
                self.autosave_due = true;
            }
            MenuOut::Pray => {
                self.overlay = None;
                if let Some(game) = &mut self.game {
                    game.heal_all();
                }
                self.flash = 0.6;
                self.audio.play_sfx(Sfx::Light);
                self.explore.notice("Warmth washes over the party.");
            }
            MenuOut::Save(slot) => {
                if slot == usize::MAX {
                    self.overlay = Some(Overlay::Saves { saving: true, list: ListState { cursor: 1, scroll: 0 } });
                } else {
                    self.save(slot);
                    self.overlay = None;
                }
            }
            MenuOut::Load(slot) => {
                if slot == usize::MAX {
                    self.overlay = Some(Overlay::Saves { saving: false, list: ListState::default() });
                } else {
                    self.load(slot);
                }
            }
            MenuOut::Title => self.to_title(),
            MenuOut::Replay(scene) => {
                self.overlay = None;
                self.start_scene(&scene);
            }
            MenuOut::OpenShop(shop) => self.overlay = Some(Overlay::Shop(ShopView::new(shop))),
            MenuOut::SettingsChanged => self.apply_settings(ctx),
        }
    }

    // ------------------------------------------------------------ frame

    fn play_frame(&mut self, ctx: &egui::Context, painter: &egui::Painter, screen: egui::Rect, input: &mut Input) {
        let dt = input.dt;
        if let Some(game) = &mut self.game {
            game.playtime += dt as f64;
        }
        self.handle_pending();

        // Battle (possibly with the phase interlude on top).
        if self.battle.is_some() {
            let mut signal = None;
            {
                let game = self.game.as_mut().unwrap();
                let view = self.battle.as_mut().unwrap();
                if self.dialogues.is_empty() || !self.interlude {
                    if self.dialogues.last().is_none_or(|d| d.waiting) {
                        signal = view.update(input, game, &mut self.audio, self.settings.battle_speed);
                    }
                }
                let mut battle_input = if self.interlude { Input { dt, ..Default::default() } } else { input.clone() };
                view.draw(painter, &self.gfx, screen, &mut battle_input, game, &mut self.audio, self.time);
            }
            match signal {
                Some(BattleSignal::PhaseChange) => {
                    self.interlude = true;
                    self.start_scene("aurelian_phase2");
                    if self.dialogues.last().is_none_or(|d| d.runner.scene != "aurelian_phase2") {
                        // No interlude written: go straight on.
                        self.interlude = false;
                        self.battle.as_mut().unwrap().begin_second_phase();
                    }
                }
                Some(s) => self.battle_over(s),
                None => {}
            }
            if self.interlude {
                self.dialogue_frame(painter, screen, input);
            }
            return;
        }

        // Exploring.
        self.explore.update(dt);
        {
            let game = self.game.as_ref().unwrap();
            let shake = if self.shake > 0.0 { egui::vec2((self.time * 80.0).sin() as f32 * self.shake, 0.0) } else { egui::vec2(0.0, 0.0) };
            self.explore.draw(painter, &self.gfx, screen.translate(shake), game, self.time, dt);
            if self.dialogues.is_empty() {
                self.explore.draw_hud(painter, &self.gfx, screen, game, self.time);
            }
        }
        if !self.dialogues.is_empty() {
            self.dialogue_frame(painter, screen, input);
        } else if let Some(mut overlay) = self.overlay.take() {
            let game = self.game.as_mut().unwrap();
            let out = overlay.show(painter, &self.gfx, screen, input, game, &mut self.audio, &mut self.settings, self.time);
            self.overlay = Some(overlay);
            self.menu_out(out, ctx);
        } else {
            self.run_after();
            if self.dialogues.is_empty() && self.overlay.is_none() && self.battle.is_none() {
                if self.autosave_due {
                    self.autosave_due = false;
                    self.save(0);
                }
                self.explore_input(input);
            }
        }
    }

    fn dialogue_frame(&mut self, painter: &egui::Painter, screen: egui::Rect, input: &mut Input) {
        let Some(mut d) = self.dialogues.pop() else { return };
        let game = self.game.as_mut().unwrap();
        let out = d.show(painter, &self.gfx, screen, input, game, &mut self.audio, self.settings.text_speed, self.time);
        match out {
            DialogueOut::Nothing => self.dialogues.push(d),
            DialogueOut::Battle(id) => {
                self.dialogues.push(d);
                self.start_battle(id.formation(), BattleKind::Story(id), None, Some(id));
            }
            DialogueOut::Ending(e) => self.ending(e),
            DialogueOut::Finished => self.dialogue_finished(d),
        }
        self.handle_pending();
    }

    fn ending(&mut self, e: Ending) {
        let Some(game) = &mut self.game else { return };
        game.set_flag(match e {
            Ending::Oath => "ending_oath_seen",
            Ending::Dark => "ending_dark_seen",
            Ending::Dawn => "ending_dawn_seen",
        });
        let _ = save::write(0, game);
        self.dialogues.clear();
        self.after.clear();
        self.audio.play_music(if e == Ending::Dawn { Track::Dawn } else { Track::Ending });
        self.screen = Screen::Ending(EndingView::new(e, game));
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.frame(ui);
    }
}

impl App {
    fn frame(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        if self.fullscreen_applied.is_none() {
            self.apply_settings(&ctx);
        }
        let mut input = Input::read(&ctx);
        self.time += input.dt as f64;
        self.audio.update(input.dt);
        let screen = ctx.content_rect();
        let painter = ui.painter().clone().with_clip_rect(screen);

        match &mut self.screen {
            Screen::Title(view) => {
                let out = view.show(&painter, &self.gfx, screen, &mut input, &mut self.audio, &mut self.settings, self.time);
                match out {
                    TitleOut::NewGame(d) => self.new_game(d),
                    TitleOut::Chapter(floor) => self.jump(floor),
                    TitleOut::Load(slot) => self.load(slot),
                    TitleOut::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                    TitleOut::SettingsChanged => self.apply_settings(&ctx),
                    TitleOut::None => {}
                }
            }
            Screen::Playing => self.play_frame(&ctx, &painter, screen, &mut input),
            Screen::GameOver(view) => match view.show(&painter, &self.gfx, screen, &mut input, &mut self.audio, self.time) {
                GameOverOut::Retry => match save::any_save() {
                    Some(slot) => self.load(slot),
                    None => self.to_title(),
                },
                GameOverOut::Title => self.to_title(),
                GameOverOut::None => {}
            },
            Screen::Ending(view) => {
                if view.show(&painter, screen, &mut input) {
                    self.to_title();
                }
            }
        }

        // Screen-wide effects.
        let dt = input.dt;
        self.shake = (self.shake - dt * 25.0).max(0.0);
        if self.flash > 0.0 {
            painter.rect_filled(screen, CornerRadius::ZERO, Color32::from_white_alpha((self.flash * 180.0) as u8));
            self.flash = (self.flash - dt * 2.0).max(0.0);
        }
        if self.fade > 0.0 {
            painter.rect_filled(screen, CornerRadius::ZERO, Color32::from_black_alpha((self.fade.min(1.0) * 255.0) as u8));
            self.fade = (self.fade - dt * 1.8).max(0.0);
        }
        let _ = Dir::Up;
        let _ = ItemId::Tonic;
        ctx.request_repaint();
    }
}
