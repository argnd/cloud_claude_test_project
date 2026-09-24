//! Drawing the map: tiles lit by the lantern and torches, the party walking
//! in a line, monsters, particles, the minimap and the HUD.

use std::collections::VecDeque;

use eframe::egui::{self, Align2, Color32, CornerRadius, Pos2, Rect, Stroke, Vec2, pos2, vec2};

use crate::game::Game;
use crate::gfx::sprites::Sprite;
use crate::gfx::{self, GOLD, Gfx, PARCHMENT};
use crate::world::{Biome, EntityKind, Pickup, Place, Shop, Tile, World};

const STEP_TIME: f32 = 0.13;

#[derive(Clone)]
struct Particle {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    max_life: f32,
    size: f32,
    colour: Color32,
}

pub struct ExploreView {
    pub move_t: f32,
    from: (f32, f32),
    /// Previous tile of every entity, for tweening monster steps.
    entity_from: Vec<(i32, i32)>,
    entity_t: f32,
    pub trail: VecDeque<(i32, i32)>,
    trail_from: Vec<(i32, i32)>,
    particles: Vec<Particle>,
    pub area_title: Option<(String, String, f32)>,
    pub notices: Vec<(String, f32)>,
    pub show_map: bool,
    camera: Option<Pos2>,
    pub bump: Option<(f32, f32, f32)>,
}

impl ExploreView {
    pub fn new() -> Self {
        Self {
            move_t: 1.0,
            from: (0.0, 0.0),
            entity_from: Vec::new(),
            entity_t: 1.0,
            trail: VecDeque::new(),
            trail_from: Vec::new(),
            particles: Vec::new(),
            area_title: None,
            notices: Vec::new(),
            show_map: true,
            camera: None,
            bump: None,
        }
    }

    pub fn moving(&self) -> bool {
        self.move_t < 1.0
    }

    /// Call before the world changes positions (player and creatures).
    pub fn snapshot(&mut self, world: &World) {
        self.from = (world.player.0 as f32, world.player.1 as f32);
        self.entity_from = world.entities.iter().map(|e| e.pos).collect();
        self.trail_from = self.trail.iter().copied().collect();
    }

    /// Call after the player actually moved.
    pub fn stepped(&mut self, world: &World) {
        self.move_t = 0.0;
        self.entity_t = 0.0;
        let old = (self.from.0 as i32, self.from.1 as i32);
        self.trail.push_front(old);
        self.trail.truncate(4);
        let _ = world;
    }

    /// New map: no tweening across it.
    pub fn reset(&mut self, world: &World) {
        self.move_t = 1.0;
        self.entity_t = 1.0;
        self.from = (world.player.0 as f32, world.player.1 as f32);
        self.entity_from = world.entities.iter().map(|e| e.pos).collect();
        self.trail = VecDeque::from(vec![world.player; 3]);
        self.trail_from = self.trail.iter().copied().collect();
        self.camera = None;
        self.particles.clear();
    }

