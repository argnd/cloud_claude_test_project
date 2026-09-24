//! Overlays drawn over the map: the pause menu, shops, services, the
//! waystone, floor selection, saving and loading, settings.

use eframe::egui::{self, Align2, Color32, CornerRadius, Rect, Stroke, pos2, vec2};

use super::input::Input;
use super::widgets::{self, ListState, Row};
use crate::audio::{Audio, Sfx};
use crate::data::Stats;
use crate::data::items::{EquipSlot, ItemId, ItemKind, Special};
use crate::data::quests::QuestId;
use crate::data::skills::{SkillKind, Target};
use crate::game::{Game, QuestState};
use crate::gfx::{self, GOLD, Gfx, PARCHMENT};
use crate::save::{self, Settings};
use crate::world::Shop;
use crate::world::floor::LAST_FLOOR;
use crate::world::Biome;

pub enum MenuOut {
    None,
    Close,
    Descend(u32),
    ToTown,
    Rest,
    Pray,
    Save(usize),
    Load(usize),
    Title,
    Replay(String),
    OpenShop(Shop),
    SettingsChanged,
}

pub enum Overlay {
    Pause(PauseMenu),
    Shop(ShopView),
    Service { kind: Shop, list: ListState },
    Confirm { text: String, yes: ConfirmYes, list: ListState },
    FloorSelect(ListState),
    Waystone(ListState),
    Saves { saving: bool, list: ListState },
}

#[derive(Clone, Copy)]
pub enum ConfirmYes {
    Descend(u32),
    Title,
}

impl Overlay {
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        painter: &egui::Painter,
        gfx: &Gfx,
        screen: Rect,
        input: &mut Input,
        game: &mut Game,
        audio: &mut Audio,
        settings: &mut Settings,
        time: f64,
    ) -> MenuOut {
        painter.rect_filled(screen, CornerRadius::ZERO, Color32::from_black_alpha(110));
        match self {
            Overlay::Pause(menu) => menu.show(painter, gfx, screen, input, game, audio, settings, time),
            Overlay::Shop(shop) => shop.show(painter, gfx, screen, input, game, audio, time),
            Overlay::Service { kind, list } => service(painter, gfx, screen, input, game, audio, *kind, list, time),
            Overlay::Confirm { text, yes, list } => {
                let rect = Rect::from_center_size(screen.center(), vec2(520.0, 200.0));
                gfx::panel(painter, rect);
                gfx::wrapped(painter, rect.left_top() + vec2(24.0, 22.0), rect.width() - 48.0, text, gfx::body_font(23.0), PARCHMENT);
                let rows = vec![Row::new("Yes"), Row::new("No")];
                let r = widgets::list(painter, gfx, Rect::from_min_size(rect.left_top() + vec2(20.0, 100.0), vec2(rect.width() - 40.0, 80.0)), &rows, list, input, true, time);
                sfx(audio, &r);
                match r.picked {
                    Some(0) => match yes {
                        ConfirmYes::Descend(n) => MenuOut::Descend(*n),
                        ConfirmYes::Title => MenuOut::Title,
                    },
                    Some(_) => MenuOut::Close,
                    None if input.cancel => MenuOut::Close,
                    None => MenuOut::None,
                }
            }
            Overlay::FloorSelect(list) => {
                let rect = Rect::from_center_size(screen.center(), vec2(620.0, 560.0));
                gfx::panel(painter, rect);
                gfx::text(painter, rect.center_top() + vec2(0.0, 30.0), Align2::CENTER_CENTER, "The Vaultgate", gfx::heading_font(28.0), GOLD);
                gfx::text(painter, rect.center_top() + vec2(0.0, 62.0), Align2::CENTER_CENTER, "The waystones remember every floor you have reached.", gfx::italic_font(18.0), gfx::DIM);
                let deepest = game.deepest.clamp(1, LAST_FLOOR);
                let rows: Vec<Row> = (1..=deepest)
                    .rev()
                    .map(|n| Row::new(format!("Floor {n}")).right(Biome::for_floor(n).name()))
                    .chain(std::iter::once(Row::new("Stay in Hollowmere")))
                    .collect();
                let r = widgets::list(painter, gfx, Rect::from_min_max(rect.left_top() + vec2(20.0, 90.0), rect.right_bottom() - vec2(20.0, 16.0)), &rows, list, input, true, time);
                sfx(audio, &r);
                match r.picked {
                    Some(i) if i < deepest as usize => MenuOut::Descend(deepest - i as u32),
                    Some(_) => MenuOut::Close,
                    None if input.cancel => MenuOut::Close,
                    None => MenuOut::None,
                }
            }
            Overlay::Waystone(list) => {
                let rect = Rect::from_center_size(screen.center(), vec2(520.0, 300.0));
                gfx::panel(painter, rect);
                gfx::text(painter, rect.center_top() + vec2(0.0, 30.0), Align2::CENTER_CENTER, "Waystone", gfx::heading_font(28.0), Color32::from_rgb(150, 200, 255));
                let rows = vec![
                    Row::new("Rest and make camp").right("restores the party"),
                    Row::new("Return to Hollowmere"),
                    Row::new("Save the game"),
                    Row::new("Leave"),
                ];
                let r = widgets::list(painter, gfx, Rect::from_min_max(rect.left_top() + vec2(20.0, 70.0), rect.right_bottom() - vec2(20.0, 16.0)), &rows, list, input, true, time);
                sfx(audio, &r);
                match r.picked {
                    Some(0) => MenuOut::Rest,
                    Some(1) => MenuOut::ToTown,
                    Some(2) => {
                        *self = Overlay::Saves { saving: true, list: ListState { cursor: 1, scroll: 0 } };
                        MenuOut::None
                    }
                    Some(_) => MenuOut::Close,
                    None if input.cancel => MenuOut::Close,
                    None => MenuOut::None,
                }
            }
            Overlay::Saves { saving, list } => saves(painter, gfx, screen, input, audio, *saving, list, time),
        }
    }
}

