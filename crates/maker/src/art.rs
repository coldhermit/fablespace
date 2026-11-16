//! Original pictures for a new project. Nothing here is taken from RPG Maker.

use std::path::Path;

use image::{Rgba, RgbaImage};

const TILE: u32 = 32;

pub fn write_all(root: &Path) -> Result<(), String> {
    let img = root.join("img");
    for folder in [
        "tilesets",
        "characters",
        "faces",
        "enemies",
        "pictures",
        "titles",
    ] {
        std::fs::create_dir_all(img.join(folder)).map_err(|e| e.to_string())?;
    }
    save(&img.join("tilesets/meadow.png"), meadow())?;
    save(&img.join("characters/hero.png"), person_sheet(Skin::hero()))?;
    save(&img.join("characters/mara.png"), person_sheet(Skin::mara()))?;
    save(&img.join("characters/chest.png"), chest_sheet())?;
    save(&img.join("characters/slime.png"), slime_sheet())?;
    save(&img.join("characters/board.png"), board_sheet())?;
    save(&img.join("faces/lark.png"), face(Skin::hero()))?;
    save(&img.join("faces/mara.png"), face(Skin::mara()))?;
    save(&img.join("enemies/slime.png"), battler())?;
    save(&img.join("pictures/board.png"), notice())?;
    save(&img.join("titles/title.png"), title_card())?;
    Ok(())
}

fn save(path: &Path, image: RgbaImage) -> Result<(), String> {
    image
        .save(path)
        .map_err(|e| format!("save {}: {e}", path.display()))
}

fn meadow() -> RgbaImage {
    let mut image = RgbaImage::new(8 * TILE, 5 * TILE);
    for row in 0..5 {
        for col in 0..8 {
            let id = row * 8 + col;
            paint_tile(&mut image, col, row, id);
        }
    }
    image
}

fn paint_tile(image: &mut RgbaImage, col: u32, row: u32, id: u32) {
    let ox = col * TILE;
    let oy = row * TILE;
    if row >= 1 && col < 4 {
        let mask = ((row - 1) * 4 + col) as u8;
        path_tile(image, ox, oy, mask);
        return;
    }
    match id {
        0 => fill(image, ox, oy, TILE, TILE, rgba(18, 24, 20, 255)),
        2 => stone(image, ox, oy),
        3 => water(image, ox, oy),
        4 => {
            grass(image, ox, oy, 1);
            flower(image, ox + 8, oy + 14, rgba(230, 210, 90, 255));
            flower(image, ox + 18, oy + 8, rgba(220, 120, 130, 255));
        }
        5 => tree(image, ox, oy),
        6 => grass(image, ox, oy, 2),
        7 => {
            grass(image, ox, oy, 3);
            fill(image, ox + 20, oy + 18, 6, 4, rgba(120, 116, 108, 255));
        }
        _ => grass(image, ox, oy, id),
    }
}

fn path_tile(image: &mut RgbaImage, ox: u32, oy: u32, mask: u8) {
    grass(image, ox, oy, 4);
    for y in 0..TILE {
        for x in 0..TILE {
            if path_pixel(x, y, mask) {
                let shade = 150 + ((x * 3 + y * 5) % 18) as u8;
                image.put_pixel(ox + x, oy + y, rgba(shade, shade - 28, 70, 255));
            } else if path_edge(x, y, mask) {
                image.put_pixel(ox + x, oy + y, rgba(90, 110, 48, 255));
            }
        }
    }
}

fn path_pixel(x: u32, y: u32, mask: u8) -> bool {
    let n = mask & 1 != 0;
    let e = mask & 2 != 0;
    let s = mask & 4 != 0;
    let w = mask & 8 != 0;
    let m = 10;
    let left = x < m;
    let right = x >= TILE - m;
    let top = y < m;
    let bottom = y >= TILE - m;
    if !left && !right && !top && !bottom {
        return true;
    }
    if top && !n || bottom && !s || left && !w || right && !e {
        return false;
    }
    true
}

