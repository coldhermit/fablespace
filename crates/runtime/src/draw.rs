//! Everything the window paints, in 544×416 game pixels.

use fablespace_data::tiles::{character_frame, tile_cell, upper_draw_order, UpperDraw};
use fablespace_data::{Priority, TILE, VIEW_TILES_X, VIEW_TILES_Y};

use crate::battle::{Battle, Fighter, Pending, Phase};
use crate::gpu::Sprite;
use crate::menu::{play_clock, MenuPage, EQUIP_LABELS};
use crate::party;
use crate::session::{Screen, Session};

pub const VIEW_W: u32 = TILE * VIEW_TILES_X;
pub const VIEW_H: u32 = TILE * VIEW_TILES_Y;

const INK: [f32; 4] = [0.95, 0.92, 0.84, 1.0];
const GOLD: [f32; 4] = [0.95, 0.78, 0.38, 1.0];
const DIM: [f32; 4] = [0.62, 0.68, 0.6, 1.0];
const EDGE: [f32; 4] = [0.55, 0.74, 0.5, 1.0];

pub fn frame(session: &Session, image: &mut dyn FnMut(&str) -> Option<(u32, u32)>) -> Vec<Sprite> {
    let mut out = Vec::new();
    match session.screen {
        Screen::Title => {
            draw_title(&mut out, session, image);
            if session.save_ui.is_some() {
                draw_save(&mut out, session);
            }
        }
        Screen::GameOver => draw_game_over(&mut out),
        Screen::Map => {
            draw_map(&mut out, session, image);
            if session.fade > 0 {
                let alpha = session.fade as f32 / 16.0;
                rect(
                    &mut out,
                    0,
                    0,
                    VIEW_W as i32,
                    VIEW_H as i32,
                    [0.0, 0.0, 0.0, alpha],
                );
            }
            if session.battle.is_some() {
                draw_battle(&mut out, session, image);
            } else if session.menu.is_some() {
                draw_menu(&mut out, session);
            }
            if session.save_ui.is_some() {
                draw_save(&mut out, session);
            }
            if session.choice.is_some() {
                draw_choice(&mut out, session);
            }
            if session.message.is_some() {
                draw_message(&mut out, session, image);
            } else if session.battle.is_none()
                && session.menu.is_none()
                && session.save_ui.is_none()
            {
                let name = session.map_name(session.map_id);
                panel(&mut out, 8, 8, text_width(&name, 1) + 16, 18);
                text(&mut out, 16, 12, 1, &name, GOLD);
            }
            if session.notice_timer > 0 && !session.notice.is_empty() {
                let w = text_width(&session.notice, 1) + 16;
                let x = (VIEW_W as i32 - w) / 2;
                panel(&mut out, x, 28, w, 18);
                text(&mut out, x + 8, 32, 1, &session.notice, INK);
            }
        }
    }
    out
}

fn draw_title(
    out: &mut Vec<Sprite>,
    session: &Session,
    image: &mut dyn FnMut(&str) -> Option<(u32, u32)>,
) {
    let key = format!("titles/{}", session.game.system.title_image);
    if let Some((w, h)) = image(&key) {
        blit(
            out,
            key,
            (0, 0, w, h),
            (0, 0, VIEW_W as i32, VIEW_H as i32),
            [1.0, 1.0, 1.0, 1.0],
        );
    } else {
        rect(out, 0, 0, VIEW_W as i32, 220, [0.45, 0.62, 0.78, 1.0]);
        rect(
            out,
            0,
            220,
            VIEW_W as i32,
            VIEW_H as i32 - 220,
            [0.36, 0.55, 0.32, 1.0],
        );
        rect(out, 0, 210, VIEW_W as i32, 24, [0.55, 0.72, 0.38, 1.0]);
    }
    let title = session.game.system.game_title.clone();
    let tw = text_width(&title, 3);
    text(out, (VIEW_W as i32 - tw) / 2, 48, 3, &title, GOLD);
    let terms = &session.game.system.terms;
    let labels = [&terms.new_game, &terms.continue_game, &terms.shutdown];
    let w = 180;
    let h = 78;
    let x = (VIEW_W as i32 - w) / 2;
    let y = VIEW_H as i32 - h - 36;
    panel(out, x, y, w, h);
    for (index, label) in labels.iter().enumerate() {
        let row = y + 8 + index as i32 * 22;
        if index == session.title_cursor {
            rect(out, x + 8, row - 2, w - 16, 18, [0.16, 0.3, 0.2, 1.0]);
            text(out, x + 16, row, 1, label, GOLD);
        } else {
            text(out, x + 16, row, 1, label, INK);
        }
    }
}

