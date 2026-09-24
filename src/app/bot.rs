//! An automated player that drives the real UI with key presses, from the
//! title screen to an ending. It fights everything, gathers every shard and
//! lore page, rests at waystones, buys and wears the best gear it can, and
//! reports how long (in game time) the journey took.
//!
//! Run with: `cargo test --release playthrough -- --ignored --nocapture`

use std::collections::VecDeque;

use eframe::egui::{self, Key, Pos2, Rect, vec2};

use super::menus::Overlay;
use super::{App, Screen};
use crate::data::items::{EquipSlot, ItemId};
use crate::game::Game;
use crate::world::{Dir, EntityKind, Place, Tile, World};

struct Driver {
    ctx: egui::Context,
    app: App,
    time: f64,
    dt: f32,
    frames: u64,
    held: Option<Key>,
}

impl Driver {
    fn new(dt: f32) -> Self {
        let ctx = egui::Context::default();
        let app = App::with_context(&ctx);
        Self { ctx, app, time: 0.0, dt, frames: 0, held: None }
    }

    /// One frame; `press` is pressed this frame and released the next.
    fn frame(&mut self, press: Option<Key>) {
        let mut raw = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1280.0, 800.0))),
            time: Some(self.time),
            predicted_dt: self.dt,
            ..Default::default()
        };
        if let Some(k) = self.held.take() {
            raw.events.push(egui::Event::Key { key: k, physical_key: None, pressed: false, repeat: false, modifiers: Default::default() });
        }
        if let Some(k) = press {
            raw.events.push(egui::Event::Key { key: k, physical_key: None, pressed: true, repeat: false, modifiers: Default::default() });
            self.held = Some(k);
        }
        self.time += self.dt as f64;
        self.frames += 1;
        let app = &mut self.app;
        let mut out = self.ctx.run_ui(raw, |ui| app.frame(ui));
        out.textures_delta.clear();
    }
}

fn key_for(dir: Dir) -> Key {
    match dir {
        Dir::Up => Key::ArrowUp,
        Dir::Down => Key::ArrowDown,
        Dir::Left => Key::ArrowLeft,
        Dir::Right => Key::ArrowRight,
    }
}

/// First step on a shortest path from the player to (next to) `goal`.
fn step_towards(world: &World, goal: (i32, i32)) -> Option<Dir> {
    let start = world.player;
    if start == goal {
        return None;
    }
    let mut prev = vec![None; (world.w * world.h) as usize];
    let mut seen = vec![false; (world.w * world.h) as usize];
    let mut queue = VecDeque::new();
    seen[world.idx(start)] = true;
    queue.push_back(start);
    while let Some(p) = queue.pop_front() {
        for dir in Dir::ALL {
            let (dx, dy) = dir.delta();
            let q = (p.0 + dx, p.1 + dy);
            if !world.in_bounds(q) || seen[world.idx(q)] {
                continue;
            }
            let t = world.tile(q);
            let passable = t.walkable() || t == Tile::Door;
            let blocked = q != goal && world.entities.iter().any(|e| e.pos == q && e.blocks());
            if !passable && q != goal || blocked {
                continue;
            }
            seen[world.idx(q)] = true;
            prev[world.idx(q)] = Some((p, dir));
            if q == goal {
                // Walk back to the first step.
                let mut cur = q;
                let mut first = dir;
                while let Some((from, d)) = prev[world.idx(cur)] {
                    first = d;
                    if from == start {
                        break;
                    }
                    cur = from;
                }
                return Some(first);
            }
            queue.push_back(q);
        }
    }
    None
}

/// Walking distance from the player to every tile; tiles holding a blocking
/// entity get a distance (you can walk up to them) but aren't walked through.
fn path_distances(world: &World) -> Vec<i32> {
    let mut dist = vec![i32::MAX; (world.w * world.h) as usize];
    let mut queue = VecDeque::new();
    dist[world.idx(world.player)] = 0;
    queue.push_back(world.player);
    while let Some(p) = queue.pop_front() {
        let d = dist[world.idx(p)];
        for dir in Dir::ALL {
            let (dx, dy) = dir.delta();
            let q = (p.0 + dx, p.1 + dy);
            if !world.in_bounds(q) || dist[world.idx(q)] != i32::MAX {
                continue;
            }
            let t = world.tile(q);
            if !(t.walkable() || t == Tile::Door) {
                continue;
            }
            dist[world.idx(q)] = d + 1;
            if !world.entities.iter().any(|e| e.pos == q && e.blocks()) {
                queue.push_back(q);
            }
        }
    }
    dist
}

fn equip_best(game: &mut Game) {
    for h in 0..game.party.len() {
        for slot in [EquipSlot::Weapon, EquipSlot::Armor, EquipSlot::Accessory] {
            let score = |i: ItemId| {
                let s = i.def().stats;
                s.atk + s.mag + s.def + s.res + s.hp / 5 + s.spd * 2 + if i.def().special != crate::data::items::Special::None { 8 } else { 0 }
            };
            let current = game.party[h].slot(slot).map(score).unwrap_or(-1);
            let best = game
                .inventory
                .list()
                .into_iter()
                .map(|(i, _)| i)
                .filter(|&i| i.def().slot() == Some(slot) && game.party[h].can_equip(i))
                .max_by_key(|&i| score(i));
            if let Some(b) = best {
                if score(b) > current {
                    let _ = game.equip(h, b);
                }
            }
        }
    }
}

