//! Playable half of Fablespace: maps, events, battle, menu, and saves.
//!
//! The window and GPU renderer live in the binary. Tests drive [`Session`]
//! with [`Input`] and never open a window.

mod battle;
pub mod draw;
mod font8x8;
pub mod gpu;
mod interpret;
mod menu;
mod party;
mod persist;
mod rng;
mod session;

pub use font8x8::FONT8X8;
pub use persist::{format_play, SLOT_COUNT};
pub use session::{Input, Screen, Session};

/// Load a project and walk a few steps, open the menu, and round-trip a save.
/// No window is opened.
pub fn self_test(paths: fablespace_data::Paths) -> Result<(), String> {
    let mut session = Session::boot(paths)?;
    session.new_game();
    if session.screen != Screen::Map {
        return Err("new game did not reach the map".into());
    }
    for _ in 0..4 {
        let (x, y) = (session.player.tile_x, session.player.tile_y);
        if !session.passable(x, y - 1, None) {
            break;
        }
        session.update(&Input::walk(fablespace_data::Direction::Up));
        for _ in 0..40 {
            if session.player.moving.is_none() {
                break;
            }
            session.update(&Input::default());
        }
        if session.player.moving.is_some() {
            return Err("a step never finished".into());
        }
    }
    session.update(&Input {
        menu: true,
        ..Input::default()
    });
    if session.menu.is_none() {
        return Err("the menu did not open".into());
    }
    session.update(&Input::cancel());
    if session.menu.is_some() {
        return Err("the menu did not close".into());
    }
    let gold = session.gold;
    session.save_slot(0)?;
    session.gold = gold.saturating_add(9);
    session.load_slot(0)?;
    if session.gold != gold {
        return Err("the save did not restore gold".into());
    }
    if session.party.is_empty() {
        return Err("the party is empty".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, VecDeque};

    use fablespace_data::command::{ActorTarget, AmountOp, Command, Condition, Operand, VarOp};
    use fablespace_data::sample::{self, clearing_map};
    use fablespace_data::{Cmp, Direction, Paths, SwitchValue};

    use crate::session::Process;
    use crate::session::{Input, Screen, Session};

    fn session_in(dir: &std::path::Path) -> Session {
        let (game, maps) = sample::fable_hollow();
        let paths = Paths {
            root: dir.to_path_buf(),
            data: dir.to_path_buf(),
        };
        Session::new(game, maps, paths)
    }

    fn fresh() -> (Session, std::path::PathBuf) {
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        let n = N.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("fablespace-rt-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        (session_in(&dir), dir)
    }

    fn step(session: &mut Session, dir: Direction) {
        let (x, y) = (session.player.tile_x, session.player.tile_y);
        session.update(&Input::walk(dir));
        assert!(
            session.player.moving.is_some(),
            "expected a step {dir:?} from ({x},{y})"
        );
        for _ in 0..40 {
            if session.player.moving.is_none() {
                break;
            }
            session.update(&Input::default());
        }
        assert!(session.player.moving.is_none());
        let (dx, dy) = dir.delta();
        assert_eq!(
            (session.player.tile_x, session.player.tile_y),
            (x + dx, y + dy)
        );
    }

    fn walk_to(session: &mut Session, goal_x: i32, goal_y: i32) {
        let start = (session.player.tile_x, session.player.tile_y);
        if start == (goal_x, goal_y) {
            return;
        }
        let mut queue = VecDeque::from([start]);
        let mut prev: HashMap<(i32, i32), (i32, i32, Direction)> = HashMap::new();
        let mut found = false;
        while let Some((x, y)) = queue.pop_front() {
            if (x, y) == (goal_x, goal_y) {
                found = true;
                break;
            }
            for dir in Direction::all() {
                let (dx, dy) = dir.delta();
                let next = (x + dx, y + dy);
                if next == start || prev.contains_key(&next) {
                    continue;
                }
                if !session.passable(next.0, next.1, None) {
                    continue;
                }
                prev.insert(next, (x, y, dir));
                queue.push_back(next);
            }
        }
        assert!(found, "no path from {start:?} to ({goal_x},{goal_y})");
        let mut dirs = Vec::new();
        let mut cursor = (goal_x, goal_y);
        while cursor != start {
            let (px, py, dir) = prev[&cursor];
            dirs.push(dir);
            cursor = (px, py);
        }
        dirs.reverse();
        for dir in dirs {
            step(session, dir);
        }
    }

    fn face(session: &mut Session, dir: Direction) {
        if session.player.dir == dir && session.player.moving.is_none() {
            return;
        }
        session.update(&Input::walk(dir));
        assert!(
            session.player.moving.is_none(),
            "facing {dir:?} walked onto a tile"
        );
        assert_eq!(session.player.dir, dir);
    }

    fn confirm_messages(session: &mut Session) {
        for _ in 0..12 {
            if session.message.is_none() {
                return;
            }
            session.update(&Input::confirm());
        }
        assert!(session.message.is_none(), "message did not close");
    }

    #[test]
    fn play_fable_hollow() {
        let (mut session, dir) = fresh();
        session.update(&Input::confirm());
        assert_eq!(session.screen, Screen::Map);
        assert_eq!((session.player.tile_x, session.player.tile_y), (3, 12));
        assert_eq!(session.gold, 20);
        assert_eq!(session.bag.count_item(1), 1);

        walk_to(&mut session, 5, 6);
        face(&mut session, Direction::Up);
        session.update(&Input::confirm());
        for _ in 0..16 {
            if session.message.is_some() {
                break;
            }
            session.update(&Input::default());
        }
        assert!(session.message.is_some(), "the board never spoke");
        assert!(session
            .pictures
            .iter()
            .any(|picture| picture.name == "board"));
        confirm_messages(&mut session);
        assert!(session.pictures.is_empty());

        walk_to(&mut session, 8, 6);
        face(&mut session, Direction::Up);
        session.update(&Input::confirm());
        confirm_messages(&mut session);
        assert!(session.choice.is_some());
        session.update(&Input::confirm());
        confirm_messages(&mut session);
        assert_eq!(session.variable(1), 1);
        assert!(session.switch(1));
        assert!(session.foreground.is_none());

        walk_to(&mut session, 14, 9);
        face(&mut session, Direction::Right);
        session.update(&Input::confirm());
        confirm_messages(&mut session);
        assert_eq!(session.bag.count_item(2), 1);
        session.update(&Input::confirm());
        confirm_messages(&mut session);
        assert_eq!(session.bag.count_item(2), 1);
        let chest = session.events.iter().find(|event| event.id == 3).unwrap();
        assert_eq!(chest.body.character_index, 1);

        walk_to(&mut session, 17, 11);
        let gold_before = session.gold;
        session.update(&Input::walk(Direction::Right));
        assert!(
            session.battle.is_some(),
            "bumping the slime should start a battle"
        );
        assert_eq!(session.player.tile_x, 17);
        for _ in 0..80 {
            if session.battle.is_none() {
                break;
            }
            session.update(&Input::confirm());
        }
        assert!(session.battle.is_none(), "the slime fight did not end");
        assert_eq!(session.gold, gold_before + 12);
        let slime = session.events.iter().find(|event| event.id == 4).unwrap();
        assert!(slime.body.character.is_empty());
        assert!(session.party[0].hp > 0);
        assert!(session.party[0].hp < party_max_hp(&session));

        let hp_before = session.party[0].hp;
        session.update(&Input {
            menu: true,
            ..Input::default()
        });
        assert!(session.menu.is_some());
        session.update(&Input::confirm());
        session.update(&Input::confirm());
        confirm_messages(&mut session);
        assert_eq!(session.bag.count_item(1), 0);
        assert!(session.party[0].hp > hp_before);

        for _ in 0..4 {
            session.update(&Input::cancel());
        }
        session.update(&Input {
            menu: true,
            ..Input::default()
        });
        for _ in 0..4 {
            session.update(&Input::walk(Direction::Down));
        }
        session.update(&Input::confirm());
        session.update(&Input::confirm());
        assert!(dir.join("save").join("slot01.json").is_file());
        session.gold = 1;
        session.load_slot(0).unwrap();
        assert_eq!(session.gold, gold_before + 12);
        assert_eq!(session.bag.count_item(2), 1);
        assert!(session.switch(1));

        session.update(&Input {
            menu: true,
            ..Input::default()
        });
        for _ in 0..5 {
            session.update(&Input::walk(Direction::Down));
        }
        session.update(&Input::confirm());
        session.update(&Input::walk(Direction::Right));
        session.update(&Input::confirm());
        assert_eq!(session.screen, Screen::Title);

        session.update(&Input::walk(Direction::Down));
        session.update(&Input::confirm());
        session.update(&Input::confirm());
        assert_eq!(session.screen, Screen::Map);
        assert_eq!(session.gold, gold_before + 12);
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn party_max_hp(session: &Session) -> i32 {
        crate::party::stats_of(&session.game, &session.party[0]).mhp
    }

    #[test]
    fn commands_transfer_branch_and_math() {
        let (mut session, dir) = fresh();
        session.maps.push(clearing_map());
        session.new_game();
        session.foreground = Some(Process::event(
            1,
            99,
            vec![
                Command::ControlVariable {
                    id: 1,
                    op: VarOp::Set,
                    operand: Operand::Constant { value: 4 },
                },
                Command::ControlVariable {
                    id: 1,
                    op: VarOp::Add,
                    operand: Operand::Constant { value: 3 },
                },
                Command::Conditional {
                    cond: Condition::Variable {
                        id: 1,
                        cmp: Cmp::Ge,
                        operand: Operand::Constant { value: 7 },
                    },
                    then_branch: vec![Command::ControlSwitch {
                        id: 1,
                        value: SwitchValue::On,
                    }],
                    else_branch: Some(vec![Command::ControlSwitch {
                        id: 1,
                        value: SwitchValue::Off,
                    }]),
                },
                Command::ChangeGold {
                    op: AmountOp::Gain,
                    amount: 5,
                },
                Command::TransferPlayer {
                    map_id: 2,
                    x: 4,
                    y: 4,
                    direction: Some(Direction::Left),
                    fade: false,
                },
            ],
        ));
        session.drive_foreground();
        assert_eq!(session.variable(1), 7);
        assert!(session.switch(1));
        assert_eq!(session.gold, 25);
        assert_eq!(session.map_id, 2);
        assert_eq!((session.player.tile_x, session.player.tile_y), (4, 4));
        assert_eq!(session.player.dir, Direction::Left);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn recover_all_and_save_roundtrip() {
        let (mut session, dir) = fresh();
        session.new_game();
        session.party[0].hp = 10;
        session.party[0].mp = 1;
        session.foreground = Some(Process::common(
            1,
            1,
            vec![Command::RecoverAll {
                target: ActorTarget::EntireParty,
            }],
        ));
        session.drive_foreground();
        assert_eq!(session.party[0].hp, party_max_hp(&session));
        session.gold = 77;
        session.set_switch(1, SwitchValue::On);
        session.save_slot(2).unwrap();
        session.gold = 0;
        session.set_switch(1, SwitchValue::Off);
        session.load_slot(2).unwrap();
        assert_eq!(session.gold, 77);
        assert!(session.switch(1));
        assert_eq!(session.screen, Screen::Map);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn meadow_tiles_are_drawn_from_the_sample_sheet() {
        let (mut session, dir) = fresh();
        session.new_game();
        let sheet = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../sample/img/tilesets/meadow.png");
        let image = image::open(&sheet).expect("sample tileset").to_rgba8();
        assert!(image.width() >= 256 && image.height() >= 160);
        let mut asked = false;
        let sprites = crate::draw::frame(&session, &mut |key| {
            if key == "tilesets/meadow" {
                asked = true;
                Some((image.width(), image.height()))
            } else if key.starts_with("characters/") {
                Some((384, 256))
            } else {
                Some((96, 96))
            }
        });
        assert!(asked, "the map never asked for the meadow tileset");
        let tiles: Vec<_> = sprites
            .iter()
            .filter(|sprite| sprite.key == "tilesets/meadow")
            .collect();
        assert!(
            tiles.len() > 40,
            "expected a meadow of tiles, drew {}",
            tiles.len()
        );
        let mut colors = std::collections::HashSet::new();
        for sprite in tiles.iter().take(24) {
            let (sx, sy, sw, sh) = sprite.src;
            assert!(sx + sw <= image.width() && sy + sh <= image.height());
            let px = image.get_pixel(sx + sw / 2, sy + sh / 2);
            colors.insert([px[0], px[1], px[2]]);
        }
        assert!(
            colors.len() > 1,
            "every sampled tile is the same color, so the sheet is not showing"
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_tap_stops_on_the_next_tile() {
        let (mut session, dir) = fresh();
        session.new_game();
        let start = (session.player.tile_x, session.player.tile_y);
        session.update(&Input::walk(Direction::Up));
        assert!(session.player.moving.is_some());
        assert!(
            session.player.offset <= 2,
            "the first frame moved {} px",
            session.player.offset
        );
        for _ in 0..40 {
            session.update(&Input::default());
        }
        assert!(session.player.moving.is_none());
        assert_eq!(
            (session.player.tile_x, session.player.tile_y),
            (start.0, start.1 - 1)
        );
        let mut held = 0;
        for _ in 0..30 {
            session.update(&Input::walk(Direction::Up));
            held += 1;
            let _ = held;
        }
        let tiles = start.1 - 1 - session.player.tile_y;
        assert!(tiles <= 2, "half a second of walking crossed {tiles} tiles");
        let _ = std::fs::remove_dir_all(dir);
    }

    fn click(session: &mut Session, at: (i32, i32)) {
        session.update(&Input {
            click: Some(at),
            ..Input::default()
        });
    }

    #[test]
    fn clicks_drive_title_menu_choices_battle_and_save() {
        let (mut session, dir) = fresh();
        click(&mut session, (200, 360));
        assert!(session.quit, "the shutdown row did not take the click");
        session.quit = false;

        session.new_game();
        session.update(&Input {
            menu: true,
            ..Input::default()
        });
        click(&mut session, (40, 45));
        assert!(
            matches!(
                session.menu.as_ref().map(|menu| &menu.page),
                Some(crate::menu::MenuPage::Skill { .. })
            ),
            "the skill row did not open"
        );
        click(&mut session, (40, 110));
        assert!(session.save_ui.is_some(), "the save row did not open");
        click(&mut session, (200, 170));
        assert!(
            dir.join("save").join("slot02.json").is_file(),
            "clicking the second slot did not write it"
        );

        session.save_ui = None;
        session.menu = None;
        session.choice = Some(crate::session::ChoiceUi {
            options: vec!["Stay".into(), "Leave".into()],
            cursor: 0,
            branches: vec![vec![], vec![]],
            cancel: fablespace_data::CancelMode::Disallow,
            cancel_branch: vec![],
        });
        click(&mut session, (360, 300));
        assert!(
            session.choice.is_none(),
            "the second choice did not take the click"
        );

        session.start_battle(1, true);
        assert!(session.message.is_some());
        click(&mut session, (100, 360));
        assert!(
            matches!(
                session.battle.as_ref().map(|battle| &battle.phase),
                Some(crate::battle::Phase::Command { .. })
            ),
            "dismissing the battle line did not reach commands"
        );
        click(&mut session, (300, 278));
        assert!(
            !matches!(
                session.battle.as_ref().map(|battle| &battle.phase),
                Some(crate::battle::Phase::Command { .. })
            ),
            "attack did not take the click"
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn window_clicks_land_in_the_letterboxed_view() {
        assert_eq!(
            crate::draw::pointer_to_view(1088, 832, 100.0, 80.0),
            Some((50, 40))
        );
        assert_eq!(crate::draw::pointer_to_view(1088, 832, -1.0, 10.0), None);
    }
}
