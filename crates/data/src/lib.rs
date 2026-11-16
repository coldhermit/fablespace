//! Project data shared by the maker and the runtime.
//!
//! The on-disk JSON is the same format a build copies into `data/`.

pub mod command;
pub mod install;
pub mod model;
pub mod project;
pub mod sample;
pub mod tiles;

pub use command::{
    ActorTarget, AmountOp, CancelMode, Cmp, Command, Condition, MoveRoute, MoveStep, MoveTarget,
    Operand, SwitchValue, VarOp,
};
pub use model::*;
pub use project::{resolve, Paths};
pub use tiles::{tile_blocked, tile_cell};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{insert_after, insert_in_branch, rows, Slot, Step, Target};
    use crate::project::save_project;

    #[test]
    fn sample_roundtrip_and_pages() {
        let (game, maps) = sample::fable_hollow();
        let text = serde_json::to_string(&game).unwrap();
        let back: Game = serde_json::from_str(&text).unwrap();
        assert_eq!(back.system.game_title, "Fable Hollow");
        assert_eq!(back.actor(1).unwrap().name, "Lark");
        assert!(back.actor(0).is_none());
        let map_text = serde_json::to_string(&maps[0]).unwrap();
        let map: Map = serde_json::from_str(&map_text).unwrap();
        assert_eq!(map.events.len(), 4);
        let chest = map.events.iter().find(|e| e.name == "Chest").unwrap();
        assert!(chest.pages[1].conditions.self_switch.is_some());
        let dir = std::env::temp_dir().join(format!("fablespace-data-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        save_project(&dir, &game, &maps).unwrap();
        let paths = resolve(Some(&dir), &dir).unwrap();
        let loaded = project::load_game(&paths).unwrap();
        let loaded_maps = project::load_maps(&paths).unwrap();
        assert_eq!(loaded.system.start_x, 3);
        assert_eq!(loaded_maps[0].name, "Hollow");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn command_tree_inserts_into_a_branch() {
        let mut list = vec![Command::blank("conditional")];
        assert!(insert_in_branch(
            &mut list,
            &[Step::Cmd(0)],
            Slot::Then,
            Command::blank("comment")
        ));
        let shown = rows(&list);
        assert!(shown.iter().any(|row| matches!(
            &row.target,
            Target::Command(path) if path.as_slice() == [Step::Cmd(0), Step::Then, Step::Cmd(0)]
        )));
        assert!(insert_after(
            &mut list,
            &[Step::Cmd(0)],
            Command::blank("wait")
        ));
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn exp_curve_grows() {
        let curve = ExpCurve {
            basis: 20,
            extra: 8,
            accel: 4,
        };
        assert_eq!(exp_to_next(&curve, 1), 20);
        assert!(exp_to_next(&curve, 3) > exp_to_next(&curve, 2));
    }

    #[test]
    fn path_autotile_connects_inside_the_sample() {
        let (_game, maps) = sample::fable_hollow();
        let tileset = Tileset::blank(1);
        let map = &maps[0];
        let isolated = tile_cell(&tileset, map, true, 3, 12).unwrap();
        let middle = tile_cell(&tileset, map, true, 3, 8).unwrap();
        assert_ne!(isolated.row, middle.row);
        assert!(!tile_blocked(&tileset, map.lower_at(3, 8), 0));
        assert!(tile_blocked(&tileset, sample::WATER, 0));
        assert!(tile_blocked(&tileset, sample::GRASS, sample::TREE));
    }
}
