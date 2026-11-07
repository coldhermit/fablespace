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
    Back, Brush, EditPage, Game, Mode, Phase, Terrain, COMMANDS, HEIGHT, MAP_H, MAP_W, SCALE,
    SYSTEM_ITEMS, TITLE_ITEMS, TRANSITION_FRAMES, WIDTH,
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
                draw_menu(fb, "", &rows, game.menu_cursor, game.notice.as_deref());
            }
            return;
        }
        World::Edit => {
            draw_edit(fb, game);
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
                    game.notice.as_deref(),
                );
            }
            Mode::SaveAsk(_) => {
                let rows = [("Overwrite", None), ("New fork", None)];
                draw_menu(
                    fb,
                    "Overwrite or fork?",
                    &rows,
                    game.menu_cursor,
                    game.notice.as_deref(),
                );
            }
            Mode::NameSlot(_) => {
                let rows = [(game.name_buf.as_str(), None)];
                draw_menu(fb, "New fork", &rows, 0, game.notice.as_deref());
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
            Mode::SkillTree => draw_skill_tree(fb, game),
            _ => {}
        }
    }
}

enum World {
    Title,
    Field,
    Battle,
    Edit,
}

fn world(mode: Mode) -> World {
    match mode {
        Mode::Title
        | Mode::LoadList {
            from_title: true, ..
        } => World::Title,
        Mode::Edit => World::Edit,
        Mode::Battle
        | Mode::System(Back::Battle)
        | Mode::QuitAsk(Back::Battle)
        | Mode::SaveAsk(Back::Battle)
        | Mode::NameSlot(Back::Battle)
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
            | Mode::SaveAsk(_)
            | Mode::NameSlot(_)
            | Mode::LoadList {
                from_title: false,
                ..
            }
            | Mode::Party
            | Mode::Skills
            | Mode::Inventory
            | Mode::SkillTree
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
            match game.project.terrain(tx, ty) {
                Some(Terrain::Grass) => {
                    let id = if tx < game.project.road_x {
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
                Some(Terrain::Void) | None => fill(fb, x, y, tile, tile, BLOCKED),
                Some(Terrain::Custom(id)) => {
                    if !blit_sheet(fb, &game.project.sheet, i32::from(id), x, y, SCALE) {
                        fill(fb, x, y, tile, tile, 0x6688AA);
                    }
                }
                Some(terrain) => {
                    let (id, fallback) = tile_paint(terrain);
                    paint_tile(fb, id, x, y, fallback);
                }
            }
        }
    }
    for &(tx, ty) in &game.project.mushrooms {
        if matches!(game.project.terrain(tx, ty), Some(Terrain::Grass)) {
            blit(
                fb,
                atlas::SPR_MUSHROOM,
                tx * tile - cam_x,
                ty * tile - cam_y,
                SCALE,
            );
        }
    }
    for entity in &game.project.extras.entities {
        if entity.map == game.project.map_name || entity.map == "field" {
            let x = entity.x * tile - cam_x + 8;
            let y = entity.y * tile - cam_y + 10;
            fill(fb, x, y, 16, 14, 0xC4893A);
            fill(fb, x + 2, y - 4, 12, 4, 0xE2B15A);
        }
    }
    if game.monster_alive {
        let x = game.mx * tile - cam_x;
        let y = game.my * tile - cam_y;
        shadow(fb, x, y, SCALE);
        blit_actor(
            fb,
            &game.project.extras.sprites,
            game.anim,
            &game.monster.name,
            "overworld",
            atlas::SPR_SLIME,
            x,
            y,
            SCALE,
        );
    }
    let (px, mut py) = game.player_px();
    if game.bobbing() {
        py -= 1;
    }
    let x = px * SCALE - cam_x;
    let y = py * SCALE - cam_y;
    shadow(fb, x, y, SCALE);
    blit_actor(
        fb,
        &game.project.extras.sprites,
        game.anim,
        &game.player.name,
        "overworld",
        atlas::SPR_PLAYER,
        x,
        y,
        SCALE,
    );

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
            | Mode::SaveAsk(Back::Talk)
            | Mode::NameSlot(Back::Talk)
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
        Terrain::Void | Terrain::Custom(_) => (atlas::SPR_GRASS, BLOCKED),
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
    text(fb, x + 60, y + 36, &game.monster.name, SLIME);
    text_clip(fb, x + 60, y + 96, game.current_line(), CREAM, x + w - 48);
    let choices = game.current_choices();
    if choices.is_empty() {
        text(fb, x + w - 108, y + 96, "Z", GOLD);
        return;
    }
    for (i, choice) in choices.iter().enumerate().take(3) {
        let cy = y + 168 + i as i32 * 48;
        let selected = i == game.talk_choice;
        if selected {
            text(fb, x + 48, cy, ">", GOLD);
        }
        let color = if selected { CREAM } else { DIM };
        text_clip(fb, x + 96, cy, &choice.label, color, x + w - 48);
    }
}

fn draw_battle(fb: &mut [u32], game: &Game) {
    fb.fill(BATTLE_BG);
    fill(fb, 0, 432, WIDTH, 288, BATTLE_GROUND);
    fill(fb, 0, 432, WIDTH, 6, HORIZON);

    let sprite = TILE * PORTRAIT;
    let slime_x = WIDTH - 288 - sprite;
    shadow(fb, 288, 132, PORTRAIT);
    blit_actor(
        fb,
        &game.project.extras.sprites,
        game.anim,
        &game.player.name,
        "combat",
        atlas::SPR_PLAYER,
        288,
        132,
        PORTRAIT,
    );
    if game.monster.hp > 0 {
        shadow(fb, slime_x, 132, PORTRAIT);
        blit_actor(
            fb,
            &game.project.extras.sprites,
            game.anim,
            &game.monster.name,
            "combat",
            atlas::SPR_SLIME,
            slime_x,
            132,
            PORTRAIT,
        );
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
    text(fb, x, 468, &fighter.name, color);
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
    draw_menu(
        fb,
        "System",
        &rows,
        game.menu_cursor,
        game.notice.as_deref(),
    );
}

fn draw_load(fb: &mut [u32], game: &Game) {
    if game.slots.is_empty() {
        let rows = [("No saves", None)];
        draw_menu(fb, "Load", &rows, usize::MAX, game.notice.as_deref());
        return;
    }
    let rows: Vec<(&str, Option<&str>)> = game
        .slots
        .iter()
        .map(|name| (name.as_str(), None))
        .collect();
    draw_menu(fb, "Load", &rows, game.menu_cursor, game.notice.as_deref());
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
    let w = 1560;
    let h = 900.min(HEIGHT - 48);
    let x = (WIDTH - w) / 2;
    let y = (HEIGHT - h) / 2;
    panel(fb, x, y, w, h);
    text_centered(fb, WIDTH / 2, y + 36, "Party", GOLD);
    let character = game.selected();
    let selected = game
        .party_index
        .min(game.characters.len().saturating_sub(1));
    let mut line = y + 120;
    for (i, member) in game.characters.iter().enumerate() {
        let active = game.menu_cursor == 0 && i == selected;
        if active {
            fill(fb, x + 36, line - 12, 460, 52, SELECT);
            text(fb, x + 48, line, ">", GOLD);
        }
        let color = if i == selected { CREAM } else { DIM };
        text(fb, x + 96, line, &member.name, color);
        line += 64;
    }
    let col_y = y + 120 + game.characters.len() as i32 * 64 + 24;
    text(fb, x + 48, col_y, "Equipment", GOLD);
    text(fb, x + 560, col_y, "Inventory", GOLD);
    text(fb, x + 1060, col_y, "Skill tree", GOLD);
    text(fb, x + 48, col_y + 56, &character.weapon_name, CREAM);
    let bonus = format!("+{} attack", character.weapon_atk);
    text(fb, x + 48, col_y + 104, &bonus, DIM);
    let mut extra = col_y + 160;
    for (stat, value) in gear_lines(game) {
        text(fb, x + 48, extra, &format!("{stat} {value:+}"), CREAM);
        extra += 40;
    }
    if character.inventory.is_empty() {
        text(fb, x + 560, col_y + 56, "(empty)", DIM);
    } else {
        for (i, item) in character.inventory.iter().enumerate().take(8) {
            text(fb, x + 560, col_y + 56 + i as i32 * 44, &item.name, CREAM);
        }
    }
    if game.project.extras.tree.is_empty() {
        text(fb, x + 1060, col_y + 56, "No nodes yet.", DIM);
    } else {
        for (i, node) in game.project.extras.tree.iter().enumerate().take(8) {
            let owned = character.skills.iter().any(|skill| skill == &node.name);
            let label = if owned {
                format!("{}  owned", node.name)
            } else {
                node.name.clone()
            };
            text(
                fb,
                x + 1060,
                col_y + 56 + i as i32 * 44,
                &label,
                if owned { GOLD } else { CREAM },
            );
        }
    }
    let mut row = y + h - 250;
    for (index, label) in [(1, "Skills"), (2, "Inventory"), (3, "Skill Tree")] {
        if game.menu_cursor == index {
            fill(fb, x + 36, row - 12, w - 72, 52, SELECT);
            text(fb, x + 48, row, ">", GOLD);
        }
        let color = if game.menu_cursor == index { CREAM } else { DIM };
        text(fb, x + 96, row, label, color);
        row += 64;
    }
}

fn gear_lines(game: &Game) -> Vec<(String, i32)> {
    let Some(item) = game
        .project
        .extras
        .catalog
        .iter()
        .find(|item| item.name == game.selected().weapon_name)
    else {
        return Vec::new();
    };
    item.equip
        .iter()
        .filter(|modi| modi.stat != "atk")
        .map(|modi| (modi.stat.clone(), modi.delta))
        .collect()
}

fn draw_skill_tree(fb: &mut [u32], game: &Game) {
    let w = 1200;
    let h = 820;
    let x = (WIDTH - w) / 2;
    let y = (HEIGHT - h) / 2;
    panel(fb, x, y, w, h);
    text_centered(fb, WIDTH / 2, y + 40, "Skill tree", GOLD);
    let character = game.selected();
    text(fb, x + 64, y + 110, &character.name, CREAM);
    if game.project.extras.tree.is_empty() {
        text(fb, x + 64, y + 190, "No nodes yet.", DIM);
        return;
    }
    for (i, node) in game.project.extras.tree.iter().enumerate().take(8) {
        let row = y + 180 + i as i32 * 72;
        let selected = i == game.list_cursor;
        if selected {
            fill(fb, x + 40, row - 12, w - 80, 60, SELECT);
            text(fb, x + 56, row, ">", GOLD);
        }
        let owned = character.skills.iter().any(|skill| skill == &node.name);
        let cost = if node.cost.is_empty() {
            "free".to_string()
        } else {
            node.cost
                .iter()
                .map(|cost| format!("-{} {}", cost.delta, cost.stat))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let label = format!("{}   {cost}", node.name);
        text(fb, x + 104, row, &label, if owned { GOLD } else { CREAM });
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

fn blit_actor(
    fb: &mut [u32],
    sprites: &[crate::content::SpriteStrip],
    anim: u32,
    owner: &str,
    role: &str,
    fallback: i32,
    x: i32,
    y: i32,
    scale: i32,
) {
    let strip = sprites
        .iter()
        .find(|sprite| sprite.owner.eq_ignore_ascii_case(owner) && sprite.role == role)
        .or_else(|| {
            sprites
                .iter()
                .find(|sprite| sprite.owner.eq_ignore_ascii_case(owner) && sprite.role == "overworld")
        });
    if let Some(strip) = strip {
        let frames = strip.frames.max(1);
        let frame = (anim as i32 / 10).rem_euclid(frames);
        blit_strip(fb, &strip.pixels, frames, frame, x, y, scale);
    } else {
        blit(fb, fallback, x, y, scale);
    }
}

fn blit_strip(fb: &mut [u32], pixels: &[u8], frames: i32, frame: i32, dx: i32, dy: i32, scale: i32) {
    let stride = (TILE * TILE * 4) as usize;
    let start = frame.clamp(0, frames - 1) as usize * stride;
    let Some(tile) = pixels.get(start..start + stride) else {
        return;
    };
    for y in 0..TILE {
        for x in 0..TILE {
            let i = ((y * TILE + x) * 4) as usize;
            if tile[i + 3] < 128 {
                continue;
            }
            let color = ((tile[i] as u32) << 16) | ((tile[i + 1] as u32) << 8) | tile[i + 2] as u32;
            for sy in 0..scale {
                for sx in 0..scale {
                    plot(fb, dx + x * scale + sx, dy + y * scale + sy, color);
                }
            }
        }
    }
}

fn blit_sheet(
    fb: &mut [u32],
    sheet: &crate::project::Sheet,
    id: i32,
    dx: i32,
    dy: i32,
    scale: i32,
) -> bool {
    let Some(tile) = sheet.tile(id) else {
        return false;
    };
    for y in 0..TILE {
        for x in 0..TILE {
            let i = ((y * TILE + x) * 4) as usize;
            if tile[i + 3] < 128 {
                continue;
            }
            let color = ((tile[i] as u32) << 16) | ((tile[i + 1] as u32) << 8) | tile[i + 2] as u32;
            for sy in 0..scale {
                for sx in 0..scale {
                    plot(fb, dx + x * scale + sx, dy + y * scale + sy, color);
                }
            }
        }
    }
    true
}

fn cell_color(terrain: Terrain) -> u32 {
    match terrain {
        Terrain::Grass => grass_color(),
        Terrain::RoadLeft | Terrain::Road | Terrain::RoadRight => DIRT,
        Terrain::RoofRed => ROOF_RED,
        Terrain::WallWood => WALL_WOOD,
        Terrain::RoofBlue => ROOF_BLUE,
        Terrain::WallStone => WALL_STONE,
        Terrain::Void => BLOCKED,
        Terrain::Custom(_) => 0x6688AA,
    }
}

fn draw_edit(fb: &mut [u32], game: &Game) {
    fb.fill(TITLE_BG);
    text(fb, 48, 36, "Edit", GOLD);
    for (i, page) in EditPage::ALL.iter().enumerate() {
        let x = 240 + i as i32 * 300;
        let selected = *page == game.edit.page;
        if selected {
            fill(fb, x - 16, 24, 270, 48, SELECT);
        }
        text(fb, x, 36, page.name(), if selected { CREAM } else { DIM });
    }
    match game.edit.page {
        EditPage::Map => draw_edit_map(fb, game),
        EditPage::Tiles => draw_edit_tiles(fb, game),
        EditPage::Characters => draw_edit_characters(fb, game),
        EditPage::Dialogue => draw_edit_dialogue(fb, game),
    }
    if let Some(notice) = game.notice.as_deref() {
        text(fb, 48, HEIGHT - 48, notice, GOLD);
    }
}

fn draw_edit_map(fb: &mut [u32], game: &Game) {
    text(fb, 48, 110, "Paint", GOLD);
    for (i, brush) in Brush::PALETTE.iter().enumerate() {
        let y = 170 + i as i32 * 56;
        let selected = game.edit.brush == *brush;
        if selected {
            fill(fb, 36, y - 10, 400, 48, SELECT);
            text(fb, 48, y, ">", GOLD);
        }
        text(fb, 96, y, brush.label(), if selected { CREAM } else { DIM });
    }
    let origin_x = 500;
    let origin_y = 160;
    let cell = 8;
    for y in 0..MAP_H {
        for x in 0..MAP_W {
            let terrain = game.project.terrain(x, y).unwrap_or(Terrain::Void);
            fill(
                fb,
                origin_x + x * cell,
                origin_y + y * cell,
                cell,
                cell,
                cell_color(terrain),
            );
        }
    }
    let cx = origin_x + game.edit.cx * cell;
    let cy = origin_y + game.edit.cy * cell;
    fill(fb, cx, cy, cell, 2, GOLD);
    fill(fb, cx, cy + cell - 2, cell, 2, GOLD);
    fill(fb, cx, cy, 2, cell, GOLD);
    fill(fb, cx + cell - 2, cy, 2, cell, GOLD);
    text(
        fb,
        origin_x,
        origin_y + MAP_H * cell + 28,
        "Arrows move    Z paints    1-7 choose a brush",
        DIM,
    );
}

fn draw_edit_tiles(fb: &mut [u32], game: &Game) {
    text(fb, 48, 120, "Kenney sheet", GOLD);
    for id in 0..atlas::SPR_COUNT {
        blit(fb, id, 48 + id * 72, 180, 4);
    }
    text(fb, 48, 320, "Import a PNG path", GOLD);
    let path = if game.edit.tile_path.is_empty() {
        "(type a path, Z loads)".to_string()
    } else {
        game.edit.tile_path.clone()
    };
    text_clip(fb, 48, 380, &path, CREAM, WIDTH - 48);
    let count = format!("{} imported tiles", game.project.sheet.count);
    text(fb, 48, 460, &count, DIM);
    let shown = game.project.sheet.count.min(16);
    for id in 0..shown {
        blit_sheet(fb, &game.project.sheet, id, 48 + id * 72, 540, 4);
    }
    text(
        fb,
        48,
        680,
        "Left and right pick an imported cell. It becomes the map brush.",
        DIM,
    );
}

fn draw_edit_characters(fb: &mut [u32], game: &Game) {
    text(fb, 48, 120, "Party", GOLD);
    let labels = [
        "Character",
        "Add character",
        "Remove character",
        "Weapon",
        "Add skill",
        "Add item",
    ];
    for (i, label) in labels.iter().enumerate() {
        let y = 190 + i as i32 * 64;
        if i == game.edit.row {
            fill(fb, 36, y - 12, 520, 52, SELECT);
            text(fb, 48, y, ">", GOLD);
        }
        text(
            fb,
            96,
            y,
            label,
            if i == game.edit.row { CREAM } else { DIM },
        );
    }
    let index = game
        .edit
        .char_index
        .min(game.project.party.len().saturating_sub(1));
    if let Some(character) = game.project.party.get(index) {
        let x = 760;
        text(fb, x, 190, &character.name, GOLD);
        let weapon = format!("{}  +{}", character.weapon_name, character.weapon_atk);
        text(fb, x, 270, &weapon, CREAM);
        text(fb, x, 360, "Skills", GOLD);
        if character.skills.is_empty() {
            text(fb, x, 420, "(none)", DIM);
        }
        for (i, skill) in character.skills.iter().enumerate().take(6) {
            text(fb, x, 420 + i as i32 * 42, skill, CREAM);
        }
        text(fb, x, 700, "Inventory", GOLD);
        if character.inventory.is_empty() {
            text(fb, x, 760, "(empty)", DIM);
        }
        for (i, item) in character.inventory.iter().enumerate().take(4) {
            text(fb, x + 280, 760 + i as i32 * 42, &item.name, CREAM);
        }
    }
    text(
        fb,
        48,
        HEIGHT - 80,
        "Mira stays if she is the only character.",
        DIM,
    );
}

fn draw_edit_dialogue(fb: &mut [u32], game: &Game) {
    text(fb, 48, 110, "Nodes", GOLD);
    for (i, node) in game.project.talk.nodes.iter().enumerate() {
        let y = 170 + i as i32 * 52;
        let selected = i == game.edit.node_index;
        if selected {
            fill(fb, 36, y - 10, 440, 46, SELECT);
            text(fb, 48, y, ">", GOLD);
        }
        text(fb, 96, y, &node.id, if selected { CREAM } else { DIM });
    }
    let Some(node) = game.project.talk.nodes.get(game.edit.node_index) else {
        text(fb, 560, 170, "No nodes", DIM);
        return;
    };
    let x = 560;
    text(fb, x, 110, &node.id, GOLD);
    let line = if node.text.is_empty() {
        "(no line)".to_string()
    } else {
        node.text.clone()
    };
    text_clip(fb, x, 170, &line, CREAM, WIDTH - 64);

    text(fb, x, 240, "Requires", GOLD);
    if node.require.is_empty() {
        text(fb, x, 292, "(none)", DIM);
    }
    for (i, need) in node.require.iter().enumerate() {
        let y = 292 + i as i32 * 48;
        let state = if need.want { "on" } else { "off" };
        text(fb, x, y, &need.flag, CREAM);
        let color = if need.want { GOLD } else { HP_LOW };
        text(fb, x + 460, y, state, color);
    }

    let mut y = 300 + (node.require.len().max(1) as i32) * 48 + 24;
    text(fb, x, y, "Choices", GOLD);
    y += 52;
    if node.choices.is_empty() {
        text(fb, x, y, "(none)", DIM);
        y += 48;
    }
    for choice in &node.choices {
        let dest = format!("{}  ->  {}", choice.label, choice.goto);
        text_clip(fb, x, y, &dest, CREAM, WIDTH - 64);
        y += 44;
        if choice.sets.is_empty() {
            text(fb, x + 36, y, "sets none", DIM);
            y += 44;
        } else {
            for flag in &choice.sets {
                let sets = format!("sets {flag}");
                text(fb, x + 36, y, &sets, GOLD);
                y += 44;
            }
        }
    }
    y += 12;
    text(fb, x, y, "Battle", GOLD);
    y += 52;
    if let Some(battle) = &node.battle {
        let lead = format!("{} on {}", battle.entity, battle.map);
        text(fb, x, y, &lead, CREAM);
    } else {
        text(fb, x, y, "no battle", DIM);
    }
    text(
        fb,
        48,
        HEIGHT - 80,
        "Left Right changes the node or a required flag. Z adds a flag.",
        DIM,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{isolated_dir, terrain_at, Game, BUILDINGS, ROAD_X};

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
