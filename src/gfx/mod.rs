//! Sprite atlas, fonts, theme, and small drawing helpers shared by every
//! screen.

pub mod sprites;

use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontFamily, FontId, Pos2, Rect, Stroke, Vec2, pos2, vec2,
};
use sprites::{ATLAS_COLUMNS, ATLAS_PNG, ATLAS_ROWS, CELL, Sprite};

/// Saved by name, so saves survive the atlas being rebuilt in another order.
impl serde::Serialize for Sprite {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!("{self:?}"))
    }
}

impl<'de> serde::Deserialize<'de> for Sprite {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let name = String::deserialize(d)?;
        Sprite::ALL
            .iter()
            .copied()
            .find(|s| format!("{s:?}") == name)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown sprite {name}")))
    }
}

pub const GOLD: Color32 = Color32::from_rgb(232, 190, 110);
pub const EMBER: Color32 = Color32::from_rgb(255, 146, 64);
pub const PARCHMENT: Color32 = Color32::from_rgb(236, 226, 205);
pub const DIM: Color32 = Color32::from_rgb(150, 140, 125);
pub const HP_RED: Color32 = Color32::from_rgb(208, 64, 58);
pub const MP_BLUE: Color32 = Color32::from_rgb(70, 130, 230);
pub const PANEL: Color32 = Color32::from_rgba_premultiplied(14, 12, 18, 232);

pub struct Gfx {
    pub atlas: egui::TextureHandle,
}

impl Gfx {
    pub fn load(ctx: &egui::Context) -> Self {
        let image = if ATLAS_PNG.is_empty() {
            placeholder_atlas()
        } else {
            let decoded = image::load_from_memory(ATLAS_PNG)
                .expect("atlas.png is valid")
                .to_rgba8();
            let size = [decoded.width() as usize, decoded.height() as usize];
            egui::ColorImage::from_rgba_unmultiplied(size, &decoded.into_raw())
        };
        let atlas = ctx.load_texture("atlas", image, egui::TextureOptions::NEAREST);
        Self { atlas }
    }

    pub fn uv(&self, sprite: Sprite) -> Rect {
        let (c, r) = sprite.cell();
        let (w, h) = ((ATLAS_COLUMNS * CELL) as f32, (ATLAS_ROWS * CELL) as f32);
        // A hair inside the cell so neighbours never bleed in.
        let inset = 0.01;
        Rect::from_min_max(
            pos2(
                (c * CELL) as f32 / w + inset / w,
                (r * CELL) as f32 / h + inset / h,
            ),
            pos2(
                ((c + 1) * CELL) as f32 / w - inset / w,
                ((r + 1) * CELL) as f32 / h - inset / h,
            ),
        )
    }

    pub fn draw(&self, painter: &egui::Painter, sprite: Sprite, rect: Rect, tint: Color32) {
        painter.image(self.atlas.id(), rect, self.uv(sprite), tint);
    }

    /// Mirrored horizontally (heroes face left in battle).
    pub fn draw_flipped(&self, painter: &egui::Painter, sprite: Sprite, rect: Rect, tint: Color32) {
        let uv = self.uv(sprite);
        let flipped = Rect::from_min_max(pos2(uv.max.x, uv.min.y), pos2(uv.min.x, uv.max.y));
        let mut mesh = egui::Mesh::with_texture(self.atlas.id());
        mesh.add_rect_with_uv(rect, flipped, tint);
        painter.add(mesh);
    }

