// 16x16 sprites from Kenney's CC0 packs (commercial use, credit not required):
// grass, flowers, mushrooms, road, roofs, and walls from Tiny Town;
// Mira and the slime from Tiny Dungeon.
// https://creativecommons.org/publicdomain/zero/1.0/
// Layout is a horizontal strip of 16x16 RGBA sprites:
// 0 grass, 1 white flower, 2 yellow flower, 3 mushroom, 4 player, 5 slime,
// 6 road left, 7 road, 8 road right, 9 red roof, 10 wood wall,
// 11 blue roof, 12 stone wall.

pub const TILE: i32 = 16;
pub const SPR_GRASS: i32 = 0;
pub const SPR_FLOWER: i32 = 1;
pub const SPR_BLOOM: i32 = 2;
pub const SPR_MUSHROOM: i32 = 3;
pub const SPR_PLAYER: i32 = 4;
pub const SPR_SLIME: i32 = 5;
pub const SPR_ROAD_L: i32 = 6;
pub const SPR_ROAD: i32 = 7;
pub const SPR_ROAD_R: i32 = 8;
pub const SPR_ROOF_RED: i32 = 9;
pub const SPR_WALL_WOOD: i32 = 10;
pub const SPR_ROOF_BLUE: i32 = 11;
pub const SPR_WALL_STONE: i32 = 12;
pub const SPR_COUNT: i32 = 13;
pub const ATLAS_W: i32 = TILE * SPR_COUNT;

pub const ATLAS: &[u8] = include_bytes!("../assets/atlas.rgba");

#[cfg(test)]
mod tests {
    #[test]
    fn atlas_is_the_field_strip() {
        assert_eq!(
            super::ATLAS.len(),
            (super::ATLAS_W * super::TILE * 4) as usize
        );
    }
}
