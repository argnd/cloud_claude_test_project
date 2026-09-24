// No console window for the shipped exe; debug builds keep it so panics stay visible.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod dungeon;
mod game;
mod ui;

use eframe::egui;
use ui::CrawlerApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 760.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Dungeon crawler",
        options,
        Box::new(|cc| Ok(Box::new(CrawlerApp::new(cc)))),
    )
}
