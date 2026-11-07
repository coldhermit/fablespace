//! Code-drawn frame: grass field, dialogue, battle, and the pause menu.
//! Panels, bars, and glyphs are rectangles and a bitmap font. No UI images.

use crate::atlas::{self, ATLAS, ATLAS_W, TILE};
use crate::font::FONT;
use crate::game::{
    Back, Game, Mode, Phase, COMMANDS, HEIGHT, TALK_LINE, TRANSITION_FRAMES, WEAPON, WIDTH,
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

const MUSHROOMS: [(i32, i32); 4] = [(4, 3), (15, 2), (12, 8), (6, 6)];

pub fn draw(fb: &mut [u32], game: &Game) {
    debug_assert_eq!(fb.len(), (WIDTH * HEIGHT) as usize);
    if shows_battle(game.mode) {
        draw_battle(fb, game);
    } else {
        draw_field(fb, game);
    }
    if let Mode::Transition { frame } = game.mode {
        shade(fb, transition_keep(frame));
    }
    if matches!(game.mode, Mode::Pause(_)) {
        shade(fb, 85);
        draw_pause(fb, game);
    }
}

fn shows_battle(mode: Mode) -> bool {
    match mode {
        Mode::Battle | Mode::Pause(Back::Battle) => true,
        Mode::Transition { frame } => frame >= TRANSITION_FRAMES / 2,
        _ => false,
    }
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
    let fill = grass_color();
    fb.fill(fill);
    for ty in 0..crate::game::MAP_H {
        for tx in 0..crate::game::MAP_W {
            let id = match tile_hash(tx, ty) % 10 {
                0 => atlas::SPR_FLOWER,
                1 => atlas::SPR_BLOOM,
                _ => atlas::SPR_GRASS,
            };
            blit(fb, id, tx * TILE, ty * TILE, 1);
        }
    }
    for &(tx, ty) in &MUSHROOMS {
        blit(fb, atlas::SPR_MUSHROOM, tx * TILE, ty * TILE, 1);
    }
    if game.monster_alive {
        let (x, y) = (game.mx * TILE, game.my * TILE);
        shadow(fb, x, y, 1);
        blit(fb, atlas::SPR_SLIME, x, y, 1);
    }
    let (x, mut y) = game.player_px();
    if game.bobbing() {
        y -= 1;
    }
    shadow(fb, x, y, 1);
    blit(fb, atlas::SPR_PLAYER, x, y, 1);

    if matches!(game.mode, Mode::Talk | Mode::Pause(Back::Talk)) {
        draw_dialogue(fb, game);
    } else if game.mode == Mode::Field && game.near_monster() {
        draw_prompt(fb);
    }
}

fn draw_prompt(fb: &mut [u32]) {
    let label = "Z Talk";
    let w = text_width(label) + 16;
    let h = 20;
    let x = (WIDTH - w) / 2;
    let y = HEIGHT - h - 6;
    panel(fb, x, y, w, h);
    text(fb, x + 8, y + 6, label, GOLD);
}

fn draw_dialogue(fb: &mut [u32], game: &Game) {
    let x = 8;
    let y = 124;
    let w = WIDTH - 16;
    let h = 50;
    panel(fb, x, y, w, h);
    text(fb, x + 10, y + 8, game.monster.name, SLIME);
    text(fb, x + 10, y + 26, TALK_LINE, CREAM);
    text(fb, x + w - 18, y + 26, "Z", GOLD);
}

fn draw_battle(fb: &mut [u32], game: &Game) {
    fb.fill(BATTLE_BG);
    fill(fb, 0, 72, WIDTH, 48, BATTLE_GROUND);
    fill(fb, 0, 72, WIDTH, 1, HORIZON);

    let scale = 3;
    let sprite = TILE * scale;
    let slime_x = WIDTH - 48 - sprite;
    shadow(fb, 48, 22, scale);
    blit(fb, atlas::SPR_PLAYER, 48, 22, scale);
    if game.monster.hp > 0 {
        shadow(fb, slime_x, 22, scale);
        blit(fb, atlas::SPR_SLIME, slime_x, 22, scale);
    }

    fighter_card(fb, 24, false, &game.player, GOLD);
    fighter_card(fb, WIDTH - 24, true, &game.monster, SLIME);

    panel(fb, 8, 118, WIDTH - 16, 56);
    let choosing = matches!(game.battle.phase, Phase::Choose);
    for (i, name) in COMMANDS.iter().enumerate() {
        let y = 126 + i as i32 * 14;
        let selected = choosing && i == game.battle.cursor;
        if selected {
            fill(fb, 16, y - 2, 78, 12, SELECT);
        }
        let color = if selected {
            GOLD
        } else if choosing {
            CREAM
        } else {
            DIM
        };
        if selected {
            text(fb, 18, y, ">", GOLD);
        }
        text(fb, 30, y, name, color);
    }
    let log_x = 128;
    let log_bottom = 118 + 56 - 10;
    let lines = game.battle.log.len() as i32;
    for (i, line) in game.battle.log.iter().enumerate() {
        let y = log_bottom - (lines - 1 - i as i32) * 12;
        let color = if i + 1 == game.battle.log.len() {
            CREAM
        } else {
            DIM
        };
        text_clip(fb, log_x, y, line, color, WIDTH - 16);
    }
}

fn fighter_card(
    fb: &mut [u32],
    edge: i32,
    right: bool,
    fighter: &crate::game::Fighter,
    color: u32,
) {
    let bar_w = 104;
    let x = if right { edge - bar_w } else { edge };
    let hp = format!("{}/{}", fighter.hp.max(0), fighter.max_hp);
    text(fb, x, 78, fighter.name, color);
    text(fb, x + bar_w - text_width(&hp), 78, &hp, CREAM);
    hp_bar(fb, x, 90, bar_w, 8, fighter.hp, fighter.max_hp);
}

fn draw_pause(fb: &mut [u32], game: &Game) {
    let w = 200;
    let h = 128;
    let x = (WIDTH - w) / 2;
    let y = (HEIGHT - h) / 2;
    panel(fb, x, y, w, h);
    let left = x + 12;
    text_centered(fb, WIDTH / 2, y + 8, "Pause", GOLD);
    text(fb, left, y + 26, "Party", DIM);
    text(fb, left, y + 40, game.player.name, CREAM);
    let hp = format!("{}/{}", game.player.hp, game.player.max_hp);
    text(fb, x + w - 12 - text_width(&hp), y + 40, &hp, CREAM);
    hp_bar(
        fb,
        left,
        y + 54,
        w - 24,
        8,
        game.player.hp,
        game.player.max_hp,
    );
    text(fb, left, y + 70, "Equip", DIM);
    let slot_w = w - 24;
    outline(fb, left, y + 84, slot_w, 16, FRAME);
    text(fb, left + 6, y + 88, WEAPON.name, CREAM);
    let bonus = format!("+{} attack", WEAPON.atk);
    text(fb, left, y + 106, &bonus, GOLD);
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
    fill(fb, x, y, w, h, FRAME);
    fill(fb, x + 1, y + 1, w - 2, h - 2, FRAME_IN);
    fill(fb, x + 3, y + 3, w - 6, h - 6, PANEL);
}

fn outline(fb: &mut [u32], x: i32, y: i32, w: i32, h: i32, color: u32) {
    fill(fb, x, y, w, 1, color);
    fill(fb, x, y + h - 1, w, 1, color);
    fill(fb, x, y, 1, h, color);
    fill(fb, x + w - 1, y, 1, h, color);
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
    fill(fb, x, y, w, h, FRAME);
    fill(fb, x + 1, y + 1, w - 2, h - 2, HP_BG);
    if max <= 0 || hp <= 0 {
        return;
    }
    let inner = w - 2;
    let filled = inner * hp.min(max) / max;
    let color = if hp * 4 <= max { HP_LOW } else { HP_OK };
    fill(fb, x + 1, y + 1, filled, h - 2, color);
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
    glyphs(fb, x + 1, y + 1, s, INK, max_x);
    glyphs(fb, x, y, s, color, max_x);
}

fn glyphs(fb: &mut [u32], x: i32, y: i32, s: &str, color: u32, max_x: i32) {
    let mut cx = x;
    for ch in s.chars() {
        if cx + 8 > max_x {
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
                    plot(fb, cx + col as i32, y + row as i32, color);
                }
            }
        }
        cx += 8;
    }
}

fn text_width(s: &str) -> i32 {
    s.chars().count() as i32 * 8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Game;

    #[test]
    fn field_corner_is_grass() {
        let mut fb = vec![0; (WIDTH * HEIGHT) as usize];
        draw(&mut fb, &Game::new());
        let p = fb[(8 * WIDTH + 8) as usize];
        let r = (p >> 16) & 255;
        let g = (p >> 8) & 255;
        let b = p & 255;
        assert!(g > r + 20 && g > b + 20, "grass pixel was {p:#08x}");
    }
}