    /// Adds a quad with a colour per corner (top-left, top-right,
    /// bottom-right, bottom-left) — used for smooth lighting.
    pub fn quad(&self, mesh: &mut egui::Mesh, sprite: Sprite, rect: Rect, colours: [Color32; 4]) {
        let uv = self.uv(sprite);
        let base = mesh.vertices.len() as u32;
        let corners = [
            (rect.left_top(), uv.left_top()),
            (rect.right_top(), uv.right_top()),
            (rect.right_bottom(), uv.right_bottom()),
            (rect.left_bottom(), uv.left_bottom()),
        ];
        for (i, (pos, uv)) in corners.into_iter().enumerate() {
            mesh.vertices.push(egui::epaint::Vertex {
                pos,
                uv,
                color: colours[i],
            });
        }
        mesh.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

/// Coloured squares standing in for the real atlas (used only before the
/// atlas has been generated).
fn placeholder_atlas() -> egui::ColorImage {
    let (w, h) = (
        (ATLAS_COLUMNS * CELL) as usize,
        (ATLAS_ROWS * CELL) as usize,
    );
    let mut img = egui::ColorImage::filled([w, h], Color32::TRANSPARENT);
    for (i, _) in Sprite::ALL.iter().enumerate() {
        let (c, r) = (
            (i as u32 % ATLAS_COLUMNS) as usize,
            (i as u32 / ATLAS_COLUMNS) as usize,
        );
        let hue = (i * 37 % 255) as u8;
        let colour = Color32::from_rgb(hue, 255 - hue, (hue / 2).wrapping_add(80));
        for y in 4..28 {
            for x in 4..28 {
                img[(c * 32 + x, r * 32 + y)] = colour;
            }
        }
    }
    img
}

pub fn title_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("title".into()))
}

pub fn heading_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("heading".into()))
}

pub fn body_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

pub fn italic_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("italic".into()))
}

pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let add = |fonts: &mut egui::FontDefinitions, name: &str, bytes: &'static [u8]| {
        fonts.font_data.insert(
            name.to_string(),
            std::sync::Arc::new(egui::FontData::from_static(bytes)),
        );
    };
    add(
        &mut fonts,
        "cinzel_decorative",
        include_bytes!("../../assets/fonts/CinzelDecorative-Bold.ttf"),
    );
    add(
        &mut fonts,
        "cinzel",
        include_bytes!("../../assets/fonts/Cinzel.ttf"),
    );
    add(
        &mut fonts,
        "crimson",
        include_bytes!("../../assets/fonts/CrimsonText-Regular.ttf"),
    );
    add(
        &mut fonts,
        "crimson_italic",
        include_bytes!("../../assets/fonts/CrimsonText-Italic.ttf"),
    );
    add(
        &mut fonts,
        "crimson_semibold",
        include_bytes!("../../assets/fonts/CrimsonText-SemiBold.ttf"),
    );
    let fallback: Vec<String> = fonts.families[&FontFamily::Proportional].clone();
    let with_fallback = |first: &str| {
        let mut v = vec![first.to_string()];
        v.extend(fallback.iter().cloned());
        v
    };
    fonts
        .families
        .insert(FontFamily::Proportional, with_fallback("crimson"));
    fonts.families.insert(
        FontFamily::Name("title".into()),
        with_fallback("cinzel_decorative"),
    );
    fonts
        .families
        .insert(FontFamily::Name("heading".into()), with_fallback("cinzel"));
    fonts.families.insert(
        FontFamily::Name("italic".into()),
        with_fallback("crimson_italic"),
    );
    fonts.families.insert(
        FontFamily::Name("bold".into()),
        with_fallback("crimson_semibold"),
    );
    ctx.set_fonts(fonts);

    ctx.global_style_mut(|style| {
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = Color32::from_rgb(8, 7, 11);
        style.visuals.window_fill = PANEL;
        style.visuals.override_text_color = Some(PARCHMENT);
        style
            .text_styles
            .insert(egui::TextStyle::Body, body_font(20.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, body_font(20.0));
        style
            .text_styles
            .insert(egui::TextStyle::Heading, heading_font(28.0));
        style
            .text_styles
            .insert(egui::TextStyle::Small, body_font(15.0));
    });
}

/// Text with a soft drop shadow.
pub fn text(
    painter: &egui::Painter,
    pos: Pos2,
    align: Align2,
    s: &str,
    font: FontId,
    colour: Color32,
) -> Rect {
    let shadow = Color32::from_black_alpha((colour.a() as f32 * 0.8) as u8);
    painter.text(pos + vec2(1.5, 2.0), align, s, font.clone(), shadow);
    painter.text(pos, align, s, font, colour)
}