fn path_edge(x: u32, y: u32, mask: u8) -> bool {
    path_pixel(x.saturating_sub(1), y, mask)
        || path_pixel(x + 1, y, mask)
        || path_pixel(x, y.saturating_sub(1), mask)
        || path_pixel(x, y + 1, mask)
}

fn grass(image: &mut RgbaImage, ox: u32, oy: u32, salt: u32) {
    for y in 0..TILE {
        for x in 0..TILE {
            let n = (x * 13 + y * 7 + salt * 17) % 11;
            let g = 118 + n as u8 * 3;
            image.put_pixel(ox + x, oy + y, rgba(48 + n as u8, g, 46, 255));
        }
    }
}

fn stone(image: &mut RgbaImage, ox: u32, oy: u32) {
    grass(image, ox, oy, 9);
    fill(image, ox + 4, oy + 6, 24, 18, rgba(128, 124, 116, 255));
    fill(image, ox + 6, oy + 8, 10, 8, rgba(150, 146, 136, 255));
    fill(image, ox + 16, oy + 14, 8, 6, rgba(108, 104, 98, 255));
}

fn water(image: &mut RgbaImage, ox: u32, oy: u32) {
    for y in 0..TILE {
        for x in 0..TILE {
            let wave = if (y / 4) % 2 == 0 && x % 8 < 5 { 18 } else { 0 };
            image.put_pixel(ox + x, oy + y, rgba(36, 90 + wave, 120, 255));
        }
    }
}

fn flower(image: &mut RgbaImage, x: u32, y: u32, color: Rgba<u8>) {
    fill(image, x + 1, y, 2, 5, rgba(40, 90, 40, 255));
    image.put_pixel(x, y, color);
    image.put_pixel(x + 1, y, color);
    image.put_pixel(x + 2, y, color);
    image.put_pixel(x + 1, y.saturating_sub(1), color);
}

fn tree(image: &mut RgbaImage, ox: u32, oy: u32) {
    fill(image, ox + 14, oy + 16, 4, 14, rgba(92, 64, 40, 255));
    for y in 2..18 {
        let reach = 8u32.saturating_sub((y as i32 - 10).unsigned_abs() / 2);
        let reach = reach.max(3);
        fill(
            image,
            ox + 16 - reach,
            oy + y,
            reach * 2,
            1,
            rgba(36, 96, 48, 255),
        );
    }
    fill(image, ox + 12, oy + 6, 8, 6, rgba(52, 120, 60, 255));
}

struct Skin {
    cloak: Rgba<u8>,
    hair: Rgba<u8>,
    hat: Rgba<u8>,
    skin: Rgba<u8>,
}

impl Skin {
    fn hero() -> Self {
        Self {
            cloak: rgba(48, 78, 150, 255),
            hair: rgba(70, 48, 30, 255),
            hat: rgba(186, 150, 64, 255),
            skin: rgba(230, 186, 146, 255),
        }
    }

    fn mara() -> Self {
        Self {
            cloak: rgba(36, 120, 78, 255),
            hair: rgba(42, 28, 24, 255),
            hat: rgba(42, 28, 24, 255),
            skin: rgba(214, 164, 130, 255),
        }
    }
}

fn person_sheet(skin: Skin) -> RgbaImage {
    let mut image = RgbaImage::new(12 * TILE, 8 * TILE);
    for dir in 0..4 {
        for frame in 0..3 {
            person(&mut image, frame * TILE, dir * TILE, dir, frame, &skin);
        }
    }
    image
}

