//! One frame's worth of player input, keyboard first, mouse welcome.

use eframe::egui::{self, Key, Pos2};

use crate::world::Dir;

#[derive(Default, Clone)]
pub struct Input {
    /// Navigation presses this frame (key repeat included).
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    /// Direction held right now (for walking).
    pub held: Option<Dir>,
    pub confirm: bool,
    pub cancel: bool,
    pub menu: bool,
    pub map: bool,
    pub fast: bool,
    pub any_key: bool,
    pub click: bool,
    pub right_click: bool,
    pub pointer: Option<Pos2>,
    pub scroll: f32,
    pub dt: f32,
    /// Digit keys 1-9 pressed this frame.
    pub digit: Option<usize>,
}

impl Input {
    pub fn read(ctx: &egui::Context) -> Self {
        ctx.input(|i| {
            let mut input = Input { dt: i.stable_dt.min(0.1), pointer: i.pointer.hover_pos(), ..Default::default() };
            for event in &i.events {
                if let egui::Event::Key { key, pressed: true, .. } = event {
                    input.any_key = true;
                    match key {
                        Key::ArrowUp | Key::W | Key::K => input.up = true,
                        Key::ArrowDown | Key::S | Key::J => input.down = true,
                        Key::ArrowLeft | Key::A | Key::H => input.left = true,
                        Key::ArrowRight | Key::D | Key::L => input.right = true,
                        Key::Enter | Key::Space | Key::E | Key::Z => input.confirm = true,
                        Key::Escape | Key::Backspace | Key::X | Key::Q => input.cancel = true,
                        Key::Tab | Key::I => input.menu = true,
                        Key::M => input.map = true,
                        Key::Num1 => input.digit = Some(1),
                        Key::Num2 => input.digit = Some(2),
                        Key::Num3 => input.digit = Some(3),
                        Key::Num4 => input.digit = Some(4),
                        Key::Num5 => input.digit = Some(5),
                        Key::Num6 => input.digit = Some(6),
                        _ => {}
                    }
                }
            }
            let held = |keys: &[Key]| keys.iter().any(|k| i.key_down(*k));
            input.held = if held(&[Key::ArrowUp, Key::W, Key::K]) {
                Some(Dir::Up)
            } else if held(&[Key::ArrowDown, Key::S, Key::J]) {
                Some(Dir::Down)
            } else if held(&[Key::ArrowLeft, Key::A, Key::H]) {
                Some(Dir::Left)
            } else if held(&[Key::ArrowRight, Key::D, Key::L]) {
                Some(Dir::Right)
            } else {
                None
            };
            input.fast = i.modifiers.shift;
            input.click = i.pointer.primary_clicked();
            input.right_click = i.pointer.secondary_clicked();
            if input.click || input.right_click {
                input.any_key = true;
            }
            if input.right_click {
                input.cancel = true;
            }
            input.scroll = i.smooth_scroll_delta.y;
            input
        })
    }

    /// Consumes navigation so a second handler this frame doesn't react.
    pub fn eat(&mut self) {
        let dt = self.dt;
        let pointer = self.pointer;
        let fast = self.fast;
        *self = Input { dt, pointer, fast, ..Default::default() };
    }
}
