// No console window for the shipped exe; debug builds keep it so panics stay visible.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod audio;
mod battle;
mod data;
mod dungeon;
mod game;
mod gfx;
mod rpg;
mod save;
mod story;
mod world;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Emberdeep — The Last Lantern")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([960.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Emberdeep",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
