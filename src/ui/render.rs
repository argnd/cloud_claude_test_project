use eframe::egui;

use super::tiles::Tiles;
use crate::dungeon::grid::Cell;
use crate::game::Game;

/// Tile size when the whole level would only fit with smaller tiles.
const TILE_SIZE: f32 = 32.0;
/// Smallest tile size used to show the whole level at once.
const MIN_FIT: f32 = 16.0;
/// Squares remembered but out of sight are drawn this dark.
const MEMORY_TINT: egui::Color32 = egui::Color32::from_gray(90);

/// Paints the level into a `size` area: the whole of it when it fits,
/// otherwise a window that follows the player. Unexplored squares stay
/// black; monsters and items show only while in sight.
pub fn draw_level(ui: &mut egui::Ui, game: &Game, tiles: &Tiles, size: egui::Vec2) {
    let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
    let view = response.rect;
    painter.rect_filled(view, egui::CornerRadius::ZERO, egui::Color32::BLACK);

    let (w, h) = (game.grid.width(), game.grid.height());
    let fit = (view.width() / w as f32)
        .min(view.height() / h as f32)
        .floor();
    let cell = if fit >= MIN_FIT {
        fit.min(48.0)
    } else {
        TILE_SIZE
    };

    let offset = egui::vec2(
        camera(
            view.width(),
            w as f32 * cell,
            game.player.pos.0 as f32,
            cell,
        ),
        camera(
            view.height(),
            h as f32 * cell,
            game.player.pos.1 as f32,
            cell,
        ),
    );
    let origin = view.min + offset;
    let cell_rect = |x: usize, y: usize| {
        let min = origin + egui::vec2(x as f32 * cell, y as f32 * cell);
        egui::Rect::from_min_size(min, egui::vec2(cell, cell))
    };
    let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));

    for y in 0..h {
        for x in 0..w {
            if !game.is_explored(x, y) {
                continue;
            }
            let rect = cell_rect(x, y);
            if !view.intersects(rect) {
                continue;
            }
            let tint = if game.is_visible(x, y) {
                egui::Color32::WHITE
            } else {
                MEMORY_TINT
            };
            let square = game.grid.get(x, y);
            if square != Cell::Wall && square != Cell::Floor {
                painter.image(tiles.floor(), rect, uv, tint);
            }
            painter.image(tiles.for_cell(square), rect, uv, tint);
        }
    }

    for item in &game.items {
        if game.is_visible(item.pos.0, item.pos.1) {
            let rect = cell_rect(item.pos.0, item.pos.1);
            painter.image(tiles.for_item(item.kind), rect, uv, egui::Color32::WHITE);
        }
    }
    for monster in &game.monsters {
        if game.is_visible(monster.pos.0, monster.pos.1) {
            let rect = cell_rect(monster.pos.0, monster.pos.1);
            painter.image(
                tiles.for_monster(monster.kind),
                rect,
                uv,
                egui::Color32::WHITE,
            );
            health_bar(&painter, rect, monster.hp, monster.kind.max_hp());
        }
    }
    let rect = cell_rect(game.player.pos.0, game.player.pos.1);
    painter.image(tiles.player(), rect, uv, egui::Color32::WHITE);
}

/// Offset of the level inside the view along one axis: centred when it
/// fits, otherwise centred on the player but never past the level's edges.
fn camera(view: f32, level: f32, player: f32, cell: f32) -> f32 {
    if level <= view {
        ((view - level) / 2.0).floor()
    } else {
        (view / 2.0 - (player + 0.5) * cell)
            .clamp(view - level, 0.0)
            .floor()
    }
}

/// A thin bar over wounded monsters.
fn health_bar(painter: &egui::Painter, rect: egui::Rect, hp: i32, max_hp: i32) {
    if hp >= max_hp {
        return;
    }
    let height = (rect.height() / 10.0).max(2.0);
    let back = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), height));
    let share = hp.max(0) as f32 / max_hp as f32;
    let front = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() * share, height));
    painter.rect_filled(
        back,
        egui::CornerRadius::ZERO,
        egui::Color32::from_rgb(90, 20, 20),
    );
    painter.rect_filled(
        front,
        egui::CornerRadius::ZERO,
        egui::Color32::from_rgb(220, 50, 50),
    );
}