fn person(image: &mut RgbaImage, ox: u32, oy: u32, dir: u32, frame: u32, skin: &Skin) {
    let bob = if frame == 1 { 0 } else { 1 };
    let y = oy + bob;
    fill(image, ox + 12, y + 18, 8, 10, skin.cloak);
    fill(image, ox + 13, y + 26, 3, 5, rgba(50, 40, 36, 255));
    fill(image, ox + 17, y + 26, 3, 5, rgba(50, 40, 36, 255));
    if frame == 0 {
        fill(image, ox + 12, y + 28, 3, 3, rgba(50, 40, 36, 255));
    }
    if frame == 2 {
        fill(image, ox + 18, y + 28, 3, 3, rgba(50, 40, 36, 255));
    }
    fill(image, ox + 12, y + 8, 8, 8, skin.skin);
    fill(image, ox + 11, y + 6, 10, 4, skin.hair);
    if dir != 3 {
        fill(image, ox + 11, y + 4, 10, 3, skin.hat);
    }
    let eye = rgba(30, 24, 28, 255);
    match dir {
        1 => {
            image.put_pixel(ox + 13, y + 11, eye);
        }
        2 => {
            image.put_pixel(ox + 18, y + 11, eye);
        }
        3 => {}
        _ => {
            image.put_pixel(ox + 14, y + 11, eye);
            image.put_pixel(ox + 17, y + 11, eye);
        }
    }
}

fn chest_sheet() -> RgbaImage {
    let mut image = RgbaImage::new(12 * TILE, 8 * TILE);
    for frame in 0..3 {
        chest(&mut image, frame * TILE, 0, false);
        chest(&mut image, (3 + frame) * TILE, 0, true);
    }
    image
}

fn chest(image: &mut RgbaImage, ox: u32, oy: u32, open: bool) {
    let wood = rgba(140, 92, 48, 255);
    let dark = rgba(80, 52, 28, 255);
    if open {
        fill(image, ox + 6, oy + 8, 20, 6, dark);
        fill(image, ox + 6, oy + 4, 20, 4, wood);
        fill(image, ox + 6, oy + 16, 20, 12, wood);
        fill(image, ox + 8, oy + 18, 16, 6, rgba(40, 28, 18, 255));
    } else {
        fill(image, ox + 6, oy + 12, 20, 14, wood);
        fill(image, ox + 6, oy + 10, 20, 4, dark);
        fill(image, ox + 14, oy + 16, 4, 3, rgba(212, 180, 70, 255));
    }
}

fn slime_sheet() -> RgbaImage {
    let mut image = RgbaImage::new(12 * TILE, 8 * TILE);
    for dir in 0..4 {
        for frame in 0..3 {
            blob(&mut image, frame * TILE, dir * TILE, frame, 14);
        }
    }
    image
}

fn blob(image: &mut RgbaImage, ox: u32, oy: u32, frame: u32, radius: i32) {
    let squish = if frame == 1 { 0 } else { 2 };
    let body = rgba(70, 170, 86, 255);
    let shine = rgba(170, 220, 150, 255);
    let cx = ox as i32 + 16;
    let cy = oy as i32 + 20;
    for y in -radius..radius {
        for x in -radius..radius {
            let yy = y + squish;
            if x * x + yy * yy * 2 <= radius * radius {
                let px = cx + x;
                let py = cy + yy;
                if px >= 0 && py >= 0 {
                    image.put_pixel(
                        px as u32,
                        py as u32,
                        if x < -2 && yy < -2 { shine } else { body },
                    );
                }
            }
        }
    }
    let eye = rgba(20, 30, 24, 255);
    image.put_pixel(ox + 12, oy + 16, eye);
    image.put_pixel(ox + 18, oy + 16, eye);
}

fn board_sheet() -> RgbaImage {
    let mut image = RgbaImage::new(12 * TILE, 8 * TILE);
    for frame in 0..3 {
        let ox = frame * TILE;
        fill(&mut image, ox + 14, 16, 4, 16, rgba(110, 78, 46, 255));
        fill(&mut image, ox + 6, 6, 20, 12, rgba(168, 132, 78, 255));
        fill(&mut image, ox + 8, 9, 16, 2, rgba(90, 60, 36, 255));
        fill(&mut image, ox + 8, 13, 12, 2, rgba(90, 60, 36, 255));
    }
    image
}