fn sfx(audio: &mut Audio, r: &widgets::ListResult) {
    if r.moved {
        audio.play_sfx(Sfx::MenuMove);
    }
    if r.picked.is_some() {
        audio.play_sfx(Sfx::MenuSelect);
    }
    if r.denied {
        audio.play_sfx(Sfx::Denied);
    }
}

#[allow(clippy::too_many_arguments)]
fn saves(painter: &egui::Painter, gfx: &Gfx, screen: Rect, input: &mut Input, audio: &mut Audio, saving: bool, list: &mut ListState, time: f64) -> MenuOut {
    let rect = Rect::from_center_size(screen.center(), vec2(760.0, 330.0));
    gfx::panel(painter, rect);
    gfx::text(painter, rect.center_top() + vec2(0.0, 30.0), Align2::CENTER_CENTER, if saving { "Save Game" } else { "Load Game" }, gfx::heading_font(28.0), GOLD);
    let rows: Vec<Row> = (0..save::SLOTS)
        .map(|s| {
            let label = if s == 0 { "Autosave".to_string() } else { format!("Slot {s}") };
            let desc = save::describe(s);
            let enabled = if saving { s != 0 } else { desc.is_some() };
            Row::new(label).right(desc.unwrap_or_else(|| "— empty —".into())).enabled(enabled)
        })
        .collect();
    let r = widgets::list(painter, gfx, Rect::from_min_max(rect.left_top() + vec2(20.0, 70.0), rect.right_bottom() - vec2(20.0, 16.0)), &rows, list, input, true, time);
    sfx(audio, &r);
    match r.picked {
        Some(s) if saving => MenuOut::Save(s),
        Some(s) => MenuOut::Load(s),
        None if input.cancel => MenuOut::Close,
        None => MenuOut::None,
    }
}

#[allow(clippy::too_many_arguments)]
fn service(painter: &egui::Painter, gfx: &Gfx, screen: Rect, input: &mut Input, game: &mut Game, audio: &mut Audio, kind: Shop, list: &mut ListState, time: f64) -> MenuOut {
    let rect = Rect::from_center_size(screen.center(), vec2(540.0, 300.0));
    gfx::panel(painter, rect);
    let (title, rows): (&str, Vec<Row>) = match kind {
        Shop::Inn => (
            "The Lantern & Lark",
            vec![Row::new("Rest the night").right("free"), Row::new("Buy provisions"), Row::new("Save the game"), Row::new("Leave")],
        ),
        _ => (
            "Temple of the Flame",
            vec![Row::new("Pray").right("heals and revives"), Row::new("Buy blessings"), Row::new("Save the game"), Row::new("Leave")],
        ),
    };
    gfx::text(painter, rect.center_top() + vec2(0.0, 30.0), Align2::CENTER_CENTER, title, gfx::heading_font(26.0), GOLD);
    gfx::text(painter, rect.center_top() + vec2(0.0, 58.0), Align2::CENTER_CENTER, &format!("{} gold", game.gold), gfx::body_font(18.0), gfx::DIM);
    let r = widgets::list(painter, gfx, Rect::from_min_max(rect.left_top() + vec2(20.0, 80.0), rect.right_bottom() - vec2(20.0, 16.0)), &rows, list, input, true, time);
    sfx(audio, &r);
    match r.picked {
        Some(0) => {
            if kind == Shop::Inn {
                MenuOut::Rest
            } else {
                MenuOut::Pray
            }
        }
        Some(1) => MenuOut::OpenShop(kind),
        Some(2) => MenuOut::Save(usize::MAX),
        Some(_) => MenuOut::Close,
        None if input.cancel => MenuOut::Close,
        None => MenuOut::None,
    }
}

// ---------------------------------------------------------------- shop

pub struct ShopView {
    pub shop: Shop,
    selling: bool,
    list: ListState,
    message: Option<(String, f32)>,
}

impl ShopView {
    pub fn new(shop: Shop) -> Self {
        Self { shop, selling: false, list: ListState::default(), message: None }
    }

