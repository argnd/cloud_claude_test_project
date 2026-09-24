//! Title screen, game over, endings and credits.

use eframe::egui::{self, Align2, Color32, CornerRadius, Rect, pos2, vec2};

use super::explore::ExploreView;
use super::input::Input;
use super::menus::{self, MenuOut};
use super::widgets::{self, ListState, Row};
use crate::audio::{Audio, Sfx};
use crate::game::{Difficulty, Game};
use crate::gfx::{self, GOLD, Gfx, PARCHMENT};
use crate::save::{self, Settings};
use crate::story::Ending;

pub enum TitleOut {
    None,
    NewGame(Difficulty),
    Chapter(u32),
    Load(usize),
    Quit,
    SettingsChanged,
}

#[derive(PartialEq)]
enum TitleMode {
    Main,
    Chapters,
    Difficulty,
    Load,
    Settings,
    Credits,
}

pub struct TitleView {
    backdrop: Game,
    view: ExploreView,
    mode: TitleMode,
    list: ListState,
    sub: ListState,
    age: f32,
    credits_t: f32,
}

/// Chapter select: jump into an act with a party ready for it.
pub const CHAPTERS: [(&str, u32); 6] = [
    ("I. The Undercroft", 1),
    ("II. The Drowned Archive", 5),
    ("III. The Mycelium Hollows", 9),
    ("IV. The Ember Forge", 13),
    ("V. The Pale Reach", 17),
    ("Finale: The Hearthflame", 20),
];

pub const CREDITS: &[(&str, &str)] = &[
    ("EMBERDEEP", "title"),
    ("The Last Lantern", "sub"),
    ("", ""),
    ("A proof of concept built with Claude Code", "line"),
    ("Design, engine, systems and story", "head"),
    ("Written in Rust with egui", "line"),
    ("", ""),
    ("Graphics", "head"),
    ("Dungeon Crawl Stone Soup tiles — CC0", "line"),
    (
        "by the DCSS artists and contributors (github.com/crawl/tiles)",
        "line",
    ),
    ("composited and recoloured for Emberdeep", "line"),
    ("", ""),
    ("Music and sound", "head"),
    (
        "Every note synthesized in real time by the game's own engine",
        "line",
    ),
    ("", ""),
    ("Fonts", "head"),
    (
        "Cinzel & Cinzel Decorative — Natanael Gama (SIL OFL)",
        "line",
    ),
    ("Crimson Text — Sebastian Kosch (SIL OFL)", "line"),
    ("", ""),
    ("Level generation", "head"),
    ("Grown from the rust_maze dungeon generator", "line"),
    ("", ""),
    ("Thank you for playing.", "sub"),
];