/// Buys this act's gear (what a sensible player does on a town trip).
fn shop(game: &mut Game) {
    let stock = game.stock(crate::world::Shop::Smith);
    for h in 0..game.party.len() {
        for slot in [EquipSlot::Weapon, EquipSlot::Armor] {
            let best = stock.iter().copied().filter(|&i| i.def().slot() == Some(slot) && game.party[h].can_equip(i)).max_by_key(|i| i.def().price);
            if let Some(b) = best {
                let have = game.party[h].slot(slot).map(|i| i.def().price).unwrap_or(0);
                if b.def().price > have && game.gold >= b.def().price {
                    let _ = game.buy(b);
                }
            }
        }
    }
    while game.gold > 400 && game.inventory.count(ItemId::Tonic) < 8 {
        let _ = game.buy(ItemId::Tonic);
    }
    equip_best(game);
}

/// What the bot walks to on the current map.
fn target(game: &Game) -> Option<(i32, i32)> {
    let w = &game.world;
    if w.place == Place::Town {
        return (0..w.h).flat_map(|y| (0..w.w).map(move |x| (x, y))).find(|&p| w.tile(p) == Tile::VaultGate);
    }
    let hp: f32 = game.party.iter().map(|h| h.hp.max(0) as f32 / h.max_hp() as f32).sum::<f32>() / game.party.len() as f32;
    let dist = path_distances(w);
    let find = |f: &dyn Fn(&EntityKind) -> bool| {
        w.entities
            .iter()
            .enumerate()
            .filter(|(_, e)| f(&e.kind) && dist[w.idx(e.pos)] != i32::MAX)
            .min_by_key(|(i, e)| (dist[w.idx(e.pos)], *i))
            .map(|(_, e)| e.pos)
    };
    if hp < 0.5 {
        if let Some(p) = find(&|k| matches!(k, EntityKind::Waystone)) {
            return Some(p);
        }
    }
    find(&|k| matches!(k, EntityKind::Npc { scene: Some(_), id, .. } if id != "brannoc"))
        .or_else(|| find(&|k| matches!(k, EntityKind::Pickup(_))))
        .or_else(|| find(&|k| matches!(k, EntityKind::Monster { .. })))
        .or_else(|| find(&|k| matches!(k, EntityKind::Chest { opened: false, .. })))
        .or_else(|| find(&|k| matches!(k, EntityKind::Npc { scene: Some(_), .. })))
        .or_else(|| find(&|k| matches!(k, EntityKind::Boss { .. })))
        .or_else(|| if hp < 0.9 { find(&|k| matches!(k, EntityKind::Waystone)) } else { None })
        .or_else(|| (0..w.h).flat_map(|y| (0..w.w).map(move |x| (x, y))).find(|&p| w.tile(p) == Tile::Stairs))
}