    #[allow(clippy::too_many_arguments)]
    fn show(&mut self, painter: &egui::Painter, gfx: &Gfx, screen: Rect, input: &mut Input, game: &mut Game, audio: &mut Audio, time: f64) -> MenuOut {
        let rect = Rect::from_center_size(screen.center(), vec2(1000.0f32.min(screen.width() - 40.0), 620.0f32.min(screen.height() - 40.0)));
        gfx::panel(painter, rect);
        let (title, keeper) = match self.shop {
            Shop::Smith => ("Dagna's Forge", "\"Steel doesn't care who you are. Neither do I. Pay up.\""),
            Shop::Apothecary => ("Fen's Apothecary", "\"Everything's labelled! Mostly! Please don't drink the blue one twice.\""),
            Shop::Inn => ("Bess's Pantry", "\"Nobody goes down there on an empty stomach. Not on my watch.\""),
            Shop::Temple => ("Temple Offerings", "\"May the Flame keep you. And may these help it along.\""),
        };
        gfx::text(painter, rect.left_top() + vec2(24.0, 18.0), Align2::LEFT_TOP, title, gfx::heading_font(28.0), GOLD);
        gfx::text(painter, rect.left_top() + vec2(24.0, 56.0), Align2::LEFT_TOP, keeper, gfx::italic_font(18.0), gfx::DIM);
        gfx::text(painter, rect.right_top() + vec2(-24.0, 22.0), Align2::RIGHT_TOP, &format!("{} gold", game.gold), gfx::heading_font(22.0), GOLD);
        let mut mode = if self.selling { 1 } else { 0 };
        let tab_rect = Rect::from_min_size(rect.left_top() + vec2(24.0, 90.0), vec2(300.0, 34.0));
        if widgets::tabs(painter, tab_rect, &["Buy", "Sell"], &mut mode, input) || input.left || input.right {
            if input.left || input.right {
                mode = 1 - mode;
            }
            self.selling = mode == 1;
            self.list = ListState::default();
            audio.play_sfx(Sfx::MenuMove);
        }
        let items: Vec<(ItemId, u32)> = if self.selling {
            game.inventory.list().into_iter().filter(|(i, _)| i.def().price > 0 && !matches!(i.def().kind, ItemKind::Quest)).collect()
        } else {
            game.stock(self.shop).into_iter().map(|i| (i, game.inventory.count(i))).collect()
        };
        let rows: Vec<Row> = items
            .iter()
            .map(|&(i, have)| {
                let d = i.def();
                let price = if self.selling { d.sell_price() } else { d.price };
                let enabled = self.selling || game.gold >= price;
                let label = if have > 0 { format!("{}  (have {have})", d.name) } else { d.name.to_string() };
                Row::new(label).right(format!("{price} g")).icon(d.sprite).enabled(enabled)
            })
            .collect();
        let list_rect = Rect::from_min_max(rect.left_top() + vec2(20.0, 136.0), pos2(rect.left() + rect.width() * 0.52, rect.bottom() - 18.0));
        let r = widgets::list(painter, gfx, list_rect, &rows, &mut self.list, input, true, time);
        if r.moved {
            audio.play_sfx(Sfx::MenuMove);
        }
        if r.denied {
            audio.play_sfx(Sfx::Denied);
            self.message = Some(("Not enough gold.".into(), 1.5));
        }
        if let Some(&(item, _)) = items.get(self.list.cursor) {
            let detail = Rect::from_min_max(pos2(list_rect.right() + 24.0, rect.top() + 136.0), rect.right_bottom() - vec2(24.0, 18.0));
            item_details(painter, gfx, detail, game, item);
        }
        if let Some(i) = r.picked {
            let item = items[i].0;
            let result = if self.selling { game.sell(item).map(|g| format!("Sold for {g} gold.")) } else { game.buy(item).map(|_| format!("Bought {}.", item.def().name)) };
            match result {
                Ok(m) => {
                    audio.play_sfx(Sfx::Coin);
                    self.message = Some((m, 1.5));
                }
                Err(m) => {
                    audio.play_sfx(Sfx::Denied);
                    self.message = Some((m, 1.5));
                }
            }
        }
        if let Some((m, t)) = &mut self.message {
            *t -= input.dt;
            gfx::text(painter, pos2(rect.center().x, rect.bottom() - 20.0), Align2::CENTER_BOTTOM, m, gfx::body_font(20.0), GOLD);
            if *t <= 0.0 {
                self.message = None;
            }
        }
        if input.cancel { MenuOut::Close } else { MenuOut::None }
    }
}

/// Description, stats and who can use it (with gains/losses vs. current gear).
fn item_details(painter: &egui::Painter, gfx: &Gfx, rect: Rect, game: &Game, item: ItemId) {
    let d = item.def();
    let icon = Rect::from_min_size(rect.left_top(), vec2(64.0, 64.0));
    gfx.draw(painter, d.sprite, icon, Color32::WHITE);
    gfx::text(painter, pos2(icon.right() + 14.0, rect.top() + 8.0), Align2::LEFT_TOP, d.name, gfx::heading_font(22.0), GOLD);
    let kind = match d.kind {
        ItemKind::Consumable(_) => "Consumable",
        ItemKind::Weapon(_) => "Weapon",
        ItemKind::Armor(_) => "Armour",
        ItemKind::Accessory => "Accessory",
        ItemKind::Quest => "Keepsake",
    };
    gfx::text(painter, pos2(icon.right() + 14.0, rect.top() + 40.0), Align2::LEFT_TOP, kind, gfx::italic_font(17.0), gfx::DIM);
    let mut y = icon.bottom() + 12.0;
    y += gfx::wrapped(painter, pos2(rect.left(), y), rect.width(), d.desc, gfx::body_font(20.0), PARCHMENT) + 12.0;
    let s = d.stats;
    let stat_line = stats_text(s);
    if !stat_line.is_empty() {
        gfx::text(painter, pos2(rect.left(), y), Align2::LEFT_TOP, &stat_line, gfx::body_font(19.0), Color32::from_rgb(200, 230, 255));
        y += 30.0;
    }
    if let Some(slot) = d.slot() {
        for h in &game.party {
            let can = h.can_equip(item);
            let text = if !can {
                format!("{}: can't equip", h.name())
            } else {
                let current = h.slot(slot).map(|i| i.def().stats).unwrap_or_default();
                let diff = s.add(current.scale(-1.0));
                let delta = |v: i32, n: &str| if v == 0 { String::new() } else { format!("{n} {}{v}  ", if v > 0 { "+" } else { "" }) };
                let t = format!(
                    "{}{}{}{}{}{}{}",
                    delta(diff.atk, "ATK"),
                    delta(diff.def, "DEF"),
                    delta(diff.mag, "MAG"),
                    delta(diff.res, "RES"),
                    delta(diff.spd, "SPD"),
                    delta(diff.hp, "HP"),
                    delta(diff.mp, "MP")
                );
                format!("{}: {}", h.name(), if t.is_empty() { "no change".into() } else { t })
            };
            let colour = if !can { Color32::from_gray(110) } else { PARCHMENT };
            gfx.draw(painter, h.def().sprite, Rect::from_min_size(pos2(rect.left(), y - 4.0), vec2(28.0, 28.0)), if can { Color32::WHITE } else { Color32::from_gray(90) });
            gfx::text(painter, pos2(rect.left() + 34.0, y), Align2::LEFT_TOP, &text, gfx::body_font(18.0), colour);
            y += 28.0;
        }
        if d.special != Special::None {
            gfx::text(painter, pos2(rect.left(), y + 4.0), Align2::LEFT_TOP, &special_text(d.special), gfx::italic_font(18.0), GOLD);
        }
    }
}

