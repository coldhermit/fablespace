//! Code-drawn frame: field, dialogue, battle, title, and menus.
//! Panels, bars, and glyphs are rectangles and a bitmap font. No UI images.
//!
//! Tiles are 16px art at 2x (32px). The camera shows about 60x33 of the
//! 120x70 map. Cells the player cannot walk, and anything outside the map,
//! stay a flat color unless a Kenney tile covers that cell. A missing
//! road, roof, or wall sprite falls back to a flat color.
//! The bitmap font is drawn at 3x. Dialogue and battle keep their 1080p layout.

use crate::atlas::{self, ATLAS, ATLAS_W, TILE};
use crate::font::FONT;
use crate::game::{
    terrain_at, Back, Game, Mode, Phase, Terrain, COMMANDS, HEIGHT, MAP_H, MAP_W, MUSHROOMS,
    ROAD_X, SCALE, SYSTEM_ITEMS, TALK_LINE, TITLE_ITEMS, TRANSITION_FRAMES, WIDTH,
};

const CREAM: u32 = 0xF3E6C8;
const GOLD: u32 = 0xF0C14A;
const DIM: u32 = 0xA89880;
const INK: u32 = 0x140E0C;
const FRAME: u32 = 0xE7D7B0;
const FRAME_IN: u32 = 0x2A1A14;
const PANEL: u32 = 0x1C120E;
const SLIME: u32 = 0x7DFFD0;
const HP_OK: u32 = 0x3DDC6A;
const HP_LOW: u32 = 0xE24B4B;
const HP_BG: u32 = 0x2A1612;
const SELECT: u32 = 0x5A3A22;
const BATTLE_BG: u32 = 0x1B3A2A;
const BATTLE_GROUND: u32 = 0x12281C;
const HORIZON: u32 = 0x3E6A4A;
/// Flat color for anything that is not walkable ground and has no tile.
const BLOCKED: u32 = 0x1A120C;
const TITLE_BG: u32 = 0x12160F;
const DIRT: u32 = 0xE6A56C;
const ROOF_RED: u32 = 0xC45A42;
const ROOF_BLUE: u32 = 0x6A7896;
const WALL_WOOD: u32 = 0xC48858;
const WALL_STONE: u32 = 0x8E96A8;

const FONT_SCALE: i32 = 3;
/// Battle portraits were three tiles tall in the old frame. 3 * 6 = 18.
const PORTRAIT: i32 = 18;

pub fn draw(fb: &mut [u32], game: &Game) {
    debug_assert_eq!(fb.len(), (WIDTH * HEIGHT) as usize);
    match world(game.mode) {
        World::Title => {
            fb.fill(TITLE_BG);
            banner(fb, 150, "Field Battle", GOLD);
            if matches!(game.mode, Mode::LoadList { .. }) {
                draw_load(fb, game);
            } else {
                let rows: Vec<(&str, Option<&str>)> =
                    TITLE_ITEMS.iter().map(|label| (*label, None)).collect();
                draw_menu(fb, "", &rows, game.menu_cursor, game.notice);
            }
            return;
        }
        World::Battle => draw_battle(fb, game),
        World::Field => draw_field(fb, game),
    }
    if let Mode::Transition { frame } = game.mode {
        shade(fb, transition_keep(frame));
    }
    if overlay(game.mode) {
        shade(fb, 85);
        match game.mode {
            Mode::System(_) => draw_system(fb, game),
            Mode::QuitAsk(_) => {
                let rows = [("Yes", None), ("No", None)];
                draw_menu(
                    fb,
                    "Save before quitting?",
                    &rows,
                    game.menu_cursor,
                    game.notice,
                );
            }
            Mode::LoadList { .. } => draw_load(fb, game),
            Mode::Party => draw_party(fb, game),
            Mode::Skills => {
                let character = game.selected();
                let placeholder = character.skills.is_empty();
                let lines: Vec<String> = if placeholder {
                    vec!["(none)".to_string()]
                } else {
                    character.skills.clone()
                };
                draw_list(fb, "Skills", &character.name, &lines, placeholder);
            }
            Mode::Inventory => {
                let character = game.selected();
                let placeholder = character.inventory.is_empty();
                let lines: Vec<String> = if placeholder {
                    vec!["(empty)".to_string()]
                } else {
                    character
                        .inventory
                        .iter()
                        .map(|item| item.name.clone())
                        .collect()
                };
                draw_list(fb, "Inventory", &character.name, &lines, placeholder);
            }
            _ => {}
        }
    }
}