impl TitleView {
    pub fn new() -> Self {
        let mut backdrop = Game::new(Difficulty::Normal, 7);
        backdrop.world.player = (22, 20);
        backdrop.world.update_fov();
        let mut view = ExploreView::new();
        view.reset(&backdrop.world);
        Self {
            backdrop,
            view,
            mode: TitleMode::Main,
            list: ListState::default(),
            sub: ListState::default(),
            age: 0.0,
            credits_t: 0.0,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        painter: &egui::Painter,
        gfx: &Gfx,
        screen: Rect,
        input: &mut Input,
        audio: &mut Audio,
        settings: &mut Settings,
        time: f64,
    ) -> TitleOut {
        self.age += input.dt;
        // A slow drift across the sleeping town.
        let t = time as f32 * 0.05;
        let target = (22.0 + (t.sin() * 10.0), 15.0 + (t * 0.7).cos() * 6.0);
        let p = (target.0.round() as i32, target.1.round() as i32);
        if self.backdrop.world.tile(p).walkable()
            && p != self.backdrop.world.player
            && !self.view.moving()
        {
            self.view.snapshot(&self.backdrop.world);
            self.backdrop.world.player = p;
            self.backdrop.world.update_fov();
            self.view.stepped(&self.backdrop.world);
        }
        self.view.update(input.dt * 0.35);
        self.view
            .draw(painter, gfx, screen, &self.backdrop, time, input.dt);
        gfx::gradient(
            painter,
            Rect::from_min_max(screen.min, pos2(screen.right(), screen.center().y)),
            Color32::from_black_alpha(215),
            Color32::from_black_alpha(60),
        );
        gfx::gradient(
            painter,
            Rect::from_min_max(pos2(screen.left(), screen.center().y), screen.max),
            Color32::from_black_alpha(60),
            Color32::from_black_alpha(200),
        );

        let a = (self.age / 1.5).min(1.0);
        let logo_y = screen.top() + screen.height() * 0.22;
        let glow = 0.75 + 0.25 * (time * 1.3).sin() as f32;
        for k in 0..3 {
            let off = (k + 1) as f32 * 2.0;
            gfx::text(
                painter,
                pos2(screen.center().x, logo_y),
                Align2::CENTER_CENTER,
                "EMBERDEEP",
                gfx::title_font(92.0 + off),
                gfx::EMBER.gamma_multiply(0.12 * glow * a),
            );
        }
        gfx::text(
            painter,
            pos2(screen.center().x, logo_y),
            Align2::CENTER_CENTER,
            "EMBERDEEP",
            gfx::title_font(92.0),
            GOLD.gamma_multiply(a),
        );
        gfx::text(
            painter,
            pos2(screen.center().x, logo_y + 70.0),
            Align2::CENTER_CENTER,
            "The Last Lantern",
            gfx::heading_font(28.0),
            PARCHMENT.gamma_multiply(a),
        );

        let menu_rect = Rect::from_center_size(
            pos2(screen.center().x, screen.top() + screen.height() * 0.67),
            vec2(420.0, 300.0),
        );
        match self.mode {
            TitleMode::Main => {
                let has_save = save::any_save().is_some();
                let rows = vec![
                    Row::new("Continue").enabled(has_save),
                    Row::new("New Game"),
                    Row::new("Load Game").enabled(has_save),
                    Row::new("Chapter Select"),
                    Row::new("Settings"),
                    Row::new("Credits"),
                    Row::new("Quit"),
                ];
                if self.age < 0.2 && has_save {
                    self.list.cursor = 0;
                } else if self.age < 0.2 {
                    self.list.cursor = 1;
                }
                gfx::panel(painter, menu_rect);
                let r = widgets::list(
                    painter,
                    gfx,
                    menu_rect.shrink2(vec2(20.0, 20.0)),
                    &rows,
                    &mut self.list,
                    input,
                    self.age > 0.6,
                    time,
                );
                sfx(audio, &r);
                match r.picked {
                    Some(0) => return TitleOut::Load(save::any_save().unwrap_or(0)),
                    Some(1) => {
                        self.mode = TitleMode::Difficulty;
                        self.sub = ListState {
                            cursor: 1,
                            scroll: 0,
                        };
                    }
                    Some(2) => {
                        self.mode = TitleMode::Load;
                        self.sub = ListState::default();
                    }
                    Some(3) => {
                        self.mode = TitleMode::Chapters;
                        self.sub = ListState::default();
                    }
                    Some(4) => {
                        self.mode = TitleMode::Settings;
                        self.sub = ListState::default();
                    }
                    Some(5) => {
                        self.mode = TitleMode::Credits;
                        self.credits_t = 0.0;
                    }
                    Some(6) => return TitleOut::Quit,
                    _ => {}
                }
            }
            TitleMode::Difficulty => {
                let rect = menu_rect.expand2(vec2(100.0, 10.0));
                gfx::panel(painter, rect);
                gfx::text(
                    painter,
                    rect.center_top() + vec2(0.0, 26.0),
                    Align2::CENTER_CENTER,
                    "Choose your path",
                    gfx::heading_font(24.0),
                    GOLD,
                );
                let rows: Vec<Row> = Difficulty::ALL.iter().map(|d| Row::new(d.name())).collect();
                let inner = Rect::from_min_max(
                    rect.left_top() + vec2(20.0, 56.0),
                    pos2(rect.right() - 20.0, rect.top() + 170.0),
                );
                let r = widgets::list(painter, gfx, inner, &rows, &mut self.sub, input, true, time);
                sfx(audio, &r);
                let d = Difficulty::ALL[self.sub.cursor.min(2)];
                gfx::wrapped(
                    painter,
                    pos2(rect.left() + 30.0, rect.top() + 190.0),
                    rect.width() - 60.0,
                    d.blurb(),
                    gfx::italic_font(20.0),
                    PARCHMENT,
                );
                if let Some(i) = r.picked {
                    return TitleOut::NewGame(Difficulty::ALL[i]);
                }
                if input.cancel {
                    audio.play_sfx(Sfx::MenuBack);
                    self.mode = TitleMode::Main;
                }
            }
            TitleMode::Chapters => {
                let rect = Rect::from_center_size(menu_rect.center(), vec2(640.0, 330.0));
                gfx::panel(painter, rect);
                gfx::text(
                    painter,
                    rect.center_top() + vec2(0.0, 26.0),
                    Align2::CENTER_CENTER,
                    "Chapter Select",
                    gfx::heading_font(24.0),
                    GOLD,
                );
                let rows: Vec<Row> = CHAPTERS
                    .iter()
                    .map(|(name, floor)| Row::new(*name).right(format!("Floor {floor}")))
                    .collect();
                let r = widgets::list(
                    painter,
                    gfx,
                    Rect::from_min_max(
                        rect.left_top() + vec2(20.0, 56.0),
                        rect.right_bottom() - vec2(20.0, 14.0),
                    ),
                    &rows,
                    &mut self.sub,
                    input,
                    true,
                    time,
                );
                sfx(audio, &r);
                if let Some(i) = r.picked {
                    return TitleOut::Chapter(CHAPTERS[i].1);
                }
                if input.cancel {
                    audio.play_sfx(Sfx::MenuBack);
                    self.mode = TitleMode::Main;
                }
            }
            TitleMode::Load => {
                let rect = Rect::from_center_size(menu_rect.center(), vec2(760.0, 260.0));
                gfx::panel(painter, rect);
                let rows: Vec<Row> = (0..save::SLOTS)
                    .map(|s| {
                        let desc = save::describe(s);
                        Row::new(if s == 0 {
                            "Autosave".to_string()
                        } else {
                            format!("Slot {s}")
                        })
                        .right(desc.clone().unwrap_or_else(|| "— empty —".into()))
                        .enabled(desc.is_some())
                    })
                    .collect();
                let r = widgets::list(
                    painter,
                    gfx,
                    rect.shrink2(vec2(20.0, 30.0)),
                    &rows,
                    &mut self.sub,
                    input,
                    true,
                    time,
                );
                sfx(audio, &r);
                if let Some(s) = r.picked {
                    return TitleOut::Load(s);
                }
                if input.cancel {
                    self.mode = TitleMode::Main;
                    audio.play_sfx(Sfx::MenuBack);
                }
            }
            TitleMode::Settings => {
                let rect = Rect::from_center_size(menu_rect.center(), vec2(560.0, 280.0));
                gfx::panel(painter, rect);
                match menus::settings_panel(
                    painter,
                    gfx,
                    rect.shrink2(vec2(20.0, 30.0)),
                    input,
                    audio,
                    settings,
                    &mut self.sub,
                    time,
                ) {
                    MenuOut::Close => self.mode = TitleMode::Main,
                    MenuOut::SettingsChanged => return TitleOut::SettingsChanged,
                    _ => {}
                }
                if input.cancel {
                    self.mode = TitleMode::Main;
                }
            }
            TitleMode::Credits => {
                self.credits_t += input.dt;
                painter.rect_filled(screen, CornerRadius::ZERO, Color32::from_black_alpha(200));
                draw_credits(painter, screen, self.credits_t);
                if input.cancel || input.confirm || self.credits_t > 45.0 {
                    self.mode = TitleMode::Main;
                }
            }
        }
        gfx::text(
            painter,
            pos2(screen.center().x, screen.bottom() - 20.0),
            Align2::CENTER_BOTTOM,
            "Arrows to choose · Enter to confirm · Esc to go back",
            gfx::body_font(16.0),
            Color32::from_white_alpha(80),
        );
        TitleOut::None
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

pub fn draw_credits(painter: &egui::Painter, screen: Rect, t: f32) {
    let mut y = screen.bottom() + 40.0 - t * 45.0;
    for (text, kind) in CREDITS {
        let (font, colour, step) = match *kind {
            "title" => (gfx::title_font(64.0), GOLD, 80.0),
            "sub" => (gfx::heading_font(26.0), PARCHMENT, 52.0),
            "head" => (gfx::heading_font(22.0), GOLD, 38.0),
            "line" => (gfx::body_font(22.0), PARCHMENT, 32.0),
            _ => (gfx::body_font(20.0), PARCHMENT, 30.0),
        };
        if y > screen.top() - 80.0 && y < screen.bottom() + 80.0 {
            gfx::text(
                painter,
                pos2(screen.center().x, y),
                Align2::CENTER_CENTER,
                text,
                font,
                colour,
            );
        }
        y += step;
    }
}

pub struct GameOverView {
    list: ListState,
    age: f32,
}

pub enum GameOverOut {
    None,
    Retry,
    Title,
}

impl GameOverView {
    pub fn new() -> Self {
        Self {
            list: ListState::default(),
            age: 0.0,
        }
    }

    pub fn show(
        &mut self,
        painter: &egui::Painter,
        gfx: &Gfx,
        screen: Rect,
        input: &mut Input,
        audio: &mut Audio,
        time: f64,
    ) -> GameOverOut {
        self.age += input.dt;
        gfx::gradient(
            painter,
            screen,
            Color32::from_rgb(20, 4, 6),
            Color32::from_rgb(4, 2, 4),
        );
        let a = (self.age / 1.2).min(1.0);
        gfx::text(
            painter,
            pos2(screen.center().x, screen.top() + screen.height() * 0.3),
            Align2::CENTER_CENTER,
            "The Lantern Gutters",
            gfx::title_font(60.0),
            Color32::from_rgb(220, 100, 80).gamma_multiply(a),
        );
        gfx::text(
            painter,
            pos2(
                screen.center().x,
                screen.top() + screen.height() * 0.3 + 60.0,
            ),
            Align2::CENTER_CENTER,
            "But a lamplighter knows: every flame can be lit again.",
            gfx::italic_font(24.0),
            PARCHMENT.gamma_multiply(a),
        );
        let rect = Rect::from_center_size(
            pos2(screen.center().x, screen.top() + screen.height() * 0.62),
            vec2(440.0, 130.0),
        );
        gfx::panel(painter, rect);
        let rows = vec![
            Row::new("Rise again (last save)").enabled(save::any_save().is_some()),
            Row::new("Return to title"),
        ];
        let r = widgets::list(
            painter,
            gfx,
            rect.shrink2(vec2(20.0, 26.0)),
            &rows,
            &mut self.list,
            input,
            self.age > 1.0,
            time,
        );
        sfx(audio, &r);
        match r.picked {
            Some(0) => GameOverOut::Retry,
            Some(1) => GameOverOut::Title,
            _ => GameOverOut::None,
        }
    }
}

pub struct EndingView {
    pub ending: Ending,
    age: f32,
    stats: Vec<String>,
}

impl EndingView {
    pub fn new(ending: Ending, game: &Game) -> Self {
        let t = game.playtime as u64;
        let stats = vec![
            format!("Time in the Deep: {}h {:02}m", t / 3600, (t / 60) % 60),
            format!("Wren reached level {}", game.party[0].level),
            format!(
                "Battles won: {} · Foes defeated: {}",
                game.stats.battles_won, game.stats.enemies_defeated
            ),
            format!(
                "Memory Shards: {} / 12 · Journal pages: {} / 5",
                game.shards().len(),
                game.journals().len()
            ),
            format!("Difficulty: {}", game.difficulty.name()),
        ];
        Self {
            ending,
            age: 0.0,
            stats,
        }
    }

    /// Returns true when the player is done.
    pub fn show(&mut self, painter: &egui::Painter, screen: Rect, input: &mut Input) -> bool {
        self.age += input.dt;
        let (title, colour, top) = match self.ending {
            Ending::Oath => (
                "The Warden's Oath",
                Color32::from_rgb(255, 180, 90),
                Color32::from_rgb(30, 14, 8),
            ),
            Ending::Dark => (
                "Let It Go Dark",
                Color32::from_rgb(170, 190, 230),
                Color32::from_rgb(6, 8, 16),
            ),
            Ending::Dawn => (
                "Dawn Below",
                Color32::from_rgb(255, 230, 150),
                Color32::from_rgb(60, 40, 70),
            ),
        };
        gfx::gradient(painter, screen, top, Color32::from_rgb(4, 3, 6));
        if self.age < 12.0 {
            let a = (self.age / 1.5).min(1.0) * ((12.0 - self.age) / 1.5).clamp(0.0, 1.0);
            gfx::text(
                painter,
                pos2(screen.center().x, screen.top() + screen.height() * 0.28),
                Align2::CENTER_CENTER,
                "Ending",
                gfx::heading_font(24.0),
                PARCHMENT.gamma_multiply(a),
            );
            gfx::text(
                painter,
                pos2(
                    screen.center().x,
                    screen.top() + screen.height() * 0.28 + 56.0,
                ),
                Align2::CENTER_CENTER,
                title,
                gfx::title_font(60.0),
                colour.gamma_multiply(a),
            );
            for (k, s) in self.stats.iter().enumerate() {
                let fade = ((self.age - 1.5 - k as f32 * 0.4) / 0.8).clamp(0.0, 1.0) * a;
                gfx::text(
                    painter,
                    pos2(
                        screen.center().x,
                        screen.top() + screen.height() * 0.5 + k as f32 * 34.0,
                    ),
                    Align2::CENTER_CENTER,
                    s,
                    gfx::body_font(23.0),
                    PARCHMENT.gamma_multiply(fade),
                );
            }
            if self.ending != Ending::Dawn && self.age > 5.0 {
                gfx::text(
                    painter,
                    pos2(screen.center().x, screen.bottom() - 70.0),
                    Align2::CENTER_CENTER,
                    "There may be another way. Every memory matters.",
                    gfx::italic_font(20.0),
                    Color32::from_rgb(160, 200, 255).gamma_multiply(a),
                );
            }
        } else {
            draw_credits(painter, screen, self.age - 12.0);
        }
        (self.age > 2.0 && (input.confirm && self.age > 12.0))
            || self.age > 55.0
            || (input.cancel && self.age > 3.0)
    }
}