fn stats_text(s: Stats) -> String {
    let mut parts = Vec::new();
    for (v, n) in [(s.atk, "ATK"), (s.def, "DEF"), (s.mag, "MAG"), (s.res, "RES"), (s.spd, "SPD"), (s.hp, "HP"), (s.mp, "MP")] {
        if v != 0 {
            parts.push(format!("{n} {}{v}", if v > 0 { "+" } else { "" }));
        }
    }
    parts.join("   ")
}

fn special_text(s: Special) -> String {
    match s {
        Special::None => String::new(),
        Special::Resist(e) => format!("Halves {} damage", e.name()),
        Special::CritUp(c) => format!("+{c}% critical chance"),
        Special::Regen => "Regenerates HP each turn".into(),
        Special::MpRegen => "Regenerates MP each turn".into(),
        Special::StatusGuard => "Ailments land half as often".into(),
        Special::FirstStrike => "Starts battles hasted".into(),
        Special::AttackElement(e) => format!("Attacks deal {} damage", e.name()),
    }
}

// ---------------------------------------------------------------- pause menu

const TABS: [&str; 7] = ["Party", "Equip", "Items", "Skills", "Quests", "Journal", "System"];

pub struct PauseMenu {
    tab: usize,
    stage: u8,
    a: ListState,
    b: ListState,
    c: ListState,
    picked_item: Option<ItemId>,
    message: Option<(String, f32)>,
    settings_open: bool,
}

impl PauseMenu {
    pub fn new() -> Self {
        Self { tab: 0, stage: 0, a: ListState::default(), b: ListState::default(), c: ListState::default(), picked_item: None, message: None, settings_open: false }
    }

    fn say(&mut self, m: impl Into<String>) {
        self.message = Some((m.into(), 2.0));
    }

    #[allow(clippy::too_many_arguments)]
    fn show(&mut self, painter: &egui::Painter, gfx: &Gfx, screen: Rect, input: &mut Input, game: &mut Game, audio: &mut Audio, settings: &mut Settings, time: f64) -> MenuOut {
        let rect = Rect::from_center_size(screen.center(), vec2((screen.width() - 60.0).min(1180.0), (screen.height() - 60.0).min(720.0)));
        gfx::panel(painter, rect);
        let tab_rect = Rect::from_min_size(rect.left_top() + vec2(20.0, 16.0), vec2(rect.width() - 40.0, 38.0));
        let mut tab = self.tab;
        let clicked = widgets::tabs(painter, tab_rect, &TABS, &mut tab, input);
        if self.stage == 0 && !self.settings_open && (input.left || input.right) {
            tab = if input.left { (tab + TABS.len() - 1) % TABS.len() } else { (tab + 1) % TABS.len() };
        }
        if tab != self.tab || clicked {
            self.tab = tab;
            self.stage = 0;
            self.a = ListState::default();
            self.b = ListState::default();
            self.c = ListState::default();
            audio.play_sfx(Sfx::MenuMove);
            input.left = false;
            input.right = false;
        }
        let body = Rect::from_min_max(rect.left_top() + vec2(20.0, 70.0), rect.right_bottom() - vec2(20.0, 20.0));
        let out = match self.tab {
            0 => self.party(painter, gfx, body, input, game, audio, time),
            1 => self.equip(painter, gfx, body, input, game, audio, time),
            2 => self.items(painter, gfx, body, input, game, audio, time),
            3 => self.skills(painter, gfx, body, input, game, audio, time),
            4 => self.quests(painter, gfx, body, input, game, audio, time),
            5 => self.journal(painter, gfx, body, input, game, audio, time),
            _ => self.system(painter, gfx, body, input, audio, settings, time),
        };
        if let Some((m, t)) = &mut self.message {
            *t -= input.dt;
            let r = Rect::from_center_size(pos2(rect.center().x, rect.bottom() - 34.0), vec2(m.len() as f32 * 10.0 + 60.0, 36.0));
            painter.rect_filled(r, CornerRadius::same(18), Color32::from_black_alpha(200));
            gfx::text(painter, r.center(), Align2::CENTER_CENTER, m, gfx::body_font(20.0), GOLD);
            if *t <= 0.0 {
                self.message = None;
            }
        }
        if !matches!(out, MenuOut::None) {
            return out;
        }
        if input.cancel || input.menu {
            if self.stage > 0 {
                self.stage -= 1;
                audio.play_sfx(Sfx::MenuBack);
            } else if self.settings_open {
                self.settings_open = false;
                audio.play_sfx(Sfx::MenuBack);
            } else {
                audio.play_sfx(Sfx::MenuBack);
                return MenuOut::Close;
            }
        }
        MenuOut::None
    }

    fn hero_rows(game: &Game) -> Vec<Row> {
        game.party
            .iter()
            .map(|h| Row::new(h.name()).right(format!("Lv {}", h.level)).icon(h.def().sprite).colour(if h.hp <= 0 { Color32::from_rgb(200, 100, 100) } else { PARCHMENT }))
            .collect()
    }