fn face(skin: Skin) -> RgbaImage {
    let mut image = RgbaImage::new(96, 96);
    fill(&mut image, 0, 0, 96, 96, rgba(24, 32, 28, 255));
    fill(&mut image, 22, 28, 52, 48, skin.skin);
    fill(&mut image, 18, 16, 60, 22, skin.hair);
    fill(&mut image, 18, 12, 60, 10, skin.hat);
    fill(&mut image, 22, 70, 52, 18, skin.cloak);
    let eye = rgba(30, 24, 28, 255);
    fill(&mut image, 34, 46, 6, 6, eye);
    fill(&mut image, 56, 46, 6, 6, eye);
    fill(&mut image, 40, 60, 16, 3, rgba(160, 80, 80, 255));
    image
}

fn battler() -> RgbaImage {
    let mut image = RgbaImage::new(160, 128);
    blob(&mut image, 48, 20, 1, 40);
    image
}

fn notice() -> RgbaImage {
    let mut image = RgbaImage::new(208, 120);
    fill(&mut image, 0, 0, 208, 120, rgba(0, 0, 0, 0));
    fill(&mut image, 8, 8, 192, 104, rgba(168, 126, 72, 255));
    fill(&mut image, 16, 16, 176, 88, rgba(214, 186, 130, 255));
    for row in 0..4 {
        fill(
            &mut image,
            28,
            28 + row * 16,
            140 - row * 12,
            4,
            rgba(110, 78, 46, 255),
        );
    }
    image
}

fn title_card() -> RgbaImage {
    let mut image = RgbaImage::new(544, 416);
    for y in 0..416 {
        let t = y as f32 / 416.0;
        let r = (70.0 + t * 40.0) as u8;
        let g = (120.0 - t * 30.0) as u8;
        let b = (170.0 - t * 80.0) as u8;
        if y < 230 {
            fill(&mut image, 0, y, 544, 1, rgba(r, g, b, 255));
        } else {
            let g = 90 + ((y * 3) % 20) as u8;
            fill(&mut image, 0, y, 544, 1, rgba(40, g, 42, 255));
        }
    }
    for x in 0..544 {
        let hill = 200 + ((x as i32 - 200).abs() / 8).min(40);
        for y in hill..240 {
            image.put_pixel(x, y as u32, rgba(46, 100, 52, 255));
        }
    }
    for x in 250..300 {
        image.put_pixel(x, 250, rgba(150, 120, 70, 255));
        image.put_pixel(x, 251, rgba(150, 120, 70, 255));
    }
    blob(&mut image, 430, 250, 1, 16);
    image
}

fn fill(image: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, color: Rgba<u8>) {
    for py in y..y.saturating_add(h).min(image.height()) {
        for px in x..x.saturating_add(w).min(image.width()) {
            let dst = image.get_pixel(px, py);
            image.put_pixel(
                px,
                py,
                if color.0[3] == 255 {
                    color
                } else {
                    blend(*dst, color)
                },
            );
        }
    }
}

fn blend(under: Rgba<u8>, over: Rgba<u8>) -> Rgba<u8> {
    let a = over.0[3] as u32;
    let inv = 255 - a;
    Rgba([
        ((over.0[0] as u32 * a + under.0[0] as u32 * inv) / 255) as u8,
        ((over.0[1] as u32 * a + under.0[1] as u32 * inv) / 255) as u8,
        ((over.0[2] as u32 * a + under.0[2] as u32 * inv) / 255) as u8,
        255,
    ])
}

fn rgba(r: u8, g: u8, b: u8, a: u8) -> Rgba<u8> {
    Rgba([r, g, b, a])
}