fn draw_game_over(out: &mut Vec<Sprite>) {
    rect(
        out,
        0,
        0,
        VIEW_W as i32,
        VIEW_H as i32,
        [0.05, 0.04, 0.06, 1.0],
    );
    let line = "Game Over";
    text(
        out,
        (VIEW_W as i32 - text_width(line, 3)) / 2,
        170,
        3,
        line,
        GOLD,
    );
    let hint = "Confirm to return";
    text(
        out,
        (VIEW_W as i32 - text_width(hint, 1)) / 2,
        220,
        1,
        hint,
        INK,
    );
}

fn draw_map(
    out: &mut Vec<Sprite>,
    session: &Session,
    image: &mut dyn FnMut(&str) -> Option<(u32, u32)>,
) {
    let Some(map) = session.current_map() else {
        rect(
            out,
            0,
            0,
            VIEW_W as i32,
            VIEW_H as i32,
            [0.1, 0.12, 0.1, 1.0],
        );
        return;
    };
    let Some(tileset) = session.game.tileset(map.tileset_id) else {
        return;
    };
    let (cam_x, cam_y) = camera(session);
    let key = format!("tilesets/{}", tileset.image);
    let tile = TILE as i32;
    let x0 = (cam_x / tile).max(0);
    let y0 = (cam_y / tile).max(0);
    let x1 = ((cam_x + VIEW_W as i32) / tile + 2).min(map.width as i32);
    let y1 = ((cam_y + VIEW_H as i32) / tile + 2).min(map.height as i32);
    let mut same: Vec<(i32, Sprite)> = Vec::new();
    let mut above: Vec<Sprite> = Vec::new();
    for y in y0..y1 {
        for x in x0..x1 {
            let dx = x * tile - cam_x;
            let dy = y * tile - cam_y;
            if let Some(cell) = tile_cell(tileset, map, true, x, y) {
                blit(
                    out,
                    key.clone(),
                    (cell.col * TILE, cell.row * TILE, TILE, TILE),
                    (dx, dy, tile, tile),
                    [1.0, 1.0, 1.0, 1.0],
                );
            } else {
                rect(out, dx, dy, tile, tile, [0.15, 0.18, 0.12, 1.0]);
            }
            let upper = map.upper_at(x, y);
            if upper == 0 {
                continue;
            }
            let Some(cell) = tile_cell(tileset, map, false, x, y) else {
                continue;
            };
            let sprite = Sprite {
                key: key.clone(),
                src: (cell.col * TILE, cell.row * TILE, TILE, TILE),
                dst: (dx, dy, tile, tile),
                color: [1.0, 1.0, 1.0, 1.0],
            };
            match upper_draw_order(tileset.passage(upper)) {
                UpperDraw::Below => out.push(sprite),
                UpperDraw::Same => same.push((dy + tile, sprite)),
                UpperDraw::Above => above.push(sprite),
            }
        }
    }
    for event in &session.events {
        push_mover(
            image,
            &event.body,
            cam_x,
            cam_y,
            &mut same,
            event.priority == Priority::Above,
        );
    }
    push_mover(image, &session.player, cam_x, cam_y, &mut same, false);
    same.sort_by_key(|(bottom, _)| *bottom);
    for (_, sprite) in same {
        out.push(sprite);
    }
    out.extend(above);
    for picture in &session.pictures {
        let key = format!("pictures/{}", picture.name);
        if let Some((w, h)) = image(&key) {
            blit(
                out,
                key,
                (0, 0, w, h),
                (picture.x, picture.y, w as i32, h as i32),
                [1.0, 1.0, 1.0, 1.0],
            );
        }
    }
}

