use eframe::egui;

use crate::dungeon::grid::Cell;
use crate::game::entities::{ItemKind, MonsterKind};

/// Every 32x32 PNG in assets/tiles/, embedded into the exe at build time.
pub struct Tiles {
    wall: egui::TextureHandle,
    floor: egui::TextureHandle,
    door: egui::TextureHandle,
    open_door: egui::TextureHandle,
    stairs: egui::TextureHandle,
    player: egui::TextureHandle,
    rat: egui::TextureHandle,
    goblin: egui::TextureHandle,
    orc: egui::TextureHandle,
    potion: egui::TextureHandle,
    gold: egui::TextureHandle,
}

macro_rules! tile {
    ($ctx:expr, $name:literal) => {
        load_png(
            $ctx,
            $name,
            include_bytes!(concat!("../../assets/tiles/", $name, ".png")),
        )
    };
}

impl Tiles {
    pub fn load(ctx: &egui::Context) -> Self {
        Self {
            wall: tile!(ctx, "wall"),
            floor: tile!(ctx, "floor"),
            door: tile!(ctx, "door"),
            open_door: tile!(ctx, "open_door"),
            stairs: tile!(ctx, "stairs"),
            player: tile!(ctx, "player"),
            rat: tile!(ctx, "rat"),
            goblin: tile!(ctx, "goblin"),
            orc: tile!(ctx, "orc"),
            potion: tile!(ctx, "potion"),
            gold: tile!(ctx, "gold"),
        }
    }

    pub fn floor(&self) -> egui::TextureId {
        self.floor.id()
    }

    pub fn player(&self) -> egui::TextureId {
        self.player.id()
    }

    pub fn for_cell(&self, cell: Cell) -> egui::TextureId {
        match cell {
            Cell::Wall => self.wall.id(),
            Cell::Floor => self.floor.id(),
            Cell::Door => self.door.id(),
            Cell::OpenDoor => self.open_door.id(),
            Cell::Stairs => self.stairs.id(),
        }
    }

    pub fn for_monster(&self, kind: MonsterKind) -> egui::TextureId {
        match kind {
            MonsterKind::Rat => self.rat.id(),
            MonsterKind::Goblin => self.goblin.id(),
            MonsterKind::Orc => self.orc.id(),
        }
    }

    pub fn for_item(&self, kind: ItemKind) -> egui::TextureId {
        match kind {
            ItemKind::Potion => self.potion.id(),
            ItemKind::Gold(_) => self.gold.id(),
        }
    }
}

fn load_png(ctx: &egui::Context, name: &str, bytes: &[u8]) -> egui::TextureHandle {
    let decoded = image::load_from_memory(bytes)
        .expect("embedded tile PNG must be valid")
        .to_rgba8();
    let size = [decoded.width() as usize, decoded.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &decoded.into_raw());
    ctx.load_texture(name, color_image, egui::TextureOptions::NEAREST)
}
