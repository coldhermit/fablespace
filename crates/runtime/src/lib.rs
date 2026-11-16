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
        for _ in 0..8 {
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
        for _ in 0..8 {
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
}
