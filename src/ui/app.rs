use eframe::egui;
use rand::RngExt;

use super::render::draw_level;
use super::tiles::Tiles;
use crate::dungeon::carvers::Algorithm;
use crate::game::{Action, Direction, FINAL_DEPTH, Game, State};

/// Lines of the message log shown under the map.
const LOG_LINES: usize = 5;

const CONTROLS: &str = "Arrows / WASD / HJKL: move, open doors, attack\n\
                        Space or . : wait a turn\n\
                        Enter or > : take the stairs down\n\
                        Walk over potions and gold to pick them up.";

enum Screen {
    Menu,
    Playing(Box<Game>),
}

pub struct CrawlerApp {
    screen: Screen,
    algorithm: Algorithm,
    tiles: Tiles,
}

impl CrawlerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            screen: Screen::Menu,
            algorithm: Algorithm::Backtracker,
            tiles: Tiles::load(&cc.egui_ctx),
        }
    }

    fn new_game(&mut self) {
        let seed = rand::rng().random();
        self.screen = Screen::Playing(Box::new(Game::new(self.algorithm, seed)));
    }

    fn menu(&mut self, ui: &mut egui::Ui) {
        ui.heading("Dungeon crawler");
        ui.label(format!(
            "Reach the stairs of depth {FINAL_DEPTH} to escape. Monsters get tougher the deeper you go."
        ));
        ui.add_space(8.0);
        egui::ComboBox::from_label("Corridors")
            .selected_text(self.algorithm.label())
            .show_ui(ui, |ui| {
                for algorithm in Algorithm::ALL {
                    ui.selectable_value(&mut self.algorithm, algorithm, algorithm.label());
                }
            });
        ui.add_space(8.0);
        if ui.button("New game").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            self.new_game();
        }
        ui.add_space(16.0);
        ui.label(CONTROLS);
    }
}

/// The action for the first game key pressed this frame, if any.
fn read_action(ctx: &egui::Context) -> Option<Action> {
    use egui::Key;
    ctx.input(|input| {
        for event in &input.events {
            match event {
                egui::Event::Key {
                    key, pressed: true, ..
                } => {
                    let action = match key {
                        Key::ArrowUp | Key::W | Key::K => Action::Move(Direction::Up),
                        Key::ArrowDown | Key::S | Key::J => Action::Move(Direction::Down),
                        Key::ArrowLeft | Key::A | Key::H => Action::Move(Direction::Left),
                        Key::ArrowRight | Key::D | Key::L => Action::Move(Direction::Right),
                        Key::Space => Action::Wait,
                        Key::Enter => Action::Descend,
                        _ => continue,
                    };
                    return Some(action);
                }
                egui::Event::Text(text) if text == "." => return Some(Action::Wait),
                egui::Event::Text(text) if text == ">" => return Some(Action::Descend),
                _ => {}
            }
        }
        None
    })
}

fn hud(ui: &mut egui::Ui, game: &Game) {
    ui.horizontal(|ui| {
        ui.label(format!("Depth {}/{FINAL_DEPTH}", game.depth));
        ui.separator();
        let share = game.player.hp as f32 / game.player.max_hp as f32;
        ui.add(
            egui::ProgressBar::new(share)
                .desired_width(160.0)
                .fill(egui::Color32::from_rgb(170, 40, 40))
                .text(format!("HP {}/{}", game.player.hp, game.player.max_hp)),
        );
        ui.separator();
        ui.label(format!("Attack {}", game.player.attack));
        ui.separator();
        ui.label(format!("Gold {}", game.player.gold));
        ui.separator();
        ui.label(format!("Kills {}", game.player.kills));
        ui.separator();
        ui.label(format!("Turn {}", game.turns));
    });
}

fn message_log(ui: &mut egui::Ui, game: &Game) {
    let start = game.log.len().saturating_sub(LOG_LINES);
    let lines = &game.log[start..];
    for (i, line) in lines.iter().enumerate() {
        // Older lines fade out.
        let age = lines.len() - 1 - i;
        let gray = 230u8.saturating_sub(age as u8 * 35);
        ui.label(egui::RichText::new(line).color(egui::Color32::from_gray(gray)));
    }
}

impl eframe::App for CrawlerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            let mut to_menu = false;
            let mut restart = false;
            match &mut self.screen {
                Screen::Menu => self.menu(ui),
                Screen::Playing(game) => {
                    // Ignored once the game is over.
                    if let Some(action) = read_action(ui.ctx()) {
                        game.act(action);
                    }

                    ui.horizontal(|ui| {
                        if ui.button("Menu").clicked() {
                            to_menu = true;
                        }
                        ui.separator();
                        hud(ui, game);
                    });

                    let log_height = LOG_LINES as f32 * 20.0 + 8.0;
                    let map_size = ui.available_size() - egui::vec2(0.0, log_height);
                    draw_level(ui, game, &self.tiles, map_size.max(egui::Vec2::ZERO));
                    message_log(ui, game);

                    if game.state != State::Playing {
                        let title = if game.state == State::Won {
                            "You escaped!"
                        } else {
                            "You died"
                        };
                        egui::Window::new(title)
                            .collapsible(false)
                            .resizable(false)
                            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                            .show(ui.ctx(), |ui| {
                                ui.label(format!(
                                    "Depth {}, {} gold, {} kills, {} turns.",
                                    game.depth, game.player.gold, game.player.kills, game.turns
                                ));
                                ui.horizontal(|ui| {
                                    if ui.button("New game").clicked() {
                                        restart = true;
                                    }
                                    if ui.button("Menu").clicked() {
                                        to_menu = true;
                                    }
                                });
                            });
                    }
                }
            }
            if restart {
                self.new_game();
            } else if to_menu {
                self.screen = Screen::Menu;
            }
        });
    }
}