    #[allow(clippy::too_many_arguments)]
    fn party(&mut self, painter: &egui::Painter, gfx: &Gfx, body: Rect, input: &mut Input, game: &mut Game, audio: &mut Audio, time: f64) -> MenuOut {
        let left = Rect::from_min_size(body.min, vec2(260.0, body.height()));
        let r = widgets::list(painter, gfx, left, &Self::hero_rows(game), &mut self.a, input, true, time);
        sfx(audio, &r);
        let h = &game.party[self.a.cursor.min(game.party.len() - 1)];
        let right = Rect::from_min_max(pos2(left.right() + 30.0, body.top()), body.max);
        let pr = Rect::from_min_size(right.min, vec2(170.0, 170.0));
        gfx::portrait(gfx, painter, pr, h.def().sprite, false);
        let x = pr.right() + 24.0;
        gfx::text(painter, pos2(x, right.top()), Align2::LEFT_TOP, h.name(), gfx::title_font(38.0), GOLD);
        gfx::text(painter, pos2(x, right.top() + 50.0), Align2::LEFT_TOP, &format!("{} · Level {}", h.def().title, h.level), gfx::heading_font(20.0), PARCHMENT);
        gfx::wrapped(painter, pos2(x, right.top() + 82.0), right.right() - x, h.def().blurb, gfx::italic_font(19.0), gfx::DIM);
        let bw = 300.0;
        gfx::bar(painter, Rect::from_min_size(pos2(x, right.top() + 124.0), vec2(bw, 14.0)), h.hp.max(0) as f32 / h.max_hp() as f32, gfx::HP_RED);
        gfx::text(painter, pos2(x + bw + 10.0, right.top() + 131.0), Align2::LEFT_CENTER, &format!("HP {} / {}", h.hp.max(0), h.max_hp()), gfx::body_font(18.0), PARCHMENT);
        gfx::bar(painter, Rect::from_min_size(pos2(x, right.top() + 146.0), vec2(bw, 10.0)), h.mp as f32 / h.max_mp().max(1) as f32, gfx::MP_BLUE);
        gfx::text(painter, pos2(x + bw + 10.0, right.top() + 151.0), Align2::LEFT_CENTER, &format!("MP {} / {}", h.mp, h.max_mp()), gfx::body_font(18.0), PARCHMENT);
        let xp_frac = h.xp as f32 / h.xp_to_next().max(1) as f32;
        gfx::bar(painter, Rect::from_min_size(pos2(x, right.top() + 164.0), vec2(bw, 6.0)), xp_frac, GOLD);
        gfx::text(painter, pos2(x + bw + 10.0, right.top() + 167.0), Align2::LEFT_CENTER, &format!("{} XP to next", h.xp_to_next().saturating_sub(h.xp)), gfx::body_font(15.0), gfx::DIM);
        let s = h.stats();
        let stats = [("Attack", s.atk), ("Defense", s.def), ("Magic", s.mag), ("Resistance", s.res), ("Speed", s.spd)];
        for (k, (n, v)) in stats.iter().enumerate() {
            let y = pr.bottom() + 26.0 + k as f32 * 32.0;
            gfx::text(painter, pos2(right.left(), y), Align2::LEFT_TOP, n, gfx::heading_font(18.0), gfx::DIM);
            gfx::text(painter, pos2(right.left() + 170.0, y), Align2::LEFT_TOP, &v.to_string(), gfx::body_font(21.0), PARCHMENT);
        }
        let gx = right.left() + 300.0;
        for (k, (slot, name)) in [(EquipSlot::Weapon, "Weapon"), (EquipSlot::Armor, "Armour"), (EquipSlot::Accessory, "Accessory")].iter().enumerate() {
            let y = pr.bottom() + 26.0 + k as f32 * 44.0;
            gfx::text(painter, pos2(gx, y), Align2::LEFT_TOP, name, gfx::heading_font(16.0), gfx::DIM);
            let item = h.slot(*slot);
            if let Some(i) = item {
                gfx.draw(painter, i.def().sprite, Rect::from_min_size(pos2(gx + 110.0, y - 6.0), vec2(32.0, 32.0)), Color32::WHITE);
            }
            gfx::text(painter, pos2(gx + 148.0, y), Align2::LEFT_TOP, item.map(|i| i.def().name).unwrap_or("—"), gfx::body_font(20.0), PARCHMENT);
        }
        let skills: Vec<&str> = h.skills().iter().map(|s| s.def().name).collect();
        gfx::text(painter, pos2(right.left(), body.bottom() - 70.0), Align2::LEFT_TOP, "Skills", gfx::heading_font(16.0), gfx::DIM);
        gfx::wrapped(painter, pos2(right.left(), body.bottom() - 46.0), right.width(), &skills.join(" · "), gfx::body_font(18.0), PARCHMENT);
        MenuOut::None
    }