enum World {
    Title,
    Field,
    Battle,
}

fn world(mode: Mode) -> World {
    match mode {
        Mode::Title
        | Mode::LoadList {
            from_title: true, ..
        } => World::Title,
        Mode::Battle
        | Mode::System(Back::Battle)
        | Mode::QuitAsk(Back::Battle)
        | Mode::LoadList {
            from_title: false,
            back: Back::Battle,
        } => World::Battle,
        Mode::Transition { frame } => {
            if frame >= TRANSITION_FRAMES / 2 {
                World::Battle
            } else {
                World::Field
            }
        }
        _ => World::Field,
    }
}

fn overlay(mode: Mode) -> bool {
    matches!(
        mode,
        Mode::System(_)
            | Mode::QuitAsk(_)
            | Mode::LoadList {
                from_title: false,
                ..
            }
            | Mode::Party
            | Mode::Skills
            | Mode::Inventory
    )
}

fn transition_keep(frame: u16) -> u32 {
    let half = u32::from(TRANSITION_FRAMES / 2).max(1);
    let frame = u32::from(frame);
    if frame < half {
        255 - frame * 255 / half
    } else {
        (frame - half) * 255 / half
    }
}

fn draw_field(fb: &mut [u32], game: &Game) {
    fb.fill(BLOCKED);
    let (cam_x, cam_y) = game.camera();
    let tile = TILE * SCALE;
    let x0 = cam_x.div_euclid(tile).clamp(0, MAP_W);
    let y0 = cam_y.div_euclid(tile).clamp(0, MAP_H);
    let x1 = (cam_x + WIDTH + tile - 1).div_euclid(tile).clamp(0, MAP_W);
    let y1 = (cam_y + HEIGHT + tile - 1).div_euclid(tile).clamp(0, MAP_H);
    let grass = grass_color();
    for ty in y0..y1 {
        for tx in x0..x1 {
            let x = tx * tile - cam_x;
            let y = ty * tile - cam_y;
            match terrain_at(tx, ty) {
                Some(Terrain::Grass) => {
                    let id = if tx < ROAD_X {
                        match tile_hash(tx, ty) % 10 {
                            0 => atlas::SPR_FLOWER,
                            1 => atlas::SPR_BLOOM,
                            _ => atlas::SPR_GRASS,
                        }
                    } else {
                        atlas::SPR_GRASS
                    };
                    paint_tile(fb, id, x, y, grass);
                }
                Some(terrain) => {
                    let (id, fallback) = tile_paint(terrain);
                    paint_tile(fb, id, x, y, fallback);
                }
                None => fill(fb, x, y, tile, tile, BLOCKED),
            }
        }
    }
    for &(tx, ty) in &MUSHROOMS {
        if matches!(terrain_at(tx, ty), Some(Terrain::Grass)) {
            blit(
                fb,
                atlas::SPR_MUSHROOM,
                tx * tile - cam_x,
                ty * tile - cam_y,
                SCALE,
            );
        }
    }
    if game.monster_alive {
        let x = game.mx * tile - cam_x;
        let y = game.my * tile - cam_y;
        shadow(fb, x, y, SCALE);
        blit(fb, atlas::SPR_SLIME, x, y, SCALE);
    }
    let (px, mut py) = game.player_px();
    if game.bobbing() {
        py -= 1;
    }
    let x = px * SCALE - cam_x;
    let y = py * SCALE - cam_y;
    shadow(fb, x, y, SCALE);
    blit(fb, atlas::SPR_PLAYER, x, y, SCALE);

    if show_talk(game.mode) {
        draw_dialogue(fb, game);
    } else if game.mode == Mode::Field && game.near_monster() {
        draw_prompt(fb);
    }
}