fn push_mover(
    image: &mut dyn FnMut(&str) -> Option<(u32, u32)>,
    mover: &crate::session::Mover,
    cam_x: i32,
    cam_y: i32,
    same: &mut Vec<(i32, Sprite)>,
    force_above: bool,
) {
    if mover.character.is_empty() {
        return;
    }
    let key = format!("characters/{}", mover.character);
    let (px, py) = Session::sprite_pixel(mover);
    let pattern = Session::anim_pattern(mover);
    let (sw, sh, src) = if let Some((w, h)) = image(&key) {
        let frame = character_frame(w, h, mover.character_index, mover.dir, pattern);
        (frame.2 as i32, frame.3 as i32, frame)
    } else {
        (TILE as i32, TILE as i32, (0, 0, TILE, TILE))
    };
    let dx = px - cam_x + (TILE as i32 - sw) / 2;
    let dy = py - cam_y + TILE as i32 - sh;
    let sprite = if image(&key).is_some() {
        Sprite {
            key,
            src,
            dst: (dx, dy, sw, sh),
            color: [1.0, 1.0, 1.0, 1.0],
        }
    } else {
        Sprite {
            key: "white".into(),
            src: (0, 0, 1, 1),
            dst: (dx + 4, dy + 4, TILE as i32 - 8, TILE as i32 - 8),
            color: [0.75, 0.55, 0.35, 1.0],
        }
    };
    if force_above {
        same.push((10_000 + dy, sprite));
    } else {
        same.push((dy + sh, sprite));
    }
}

fn camera(session: &Session) -> (i32, i32) {
    let Some(map) = session.current_map() else {
        return (0, 0);
    };
    let map_w = map.width as i32 * TILE as i32;
    let map_h = map.height as i32 * TILE as i32;
    let (px, py) = Session::sprite_pixel(&session.player);
    let mut cam_x = px + TILE as i32 / 2 - VIEW_W as i32 / 2;
    let mut cam_y = py + TILE as i32 / 2 - VIEW_H as i32 / 2;
    if map_w <= VIEW_W as i32 {
        cam_x = -(VIEW_W as i32 - map_w) / 2;
    } else {
        cam_x = cam_x.clamp(0, map_w - VIEW_W as i32);
    }
    if map_h <= VIEW_H as i32 {
        cam_y = -(VIEW_H as i32 - map_h) / 2;
    } else {
        cam_y = cam_y.clamp(0, map_h - VIEW_H as i32);
    }
    (cam_x, cam_y)
}

fn draw_message(
    out: &mut Vec<Sprite>,
    session: &Session,
    image: &mut dyn FnMut(&str) -> Option<(u32, u32)>,
) {
    let Some(message) = &session.message else {
        return;
    };
    let face = !message.face.is_empty();
    let h = if face { 112 } else { 78 };
    let y = VIEW_H as i32 - h - 8;
    panel(out, 8, y, VIEW_W as i32 - 16, h);
    let mut text_x = 16;
    if face {
        let key = format!("faces/{}", message.face);
        if let Some((w, h)) = image(&key) {
            blit(
                out,
                key,
                (0, 0, w, h),
                (16, y + 8, 96, 96),
                [1.0, 1.0, 1.0, 1.0],
            );
        }
        text_x = 124;
    }
    if !message.speaker.is_empty() {
        text(out, text_x, y - 14, 1, &message.speaker, GOLD);
    }
    let lines = message
        .pages
        .get(message.page)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    for (index, line) in lines.iter().enumerate() {
        text(out, text_x, y + 12 + index as i32 * 14, 1, line, INK);
    }
}

