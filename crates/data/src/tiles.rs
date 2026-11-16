//! Tile sources and the 4-neighbor autotile.
//!
//! Neighbor bits are N=1, E=2, S=4, W=8. The sixteen masks fill a 4 by 4
//! block in reading order: mask 0 is the isolated tile at the origin, mask 15
//! is the fully connected tile at the bottom right of the block.

use crate::model::{Autotile, Map, Passage, Tileset};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub col: u32,
    pub row: u32,
}

pub fn autotile_mask(same: impl Fn(i32, i32) -> bool, x: i32, y: i32) -> u8 {
    let n = u8::from(same(x, y - 1));
    let e = u8::from(same(x + 1, y)) << 1;
    let s = u8::from(same(x, y + 1)) << 2;
    let w = u8::from(same(x - 1, y)) << 3;
    n | e | s | w
}

pub fn variant_cell(auto: &Autotile, mask: u8) -> Cell {
    Cell {
        col: auto.origin_col + (mask as u32 % 4),
        row: auto.origin_row + (mask as u32 / 4),
    }
}

pub fn autotile_for(tileset: &Tileset, tile_id: u32) -> Option<&Autotile> {
    tileset.autotiles.iter().find(|a| a.tile_id == tile_id)
}

fn layer_id(map: &Map, lower: bool, x: i32, y: i32) -> u32 {
    if lower {
        map.lower_at(x, y)
    } else {
        map.upper_at(x, y)
    }
}

/// Source cell for a painted tile. Empty tiles return `None`.
pub fn tile_cell(tileset: &Tileset, map: &Map, lower: bool, x: i32, y: i32) -> Option<Cell> {
    let id = layer_id(map, lower, x, y);
    if id == 0 {
        return None;
    }
    if let Some(auto) = autotile_for(tileset, id) {
        let mask = autotile_mask(|nx, ny| layer_id(map, lower, nx, ny) == id, x, y);
        return Some(variant_cell(auto, mask));
    }
    let cols = tileset.columns.max(1);
    Some(Cell {
        col: id % cols,
        row: id / cols,
    })
}

/// Empty ground does not pass. A blocking upper tile blocks. Star never blocks.
pub fn tile_blocked(tileset: &Tileset, lower_id: u32, upper_id: u32) -> bool {
    if lower_id == 0 || matches!(tileset.passage(lower_id), Passage::Block) {
        return true;
    }
    upper_id != 0 && matches!(tileset.passage(upper_id), Passage::Block)
}

pub fn upper_draw_order(passage: Passage) -> UpperDraw {
    match passage {
        Passage::Pass => UpperDraw::Below,
        Passage::Block => UpperDraw::Same,
        Passage::Star => UpperDraw::Above,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpperDraw {
    Below,
    Same,
    Above,
}

/// Character sheets are 12 frames wide and 8 frames tall: four characters
/// across, two down, each character three frames by four directions.
pub fn character_frame(
    sheet_w: u32,
    sheet_h: u32,
    index: u32,
    direction: crate::model::Direction,
    pattern: u8,
) -> (u32, u32, u32, u32) {
    let fw = (sheet_w / 12).max(1);
    let fh = (sheet_h / 8).max(1);
    let col = (index % 4) * 3 + u32::from(pattern % 3);
    let row_in = match direction {
        crate::model::Direction::Down => 0,
        crate::model::Direction::Left => 1,
        crate::model::Direction::Right => 2,
        crate::model::Direction::Up => 3,
    };
    let row = (index / 4) * 4 + row_in;
    (col * fw, row * fh, fw, fh)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Autotile;

    #[test]
    fn mask_bits_and_variant_block() {
        let mask = autotile_mask(|x, y| matches!((x, y), (0, -1) | (1, 0) | (-1, 0)), 0, 0);
        assert_eq!(mask, 1 | 2 | 8);
        let auto = Autotile {
            tile_id: 8,
            origin_col: 0,
            origin_row: 1,
        };
        assert_eq!(variant_cell(&auto, 0), Cell { col: 0, row: 1 });
        assert_eq!(variant_cell(&auto, 15), Cell { col: 3, row: 4 });
    }

    #[test]
    fn empty_ground_blocks_and_star_does_not() {
        let tileset = Tileset::blank(1);
        assert!(tile_blocked(&tileset, 0, 0));
        assert!(!tile_blocked(&tileset, 1, 0));
        assert!(tile_blocked(&tileset, 1, 3));
        assert!(tile_blocked(&tileset, 1, 5));
    }
}