fn show_talk(mode: Mode) -> bool {
    matches!(
        mode,
        Mode::Talk
            | Mode::System(Back::Talk)
            | Mode::QuitAsk(Back::Talk)
            | Mode::LoadList {
                from_title: false,
                back: Back::Talk,
            }
    )
}

fn tile_paint(terrain: Terrain) -> (i32, u32) {
    match terrain {
        Terrain::Grass => (atlas::SPR_GRASS, grass_color()),
        Terrain::RoadLeft => (atlas::SPR_ROAD_L, DIRT),
        Terrain::Road => (atlas::SPR_ROAD, DIRT),
        Terrain::RoadRight => (atlas::SPR_ROAD_R, DIRT),
        Terrain::RoofRed => (atlas::SPR_ROOF_RED, ROOF_RED),
        Terrain::WallWood => (atlas::SPR_WALL_WOOD, WALL_WOOD),
        Terrain::RoofBlue => (atlas::SPR_ROOF_BLUE, ROOF_BLUE),
        Terrain::WallStone => (atlas::SPR_WALL_STONE, WALL_STONE),
    }
}

fn paint_tile(fb: &mut [u32], id: i32, x: i32, y: i32, fallback: u32) {
    if has_sprite(id) {
        blit(fb, id, x, y, SCALE);
    } else {
        fill(fb, x, y, TILE * SCALE, TILE * SCALE, fallback);
    }
}

fn has_sprite(id: i32) -> bool {
    id >= 0 && (id + 1) * TILE <= ATLAS_W && ATLAS.len() >= (ATLAS_W * TILE * 4) as usize
}

fn draw_prompt(fb: &mut [u32]) {
    let label = "Z Talk";
    let w = text_width(label) + 96;
    let h = 120;
    let x = (WIDTH - w) / 2;
    let y = HEIGHT - h - 36;
    panel(fb, x, y, w, h);
    text(fb, x + 48, y + 36, label, GOLD);
}

fn draw_dialogue(fb: &mut [u32], game: &Game) {
    let x = 48;
    let y = 744;
    let w = WIDTH - 96;
    let h = 300;
    panel(fb, x, y, w, h);
    text(fb, x + 60, y + 48, game.monster.name, SLIME);
    text(fb, x + 60, y + 156, TALK_LINE, CREAM);
    text(fb, x + w - 108, y + 156, "Z", GOLD);
}

fn draw_battle(fb: &mut [u32], game: &Game) {
    fb.fill(BATTLE_BG);
    fill(fb, 0, 432, WIDTH, 288, BATTLE_GROUND);
    fill(fb, 0, 432, WIDTH, 6, HORIZON);

    let sprite = TILE * PORTRAIT;
    let slime_x = WIDTH - 288 - sprite;
    shadow(fb, 288, 132, PORTRAIT);
    blit(fb, atlas::SPR_PLAYER, 288, 132, PORTRAIT);
    if game.monster.hp > 0 {
        shadow(fb, slime_x, 132, PORTRAIT);
        blit(fb, atlas::SPR_SLIME, slime_x, 132, PORTRAIT);
    }

    fighter_card(fb, 144, false, &game.player, GOLD);
    fighter_card(fb, WIDTH - 144, true, &game.monster, SLIME);

    panel(fb, 48, 708, WIDTH - 96, 336);
    let choosing = matches!(game.battle.phase, Phase::Choose);
    for (i, name) in COMMANDS.iter().enumerate() {
        let y = 756 + i as i32 * 84;
        let selected = choosing && i == game.battle.cursor;
        if selected {
            fill(fb, 96, y - 12, 468, 72, SELECT);
        }
        let color = if selected {
            GOLD
        } else if choosing {
            CREAM
        } else {
            DIM
        };
        if selected {
            text(fb, 108, y, ">", GOLD);
        }
        text(fb, 180, y, name, color);
    }
    let log_x = 768;
    let log_bottom = 708 + 336 - 60;
    let lines = game.battle.log.len() as i32;
    for (i, line) in game.battle.log.iter().enumerate() {
        let y = log_bottom - (lines - 1 - i as i32) * 72;
        let color = if i + 1 == game.battle.log.len() {
            CREAM
        } else {
            DIM
        };
        text_clip(fb, log_x, y, line, color, WIDTH - 96);
    }
}

