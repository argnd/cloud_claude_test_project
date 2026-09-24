//! One-shot generator for the placeholder tiles in assets/tiles/.
//! Run with `cargo run --example make_tiles`; repaint the PNGs by hand afterwards.

use image::{Rgba, RgbaImage};

const SIZE: u32 = 32;
const CLEAR: Rgba<u8> = Rgba([0, 0, 0, 0]);

fn main() {
    // Wall: stone bricks, rows offset by half a brick.
    let mut wall = flat(Rgba([52, 50, 62, 255]));
    for row in 0..4 {
        let y0 = row * 8;
        let shift: i32 = if row % 2 == 0 { 0 } else { 8 };
        for col in -1..3i32 {
            let x0 = (col * 16 + shift).max(0) as u32;
            let x1 = ((col + 1) * 16 + shift).clamp(0, SIZE as i32) as u32;
            if x1 > x0 + 1 {
                rect(
                    &mut wall,
                    x0 + 1,
                    y0 + 1,
                    x1 - 1,
                    y0 + 7,
                    Rgba([88, 84, 100, 255]),
                );
                rect(
                    &mut wall,
                    x0 + 1,
                    y0 + 1,
                    x1 - 1,
                    y0 + 2,
                    Rgba([110, 106, 124, 255]),
                );
            }
        }
    }
    save(&wall, "wall");

    // Floor: dark flagstones with a few specks.
    let mut floor = flat(Rgba([38, 34, 32, 255]));
    rect(&mut floor, 1, 1, 15, 15, Rgba([58, 52, 48, 255]));
    rect(&mut floor, 17, 1, 31, 15, Rgba([54, 49, 46, 255]));
    rect(&mut floor, 1, 17, 15, 31, Rgba([54, 49, 46, 255]));
    rect(&mut floor, 17, 17, 31, 31, Rgba([58, 52, 48, 255]));
    for &(x, y) in &[
        (5, 4),
        (11, 9),
        (22, 6),
        (27, 12),
        (6, 25),
        (20, 21),
        (26, 28),
    ] {
        floor.put_pixel(x, y, Rgba([74, 67, 62, 255]));
    }
    save(&floor, "floor");

    // Door: planks in a frame, drawn over the floor.
    let mut door = RgbaImage::from_pixel(SIZE, SIZE, CLEAR);
    rect(&mut door, 3, 1, 29, 32, Rgba([70, 44, 22, 255]));
    rect(&mut door, 5, 3, 27, 32, Rgba([140, 90, 45, 255]));
    for x in [10, 16, 22] {
        rect(&mut door, x, 3, x + 1, 32, Rgba([100, 62, 30, 255]));
    }
    circle(&mut door, 23.0, 17.0, 1.6, Rgba([230, 200, 90, 255]));
    save(&door, "door");

    // Open door: only the frame is left, the door swung against the side.
    let mut open_door = RgbaImage::from_pixel(SIZE, SIZE, CLEAR);
    rect(&mut open_door, 3, 1, 29, 3, Rgba([70, 44, 22, 255]));
    rect(&mut open_door, 3, 1, 5, 32, Rgba([70, 44, 22, 255]));
    rect(&mut open_door, 27, 1, 29, 32, Rgba([70, 44, 22, 255]));
    rect(&mut open_door, 5, 3, 9, 32, Rgba([140, 90, 45, 255]));
    save(&open_door, "open_door");

    // Stairs: steps getting darker as they go down.
    let mut stairs = flat(Rgba([20, 18, 18, 255]));
    for i in 0..5u32 {
        let shade = 150 - i as u8 * 25;
        let y0 = 3 + i * 5;
        rect(
            &mut stairs,
            3 + i * 2,
            y0,
            29 - i * 2,
            y0 + 4,
            Rgba([shade, shade - 10, shade - 20, 255]),
        );
    }
    save(&stairs, "stairs");

    // Player: blue-cloaked figure with a sword.
    let mut player = RgbaImage::from_pixel(SIZE, SIZE, CLEAR);
    rect(&mut player, 10, 14, 22, 28, Rgba([40, 80, 190, 255]));
    rect(&mut player, 11, 28, 14, 31, Rgba([60, 40, 30, 255]));
    rect(&mut player, 18, 28, 21, 31, Rgba([60, 40, 30, 255]));
    circle(&mut player, 16.0, 9.0, 5.0, Rgba([235, 195, 160, 255]));
    rect(&mut player, 11, 3, 21, 6, Rgba([120, 70, 30, 255]));
    rect(&mut player, 24, 6, 26, 22, Rgba([210, 215, 225, 255]));
    rect(&mut player, 22, 20, 28, 22, Rgba([150, 110, 40, 255]));
    save(&player, "player");

    // Rat: small grey body, pink tail.
    let mut rat = RgbaImage::from_pixel(SIZE, SIZE, CLEAR);
    ellipse(&mut rat, 15.0, 22.0, 9.0, 5.5, Rgba([120, 115, 110, 255]));
    circle(&mut rat, 24.0, 19.0, 4.0, Rgba([130, 125, 120, 255]));
    circle(&mut rat, 23.0, 15.0, 2.0, Rgba([200, 150, 150, 255]));
    rat.put_pixel(26, 18, Rgba([200, 30, 30, 255]));
    rect(&mut rat, 2, 24, 7, 25, Rgba([220, 150, 150, 255]));
    rect(&mut rat, 1, 21, 3, 24, Rgba([220, 150, 150, 255]));
    save(&rat, "rat");

    // Goblin: green, big ears, short.
    let mut goblin = RgbaImage::from_pixel(SIZE, SIZE, CLEAR);
    rect(&mut goblin, 11, 17, 21, 28, Rgba([110, 70, 40, 255]));
    rect(&mut goblin, 12, 28, 15, 31, Rgba([70, 120, 50, 255]));
    rect(&mut goblin, 17, 28, 20, 31, Rgba([70, 120, 50, 255]));
    circle(&mut goblin, 16.0, 12.0, 6.0, Rgba([90, 160, 60, 255]));
    rect(&mut goblin, 6, 9, 11, 12, Rgba([90, 160, 60, 255]));
    rect(&mut goblin, 21, 9, 26, 12, Rgba([90, 160, 60, 255]));
    goblin.put_pixel(14, 11, Rgba([250, 220, 40, 255]));
    goblin.put_pixel(18, 11, Rgba([250, 220, 40, 255]));
    save(&goblin, "goblin");

    // Orc: bulky, grey-green, red eyes, club.
    let mut orc = RgbaImage::from_pixel(SIZE, SIZE, CLEAR);
    rect(&mut orc, 7, 13, 25, 28, Rgba([90, 60, 50, 255]));
    rect(&mut orc, 8, 28, 13, 31, Rgba([80, 100, 70, 255]));
    rect(&mut orc, 19, 28, 24, 31, Rgba([80, 100, 70, 255]));
    circle(&mut orc, 16.0, 8.0, 6.5, Rgba([100, 125, 80, 255]));
    rect(&mut orc, 12, 7, 14, 9, Rgba([220, 30, 30, 255]));
    rect(&mut orc, 18, 7, 20, 9, Rgba([220, 30, 30, 255]));
    rect(&mut orc, 13, 12, 14, 14, Rgba([240, 240, 220, 255]));
    rect(&mut orc, 18, 12, 19, 14, Rgba([240, 240, 220, 255]));
    rect(&mut orc, 26, 10, 30, 26, Rgba([110, 75, 40, 255]));
    save(&orc, "orc");

    // Potion: red liquid in a round flask.
    let mut potion = RgbaImage::from_pixel(SIZE, SIZE, CLEAR);
    rect(&mut potion, 13, 6, 19, 13, Rgba([190, 210, 230, 255]));
    rect(&mut potion, 12, 4, 20, 7, Rgba([130, 90, 50, 255]));
    circle(&mut potion, 16.0, 20.0, 8.0, Rgba([190, 210, 230, 255]));
    circle(&mut potion, 16.0, 21.0, 6.5, Rgba([210, 30, 50, 255]));
    circle(&mut potion, 13.0, 18.0, 1.5, Rgba([255, 150, 160, 255]));
    save(&potion, "potion");

    // Gold: a small pile of coins.
    let mut gold = RgbaImage::from_pixel(SIZE, SIZE, CLEAR);
    for &(x, y) in &[
        (11.0, 23.0),
        (21.0, 23.0),
        (16.0, 21.0),
        (13.0, 17.0),
        (19.0, 16.0),
    ] {
        ellipse(&mut gold, x, y, 5.5, 3.5, Rgba([170, 120, 20, 255]));
        ellipse(&mut gold, x, y - 0.5, 4.5, 2.5, Rgba([250, 205, 60, 255]));
    }
    save(&gold, "gold");
}

fn flat(color: Rgba<u8>) -> RgbaImage {
    RgbaImage::from_pixel(SIZE, SIZE, color)
}

/// Fills x0..x1 by y0..y1, clipped to the tile.
fn rect(img: &mut RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32, color: Rgba<u8>) {
    for y in y0..y1.min(SIZE) {
        for x in x0..x1.min(SIZE) {
            img.put_pixel(x, y, color);
        }
    }
}

fn circle(img: &mut RgbaImage, cx: f32, cy: f32, radius: f32, color: Rgba<u8>) {
    ellipse(img, cx, cy, radius, radius, color);
}

fn ellipse(img: &mut RgbaImage, cx: f32, cy: f32, rx: f32, ry: f32, color: Rgba<u8>) {
    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = (x as f32 + 0.5 - cx) / rx;
            let dy = (y as f32 + 0.5 - cy) / ry;
            if dx * dx + dy * dy <= 1.0 {
                img.put_pixel(x, y, color);
            }
        }
    }
}

fn save(img: &RgbaImage, name: &str) {
    let path = format!("assets/tiles/{name}.png");
    img.save(&path).expect("write tile PNG");
    println!("wrote {path}");
}