    #[allow(clippy::too_many_arguments)]
    fn equip(&mut self, painter: &egui::Painter, gfx: &Gfx, body: Rect, input: &mut Input, game: &mut Game, audio: &mut Audio, time: f64) -> MenuOut {
        let col = body.width() / 3.0 - 10.0;
        let a = Rect::from_min_size(body.min, vec2(col, body.height()));
        let b = Rect::from_min_size(pos2(a.right() + 15.0, body.top()), vec2(col, body.height() * 0.45));
        let c = Rect::from_min_size(pos2(b.right() + 15.0, body.top()), vec2(col, body.height()));
        let r = widgets::list(painter, gfx, a, &Self::hero_rows(game), &mut self.a, input, self.stage == 0, time);
        sfx(audio, &r);
        if r.picked.is_some() {
            self.stage = 1;
            input.eat();
        }
        let h = self.a.cursor.min(game.party.len() - 1);
        let slots = [EquipSlot::Weapon, EquipSlot::Armor, EquipSlot::Accessory];
        let slot_rows: Vec<Row> = slots
            .iter()
            .map(|&s| {
                let item = game.party[h].slot(s);
                let n = match s {
                    EquipSlot::Weapon => "Weapon",
                    EquipSlot::Armor => "Armour",
                    EquipSlot::Accessory => "Accessory",
                };
                let mut row = Row::new(item.map(|i| i.def().name).unwrap_or("—")).right(n);
                if let Some(i) = item {
                    row = row.icon(i.def().sprite);
                }
                row
            })
            .collect();
        let r = widgets::list(painter, gfx, b, &slot_rows, &mut self.b, input, self.stage == 1, time);
        if self.stage == 1 {
            sfx(audio, &r);
            if r.picked.is_some() {
                self.stage = 2;
                self.c = ListState::default();
                input.eat();
            }
        }
        let slot = slots[self.b.cursor.min(2)];
        // Candidates.
        let mut options: Vec<Option<ItemId>> = vec![None];
        options.extend(
            game.inventory
                .list()
                .into_iter()
                .filter(|(i, _)| i.def().slot() == Some(slot) && game.party[h].can_equip(*i))
                .map(|(i, _)| Some(i)),
        );
        let rows: Vec<Row> = options
            .iter()
            .map(|o| match o {
                None => Row::new("(remove)"),
                Some(i) => Row::new(i.def().name).icon(i.def().sprite).right(stats_text(i.def().stats).split("   ").next().unwrap_or("").to_string()),
            })
            .collect();
        let r = widgets::list(painter, gfx, c, &rows, &mut self.c, input, self.stage == 2, time);
        if self.stage == 2 {
            sfx(audio, &r);
            if let Some(k) = r.picked {
                match options[k] {
                    None => game.unequip(h, slot),
                    Some(i) => {
                        if let Err(e) = game.equip(h, i) {
                            self.say(e);
                        }
                    }
                }
                audio.play_sfx(Sfx::Buff);
                self.stage = 1;
            }
            if let Some(Some(i)) = options.get(self.c.cursor) {
                let detail = Rect::from_min_max(pos2(b.left(), b.bottom() + 20.0), pos2(b.right(), body.bottom()));
                item_details(painter, gfx, detail, game, *i);
            }
        } else {
            let hero = &game.party[h];
            let s = hero.stats();
            let detail = pos2(b.left(), b.bottom() + 20.0);
            gfx::text(painter, detail, Align2::LEFT_TOP, &format!("ATK {}   DEF {}   MAG {}", s.atk, s.def, s.mag), gfx::body_font(20.0), PARCHMENT);
            gfx::text(painter, detail + vec2(0.0, 30.0), Align2::LEFT_TOP, &format!("RES {}   SPD {}   HP {}   MP {}", s.res, s.spd, s.hp, s.mp), gfx::body_font(20.0), PARCHMENT);
        }
        MenuOut::None
    }

    #[allow(clippy::too_many_arguments)]
    fn items(&mut self, painter: &egui::Painter, gfx: &Gfx, body: Rect, input: &mut Input, game: &mut Game, audio: &mut Audio, time: f64) -> MenuOut {
        let items = game.inventory.list();
        let left = Rect::from_min_size(body.min, vec2(body.width() * 0.5, body.height()));
        let rows: Vec<Row> = items
            .iter()
            .map(|&(i, c)| Row::new(i.def().name).right(format!("×{c}")).icon(i.def().sprite).enabled(true))
            .collect();
        let r = widgets::list(painter, gfx, left, &rows, &mut self.a, input, self.stage == 0, time);
        let right = Rect::from_min_max(pos2(left.right() + 30.0, body.top()), body.max);
        if self.stage == 0 {
            sfx(audio, &r);
            if let Some(k) = r.picked {
                let item = items[k].0;
                if item.def().usable_in_field() {
                    self.picked_item = Some(item);
                    self.stage = 1;
                    self.b = ListState::default();
                    input.eat();
                } else {
                    audio.play_sfx(Sfx::Denied);
                    self.say("That can't be used here.");
                }
            }
            if let Some(&(i, _)) = items.get(self.a.cursor) {
                item_details(painter, gfx, right, game, i);
            }
        } else if let Some(item) = self.picked_item {
            gfx::text(painter, right.left_top(), Align2::LEFT_TOP, &format!("Use {} on…", item.def().name), gfx::heading_font(20.0), GOLD);
            let rows: Vec<Row> = game
                .party
                .iter()
                .map(|h| Row::new(h.name()).icon(h.def().sprite).right(format!("HP {}/{}  MP {}/{}", h.hp.max(0), h.max_hp(), h.mp, h.max_mp())))
                .collect();
            let r = widgets::list(painter, gfx, Rect::from_min_max(right.left_top() + vec2(0.0, 40.0), right.right_bottom()), &rows, &mut self.b, input, true, time);
            sfx(audio, &r);
            if let Some(h) = r.picked {
                let was_town = game.world.place == crate::world::Place::Town;
                match game.use_item_on(item, h) {
                    Ok(m) => {
                        audio.play_sfx(Sfx::Heal);
                        self.say(m);
                        if !game.inventory.has(item) {
                            self.stage = 0;
                        }
                        if item == ItemId::WaystoneShard && !was_town {
                            return MenuOut::ToTown;
                        }
                    }
                    Err(e) => {
                        audio.play_sfx(Sfx::Denied);
                        self.say(e);
                    }
                }
            }
        }
        MenuOut::None
    }

