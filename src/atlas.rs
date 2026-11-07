// 16x16 sprites from Kenney's CC0 packs (commercial use, credit not required):
// grass and mushrooms from Tiny Town, Mira and the slime from Tiny Dungeon.
// https://creativecommons.org/publicdomain/zero/1.0/
// Layout is a horizontal strip of six 16x16 RGBA sprites:
// 0 grass, 1 grass with a white flower, 2 grass with a yellow flower,
// 3 mushroom, 4 player, 5 slime.

pub const TILE: i32 = 16;
pub const SPR_GRASS: i32 = 0;
pub const SPR_FLOWER: i32 = 1;
pub const SPR_BLOOM: i32 = 2;
pub const SPR_MUSHROOM: i32 = 3;
pub const SPR_PLAYER: i32 = 4;
pub const SPR_SLIME: i32 = 5;
pub const ATLAS_W: i32 = TILE * 6;

pub const ATLAS: &[u8] = include_bytes!("../assets/atlas.rgba");

#[cfg(test)]
mod tests {
    #[test]
    fn atlas_is_six_sprites() {
        assert_eq!(
            super::ATLAS.len(),
            (super::ATLAS_W * super::TILE * 4) as usize
        );
    }
}
