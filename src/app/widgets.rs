//! A keyboard-and-mouse list, the building block of every menu.

use eframe::egui::{self, Align2, Color32, CornerRadius, Rect, Stroke, pos2, vec2};

use super::input::Input;
use crate::gfx::sprites::Sprite;
use crate::gfx::{self, GOLD, Gfx, PARCHMENT};

pub struct Row {
    pub label: String,
    pub right: String,
    pub icon: Option<Sprite>,
    pub enabled: bool,
    pub colour: Option<Color32>,
}

impl Row {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            right: String::new(),
            icon: None,
            enabled: true,
            colour: None,
        }
    }
    pub fn right(mut self, right: impl Into<String>) -> Self {
        self.right = right.into();
        self
    }
    pub fn icon(mut self, icon: Sprite) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn colour(mut self, colour: Color32) -> Self {
        self.colour = Some(colour);
        self
    }
}

#[derive(Default, Clone, Copy)]
pub struct ListState {
    pub cursor: usize,
    pub scroll: usize,
}

#[derive(Default)]
pub struct ListResult {
    pub moved: bool,
    pub picked: Option<usize>,
    /// Picked but disabled.
    pub denied: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn list(
    painter: &egui::Painter,
    gfx: &Gfx,
    rect: Rect,
    rows: &[Row],
    state: &mut ListState,
    input: &mut Input,
    active: bool,
    time: f64,
) -> ListResult {
    let mut result = ListResult::default();
    let row_h = 34.0;
    let visible = ((rect.height() / row_h).floor() as usize).max(1);
    if rows.is_empty() {
        gfx::text(
            painter,
            rect.left_top() + vec2(12.0, 8.0),
            Align2::LEFT_TOP,
            "(nothing)",
            gfx::italic_font(19.0),
            gfx::DIM,
        );
        return result;
    }
    state.cursor = state.cursor.min(rows.len() - 1);
    if active {
        if input.up {
            state.cursor = (state.cursor + rows.len() - 1) % rows.len();
            result.moved = true;
        }
        if input.down {
            state.cursor = (state.cursor + 1) % rows.len();
            result.moved = true;
        }
        if input.scroll.abs() > 1.0 && input.pointer.is_some_and(|p| rect.contains(p)) {
            let max_scroll = rows.len().saturating_sub(visible);
            if input.scroll > 0.0 {
                state.scroll = state.scroll.saturating_sub(1);
            } else {
                state.scroll = (state.scroll + 1).min(max_scroll);
            }
        }
    }
    if state.cursor < state.scroll {
        state.scroll = state.cursor;
    }
    if state.cursor >= state.scroll + visible {
        state.scroll = state.cursor + 1 - visible;
    }
    for (slot, i) in (state.scroll..rows.len().min(state.scroll + visible)).enumerate() {
        let row = &rows[i];
        let r = Rect::from_min_size(
            pos2(rect.left(), rect.top() + slot as f32 * row_h),
            vec2(rect.width(), row_h - 2.0),
        );
        if active {
            if let Some(p) = input.pointer
                && r.contains(p)
                && state.cursor != i
                && !input.up
                && !input.down
            {
                state.cursor = i;
                result.moved = true;
            }
            if input.click && input.pointer.is_some_and(|p| r.contains(p)) {
                state.cursor = i;
                if row.enabled {
                    result.picked = Some(i);
                } else {
                    result.denied = true;
                }
            }
        }
        let selected = i == state.cursor;
        if selected {
            let pulse = (time * 4.0).sin() as f32 * 0.5 + 0.5;
            let fill = if active {
                GOLD.gamma_multiply(0.16 + pulse * 0.08)
            } else {
                GOLD.gamma_multiply(0.07)
            };
            painter.rect_filled(r, CornerRadius::same(5), fill);
            if active {
                painter.rect_stroke(
                    r,
                    CornerRadius::same(5),
                    Stroke::new(1.0, GOLD.gamma_multiply(0.6)),
                    egui::StrokeKind::Inside,
                );
                let tri_x = r.left() + 6.0 + pulse * 3.0;
                let c = r.center().y;
                painter.add(egui::Shape::convex_polygon(
                    vec![
                        pos2(tri_x, c - 6.0),
                        pos2(tri_x + 8.0, c),
                        pos2(tri_x, c + 6.0),
                    ],
                    GOLD,
                    Stroke::NONE,
                ));
            }
        }
        let mut x = r.left() + 22.0;
        if let Some(icon) = row.icon {
            let ir = Rect::from_center_size(pos2(x + 13.0, r.center().y), vec2(28.0, 28.0));
            gfx.draw(
                painter,
                icon,
                ir,
                if row.enabled {
                    Color32::WHITE
                } else {
                    Color32::from_gray(110)
                },
            );
            x += 34.0;
        }
        let colour = if !row.enabled {
            Color32::from_gray(105)
        } else {
            row.colour
                .unwrap_or(if selected { Color32::WHITE } else { PARCHMENT })
        };
        gfx::text(
            painter,
            pos2(x, r.center().y),
            Align2::LEFT_CENTER,
            &row.label,
            gfx::body_font(21.0),
            colour,
        );
        if !row.right.is_empty() {
            gfx::text(
                painter,
                pos2(r.right() - 10.0, r.center().y),
                Align2::RIGHT_CENTER,
                &row.right,
                gfx::body_font(19.0),
                colour.gamma_multiply(0.85),
            );
        }
    }
    // Scroll hints.
    if state.scroll > 0 {
        gfx::triangle(
            painter,
            pos2(rect.center().x, rect.top() - 8.0),
            12.0,
            false,
            GOLD,
        );
    }
    if state.scroll + visible < rows.len() {
        gfx::triangle(
            painter,
            pos2(rect.center().x, rect.top() + visible as f32 * row_h + 6.0),
            12.0,
            true,
            GOLD,
        );
    }
    if active && input.confirm {
        if rows[state.cursor].enabled {
            result.picked = Some(state.cursor);
        } else {
            result.denied = true;
        }
    }
    result
}

/// Horizontal tabs; returns true when the selection changed.
pub fn tabs(
    painter: &egui::Painter,
    rect: Rect,
    names: &[&str],
    current: &mut usize,
    input: &Input,
) -> bool {
    let w = rect.width() / names.len() as f32;
    let mut changed = false;
    for (i, name) in names.iter().enumerate() {
        let r = Rect::from_min_size(
            pos2(rect.left() + i as f32 * w, rect.top()),
            vec2(w - 4.0, rect.height()),
        );
        if input.click && input.pointer.is_some_and(|p| r.contains(p)) && *current != i {
            *current = i;
            changed = true;
        }
        let sel = *current == i;
        painter.rect_filled(
            r,
            CornerRadius::same(6),
            if sel {
                GOLD.gamma_multiply(0.22)
            } else {
                Color32::from_black_alpha(90)
            },
        );
        if sel {
            painter.rect_stroke(
                r,
                CornerRadius::same(6),
                Stroke::new(1.0, GOLD.gamma_multiply(0.8)),
                egui::StrokeKind::Inside,
            );
        }
        gfx::text(
            painter,
            r.center(),
            Align2::CENTER_CENTER,
            name,
            gfx::heading_font(17.0),
            if sel { GOLD } else { gfx::DIM },
        );
    }
    changed
}