    pub fn notice(&mut self, text: impl Into<String>) {
        self.notices.push((text.into(), 0.0));
        if self.notices.len() > 5 {
            self.notices.remove(0);
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.move_t = (self.move_t + dt / STEP_TIME).min(1.0);
        self.entity_t = (self.entity_t + dt / STEP_TIME).min(1.0);
        for n in &mut self.notices {
            n.1 += dt;
        }
        self.notices.retain(|n| n.1 < 3.5);
        if let Some(t) = &mut self.area_title {
            t.2 += dt;
            if t.2 > 4.5 {
                self.area_title = None;
            }
        }
        if let Some(b) = &mut self.bump {
            b.2 += dt;
            if b.2 > 0.15 {
                self.bump = None;
            }
        }
    }

    fn player_pos(&self, world: &World) -> (f32, f32) {
        let t = ease(self.move_t);
        let to = (world.player.0 as f32, world.player.1 as f32);
        let mut p = (self.from.0 + (to.0 - self.from.0) * t, self.from.1 + (to.1 - self.from.1) * t);
        if let Some((dx, dy, bt)) = self.bump {
            let k = (bt / 0.15 * std::f32::consts::PI).sin() * 0.18;
            p.0 += dx * k;
            p.1 += dy * k;
        }
        p
    }

    fn entity_pos(&self, world: &World, i: usize) -> (f32, f32) {
        let to = world.entities[i].pos;
        let from = self.entity_from.get(i).copied().unwrap_or(to);
        let far = (from.0 - to.0).abs() + (from.1 - to.1).abs() > 1;
        let t = if far { 1.0 } else { ease(self.entity_t) };
        (from.0 as f32 + (to.0 - from.0) as f32 * t, from.1 as f32 + (to.1 - from.1) as f32 * t)
    }

    fn follower_pos(&self, k: usize) -> Option<(f32, f32)> {
        let to = *self.trail.get(k)?;
        let from = self.trail_from.get(k).copied().unwrap_or(to);
        let t = ease(self.move_t);
        Some((from.0 as f32 + (to.0 - from.0) as f32 * t, from.1 as f32 + (to.1 - from.1) as f32 * t))
    }

    pub fn draw(&mut self, painter: &egui::Painter, gfx: &Gfx, screen: Rect, game: &Game, time: f64, dt: f32) {
        let world = &game.world;
        painter.rect_filled(screen, CornerRadius::ZERO, Color32::from_rgb(4, 4, 7));
        let ts = tile_size(screen);
        let (px, py) = self.player_pos(world);
        let target = pos2((px + 0.5) * ts, (py + 0.5) * ts);
        let camera = match self.camera {
            None => target,
            Some(c) => c + (target - c) * (1.0 - (-dt * 14.0).exp()),
        };
        self.camera = Some(camera);
        let map_w = world.w as f32 * ts;
        let map_h = world.h as f32 * ts;
        let clamp_axis = |c: f32, view: f32, map: f32| {
            if map <= view { map / 2.0 } else { c.clamp(view / 2.0, map - view / 2.0) }
        };
        let cam = pos2(clamp_axis(camera.x, screen.width(), map_w), clamp_axis(camera.y, screen.height(), map_h));
        let origin = screen.center() - cam.to_vec2();
        let origin = pos2(origin.x.round(), origin.y.round());
        let to_screen = |x: f32, y: f32| origin + vec2(x * ts, y * ts);

        let x0 = (((screen.left() - origin.x) / ts).floor() as i32 - 1).max(0);
        let y0 = (((screen.top() - origin.y) / ts).floor() as i32 - 1).max(0);
        let x1 = (((screen.right() - origin.x) / ts).ceil() as i32 + 1).min(world.w - 1);
        let y1 = (((screen.bottom() - origin.y) / ts).ceil() as i32 + 1).min(world.h - 1);

        // ---- light
        let lights = collect_lights(world, (px, py), time);
        let base = world.biome.light_colour();
        let ambient = world.biome.ambient();
        let vertex_light = |vx: i32, vy: i32| -> [f32; 3] {
            let p = (vx as f32, vy as f32);
            let mut c = [ambient * 0.7, ambient * 0.75, ambient * 0.95];
            for l in &lights {
                let d = ((p.0 - l.x).powi(2) + (p.1 - l.y).powi(2)).sqrt();
                if d < l.radius {
                    let f = (1.0 - d / l.radius).powf(1.25) * l.intensity;
                    for k in 0..3 {
                        c[k] += l.colour[k] * f;
                    }
                }
            }
            c.map(|v| v.min(1.25))
        };
        let (vw, vh) = ((x1 - x0 + 2) as usize, (y1 - y0 + 2) as usize);
        let mut vl = vec![[0.0f32; 3]; vw * vh];
        for vy in 0..vh {
            for vx in 0..vw {
                vl[vy * vw + vx] = vertex_light(x0 + vx as i32, y0 + vy as i32);
            }
        }
        let corner = |x: i32, y: i32| vl[((y - y0) as usize) * vw + (x - x0) as usize];
        let to_colour = |c: [f32; 3]| {
            Color32::from_rgb((c[0] * base[0] * 255.0).min(255.0) as u8, (c[1] * base[1] * 255.0).min(255.0) as u8, (c[2] * base[2] * 255.0).min(255.0) as u8)
        };
        let memory = Color32::from_rgb(38, 40, 58);

        let mut floor_mesh = egui::Mesh::with_texture(gfx.atlas.id());
        let mut over_mesh = egui::Mesh::with_texture(gfx.atlas.id());
        let mut signs = Vec::new();
        for y in y0..=y1 {
            for x in x0..=x1 {
                let p = (x, y);
                if !world.is_explored(p) {
                    continue;
                }
                let tile = world.tile(p);
                let v = world.variant[world.idx(p)];
                let r = Rect::from_min_size(to_screen(x as f32, y as f32), vec2(ts, ts));
                let colours = if world.is_visible(p) {
                    [to_colour(corner(x, y)), to_colour(corner(x + 1, y)), to_colour(corner(x + 1, y + 1)), to_colour(corner(x, y + 1))]
                } else {
                    [memory; 4]
                };
                gfx.quad(&mut floor_mesh, base_sprite(world.biome, tile, v), r, colours);
                if let Some(o) = overlay_sprite(tile, v) {
                    gfx.quad(&mut over_mesh, o, r, colours);
                }
                // Shop signs hang on the wall beside the door.
                if let Tile::ShopDoor(shop) = tile {
                    let sign = match shop {
                        Shop::Inn => Sprite::ShopInn,
                        Shop::Smith => Sprite::ShopWeapon,
                        Shop::Apothecary => Sprite::ShopPotion,
                        Shop::Temple => Sprite::ShopGeneral,
                    };
                    let sr = r.translate(vec2(-ts, -ts * 0.1));
                    signs.push((sign, sr, colours));
                }
            }
        }
        for (sprite, r, colours) in signs {
            gfx.quad(&mut over_mesh, sprite, r, colours);
        }
        painter.add(floor_mesh);
        painter.add(over_mesh);

        // ---- entities (sorted by row so lower ones overlap upper)
        let mut order: Vec<usize> = (0..world.entities.len()).collect();
        order.sort_by_key(|&i| world.entities[i].pos.1);
        let light_at = |x: f32, y: f32| to_colour(vertex_light((x + 0.5) as i32, (y + 0.5) as i32));
        for &i in &order {
            let e = &world.entities[i];
            let (ex, ey) = self.entity_pos(world, i);
            let tile_p = e.pos;
            let is_town = world.place == Place::Town;
            let seen = world.is_visible(tile_p) || (is_town && world.is_explored(tile_p));
            let remembered = world.is_explored(tile_p);
            let (show, tint) = match &e.kind {
                EntityKind::Monster { .. } | EntityKind::Npc { .. } | EntityKind::Boss { .. } => (seen, light_at(ex, ey)),
                _ if world.is_visible(tile_p) => (true, light_at(ex, ey)),
                _ => (remembered, memory),
            };
            if !show {
                continue;
            }
            let bob = match &e.kind {
                EntityKind::Monster { .. } => ((time * 5.0 + i as f64).sin() as f32) * ts * 0.03,
                EntityKind::Npc { .. } => ((time * 2.0 + i as f64).sin() as f32).abs() * ts * -0.04,
                EntityKind::Pickup(_) => ((time * 3.0 + i as f64).sin() as f32) * ts * 0.06 - ts * 0.05,
                _ => 0.0,
            };
            let mut r = Rect::from_min_size(to_screen(ex, ey) + vec2(0.0, bob), vec2(ts, ts));
            if let EntityKind::Boss { .. } = e.kind {
                r = Rect::from_center_size(r.center_bottom() - vec2(0.0, ts * 0.8), vec2(ts * 1.6, ts * 1.6));
            }
            // Soft glows for magical things.
            let glow = match &e.kind {
                EntityKind::Pickup(Pickup::Shard(_)) => Some(Color32::from_rgba_unmultiplied(120, 200, 255, 60)),
                EntityKind::Waystone => Some(Color32::from_rgba_unmultiplied(120, 170, 255, 45)),
                EntityKind::Boss { .. } => Some(Color32::from_rgba_unmultiplied(180, 60, 60, 40)),
                _ => None,
            };
            if let Some(g) = glow {
                let pulse = 0.8 + 0.2 * (time * 2.5 + i as f64).sin() as f32;
                painter.circle_filled(r.center(), ts * 0.7 * pulse, g);
            }
            if matches!(e.kind, EntityKind::Monster { .. } | EntityKind::Npc { .. } | EntityKind::Boss { .. }) {
                shadow(painter, r, ts);
            }
            gfx.draw(painter, e.sprite(), r, tint);
            if let EntityKind::Monster { awake: true, group, .. } = &e.kind {
                gfx::text(painter, r.center_top() - vec2(0.0, 4.0), Align2::CENTER_BOTTOM, "!", gfx::heading_font(ts * 0.4), Color32::from_rgb(255, 80, 60));
                if group.len() > 1 {
                    gfx::text(painter, r.right_bottom() - vec2(4.0, 2.0), Align2::RIGHT_BOTTOM, &format!("×{}", group.len()), gfx::body_font(ts * 0.3), PARCHMENT);
                }
            }
            if let EntityKind::Npc { scene: Some(_), .. } = &e.kind {
                let y = r.top() - 6.0 + ((time * 3.0).sin() as f32) * 3.0;
                gfx::diamond(painter, pos2(r.center().x, y - ts * 0.12), ts * 0.28, GOLD);
            }
        }

        // ---- the party, leader last so she's on top
        let followers: Vec<Sprite> = game.party.iter().skip(1).map(|h| h.def().sprite).collect();
        for (k, sprite) in followers.iter().enumerate().rev() {
            if let Some((fx, fy)) = self.follower_pos(k) {
                if (fx - px).abs() < 0.01 && (fy - py).abs() < 0.01 {
                    continue;
                }
                let r = Rect::from_min_size(to_screen(fx, fy), vec2(ts, ts));
                shadow(painter, r, ts);
                gfx.draw(painter, *sprite, r, light_at(fx, fy));
            }
        }
        let walk_bob = if self.moving() { (self.move_t * std::f32::consts::PI).sin() * ts * -0.06 } else { 0.0 };
        let pr = Rect::from_min_size(to_screen(px, py) + vec2(0.0, walk_bob), vec2(ts, ts));
        shadow(painter, pr, ts);
        let lead = game.party[0].def().sprite;
        let tint = Color32::from_rgb(255, 245, 230);
        if world.facing == crate::world::Dir::Left {
            gfx.draw_flipped(painter, lead, pr, tint);
        } else {
            gfx.draw(painter, lead, pr, tint);
        }

        // ---- atmosphere
        self.particles(painter, screen, world.biome, dt, time);
        gfx::vignette(painter, screen, if world.biome == Biome::Town { 0.45 } else { 0.7 });
    }

    fn particles(&mut self, painter: &egui::Painter, screen: Rect, biome: Biome, dt: f32, time: f64) {
        let (count, colour, size, vel): (usize, Color32, f32, Vec2) = match biome {
            Biome::Town => (26, Color32::from_rgb(255, 230, 120), 2.2, vec2(0.0, -6.0)),
            Biome::Undercroft => (40, Color32::from_rgb(200, 180, 150), 1.6, vec2(4.0, 3.0)),
            Biome::Archive => (35, Color32::from_rgb(150, 200, 255), 2.4, vec2(0.0, -18.0)),
            Biome::Hollows => (55, Color32::from_rgb(120, 255, 210), 2.2, vec2(5.0, -10.0)),
            Biome::Forge => (60, Color32::from_rgb(255, 140, 50), 2.0, vec2(6.0, -40.0)),
            Biome::Pale => (90, Color32::from_rgb(235, 242, 255), 2.4, vec2(-12.0, 40.0)),
        };
        let seed = time.to_bits();
        while self.particles.len() < count {
            let k = self.particles.len() as u64 * 2654435761 ^ seed;
            let rx = ((k % 10_000) as f32) / 10_000.0;
            let ry = (((k / 10_000) % 10_000) as f32) / 10_000.0;
            let life = 3.0 + ((k / 7) % 400) as f32 / 100.0;
            self.particles.push(Particle {
                pos: vec2(rx * screen.width(), ry * screen.height()),
                vel: vel * (0.6 + rx * 0.8) + vec2((ry - 0.5) * 10.0, 0.0),
                life: 0.0,
                max_life: life,
                size: size * (0.6 + ry * 0.8),
                colour,
            });
        }
        for p in &mut self.particles {
            p.life += dt;
            p.pos += p.vel * dt;
            p.pos.x += ((time as f32 * 0.7 + p.max_life * 3.0).sin()) * 6.0 * dt;
            let fade = (p.life / 0.8).min(1.0) * ((p.max_life - p.life) / 0.8).clamp(0.0, 1.0);
            let flicker = if biome == Biome::Town { 0.5 + 0.5 * ((time as f32 * 3.0 + p.max_life * 10.0).sin()) } else { 1.0 };
            let a = (fade * flicker * 200.0) as u8;
            let c = Color32::from_rgba_unmultiplied(p.colour.r(), p.colour.g(), p.colour.b(), a);
            let pos = screen.min + p.pos;
            if biome == Biome::Forge || biome == Biome::Hollows || biome == Biome::Town {
                painter.circle_filled(pos, p.size * 2.5, Color32::from_rgba_unmultiplied(p.colour.r(), p.colour.g(), p.colour.b(), a / 6));
            }
            painter.circle_filled(pos, p.size, c);
        }
        self.particles.retain(|p| p.life < p.max_life && screen.expand(40.0).contains(screen.min + p.pos));
    }

    pub fn draw_hud(&self, painter: &egui::Painter, gfx: &Gfx, screen: Rect, game: &Game, time: f64) {
        // Place and objective.
        let top = Rect::from_min_size(screen.left_top() + vec2(16.0, 14.0), vec2(560.0, 64.0));
        painter.rect_filled(top, CornerRadius::same(8), Color32::from_black_alpha(140));
        gfx::text(painter, top.left_top() + vec2(14.0, 8.0), Align2::LEFT_TOP, &game.place_name(), gfx::heading_font(21.0), GOLD);
        gfx::text(painter, top.left_top() + vec2(14.0, 36.0), Align2::LEFT_TOP, game.objective(), gfx::italic_font(17.0), PARCHMENT);

        // Gold and time.
        let t = game.playtime as u64;
        let info = format!("{} gold   ·   {}:{:02}:{:02}", game.gold, t / 3600, (t / 60) % 60, t % 60);
        let ir = Rect::from_min_size(pos2(screen.right() - 290.0, screen.top() + 14.0), vec2(274.0, 34.0));
        painter.rect_filled(ir, CornerRadius::same(8), Color32::from_black_alpha(140));
        gfx::text(painter, ir.center(), Align2::CENTER_CENTER, &info, gfx::body_font(19.0), GOLD);

        // Party cards.
        let card_w = 230.0;
        for (i, h) in game.party.iter().enumerate() {
            let r = Rect::from_min_size(pos2(screen.left() + 16.0 + i as f32 * (card_w + 10.0), screen.bottom() - 86.0), vec2(card_w, 70.0));
            painter.rect_filled(r, CornerRadius::same(8), Color32::from_black_alpha(160));
            painter.rect_stroke(r, CornerRadius::same(8), Stroke::new(1.0, GOLD.gamma_multiply(0.4)), egui::StrokeKind::Inside);
            let pr = Rect::from_min_size(r.left_top() + vec2(6.0, 6.0), vec2(58.0, 58.0));
            gfx::portrait(gfx, painter, pr, h.def().sprite, false);
            let x = pr.right() + 10.0;
            let name_col = if h.hp <= 0 { gfx::DIM } else { PARCHMENT };
            gfx::text(painter, pos2(x, r.top() + 6.0), Align2::LEFT_TOP, h.name(), gfx::heading_font(16.0), name_col);
            gfx::text(painter, pos2(r.right() - 8.0, r.top() + 7.0), Align2::RIGHT_TOP, &format!("Lv {}", h.level), gfx::body_font(16.0), GOLD);
            let bw = r.right() - x - 10.0;
            gfx::bar(painter, Rect::from_min_size(pos2(x, r.top() + 31.0), vec2(bw, 11.0)), h.hp as f32 / h.max_hp() as f32, gfx::HP_RED);
            gfx::bar(painter, Rect::from_min_size(pos2(x, r.top() + 48.0), vec2(bw, 8.0)), h.mp as f32 / h.max_mp().max(1) as f32, gfx::MP_BLUE);
            gfx::text(painter, pos2(x + bw, r.top() + 36.0), Align2::RIGHT_CENTER, &format!("{}", h.hp.max(0)), gfx::body_font(13.0), Color32::WHITE);
        }

        // Minimap.
        if self.show_map && game.world.place != Place::Town {
            self.minimap(painter, screen, &game.world, time);
        }

        // Notices.
        for (k, (text, age)) in self.notices.iter().enumerate() {
            let a = (age / 0.25).min(1.0) * ((3.5 - age) / 0.6).clamp(0.0, 1.0);
            let y = screen.top() + 110.0 + k as f32 * 42.0;
            let galley_w = text.len() as f32 * 10.0 + 60.0;
            let r = Rect::from_center_size(pos2(screen.center().x, y), vec2(galley_w, 36.0));
            painter.rect_filled(r, CornerRadius::same(18), Color32::from_black_alpha((180.0 * a) as u8));
            painter.rect_stroke(r, CornerRadius::same(18), Stroke::new(1.0, GOLD.gamma_multiply(0.6 * a)), egui::StrokeKind::Inside);
            gfx::text(painter, r.center(), Align2::CENTER_CENTER, text, gfx::body_font(20.0), PARCHMENT.gamma_multiply(a));
        }

        // Area title card.
        if let Some((title, subtitle, age)) = &self.area_title {
            let a = (age / 0.8).min(1.0) * ((4.5 - age) / 1.0).clamp(0.0, 1.0);
            let c = screen.center() - vec2(0.0, screen.height() * 0.18);
            let band = Rect::from_center_size(c, vec2(screen.width(), 130.0));
            gfx::gradient(painter, Rect::from_min_max(band.left_top(), pos2(band.right(), band.center().y)), Color32::TRANSPARENT, Color32::from_black_alpha((150.0 * a) as u8));
            gfx::gradient(painter, Rect::from_min_max(pos2(band.left(), band.center().y), band.right_bottom()), Color32::from_black_alpha((150.0 * a) as u8), Color32::TRANSPARENT);
            gfx::text(painter, c - vec2(0.0, 14.0), Align2::CENTER_CENTER, title, gfx::title_font(46.0), GOLD.gamma_multiply(a));
            gfx::text(painter, c + vec2(0.0, 34.0), Align2::CENTER_CENTER, subtitle, gfx::heading_font(20.0), PARCHMENT.gamma_multiply(a));
        }

        gfx::text(
            painter,
            pos2(screen.right() - 16.0, screen.bottom() - 12.0),
            Align2::RIGHT_BOTTOM,
            "Move: arrows/WASD   Talk/act: Enter   Menu: Tab   Map: M",
            gfx::body_font(15.0),
            Color32::from_white_alpha(90),
        );
    }

    fn minimap(&self, painter: &egui::Painter, screen: Rect, world: &World, time: f64) {
        let cell = (230.0 / world.w as f32).min(170.0 / world.h as f32).floor().max(2.0);
        let size = vec2(world.w as f32 * cell, world.h as f32 * cell);
        let r = Rect::from_min_size(pos2(screen.right() - 16.0 - size.x - 12.0, screen.top() + 58.0), size + vec2(12.0, 12.0));
        painter.rect_filled(r, CornerRadius::same(8), Color32::from_black_alpha(150));
        let o = r.min + vec2(6.0, 6.0);
        for y in 0..world.h {
            for x in 0..world.w {
                if !world.is_explored((x, y)) {
                    continue;
                }
                let c = match world.tile((x, y)) {
                    Tile::Wall | Tile::Bookshelf => continue,
                    Tile::Stairs => GOLD,
                    Tile::Door | Tile::OpenDoor => Color32::from_rgb(150, 100, 60),
                    Tile::Lava => Color32::from_rgb(200, 70, 30),
                    Tile::Shallow | Tile::Water => Color32::from_rgb(60, 100, 160),
                    _ => Color32::from_rgb(90, 88, 96),
                };
                painter.rect_filled(Rect::from_min_size(o + vec2(x as f32 * cell, y as f32 * cell), vec2(cell, cell)), CornerRadius::ZERO, c);
            }
        }
        for e in &world.entities {
            if !world.is_explored(e.pos) {
                continue;
            }
            let c = match &e.kind {
                EntityKind::Waystone => Color32::from_rgb(110, 170, 255),
                EntityKind::Chest { opened: false, .. } => Color32::from_rgb(240, 200, 80),
                EntityKind::Pickup(Pickup::Shard(_)) => Color32::from_rgb(140, 230, 255),
                EntityKind::Pickup(_) => Color32::from_rgb(240, 240, 240),
                EntityKind::Boss { .. } => Color32::from_rgb(220, 40, 40),
                EntityKind::Npc { .. } => Color32::from_rgb(120, 230, 120),
                EntityKind::Monster { .. } if world.is_visible(e.pos) => Color32::from_rgb(230, 90, 70),
                _ => continue,
            };
            painter.circle_filled(o + vec2((e.pos.0 as f32 + 0.5) * cell, (e.pos.1 as f32 + 0.5) * cell), cell * 0.7, c);
        }
        let pulse = 0.7 + 0.3 * (time * 5.0).sin() as f32;
        painter.circle_filled(o + vec2((world.player.0 as f32 + 0.5) * cell, (world.player.1 as f32 + 0.5) * cell), cell * pulse + 1.0, Color32::WHITE);
    }
}

fn ease(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

pub fn tile_size(screen: Rect) -> f32 {
    let scale = (screen.height() / 420.0).round().max(1.0);
    32.0 * scale
}

fn shadow(painter: &egui::Painter, r: Rect, ts: f32) {
    let c = r.center_bottom() - vec2(0.0, ts * 0.1);
    painter.add(egui::Shape::ellipse_filled(c, vec2(ts * 0.3, ts * 0.09), Color32::from_black_alpha(90)));
}

struct Light {
    x: f32,
    y: f32,
    radius: f32,
    intensity: f32,
    colour: [f32; 3],
}

fn collect_lights(world: &World, player: (f32, f32), time: f64) -> Vec<Light> {
    let t = time as f32;
    let flicker = 1.0 + 0.04 * (t * 7.3).sin() + 0.03 * (t * 13.1).sin();
    let lantern = if world.place == Place::Town { 5.0 } else { 6.2 };
    let mut lights = vec![Light { x: player.0 + 0.5, y: player.1 + 0.5, radius: lantern * flicker, intensity: 1.05, colour: [1.0, 0.92, 0.78] }];
    for (i, e) in world.entities.iter().enumerate() {
        let r = e.light();
        if r <= 0.0 {
            continue;
        }
        let f = 1.0 + 0.08 * (t * 6.0 + i as f32 * 1.7).sin();
        let colour = match &e.kind {
            EntityKind::Waystone => [0.6, 0.8, 1.3],
            EntityKind::Pickup(_) => [0.6, 0.9, 1.3],
            EntityKind::Boss { .. } => [1.2, 0.4, 0.4],
            EntityKind::Decor { sprite: Sprite::FungusDecor, .. } => [0.4, 1.1, 1.0],
            EntityKind::Decor { sprite: Sprite::IceCrystal, .. } => [0.7, 0.85, 1.2],
            _ => [1.2, 0.85, 0.55],
        };
        lights.push(Light { x: e.pos.0 as f32 + 0.5, y: e.pos.1 as f32 + 0.5, radius: r * f, intensity: 0.9, colour });
    }
    // Lava glows (only near the player, to keep it cheap).
    let (px, py) = (player.0 as i32, player.1 as i32);
    for y in (py - 12).max(0)..(py + 12).min(world.h) {
        for x in (px - 16).max(0)..(px + 16).min(world.w) {
            if world.tile((x, y)) == Tile::Lava {
                lights.push(Light { x: x as f32 + 0.5, y: y as f32 + 0.5, radius: 2.6, intensity: 0.8, colour: [1.3, 0.5, 0.2] });
            }
        }
    }
    lights
}

fn base_sprite(biome: Biome, tile: Tile, v: u8) -> Sprite {
    let floors = biome.floors();
    let walls = biome.walls();
    match tile {
        Tile::Floor | Tile::Door | Tile::OpenDoor if biome != Biome::Town => floors[(v % 3) as usize],
        Tile::Wall => walls[(v % 2) as usize],
        Tile::Stairs => Sprite::StairsDown,
        Tile::Water => Sprite::TownWater,
        Tile::Shallow => Sprite::ShallowWater,
        Tile::Lava => Sprite::Lava,
        Tile::Grass | Tile::Tree | Tile::Statue => [Sprite::TownGrassA, Sprite::TownGrassB, Sprite::TownGrassC][(v % 3) as usize],
        Tile::Path | Tile::Fountain => [Sprite::TownPathA, Sprite::TownPathB][(v % 2) as usize],
        Tile::Wood | Tile::Altar | Tile::ShopDoor(_) | Tile::OpenDoor | Tile::Door | Tile::Floor => Sprite::WoodFloor,
        Tile::TownWall => Sprite::TownWall,
        Tile::VaultGate => Sprite::VaultGate,
        Tile::Bookshelf => Sprite::Bookshelf,
    }
}

fn overlay_sprite(tile: Tile, v: u8) -> Option<Sprite> {
    Some(match tile {
        Tile::Door => Sprite::DoorClosed,
        Tile::OpenDoor => Sprite::DoorOpen,
        Tile::Tree => if v % 2 == 0 { Sprite::TreeA } else { Sprite::TreeB },
        Tile::Fountain => Sprite::Fountain,
        Tile::Statue => Sprite::Statue,
        Tile::Altar => Sprite::TempleAltar,
        Tile::ShopDoor(_) => Sprite::DoorOpen,
        _ => return None,
    })
}