fn draw_choice(out: &mut Vec<Sprite>, session: &Session) {
    let Some(choice) = &session.choice else {
        return;
    };
    let w = 200;
    let h = 12 + choice.options.len() as i32 * 16;
    let x = VIEW_W as i32 - w - 16;
    let y = VIEW_H as i32 - h - 96;
    panel(out, x, y, w, h);
    for (index, option) in choice.options.iter().enumerate() {
        let row = y + 6 + index as i32 * 16;
        if index == choice.cursor {
            rect(out, x + 6, row - 2, w - 12, 14, [0.16, 0.3, 0.2, 1.0]);
            text(out, x + 12, row, 1, option, GOLD);
        } else {
            text(out, x + 12, row, 1, option, INK);
        }
    }
}

fn draw_battle(
    out: &mut Vec<Sprite>,
    session: &Session,
    image: &mut dyn FnMut(&str) -> Option<(u32, u32)>,
) {
    let Some(battle) = &session.battle else {
        return;
    };
    rect(
        out,
        0,
        0,
        VIEW_W as i32,
        VIEW_H as i32,
        [0.02, 0.04, 0.05, 0.55],
    );
    rect(out, 0, 0, VIEW_W as i32, 200, [0.12, 0.16, 0.18, 0.35]);
    for enemy in &battle.enemies {
        if enemy.hp <= 0 && enemy.flash == 0 {
            continue;
        }
        let key = format!("enemies/{}", battler_name(session, enemy.enemy_id));
        let color = if enemy.flash > 0 {
            [1.0, 0.45, 0.4, 1.0]
        } else if enemy.hp <= 0 {
            [0.4, 0.4, 0.4, 0.4]
        } else {
            [1.0, 1.0, 1.0, 1.0]
        };
        if let Some((w, h)) = image(&key) {
            blit(
                out,
                key,
                (0, 0, w, h),
                (
                    enemy.x - w as i32 / 2,
                    enemy.y - h as i32 / 2,
                    w as i32,
                    h as i32,
                ),
                color,
            );
        } else {
            rect(
                out,
                enemy.x - 24,
                enemy.y - 20,
                48,
                36,
                [0.35, 0.7, 0.32, 1.0],
            );
        }
    }
    if let Some(index) = target_highlight(battle) {
        if let Some(enemy) = battle.enemies.get(index) {
            rect(out, enemy.x - 28, enemy.y + 28, 56, 3, GOLD);
        }
    }
    panel(out, 8, 250, 250, 156);
    text(out, 16, 256, 1, "Party", GOLD);
    for (index, actor) in battle.actors.iter().enumerate() {
        let y = 274 + index as i32 * 28;
        let name_color = if actor.hp <= 0 { DIM } else { INK };
        text(out, 16, y, 1, &actor.name, name_color);
        let hp = format!("{}/{}", actor.hp, actor.stats.mhp);
        text(out, 120, y, 1, &hp, INK);
        bar(
            out,
            16,
            y + 12,
            90,
            actor.hp,
            actor.stats.mhp,
            [0.75, 0.28, 0.28, 1.0],
        );
        bar(
            out,
            112,
            y + 12,
            90,
            actor.mp,
            actor.stats.mmp.max(1),
            [0.28, 0.45, 0.8, 1.0],
        );
    }
    if session.message.is_none() {
        draw_battle_commands(out, session, battle);
    }
}

fn battler_name(session: &Session, enemy_id: u32) -> String {
    session
        .game
        .enemy(enemy_id)
        .map(|enemy| {
            if enemy.battler.is_empty() {
                "enemy".into()
            } else {
                enemy.battler.clone()
            }
        })
        .unwrap_or_else(|| "enemy".into())
}

fn target_highlight(battle: &Battle) -> Option<usize> {
    let Phase::Target { cursor, pending } = &battle.phase else {
        return None;
    };
    let _ = pending;
    let enemies: Vec<usize> = battle
        .enemies
        .iter()
        .enumerate()
        .filter(|(_, fighter)| fighter.hp > 0)
        .map(|(index, _)| index)
        .collect();
    enemies.get(*cursor).copied()
}

