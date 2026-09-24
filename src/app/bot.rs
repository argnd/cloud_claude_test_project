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
    let reachable = |p: (i32, i32)| step_towards(w, p).is_some();
    let find = |f: &dyn Fn(&EntityKind) -> bool| {
        w.entities
            .iter()
            .filter(|e| f(&e.kind) && reachable(e.pos))
            .min_by_key(|e| (e.pos.0 - w.player.0).abs() + (e.pos.1 - w.player.1).abs())
            .map(|e| e.pos)
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
    let mut last_floor = 0;
    let mut stuck = 0u32;
    let mut last_pos = (0, 0, 0u32);
    let mut losses = 0;
    let mut report = String::new();
    let mut last_act = 0;
    let limit = 3_000_000u64;
    let mut cooldown = 0;
    while d.frames < limit {
        cooldown = (cooldown as i32 - 1).max(0);
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
                report += &format!("  defeat #{losses} on floor {last_floor}\n");
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
            if !dl.waiting || d.app.interlude {
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
            report += &format!(
                "floor {floor:>2}: {:>5.0} min game time, levels {:?}, gold {}\n",
                game.playtime / 60.0,
                game.party.iter().map(|h| h.level).collect::<Vec<_>>(),
                game.gold
            );
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