fn fighter_card(
    fb: &mut [u32],
    edge: i32,
    right: bool,
    fighter: &crate::game::Fighter,
    color: u32,
) {
    let bar_w = 624;
    let x = if right { edge - bar_w } else { edge };
    let hp = format!("{}/{}", fighter.hp.max(0), fighter.max_hp);
    text(fb, x, 468, fighter.name, color);
    text(fb, x + bar_w - text_width(&hp), 468, &hp, CREAM);
    hp_bar(fb, x, 540, bar_w, 48, fighter.hp, fighter.max_hp);
}

fn draw_system(fb: &mut [u32], game: &Game) {
    let audio = format!("{}%", (game.master_gain() * 100.0).round() as i32);
    let resolution = format!("{}x{}", game.window_w, game.window_h);
    let rows = [
        (SYSTEM_ITEMS[0], None),
        (SYSTEM_ITEMS[1], None),
        (SYSTEM_ITEMS[2], Some(audio.as_str())),
        (SYSTEM_ITEMS[3], Some(resolution.as_str())),
        (SYSTEM_ITEMS[4], None),
        (SYSTEM_ITEMS[5], None),
    ];
    draw_menu(fb, "System", &rows, game.menu_cursor, game.notice);
}

fn draw_load(fb: &mut [u32], game: &Game) {
    if game.slots.is_empty() {
        let rows = [("No saves", None)];
        draw_menu(fb, "Load", &rows, usize::MAX, game.notice);
        return;
    }
    let owned: Vec<String> = game
        .slots
        .iter()
        .map(|id| format!("save-{id:03}"))
        .collect();
    let rows: Vec<(&str, Option<&str>)> =
        owned.iter().map(|label| (label.as_str(), None)).collect();
    draw_menu(fb, "Load", &rows, game.menu_cursor, game.notice);
}

fn draw_menu(
    fb: &mut [u32],
    title: &str,
    rows: &[(&str, Option<&str>)],
    cursor: usize,
    notice: Option<&str>,
) {
    let w = 960;
    let row_h = 78;
    let head = if title.is_empty() { 48 } else { 132 };
    let foot = if notice.is_some() { 96 } else { 48 };
    let h = head + rows.len() as i32 * row_h + foot;
    let x = (WIDTH - w) / 2;
    let y = (HEIGHT - h) / 2;
    panel(fb, x, y, w, h);
    if !title.is_empty() {
        text_centered(fb, WIDTH / 2, y + 40, title, GOLD);
    }
    let left = x + 72;
    for (i, (label, value)) in rows.iter().enumerate() {
        let ry = y + head + i as i32 * row_h;
        let selected = i == cursor;
        if selected {
            fill(fb, x + 40, ry - 16, w - 80, row_h - 14, SELECT);
            text(fb, left, ry, ">", GOLD);
        }
        text(fb, left + 48, ry, label, if selected { CREAM } else { DIM });
        if let Some(value) = value {
            let color = if selected { GOLD } else { CREAM };
            text(fb, x + w - 72 - text_width(value), ry, value, color);
        }
    }
    if let Some(notice) = notice {
        text_centered(fb, WIDTH / 2, y + h - 64, notice, GOLD);
    }
}