fn draw_battle_commands(out: &mut Vec<Sprite>, session: &Session, battle: &Battle) {
    let terms = &session.game.system.terms;
    panel(out, 268, 250, 268, 156);
    let lines: Vec<(String, bool)> = match &battle.phase {
        Phase::Command { cursor } => {
            let mut lines = vec![
                (terms.attack.clone(), *cursor == 0),
                (terms.guard.clone(), *cursor == 1),
                (terms.skill.clone(), *cursor == 2),
                (terms.item.clone(), *cursor == 3),
            ];
            let escape =
                battle.can_escape && first_conscious(session, battle) == Some(battle.actor_slot);
            if escape {
                lines.push((terms.escape.clone(), *cursor == 4));
            }
            lines
        }
        Phase::Skill { cursor } => session
            .command_skills()
            .into_iter()
            .enumerate()
            .map(|(index, id)| {
                let skill = session.game.skill(id);
                let label = skill
                    .map(|skill| format!("{}  {}", skill.name, skill.mp_cost))
                    .unwrap_or_else(|| format!("Skill {id}"));
                (label, index == *cursor)
            })
            .collect(),
        Phase::Item { cursor } => session
            .battle_items()
            .into_iter()
            .enumerate()
            .map(|(index, id)| {
                let name = session
                    .game
                    .item(id)
                    .map(|item| item.name.as_str())
                    .unwrap_or("Item");
                let count = session.bag.count_item(id);
                (format!("{name}  x{count}"), index == *cursor)
            })
            .collect(),
        Phase::Target { pending, .. } => {
            let label = match pending {
                Pending::Attack => terms.attack.clone(),
                Pending::Skill(id) => session
                    .game
                    .skill(*id)
                    .map(|s| s.name.clone())
                    .unwrap_or_default(),
                Pending::Item(id) => session
                    .game
                    .item(*id)
                    .map(|s| s.name.clone())
                    .unwrap_or_default(),
            };
            vec![(format!("Choose target: {label}"), true)]
        }
        Phase::Boot | Phase::Resolving | Phase::Victory | Phase::Defeat | Phase::Escaped => {
            vec![("...".into(), false)]
        }
    };
    let title = battle
        .actors
        .get(battle.actor_slot)
        .map(|actor| actor.name.as_str())
        .unwrap_or("Battle");
    text(out, 280, 256, 1, title, GOLD);
    let start = match &battle.phase {
        Phase::Skill { cursor } | Phase::Item { cursor } => cursor.saturating_sub(6),
        _ => 0,
    };
    for (index, (label, selected)) in lines.iter().enumerate().skip(start).take(7) {
        let row = 274 + (index - start) as i32 * 16;
        if *selected {
            rect(out, 276, row - 2, 252, 14, [0.16, 0.3, 0.2, 1.0]);
            text(out, 284, row, 1, label, GOLD);
        } else {
            text(out, 284, row, 1, label, INK);
        }
    }
}

fn first_conscious(session: &Session, battle: &Battle) -> Option<usize> {
    let knockout = session.knockout_state();
    battle
        .actors
        .iter()
        .position(|fighter| conscious(fighter, knockout))
}

fn conscious(fighter: &Fighter, knockout: u32) -> bool {
    fighter.hp > 0 && !fighter.states.iter().any(|state| state.id == knockout)
}