    #[allow(clippy::too_many_arguments)]
    fn skills(&mut self, painter: &egui::Painter, gfx: &Gfx, body: Rect, input: &mut Input, game: &mut Game, audio: &mut Audio, time: f64) -> MenuOut {
        let col = body.width() / 3.0 - 10.0;
        let a = Rect::from_min_size(body.min, vec2(col, body.height()));
        let b = Rect::from_min_size(pos2(a.right() + 15.0, body.top()), vec2(col, body.height() * 0.65));
        let c = Rect::from_min_size(pos2(b.right() + 15.0, body.top()), vec2(col, body.height()));
        let r = widgets::list(painter, gfx, a, &Self::hero_rows(game), &mut self.a, input, self.stage == 0, time);
        if self.stage == 0 {
            sfx(audio, &r);
            if r.picked.is_some() {
                self.stage = 1;
                input.eat();
            }
        }
        let caster = self.a.cursor.min(game.party.len() - 1);
        let skills = game.party[caster].skills();
        let rows: Vec<Row> = skills
            .iter()
            .map(|s| {
                let d = s.def();
                Row::new(d.name).icon(d.icon).right(format!("{} MP", d.mp)).enabled(d.usable_outside_battle() && game.party[caster].mp >= d.mp && game.party[caster].hp > 0)
            })
            .collect();
        let r = widgets::list(painter, gfx, b, &rows, &mut self.b, input, self.stage == 1, time);
        if let Some(s) = skills.get(self.b.cursor) {
            let d = s.def();
            gfx::wrapped(painter, pos2(b.left(), b.bottom() + 16.0), b.width(), d.desc, gfx::italic_font(19.0), PARCHMENT);
        }
        if self.stage == 1 {
            sfx(audio, &r);
            if r.picked.is_some() {
                self.stage = 2;
                self.c = ListState::default();
                input.eat();
            }
        }
        if self.stage == 2 {
            let skill = skills[self.b.cursor.min(skills.len().saturating_sub(1))];
            let d = skill.def();
            let rows: Vec<Row> = game.party.iter().map(|h| Row::new(h.name()).icon(h.def().sprite).right(format!("{}/{}", h.hp.max(0), h.max_hp()))).collect();
            let r = widgets::list(painter, gfx, c, &rows, &mut self.c, input, true, time);
            sfx(audio, &r);
            if let Some(t) = r.picked {
                // Field casting of healing skills.
                let mag = game.party[caster].stats().mag as f32;
                let targets: Vec<usize> = if d.target == Target::AllAllies { (0..game.party.len()).collect() } else { vec![t] };
                let mut did = false;
                for &t in &targets {
                    let h = &mut game.party[t];
                    match d.kind {
                        SkillKind::Heal { power, flat } if h.hp > 0 && h.hp < h.max_hp() => {
                            h.hp = (h.hp + (mag * power) as i32 + flat).min(h.max_hp());
                            did = true;
                        }
                        SkillKind::Revive { fraction } if h.hp <= 0 => {
                            h.hp = ((h.max_hp() as f32 * fraction) as i32).max(1);
                            did = true;
                        }
                        SkillKind::Cleanse => did = true,
                        _ => {}
                    }
                }
                if skill == crate::data::skills::SkillId::Lifebloom {
                    for h in &mut game.party {
                        if h.hp <= 0 {
                            h.hp = h.max_hp() / 2;
                            did = true;
                        }
                    }
                }
                if did {
                    game.party[caster].mp -= d.mp;
                    audio.play_sfx(Sfx::Heal);
                    self.say(format!("{} casts {}.", game.party[caster].name(), d.name));
                } else {
                    audio.play_sfx(Sfx::Denied);
                    self.say("It would have no effect.");
                }
            }
        }
        MenuOut::None
    }

    #[allow(clippy::too_many_arguments)]
    fn quests(&mut self, painter: &egui::Painter, gfx: &Gfx, body: Rect, input: &mut Input, game: &mut Game, audio: &mut Audio, time: f64) -> MenuOut {
        gfx::text(painter, body.left_top(), Align2::LEFT_TOP, "The Descent", gfx::heading_font(20.0), GOLD);
        gfx::text(painter, body.left_top() + vec2(0.0, 30.0), Align2::LEFT_TOP, game.objective(), gfx::italic_font(20.0), PARCHMENT);
        let quests: Vec<QuestId> = QuestId::ALL.into_iter().filter(|&q| game.quest_state(q) != QuestState::Hidden).collect();
        let left = Rect::from_min_max(body.left_top() + vec2(0.0, 80.0), pos2(body.left() + body.width() * 0.45, body.bottom()));
        let rows: Vec<Row> = quests
            .iter()
            .map(|&q| {
                let (tag, colour) = match game.quest_state(q) {
                    QuestState::Done => ("done", gfx::DIM),
                    QuestState::Ready => ("ready!", GOLD),
                    _ => ("active", PARCHMENT),
                };
                Row::new(q.def().name).right(tag).colour(colour)
            })
            .collect();
        let r = widgets::list(painter, gfx, left, &rows, &mut self.a, input, true, time);
        if r.moved {
            audio.play_sfx(Sfx::MenuMove);
        }
        if let Some(&q) = quests.get(self.a.cursor) {
            let d = q.def();
            let right = Rect::from_min_max(pos2(left.right() + 30.0, left.top()), body.max);
            gfx::text(painter, right.left_top(), Align2::LEFT_TOP, d.name, gfx::heading_font(22.0), GOLD);
            gfx::text(painter, right.left_top() + vec2(0.0, 32.0), Align2::LEFT_TOP, &format!("From {}", d.giver_name), gfx::italic_font(18.0), gfx::DIM);
            let mut y = right.top() + 66.0;
            y += gfx::wrapped(painter, pos2(right.left(), y), right.width(), d.summary, gfx::body_font(20.0), PARCHMENT) + 12.0;
            gfx::wrapped(painter, pos2(right.left(), y), right.width(), d.hint, gfx::italic_font(19.0), Color32::from_rgb(200, 220, 255));
            if let crate::data::quests::Requirement::Items(item, n) = d.requirement {
                if game.quest_state(q) != QuestState::Done {
                    gfx::text(painter, pos2(right.left(), right.bottom() - 30.0), Align2::LEFT_TOP, &format!("{}: {} / {n}", item.def().name, game.inventory.count(item).min(n)), gfx::body_font(20.0), GOLD);
                }
            }
        }
        MenuOut::None
    }