fn draw_party(fb: &mut [u32], game: &Game) {
    let w = 1080;
    let h = (760 + game.characters.len() as i32 * 72).min(HEIGHT - 48);
    let x = (WIDTH - w) / 2;
    let y = (HEIGHT - h) / 2;
    panel(fb, x, y, w, h);
    text_centered(fb, WIDTH / 2, y + 40, "Party", GOLD);
    let left = x + 72;
    let mut line = y + 140;
    let selected = game
        .party_index
        .min(game.characters.len().saturating_sub(1));
    for (i, character) in game.characters.iter().enumerate() {
        let active = game.menu_cursor == 0 && i == selected;
        if active {
            fill(fb, x + 40, line - 16, w - 80, 64, SELECT);
            text(fb, left, line, ">", GOLD);
        }
        let color = if i == selected { CREAM } else { DIM };
        text(fb, left + 48, line, &character.name, color);
        line += 72;
    }
    line += 16;
    text(fb, left, line, "Equip", DIM);
    line += 72;
    let character = game.selected();
    text(fb, left, line, &character.weapon_name, CREAM);
    line += 64;
    let bonus = format!("+{} attack", character.weapon_atk);
    text(fb, left, line, &bonus, GOLD);
    line += 100;
    for (index, label) in [(1, "Skills"), (2, "Inventory")] {
        if game.menu_cursor == index {
            fill(fb, x + 40, line - 16, w - 80, 64, SELECT);
            text(fb, left, line, ">", GOLD);
        }
        let color = if game.menu_cursor == index {
            CREAM
        } else {
            DIM
        };
        text(fb, left + 48, line, label, color);
        line += 78;
    }
}

fn draw_list(fb: &mut [u32], title: &str, name: &str, lines: &[String], placeholder: bool) {
    let w = 960;
    let h = 640.max(220 + lines.len() as i32 * 64);
    let x = (WIDTH - w) / 2;
    let y = (HEIGHT - h) / 2;
    panel(fb, x, y, w, h);
    text_centered(fb, WIDTH / 2, y + 40, title, GOLD);
    let left = x + 72;
    text(fb, left, y + 150, name, CREAM);
    let color = if placeholder { DIM } else { CREAM };
    for (i, line) in lines.iter().enumerate() {
        text(fb, left, y + 250 + i as i32 * 64, line, color);
    }
}

fn grass_color() -> u32 {
    let i = ((8 * ATLAS_W + 8) * 4) as usize;
    ((ATLAS[i] as u32) << 16) | ((ATLAS[i + 1] as u32) << 8) | ATLAS[i + 2] as u32
}

fn tile_hash(x: i32, y: i32) -> u32 {
    let n = x
        .wrapping_mul(374761393)
        .wrapping_add(y.wrapping_mul(668265263));
    let n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    n as u32
}

fn blit(fb: &mut [u32], id: i32, dx: i32, dy: i32, scale: i32) {
    let origin = id * TILE;
    for y in 0..TILE {
        for x in 0..TILE {
            let si = ((y * ATLAS_W + origin + x) * 4) as usize;
            if ATLAS[si + 3] < 128 {
                continue;
            }
            let color =
                ((ATLAS[si] as u32) << 16) | ((ATLAS[si + 1] as u32) << 8) | ATLAS[si + 2] as u32;
            for sy in 0..scale {
                for sx in 0..scale {
                    plot(fb, dx + x * scale + sx, dy + y * scale + sy, color);
                }
            }
        }
    }
}

fn shadow(fb: &mut [u32], x: i32, y: i32, scale: i32) {
    let w = 10 * scale;
    let h = 3 * scale;
    let ox = x + 3 * scale;
    let oy = y + 13 * scale;
    for yy in 0..h {
        for xx in 0..w {
            dim(fb, ox + xx, oy + yy, 150);
        }
    }
}

fn dim(fb: &mut [u32], x: i32, y: i32, keep: u32) {
    if x < 0 || y < 0 || x >= WIDTH || y >= HEIGHT {
        return;
    }
    let i = (y * WIDTH + x) as usize;
    let p = fb[i];
    let r = ((p >> 16) & 255) * keep / 255;
    let g = ((p >> 8) & 255) * keep / 255;
    let b = (p & 255) * keep / 255;
    fb[i] = (r << 16) | (g << 8) | b;
}

fn shade(fb: &mut [u32], keep: u32) {
    if keep >= 255 {
        return;
    }
    for p in fb {
        let r = ((*p >> 16) & 255) * keep / 255;
        let g = ((*p >> 8) & 255) * keep / 255;
        let b = (*p & 255) * keep / 255;
        *p = (r << 16) | (g << 8) | b;
    }
}