fn draw_menu(out: &mut Vec<Sprite>, session: &Session) {
    let Some(menu) = &session.menu else {
        return;
    };
    rect(
        out,
        0,
        0,
        VIEW_W as i32,
        VIEW_H as i32,
        [0.02, 0.04, 0.04, 0.45],
    );
    let terms = &session.game.system.terms;
    let root = [
        terms.item.clone(),
        terms.skill.clone(),
        terms.equip.clone(),
        terms.status.clone(),
        terms.save.clone(),
        terms.game_end.clone(),
    ];
    panel(out, 8, 8, 150, 200);
    for (index, label) in root.iter().enumerate() {
        let row = 18 + index as i32 * 22;
        let selected = matches!(menu.page, MenuPage::Root) && menu.cursor == index;
        if selected {
            rect(out, 14, row - 3, 138, 18, [0.16, 0.3, 0.2, 1.0]);
            text(out, 22, row, 1, label, GOLD);
        } else {
            text(out, 22, row, 1, label, INK);
        }
    }
    let gold = format!("{} {}", session.gold, session.game.system.currency);
    panel(out, 8, 216, 150, 24);
    text(out, 16, 222, 1, &gold, GOLD);
    panel(out, 168, 8, 368, 360);
    match &menu.page {
        MenuPage::Root => {
            text(out, 184, 24, 1, "Choose a command.", INK);
            if let Some(member) = session.party.first() {
                let stats = party::stats_of(&session.game, member);
                let name = session.actor_label(0);
                text(
                    out,
                    184,
                    52,
                    1,
                    &format!("{name}  Lv {}", member.level),
                    GOLD,
                );
                text(
                    out,
                    184,
                    72,
                    1,
                    &format!("HP {}/{}", member.hp, stats.mhp),
                    INK,
                );
                text(
                    out,
                    184,
                    88,
                    1,
                    &format!("MP {}/{}", member.mp, stats.mmp),
                    INK,
                );
            }
        }
        MenuPage::Item { cursor, actor } => draw_item_page(out, session, *cursor, *actor),
        MenuPage::Skill { cursor, actor } => draw_skill_page(out, session, *cursor, *actor),
        MenuPage::Equip { slot, actor } => draw_equip_page(out, session, *slot, *actor, None),
        MenuPage::EquipPick {
            slot,
            actor,
            cursor,
        } => draw_equip_page(out, session, *slot, *actor, Some(*cursor)),
        MenuPage::Status { actor } => draw_status_page(out, session, *actor),
        MenuPage::End { yes } => {
            text(out, 184, 24, 1, &terms.to_title, GOLD);
            text(
                out,
                184,
                52,
                1,
                "Leave this game and return to the title?",
                INK,
            );
            let no = if *yes { INK } else { GOLD };
            let yes_c = if *yes { GOLD } else { INK };
            text(out, 184, 84, 1, "No", no);
            text(out, 240, 84, 1, "Yes", yes_c);
        }
    }
}

fn draw_item_page(out: &mut Vec<Sprite>, session: &Session, cursor: usize, actor: usize) {
    text(out, 184, 16, 1, &session.actor_label(actor), GOLD);
    let items: Vec<u32> = session
        .bag
        .items
        .iter()
        .filter(|(id, count)| {
            **count > 0
                && session
                    .game
                    .item(**id)
                    .is_some_and(|item| item.occasion.allows_menu())
        })
        .map(|(id, _)| *id)
        .collect();
    if items.is_empty() {
        text(out, 184, 44, 1, "The pack is empty.", DIM);
        return;
    }
    let start = cursor.saturating_sub(10);
    for (index, id) in items.iter().enumerate().skip(start).take(12) {
        let row = 40 + (index - start) as i32 * 16;
        let name = session
            .game
            .item(*id)
            .map(|item| item.name.clone())
            .unwrap_or_default();
        let count = session.bag.count_item(*id);
        let label = format!("{name}  x{count}");
        if index == cursor {
            rect(out, 176, row - 2, 352, 14, [0.16, 0.3, 0.2, 1.0]);
            text(out, 184, row, 1, &label, GOLD);
            if let Some(item) = session.game.item(*id) {
                text(out, 184, 340, 1, &item.description, DIM);
            }
        } else {
            text(out, 184, row, 1, &label, INK);
        }
    }
}

fn draw_skill_page(out: &mut Vec<Sprite>, session: &Session, cursor: usize, actor: usize) {
    text(out, 184, 16, 1, &session.actor_label(actor), GOLD);
    let Some(member) = session.party.get(actor) else {
        return;
    };
    let skills: Vec<u32> = party::known_skills(&session.game, member)
        .into_iter()
        .filter(|id| {
            session
                .game
                .skill(*id)
                .is_some_and(|skill| skill.occasion.allows_menu())
        })
        .collect();
    text(out, 360, 16, 1, &format!("MP {}", member.mp), INK);
    if skills.is_empty() {
        text(out, 184, 44, 1, "No field skills.", DIM);
        return;
    }
    for (index, id) in skills.iter().enumerate().take(12) {
        let row = 40 + index as i32 * 16;
        let label = session
            .game
            .skill(*id)
            .map(|skill| format!("{}  {}", skill.name, skill.mp_cost))
            .unwrap_or_default();
        if index == cursor {
            rect(out, 176, row - 2, 352, 14, [0.16, 0.3, 0.2, 1.0]);
            text(out, 184, row, 1, &label, GOLD);
            if let Some(skill) = session.game.skill(*id) {
                text(out, 184, 340, 1, &skill.description, DIM);
            }
        } else {
            text(out, 184, row, 1, &label, INK);
        }
    }
}