    #[allow(clippy::too_many_arguments)]
    fn journal(&mut self, painter: &egui::Painter, gfx: &Gfx, body: Rect, input: &mut Input, game: &mut Game, audio: &mut Audio, time: f64) -> MenuOut {
        let shards = game.shards();
        let journals = game.journals();
        let mut entries: Vec<(String, String, bool)> = Vec::new();
        for n in 1..=12u8 {
            let have = shards.contains(&n);
            entries.push((if have { format!("Memory Shard {n}") } else { "Memory Shard — ???".into() }, format!("shard_{n}"), have));
        }
        for n in 1..=5u8 {
            let have = journals.contains(&n);
            entries.push((if have { format!("Ilsa's Journal, page {n}") } else { "Journal page — ???".into() }, format!("journal_{n}"), have));
        }
        let left = Rect::from_min_size(body.min, vec2(body.width() * 0.45, body.height()));
        let rows: Vec<Row> = entries
            .iter()
            .map(|(label, id, have)| {
                let icon = if id.starts_with("shard") { crate::gfx::sprites::Sprite::MemoryShard } else { crate::gfx::sprites::Sprite::NoteScroll };
                Row::new(label.clone()).icon(icon).enabled(*have)
            })
            .collect();
        let r = widgets::list(painter, gfx, left, &rows, &mut self.a, input, true, time);
        sfx(audio, &r);
        let right = Rect::from_min_max(pos2(left.right() + 30.0, body.top()), body.max);
        gfx::text(painter, right.left_top(), Align2::LEFT_TOP, &format!("Memory Shards: {} / 12", shards.len()), gfx::heading_font(22.0), Color32::from_rgb(160, 210, 255));
        gfx::wrapped(
            painter,
            right.left_top() + vec2(0.0, 40.0),
            right.width(),
            "Crystallised fragments of someone's life, scattered through the Deep. They hum when you hold them, as if trying to remember something. Perhaps, gathered together, they could.",
            gfx::italic_font(20.0),
            PARCHMENT,
        );
        gfx::text(painter, right.left_top() + vec2(0.0, 170.0), Align2::LEFT_TOP, &format!("Ilsa's pages: {} / 5", journals.len()), gfx::heading_font(22.0), GOLD);
        let known = game.known_weak.len();
        gfx::text(painter, right.left_top() + vec2(0.0, 220.0), Align2::LEFT_TOP, &format!("Weaknesses discovered: {known}"), gfx::heading_font(18.0), gfx::DIM);
        gfx::text(painter, pos2(right.left(), body.bottom() - 30.0), Align2::LEFT_TOP, "Enter: read again", gfx::italic_font(17.0), gfx::DIM);
        if let Some(k) = r.picked {
            return MenuOut::Replay(entries[k].1.clone());
        }
        MenuOut::None
    }

    #[allow(clippy::too_many_arguments)]
    fn system(&mut self, painter: &egui::Painter, gfx: &Gfx, body: Rect, input: &mut Input, audio: &mut Audio, settings: &mut Settings, time: f64) -> MenuOut {
        let left = Rect::from_min_size(body.min + vec2(body.width() * 0.25, 20.0), vec2(body.width() * 0.5, body.height() - 20.0));
        if self.settings_open {
            return settings_panel(painter, gfx, left, input, audio, settings, &mut self.b, time);
        }
        let rows = vec![Row::new("Save game"), Row::new("Load game"), Row::new("Settings"), Row::new("Quit to title")];
        let r = widgets::list(painter, gfx, left, &rows, &mut self.a, input, true, time);
        sfx(audio, &r);
        match r.picked {
            Some(0) => MenuOut::Save(usize::MAX),
            Some(1) => MenuOut::Load(usize::MAX),
            Some(2) => {
                self.settings_open = true;
                self.b = ListState::default();
                input.eat();
                MenuOut::None
            }
            Some(3) => MenuOut::Title,
            _ => MenuOut::None,
        }
    }
}

/// Volume and speed settings; left/right adjust.
#[allow(clippy::too_many_arguments)]
pub fn settings_panel(painter: &egui::Painter, gfx: &Gfx, rect: Rect, input: &mut Input, audio: &mut Audio, settings: &mut Settings, list: &mut ListState, time: f64) -> MenuOut {
    let speed_name = |s: f32| match s as i32 {
        0..=35 => "Slow",
        36..=70 => "Normal",
        71..=500 => "Fast",
        _ => "Instant",
    };
    let rows = vec![
        Row::new("Music volume").right(format!("<  {:>3}%  >", (settings.music * 100.0).round())),
        Row::new("Effects volume").right(format!("<  {:>3}%  >", (settings.sfx * 100.0).round())),
        Row::new("Text speed").right(format!("<  {}  >", speed_name(settings.text_speed))),
        Row::new("Battle speed").right(format!("<  {:.1}×  >", settings.battle_speed)),
        Row::new("Fullscreen").right(if settings.fullscreen { "<  On  >" } else { "<  Off  >" }),
        Row::new("Back"),
    ];
    let r = widgets::list(painter, gfx, rect, &rows, list, input, true, time);
    if r.moved {
        audio.play_sfx(Sfx::MenuMove);
    }
    let delta = if input.left { -1.0 } else if input.right { 1.0 } else { 0.0 };
    let mut changed = false;
    if delta != 0.0 || r.picked.is_some_and(|i| i < 5) {
        let d = if delta == 0.0 { 1.0 } else { delta };
        match list.cursor {
            0 => settings.music = (settings.music + d * 0.1).clamp(0.0, 1.0),
            1 => settings.sfx = (settings.sfx + d * 0.1).clamp(0.0, 1.0),
            2 => {
                let steps = [30.0, 55.0, 110.0, 1000.0];
                let i = steps.iter().position(|&s| s >= settings.text_speed - 1.0).unwrap_or(1) as i32;
                settings.text_speed = steps[(i + d as i32).clamp(0, 3) as usize];
            }
            3 => settings.battle_speed = (settings.battle_speed + d * 0.5).clamp(1.0, 3.0),
            4 => settings.fullscreen = !settings.fullscreen,
            _ => {}
        }
        audio.play_sfx(Sfx::MenuMove);
        changed = true;
    }
    if r.picked == Some(5) {
        return MenuOut::Close;
    }
    let _ = Stroke::NONE;
    if changed { MenuOut::SettingsChanged } else { MenuOut::None }
}