fn panel(fb: &mut [u32], x: i32, y: i32, w: i32, h: i32) {
    let edge = 6;
    fill(fb, x, y, w, h, FRAME);
    fill(fb, x + edge, y + edge, w - edge * 2, h - edge * 2, FRAME_IN);
    fill(
        fb,
        x + edge * 3,
        y + edge * 3,
        w - edge * 6,
        h - edge * 6,
        PANEL,
    );
}

fn fill(fb: &mut [u32], x: i32, y: i32, w: i32, h: i32, color: u32) {
    if w <= 0 || h <= 0 {
        return;
    }
    for yy in y..(y + h) {
        if yy < 0 || yy >= HEIGHT {
            continue;
        }
        for xx in x..(x + w) {
            if xx < 0 || xx >= WIDTH {
                continue;
            }
            fb[(yy * WIDTH + xx) as usize] = color;
        }
    }
}

fn hp_bar(fb: &mut [u32], x: i32, y: i32, w: i32, h: i32, hp: i32, max: i32) {
    let t = 6;
    fill(fb, x, y, w, h, FRAME);
    fill(fb, x + t, y + t, w - t * 2, h - t * 2, HP_BG);
    if max <= 0 || hp <= 0 || w <= t * 2 || h <= t * 2 {
        return;
    }
    let inner = w - t * 2;
    let filled = inner * hp.min(max) / max;
    let color = if hp * 4 <= max { HP_LOW } else { HP_OK };
    fill(fb, x + t, y + t, filled, h - t * 2, color);
}

fn plot(fb: &mut [u32], x: i32, y: i32, color: u32) {
    if x < 0 || y < 0 || x >= WIDTH || y >= HEIGHT {
        return;
    }
    fb[(y * WIDTH + x) as usize] = color;
}

fn text(fb: &mut [u32], x: i32, y: i32, s: &str, color: u32) {
    text_clip(fb, x, y, s, color, WIDTH);
}

fn text_centered(fb: &mut [u32], cx: i32, y: i32, s: &str, color: u32) {
    text(fb, cx - text_width(s) / 2, y, s, color);
}

fn text_clip(fb: &mut [u32], x: i32, y: i32, s: &str, color: u32, max_x: i32) {
    glyphs(
        fb,
        x + FONT_SCALE,
        y + FONT_SCALE,
        s,
        INK,
        FONT_SCALE,
        max_x,
    );
    glyphs(fb, x, y, s, color, FONT_SCALE, max_x);
}

fn banner(fb: &mut [u32], y: i32, s: &str, color: u32) {
    let scale = 6;
    let width = s.chars().count() as i32 * 8 * scale;
    let x = (WIDTH - width) / 2;
    glyphs(fb, x + scale, y + scale, s, INK, scale, WIDTH);
    glyphs(fb, x, y, s, color, scale, WIDTH);
}

fn glyphs(fb: &mut [u32], x: i32, y: i32, s: &str, color: u32, scale: i32, max_x: i32) {
    let step = 8 * scale;
    let mut cx = x;
    for ch in s.chars() {
        if cx + step > max_x {
            break;
        }
        let index = u32::from(ch);
        let glyph = if index < 128 {
            FONT[index as usize]
        } else {
            [0; 8]
        };
        for row in 0..8 {
            let bits = glyph[row];
            for col in 0..8 {
                if bits & (1 << col) != 0 {
                    let row = row as i32;
                    let col = col as i32;
                    fill(fb, cx + col * scale, y + row * scale, scale, scale, color);
                }
            }
        }
        cx += step;
    }
}

