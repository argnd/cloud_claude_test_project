//! Dialogue boxes and cinematic narration, driven by a story `Runner`.

use eframe::egui::{self, Align2, Color32, CornerRadius, Rect, Stroke, pos2, vec2};

use super::input::Input;
use super::widgets::{self, ListState, Row};
use crate::audio::{Audio, Sfx};
use crate::data::enemies::BattleId;
use crate::game::Game;
use crate::gfx::{self, GOLD, Gfx, PARCHMENT};
use crate::story::{self, Beat, Ending, Runner};

pub enum DialogueOut {
    Nothing,
    Battle(BattleId),
    Ending(Ending),
    Finished,
}

#[derive(Clone, Copy, PartialEq)]
enum Style {
    Box,
    Cinema,
    Memory,
    Journal,
}

pub struct Dialogue {
    pub runner: Runner,
    beat: Option<Beat>,
    shown: f32,
    choice: ListState,
    /// Suspended while a battle it started is fought.
    pub waiting: bool,
    blip: f32,
    age: f32,
    line_age: f32,
    /// Scenes this dialogue passed through (for "seen" bookkeeping).
    pub visited: Vec<String>,
}

fn style_for(scene: &str) -> Style {
    if scene.starts_with("shard_") {
        Style::Memory
    } else if scene.starts_with("journal_") {
        Style::Journal
    } else if scene == "intro" || scene.starts_with("ending_") || scene == "credits" {
        Style::Cinema
    } else {
        Style::Box
    }
}

impl Dialogue {
    pub fn new(scene: &str) -> Self {
        Self {
            runner: Runner::new(scene),
            beat: None,
            shown: 0.0,
            choice: ListState::default(),
            waiting: false,
            blip: 0.0,
            age: 0.0,
            line_age: 0.0,
            visited: vec![scene.to_string()],
        }
    }

    /// A menu of choices is on screen.
    #[cfg(test)]
    pub fn is_choice(&self) -> bool {
        matches!(self.beat, Some(Beat::Choice(_)))
    }

    pub fn resume(&mut self) {
        self.waiting = false;
    }

    fn fetch(&mut self, game: &mut Game) -> Option<DialogueOut> {
        let beat = self.runner.next(story::script(), game);
        if self.visited.last() != Some(&self.runner.scene) {
            self.visited.push(self.runner.scene.clone());
        }
        self.shown = 0.0;
        self.line_age = 0.0;
        self.choice = ListState::default();
        match beat {
            Beat::Battle(b) => {
                self.waiting = true;
                self.beat = None;
                Some(DialogueOut::Battle(b))
            }
            Beat::Ending(e) => {
                self.beat = None;
                Some(DialogueOut::Ending(e))
            }
            Beat::Done => {
                self.beat = None;
                Some(DialogueOut::Finished)
            }
            other => {
                self.beat = Some(other);
                None
            }
        }
    }