fn draw_equip_page(
    out: &mut Vec<Sprite>,
    session: &Session,
    slot: usize,
    actor: usize,
    picking: Option<usize>,
) {
    text(out, 184, 16, 1, &session.actor_label(actor), GOLD);
    for (index, label) in EQUIP_LABELS.iter().enumerate() {
        let row = 40 + index as i32 * 18;
        let name = session.equipped_name(actor, index);
        let line = format!("{label}: {name}");
        if picking.is_none() && index == slot {
            rect(out, 176, row - 2, 200, 16, [0.16, 0.3, 0.2, 1.0]);
            text(out, 184, row, 1, &line, GOLD);
        } else {
            text(out, 184, row, 1, &line, INK);
        }
    }
    if let Some(cursor) = picking {
        let choices = session.equip_choices(slot);
        text(out, 184, 150, 1, "Bag", GOLD);
        for (index, choice) in choices.iter().enumerate().take(8) {
            let row = 170 + index as i32 * 16;
            let label = choice.label(session);
            if index == cursor {
                rect(out, 176, row - 2, 340, 14, [0.16, 0.3, 0.2, 1.0]);
                text(out, 184, row, 1, &label, GOLD);
            } else {
                text(out, 184, row, 1, &label, INK);
            }
        }
    }
    if let Some(member) = session.party.get(actor) {
        let stats = party::stats_of(&session.game, member);
        text(
            out,
            184,
            320,
            1,
            &format!("ATK {}  DEF {}", stats.atk, stats.def),
            DIM,
        );
    }
}

fn draw_status_page(out: &mut Vec<Sprite>, session: &Session, actor: usize) {
    let Some(member) = session.party.get(actor) else {
        return;
    };
    let stats = party::stats_of(&session.game, member);
    let name = session.actor_label(actor);
    let class = session
        .game
        .actor(member.actor_id)
        .and_then(|actor| session.game.class(actor.class_id))
        .map(|class| class.name.clone())
        .unwrap_or_default();
    text(out, 184, 20, 2, &name, GOLD);
    text(out, 184, 44, 1, &class, INK);
    text(out, 184, 68, 1, &format!("Level {}", member.level), INK);
    text(
        out,
        184,
        88,
        1,
        &format!("HP {}/{}", member.hp, stats.mhp),
        INK,
    );
    text(
        out,
        184,
        104,
        1,
        &format!("MP {}/{}", member.mp, stats.mmp),
        INK,
    );
    let rows = [
        ("ATK", stats.atk),
        ("DEF", stats.def),
        ("MAT", stats.mat),
        ("MDF", stats.mdf),
        ("AGI", stats.agi),
        ("LUK", stats.luk),
    ];
    for (index, (label, value)) in rows.iter().enumerate() {
        let col = index % 2;
        let row = index / 2;
        text(
            out,
            184 + col as i32 * 120,
            132 + row as i32 * 18,
            1,
            &format!("{label} {value}"),
            INK,
        );
    }
    text(
        out,
        184,
        200,
        1,
        &format!("Weapon  {}", session.equipped_name(actor, 0)),
        INK,
    );
    text(
        out,
        184,
        218,
        1,
        &format!("Body    {}", session.equipped_name(actor, 3)),
        INK,
    );
    if let Some(actor) = session.game.actor(member.actor_id) {
        if !actor.profile.is_empty() {
            text(out, 184, 260, 1, &actor.profile, DIM);
        }
    }
}