/// Wrapped text inside a width; returns the height used.
pub fn wrapped(
    painter: &egui::Painter,
    pos: Pos2,
    width: f32,
    s: &str,
    font: FontId,
    colour: Color32,
) -> f32 {
    let galley = painter.layout(s.to_string(), font.clone(), colour, width);
    let h = galley.size().y;
    let shadow = painter.layout(s.to_string(), font, Color32::from_black_alpha(160), width);
    painter.galley(pos + vec2(1.0, 1.5), shadow, Color32::from_black_alpha(160));
    painter.galley(pos, galley, colour);
    h
}

/// The standard framed panel: dark glass, a gold edge.
pub fn panel(painter: &egui::Painter, rect: Rect) {
    painter.rect_filled(
        rect.translate(vec2(0.0, 4.0)),
        CornerRadius::same(10),
        Color32::from_black_alpha(90),
    );
    painter.rect_filled(rect, CornerRadius::same(10), PANEL);
    painter.rect_stroke(
        rect,
        CornerRadius::same(10),
        Stroke::new(1.5, GOLD.gamma_multiply(0.75)),
        egui::StrokeKind::Inside,
    );
    painter.rect_stroke(
        rect.shrink(4.0),
        CornerRadius::same(7),
        Stroke::new(1.0, GOLD.gamma_multiply(0.18)),
        egui::StrokeKind::Inside,
    );
}

pub fn bar(painter: &egui::Painter, rect: Rect, fraction: f32, colour: Color32) {
    painter.rect_filled(rect, CornerRadius::same(3), Color32::from_black_alpha(170));
    let f = fraction.clamp(0.0, 1.0);
    if f > 0.0 {
        let fill = Rect::from_min_size(rect.min, vec2(rect.width() * f, rect.height()));
        painter.rect_filled(fill, CornerRadius::same(3), colour);
        let shine =
            Rect::from_min_size(fill.min, vec2(fill.width(), (fill.height() * 0.4).max(1.0)));
        painter.rect_filled(shine, CornerRadius::same(3), Color32::from_white_alpha(40));
    }
    painter.rect_stroke(
        rect,
        CornerRadius::same(3),
        Stroke::new(1.0, Color32::from_black_alpha(200)),
        egui::StrokeKind::Outside,
    );
}

/// A portrait frame with a sprite inside.
pub fn portrait(gfx: &Gfx, painter: &egui::Painter, rect: Rect, sprite: Sprite, flipped: bool) {
    painter.rect_filled(rect, CornerRadius::same(8), Color32::from_rgb(30, 24, 30));
    let inner = rect.shrink(rect.width() * 0.08);
    let glow = egui::epaint::RectShape::filled(
        inner,
        CornerRadius::same(6),
        Color32::from_rgb(52, 40, 44),
    );
    painter.add(glow);
    if flipped {
        gfx.draw_flipped(painter, sprite, inner, Color32::WHITE);
    } else {
        gfx.draw(painter, sprite, inner, Color32::WHITE);
    }
    painter.rect_stroke(
        rect,
        CornerRadius::same(8),
        Stroke::new(2.0, GOLD.gamma_multiply(0.8)),
        egui::StrokeKind::Inside,
    );
}

/// A vertical gradient filling `rect`.
pub fn gradient(painter: &egui::Painter, rect: Rect, top: Color32, bottom: Color32) {
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), top);
    mesh.colored_vertex(rect.right_top(), top);
    mesh.colored_vertex(rect.right_bottom(), bottom);
    mesh.colored_vertex(rect.left_bottom(), bottom);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(mesh);
}