    /// Updates and draws; returns what the app must do next.
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        painter: &egui::Painter,
        gfx: &Gfx,
        screen: Rect,
        input: &mut Input,
        game: &mut Game,
        audio: &mut Audio,
        text_speed: f32,
        time: f64,
    ) -> DialogueOut {
        if self.waiting {
            return DialogueOut::Nothing;
        }
        self.age += input.dt;
        self.line_age += input.dt;
        if self.beat.is_none() {
            if let Some(out) = self.fetch(game) {
                return out;
            }
        }
        let style = style_for(&self.runner.scene);
        let beat = self.beat.clone().unwrap();
        match &beat {
            Beat::Line { speaker, text } => {
                let len = text.chars().count() as f32;
                let before = self.shown as usize;
                let speed = if input.fast { 2000.0 } else { text_speed };
                self.shown = (self.shown + input.dt * speed).min(len);
                if self.shown as usize > before && style != Style::Journal {
                    self.blip -= (self.shown as usize - before) as f32;
                    if self.blip <= 0.0 {
                        audio.play_sfx(Sfx::TextBlip);
                        self.blip = 3.0;
                    }
                }
                let complete = self.shown >= len;
                let visible: String = text.chars().take(self.shown as usize).collect();
                self.draw_line(painter, gfx, screen, style, speaker, &visible, complete, time);
                if (input.confirm || input.click) && self.line_age > 0.12 {
                    if !complete {
                        self.shown = len;
                    } else if let Some(out) = self.fetch(game) {
                        return out;
                    }
                }
            }
            Beat::Choice(options) => {
                let rows: Vec<Row> = options.iter().map(|(t, _)| Row::new(t.clone())).collect();
                let h = rows.len() as f32 * 34.0 + 24.0;
                let w = 620.0f32.min(screen.width() - 80.0);
                let rect = Rect::from_center_size(pos2(screen.center().x, screen.bottom() - 240.0 - h / 2.0), vec2(w, h));
                if style != Style::Box {
                    painter.rect_filled(screen, CornerRadius::ZERO, Color32::from_black_alpha(200));
                }
                gfx::panel(painter, rect);
                let r = widgets::list(painter, gfx, rect.shrink2(vec2(12.0, 12.0)), &rows, &mut self.choice, input, true, time);
                if r.moved {
                    audio.play_sfx(Sfx::MenuMove);
                }
                if let Some(i) = r.picked {
                    audio.play_sfx(Sfx::MenuSelect);
                    let target = options[i].1.clone();
                    self.runner.choose(&target);
                    if let Some(out) = self.fetch(game) {
                        return out;
                    }
                }
            }
            _ => {}
        }
        DialogueOut::Nothing
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_line(&self, painter: &egui::Painter, gfx: &Gfx, screen: Rect, style: Style, speaker: &str, text: &str, complete: bool, time: f64) {
        let (name, portrait) = story::speaker_info(speaker).unwrap_or(("", None));
        let blink = ((time * 3.0) as i64 % 2 == 0) && complete;
        match style {
            Style::Box => {
                let h = 190.0;
                let rect = Rect::from_min_size(pos2(screen.left() + 40.0, screen.bottom() - h - 28.0), vec2(screen.width() - 80.0, h));
                gfx::panel(painter, rect);
                let mut text_left = rect.left() + 28.0;
                if let Some(p) = portrait {
                    let pr = Rect::from_min_size(rect.left_top() + vec2(18.0, 20.0), vec2(150.0, 150.0));
                    gfx::portrait(gfx, painter, pr, p, false);
                    text_left = pr.right() + 26.0;
                }
                if !name.is_empty() {
                    let plate = Rect::from_min_size(pos2(text_left - 8.0, rect.top() - 20.0), vec2(name.len() as f32 * 14.0 + 40.0, 38.0));
                    painter.rect_filled(plate, CornerRadius::same(6), Color32::from_rgb(40, 28, 22));
                    painter.rect_stroke(plate, CornerRadius::same(6), Stroke::new(1.5, GOLD), egui::StrokeKind::Inside);
                    gfx::text(painter, plate.center(), Align2::CENTER_CENTER, name, gfx::heading_font(21.0), GOLD);
                }
                let font = if speaker == "narrator" { gfx::italic_font(25.0) } else { gfx::body_font(25.0) };
                let colour = if speaker == "narrator" { Color32::from_rgb(210, 200, 230) } else { PARCHMENT };
                gfx::wrapped(painter, pos2(text_left, rect.top() + 30.0), rect.right() - text_left - 30.0, text, font, colour);
                if blink {
                    gfx::triangle(painter, rect.right_bottom() - vec2(26.0, 20.0), 14.0, true, GOLD);
                }
            }
            Style::Cinema | Style::Memory | Style::Journal => {
                let overlay = match style {
                    Style::Memory => Color32::from_rgba_unmultiplied(10, 22, 40, 225),
                    _ => Color32::from_rgba_unmultiplied(4, 3, 6, 235),
                };
                painter.rect_filled(screen, CornerRadius::ZERO, overlay);
                if style == Style::Memory {
                    // Drifting motes of remembered light.
                    for i in 0..40 {
                        let f = i as f32 * 12.9898;
                        let x = screen.left() + ((f.sin() * 43758.545).fract().abs()) * screen.width();
                        let speed = 12.0 + (i % 7) as f32 * 5.0;
                        let y = screen.bottom() - ((time as f32 * speed + i as f32 * 97.0) % screen.height());
                        let a = (0.3 + 0.3 * ((time as f32 + f).sin())).max(0.0);
                        painter.circle_filled(pos2(x, y), 1.5 + (i % 3) as f32, Color32::from_rgba_unmultiplied(170, 210, 255, (a * 255.0) as u8));
                    }
                    gfx::text(painter, pos2(screen.center().x, screen.top() + 60.0), Align2::CENTER_CENTER, "— A Memory —", gfx::heading_font(22.0), Color32::from_rgb(160, 200, 255));
                }
                let width = (screen.width() - 200.0).min(900.0);
                if style == Style::Journal {
                    let page = Rect::from_center_size(screen.center(), vec2(width + 80.0, 420.0));
                    painter.rect_filled(page, CornerRadius::same(4), Color32::from_rgb(226, 212, 180));
                    painter.rect_stroke(page, CornerRadius::same(4), Stroke::new(2.0, Color32::from_rgb(120, 90, 60)), egui::StrokeKind::Inside);
                    gfx::text(painter, pos2(page.center().x, page.top() + 36.0), Align2::CENTER_CENTER, "From Ilsa's journal", gfx::heading_font(20.0), Color32::from_rgb(110, 70, 40));
                    let est = painter.layout(text.to_string(), gfx::italic_font(27.0), Color32::BLACK, width).size().y;
                    gfx::centred(painter, page.center().x, page.center().y - est / 2.0 + 10.0, width, text, gfx::italic_font(27.0), Color32::from_rgb(50, 36, 30));
                    if blink {
                        gfx::triangle(painter, page.right_bottom() - vec2(30.0, 24.0), 14.0, true, Color32::from_rgb(110, 70, 40));
                    }
                    return;
                }
                let mut y = screen.center().y - 40.0;
                if let Some(p) = portrait {
                    let pr = Rect::from_center_size(pos2(screen.center().x, y - 110.0), vec2(120.0, 120.0));
                    gfx::portrait(gfx, painter, pr, p, false);
                    gfx::text(painter, pos2(screen.center().x, pr.bottom() + 20.0), Align2::CENTER_CENTER, name, gfx::heading_font(20.0), GOLD);
                    y += 10.0;
                } else if !name.is_empty() {
                    gfx::text(painter, pos2(screen.center().x, y - 40.0), Align2::CENTER_CENTER, name, gfx::heading_font(20.0), Color32::from_rgb(190, 215, 255));
                }
                let font = if speaker == "narrator" || speaker == "lira" { gfx::italic_font(29.0) } else { gfx::body_font(28.0) };
                let colour = if style == Style::Memory { Color32::from_rgb(214, 230, 255) } else { PARCHMENT };
                gfx::centred(painter, screen.center().x, y, width, text, font, colour);
                if blink {
                    gfx::triangle(painter, pos2(screen.center().x, screen.bottom() - 50.0), 16.0, true, GOLD);
                }
            }
        }
    }
}