fn draw_save(out: &mut Vec<Sprite>, session: &Session) {
    let Some(ui) = &session.save_ui else {
        return;
    };
    rect(
        out,
        0,
        0,
        VIEW_W as i32,
        VIEW_H as i32,
        [0.0, 0.0, 0.0, 0.45],
    );
    let w = 360;
    let h = 280;
    let x = (VIEW_W as i32 - w) / 2;
    let y = (VIEW_H as i32 - h) / 2;
    panel(out, x, y, w, h);
    let title = if ui.loading { "Load" } else { "Save" };
    text(out, x + 16, y + 12, 1, title, GOLD);
    for index in 0..ui.slots.len() {
        let row = y + 36 + index as i32 * 52;
        if index == ui.cursor {
            rect(out, x + 10, row - 4, w - 20, 48, [0.16, 0.3, 0.2, 1.0]);
        }
        text(
            out,
            x + 18,
            row,
            1,
            &format!("File {}", index + 1),
            if index == ui.cursor { GOLD } else { INK },
        );
        if let Some(preview) = &ui.slots[index] {
            text(out, x + 18, row + 14, 1, &preview.map_name, INK);
            let line = format!(
                "{}   {} {}",
                play_clock(preview.play_frames),
                preview.gold,
                session.game.system.currency
            );
            text(out, x + 18, row + 28, 1, &line, DIM);
        } else {
            text(out, x + 18, row + 16, 1, "Empty", DIM);
        }
    }
    if ui.confirm {
        text(
            out,
            x + 16,
            y + h - 22,
            1,
            "Overwrite this file? Confirm again.",
            GOLD,
        );
    }
}

fn bar(out: &mut Vec<Sprite>, x: i32, y: i32, w: i32, value: i32, max: i32, color: [f32; 4]) {
    rect(out, x, y, w, 4, [0.1, 0.1, 0.1, 1.0]);
    if max > 0 && value > 0 {
        let fill = (w as i64 * value as i64 / max as i64).clamp(0, w as i64) as i32;
        rect(out, x, y, fill.max(1), 4, color);
    }
}

fn panel(out: &mut Vec<Sprite>, x: i32, y: i32, w: i32, h: i32) {
    rect(out, x, y, w, h, [0.06, 0.1, 0.08, 0.94]);
    rect(out, x, y, w, 1, EDGE);
    rect(out, x, y + h - 1, w, 1, EDGE);
    rect(out, x, y, 1, h, EDGE);
    rect(out, x + w - 1, y, 1, h, EDGE);
}

fn rect(out: &mut Vec<Sprite>, x: i32, y: i32, w: i32, h: i32, color: [f32; 4]) {
    if w <= 0 || h <= 0 {
        return;
    }
    out.push(Sprite {
        key: "white".into(),
        src: (0, 0, 1, 1),
        dst: (x, y, w, h),
        color,
    });
}

fn blit(
    out: &mut Vec<Sprite>,
    key: String,
    src: (u32, u32, u32, u32),
    dst: (i32, i32, i32, i32),
    color: [f32; 4],
) {
    out.push(Sprite {
        key,
        src,
        dst,
        color,
    });
}

fn text(out: &mut Vec<Sprite>, x: i32, y: i32, scale: i32, line: &str, color: [f32; 4]) {
    glyphs(
        out,
        x + scale,
        y + scale,
        scale,
        line,
        [0.02, 0.03, 0.02, 0.85],
    );
    glyphs(out, x, y, scale, line, color);
}

fn glyphs(out: &mut Vec<Sprite>, mut x: i32, y: i32, scale: i32, line: &str, color: [f32; 4]) {
    let scale = scale.max(1);
    for ch in line.chars() {
        let index = if ch.is_ascii() {
            ch as usize
        } else {
            b'?' as usize
        };
        if ch != ' ' {
            out.push(Sprite {
                key: "font".into(),
                src: ((index as u32) * 8, 0, 8, 8),
                dst: (x, y, 8 * scale, 8 * scale),
                color,
            });
        }
        x += 8 * scale;
    }
}

fn text_width(line: &str, scale: i32) -> i32 {
    line.chars().count() as i32 * 8 * scale.max(1)
}