/// Darkens the screen edges.
pub fn vignette(painter: &egui::Painter, rect: Rect, strength: f32) {
    let steps = 14;
    let centre = rect.center();
    let mut mesh = egui::Mesh::default();
    let ring = |t: f32| -> Vec<Pos2> {
        (0..=steps * 4)
            .map(|i| {
                let a = i as f32 / (steps * 4) as f32 * std::f32::consts::TAU;
                let r = Vec2::new(rect.width() * 0.5, rect.height() * 0.5) * (0.62 + t * 0.9);
                centre + vec2(a.cos() * r.x * 1.1, a.sin() * r.y * 1.1)
            })
            .collect()
    };
    let inner = ring(0.0);
    let outer = ring(1.0);
    for i in 0..inner.len() {
        mesh.colored_vertex(inner[i], Color32::TRANSPARENT);
        mesh.colored_vertex(
            outer[i],
            Color32::from_black_alpha((255.0 * strength) as u8),
        );
    }
    for i in 0..inner.len() as u32 - 1 {
        let a = i * 2;
        mesh.add_triangle(a, a + 1, a + 3);
        mesh.add_triangle(a, a + 3, a + 2);
    }
    painter.add(mesh);
}

pub fn lerp_colour(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color32::from_rgba_unmultiplied(
        l(a.r(), b.r()),
        l(a.g(), b.g()),
        l(a.b(), b.b()),
        l(a.a(), b.a()),
    )
}

/// A small filled triangle pointing down (or up).
pub fn triangle(painter: &egui::Painter, centre: Pos2, size: f32, down: bool, colour: Color32) {
    let h = size * 0.5;
    let pts = if down {
        vec![
            pos2(centre.x - h, centre.y - h * 0.6),
            pos2(centre.x + h, centre.y - h * 0.6),
            pos2(centre.x, centre.y + h * 0.6),
        ]
    } else {
        vec![
            pos2(centre.x - h, centre.y + h * 0.6),
            pos2(centre.x + h, centre.y + h * 0.6),
            pos2(centre.x, centre.y - h * 0.6),
        ]
    };
    painter.add(egui::Shape::convex_polygon(pts, colour, Stroke::NONE));
}

/// A four-pointed sparkle marker.
pub fn diamond(painter: &egui::Painter, centre: Pos2, size: f32, colour: Color32) {
    let s = size * 0.5;
    let pts = vec![
        pos2(centre.x, centre.y - s),
        pos2(centre.x + s * 0.45, centre.y),
        pos2(centre.x, centre.y + s),
        pos2(centre.x - s * 0.45, centre.y),
    ];
    painter.add(egui::Shape::convex_polygon(
        pts,
        colour,
        Stroke::new(1.0, Color32::from_black_alpha(160)),
    ));
}

/// Lays out all of `s` but only shows its first `shown` characters, so the
/// words don't jump around while a typewriter effect reveals them.
#[allow(clippy::too_many_arguments)]
pub fn reveal(
    painter: &egui::Painter,
    pos: Pos2,
    width: f32,
    s: &str,
    shown: usize,
    font: FontId,
    colour: Color32,
    centre: bool,
) -> f32 {
    let split = s
        .char_indices()
        .nth(shown)
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = width;
    if centre {
        job.halign = egui::Align::Center;
    }
    let fmt = |c: Color32| egui::text::TextFormat {
        font_id: font.clone(),
        color: c,
        ..Default::default()
    };
    job.append(&s[..split], 0.0, fmt(colour));
    job.append(&s[split..], 0.0, fmt(Color32::TRANSPARENT));
    let mut shadow_job = job.clone();
    for section in &mut shadow_job.sections {
        if section.format.color != Color32::TRANSPARENT {
            section.format.color = Color32::from_black_alpha(150);
        }
    }
    let galley = painter.layout_job(job);
    let h = galley.size().y;
    // Shadows help light text on dark; on parchment they only smudge.
    if colour.r() as u32 + colour.g() as u32 + colour.b() as u32 > 300 {
        let shadow = painter.layout_job(shadow_job);
        painter.galley(pos + vec2(1.0, 1.5), shadow, Color32::BLACK);
    }
    painter.galley(pos, galley, colour);
    h
}