#[test]
#[ignore = "plays the whole game; slow"]
fn playthrough() {
    let dir = std::env::temp_dir().join(format!("emberdeep_bot_{}", std::process::id()));
    unsafe { std::env::set_var("EMBERDEEP_SAVE_DIR", &dir) };
    let _ = std::fs::remove_dir_all(&dir);

    let mut d = Driver::new(0.05);
    d.app.settings.battle_speed = 1.0;
    d.app.settings.text_speed = 55.0;
    if let Some(floor) = std::env::var("BOT_START").ok().and_then(|v| v.parse().ok()) {
        d.app.jump(floor);
    }
    let mut last_floor = 0;
    let mut stuck = 0u32;
    let mut last_pos = (0, 0, 0u32);
    let mut losses = 0;
    let mut report = String::new();
    let mut last_act = 0;
    let limit = 3_000_000u64;
    let mut cooldown = 0;
    let mut trace = 0;
    let mut last_battle = String::new();
    while d.frames < limit {
        cooldown = (cooldown as i32 - 1).max(0);
        if d.frames % 50_000 == 0 && d.frames > 0 {
            trace = 12;
            if let Some(g) = d.app.game.as_ref() {
                eprintln!(
                    "[{}] moving {} move_t {} cooldown {cooldown} stuck {stuck} after {:?} floor {:?} at {:?} target {:?} dialogue {:?} battle {} overlay {} hp {:?}",
                    d.frames,
                    d.app.explore.moving(),
                    d.app.explore.move_t,
                    d.app.after,
                    g.world.floor_number(),
                    g.world.player,
                    target(g),
                    d.app.dialogues.last().map(|dl| dl.runner.scene.clone()),
                    d.app.battle.is_some(),
                    d.app.overlay.is_some(),
                    g.party.iter().map(|h| h.hp).collect::<Vec<_>>()
                );
            }
        }
        match &d.app.screen {
            Screen::Title(_) => {
                d.frame(if d.frames % 20 == 10 { Some(Key::Enter) } else { None });
                continue;
            }
            Screen::GameOver(_) => {
                losses += 1;
                for _ in 0..30 {
                    d.frame(None);
                }
                let line = format!("  defeat #{losses} on floor {last_floor}: {last_battle}\n");
                eprint!("{line}");
                report += &line;
                assert!(losses < 25, "too many defeats\n{report}");
                for _ in 0..40 {
                    d.frame(None);
                }
                d.frame(Some(Key::Enter));
                continue;
            }
            Screen::Ending(view) => {
                let game = d.app.game.as_ref().unwrap();
                report += &format!(
                    "ENDING {:?} after {:.1} h of game time, {} frames, party levels {:?}, shards {}/12\n",
                    view.ending,
                    game.playtime / 3600.0,
                    d.frames,
                    game.party.iter().map(|h| h.level).collect::<Vec<_>>(),
                    game.shards().len()
                );
                println!("{report}");
                return;
            }
            Screen::Playing => {}
        }
        // Dialogue: read, pick the last (most hopeful) choice.
        if let Some(dl) = d.app.dialogues.last() {
            if !dl.waiting {
                if dl.is_choice() {
                    d.frame(Some(Key::ArrowUp));
                    d.frame(None);
                    d.frame(Some(Key::Enter));
                } else {
                    d.frame(if d.frames % 3 == 0 { Some(Key::Enter) } else { None });
                }
                continue;
            }
        }
        if let Some(b) = &mut d.app.battle {
            if !b.auto {
                last_battle = b.battle.units.iter().filter(|u| u.enemy.is_some()).map(|u| format!("{} L{}", u.name, u.level)).collect::<Vec<_>>().join(", ");
                if let Some(g) = d.app.game.as_ref() {
                    last_battle += &format!(" | party {:?} items {:?}", g.party.iter().map(|h| (h.level, h.max_hp())).collect::<Vec<_>>(), g.inventory.list().iter().filter(|(i, _)| i.def().usable_in_battle()).map(|(i, c)| format!("{}x{c}", i.def().name)).collect::<Vec<_>>());
                }
            }
            b.auto = true;
            d.frame(if d.frames % 15 == 0 { Some(Key::Enter) } else { None });
            continue;
        }
        if let Some(o) = &d.app.overlay {
            let key = match o {
                Overlay::Confirm { .. } | Overlay::FloorSelect(_) => Key::Enter,
                Overlay::Waystone(list) => {
                    let hurt = d.app.game.as_ref().is_some_and(|g| g.party.iter().any(|h| h.hp < h.max_hp()));
                    if hurt && list.cursor == 0 { Key::Enter } else { Key::Escape }
                }
                _ => Key::Escape,
            };
            d.frame(Some(key));
            d.frame(None);
            continue;
        }
        // Exploring.
        let Some(game) = d.app.game.as_mut() else {
            d.frame(None);
            continue;
        };
        let floor = game.world.floor_number().unwrap_or(0);
        if floor != last_floor && floor > 0 {
            let act = (floor - 1) / 4 + 1;
            if act != last_act {
                last_act = act;
                shop(game);
            }
            equip_best(game);
            let line = format!(
                "floor {floor:>2}: {:>5.0} min game time, levels {:?}, gold {}, shards {}\n",
                game.playtime / 60.0,
                game.party.iter().map(|h| h.level).collect::<Vec<_>>(),
                game.gold,
                game.shards().len()
            );
            eprint!("{line}");
            report += &line;
            last_floor = floor;
        }
        equip_best(game);
        if d.app.explore.moving() || cooldown > 0 {
            d.frame(None);
            continue;
        }
        let pos = (game.world.player.0, game.world.player.1, floor);
        if pos == last_pos {
            stuck += 1;
        } else {
            stuck = 0;
            last_pos = pos;
        }
        assert!(stuck < 4000, "bot stuck on floor {floor} at {:?}\n{report}", game.world.player);
        let Some(goal) = target(game) else {
            d.frame(None);
            continue;
        };
        if trace > 0 {
            trace -= 1;
            let near: Vec<String> = game.world.entities.iter().filter(|e| (e.pos.0 - game.world.player.0).abs() + (e.pos.1 - game.world.player.1).abs() <= 6).map(|e| format!("{:?}@{:?}", std::mem::discriminant(&e.kind), e.pos)).collect();
            eprintln!("  at {:?} facing {:?} goal {goal:?} step {:?} tile-here {:?} near {:?}", game.world.player, game.world.facing, step_towards(&game.world, goal), game.world.tile(game.world.player), near);
        }
        match step_towards(&game.world, goal) {
            Some(dir) => {
                d.frame(Some(key_for(dir)));
                cooldown = 1;
            }
            None => {
                // Standing on the goal (stairs): interact.
                d.frame(Some(Key::Enter));
                d.frame(None);
            }
        }
    }
    panic!("no ending within {limit} frames\n{report}");
}