fn text_width(s: &str) -> i32 {
    s.chars().count() as i32 * 8 * FONT_SCALE
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{isolated_dir, Game, BUILDINGS};

    fn field() -> Game {
        let mut game = Game::with_save_dir(isolated_dir());
        game.start_new_game();
        game
    }

    fn is_green(p: u32) -> bool {
        let r = (p >> 16) & 255;
        let g = (p >> 8) & 255;
        let b = p & 255;
        g > r + 15 && g > b + 15
    }

    #[test]
    fn title_screen_is_not_the_field() {
        let mut fb = vec![0; (WIDTH * HEIGHT) as usize];
        let game = Game::with_save_dir(isolated_dir());
        assert_eq!(game.mode, crate::game::Mode::Title);
        draw(&mut fb, &game);
        assert_eq!(fb[0], TITLE_BG);
        let greens = fb.iter().filter(|p| is_green(**p)).count();
        assert!(greens < fb.len() / 20);
    }

    #[test]
    fn field_is_grass_around_an_inset_start() {
        let mut fb = vec![0; (WIDTH * HEIGHT) as usize];
        let game = field();
        let (cam_x, cam_y) = game.camera();
        assert!(cam_x > 0 && cam_y > 0);
        draw(&mut fb, &game);
        let greens = fb.iter().filter(|p| is_green(**p)).count();
        assert!(greens > fb.len() / 3, "grass pixels {greens}");
    }

    #[test]
    fn town_draws_roofs_walls_and_the_road() {
        let mut game = field();
        game.px = ROAD_X + 1;
        game.py = 20;
        let mut fb = vec![0; (WIDTH * HEIGHT) as usize];
        draw(&mut fb, &game);
        let (cam_x, cam_y) = game.camera();
        let (r, g, b) = average(&fb, BUILDINGS[0].x, BUILDINGS[0].y, cam_x, cam_y);
        assert!(r > g && r > b, "red roof was {r},{g},{b}");
        let (r, g, b) = average(&fb, BUILDINGS[1].x, BUILDINGS[1].y, cam_x, cam_y);
        assert!(b > r && b > g, "blue roof was {r},{g},{b}");
        let wall = BUILDINGS[0];
        let (r, g, b) = average(&fb, wall.x, wall.y + wall.roof_rows, cam_x, cam_y);
        assert!(r > b + 15, "wood wall was {r},{g},{b}");
        let (r, g, b) = average(&fb, ROAD_X + 1, 28, cam_x, cam_y);
        assert!(
            r > b + 20 && g > b + 20 && r + 30 > g,
            "road was {r},{g},{b}"
        );
    }

    fn average(fb: &[u32], tx: i32, ty: i32, cam_x: i32, cam_y: i32) -> (u32, u32, u32) {
        let origin_x = tx * TILE * SCALE - cam_x;
        let origin_y = ty * TILE * SCALE - cam_y;
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;
        let mut n = 0;
        for y in 8..24 {
            for x in 8..24 {
                let sx = origin_x + x;
                let sy = origin_y + y;
                assert!(
                    sx >= 0 && sy >= 0 && sx < WIDTH && sy < HEIGHT,
                    "tile {tx},{ty} at {sx},{sy} is off screen"
                );
                let p = fb[(sy * WIDTH + sx) as usize];
                r += (p >> 16) & 255;
                g += (p >> 8) & 255;
                b += p & 255;
                n += 1;
            }
        }
        (r / n, g / n, b / n)
    }

    #[test]
    fn west_of_the_road_still_scatters_flowers() {
        let mut flowers = 0;
        let mut center = 0;
        for y in 0..MAP_H {
            for x in 0..MAP_W {
                if x >= ROAD_X || !matches!(terrain_at(x, y), Some(Terrain::Grass)) {
                    continue;
                }
                if x > MAP_W / 4 && x < MAP_W / 2 {
                    center += 1;
                }
                if tile_hash(x, y) % 10 < 2 {
                    flowers += 1;
                }
            }
        }
        assert!(center > 100);
        assert!(flowers > 100);
    }

    #[test]
    fn kenney_road_roof_and_wall_are_packed() {
        assert!(has_sprite(atlas::SPR_ROAD));
        assert!(has_sprite(atlas::SPR_ROAD_L));
        assert!(has_sprite(atlas::SPR_ROAD_R));
        assert!(has_sprite(atlas::SPR_ROOF_RED));
        assert!(has_sprite(atlas::SPR_ROOF_BLUE));
        assert!(has_sprite(atlas::SPR_WALL_WOOD));
        assert!(has_sprite(atlas::SPR_WALL_STONE));
    }
}
