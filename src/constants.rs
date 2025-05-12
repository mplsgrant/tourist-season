#![allow(unused)]

use bevy::prelude::*;

pub const Z_TILEMAP: i32 = 0;

pub const BITCOIN_DIR: &str = "bitcoind";
pub const MAP_DIR: &str = "map";

/// Marks an entity as being a Popup.
/// Current use: tilemap interactions query to see if the node with this marker is displayed and if it is displayed, the system disables tilemap interaction.
/// In other words, don't register clicks to the tilemap if the Popup is visible to the user.
#[derive(Component, Clone)]
pub struct PopupBase;

pub const GRASS_PATH: &str = "tiles-test/tile_0000.png";
pub const GRASS_IDX: u32 = 0;
pub const GRASS_BORDER_UPPER_LEFT_PATH: &str = "tiles-test/tile_0012.png";
pub const GRASS_BORDER_UPPER_LEFT_IDX: u32 = 1;
pub const GRASS_BORDER_UPPER: &str = "tiles-test/tile_0013.png";
pub const GRASS_BORDER_UPPER_IDX: u32 = 2;
pub const GRASS_BORDER_UPPER_RIGHT: &str = "tiles-test/tile_0014.png";
pub const GRASS_BORDER_UPPER_RIGHT_IDX: u32 = 3;
pub const GRASS_BORDER_LEFT: &str = "tiles-test/tile_0024.png";
pub const GRASS_BORDER_LEFT_IDX: u32 = 4;
pub const DIRT: &str = "tiles-test/tile_0025.png";
pub const DIRT_IDX: u32 = 5;
pub const GRASS_BORDER_RIGHT: &str = "tiles-test/tile_0026.png";
pub const GRASS_BORDER_RIGHT_IDX: u32 = 6;
pub const GRASS_BORDER_LOWER_LEFT: &str = "tiles-test/tile_0036.png";
pub const GRASS_BORDER_LOWER_LEFT_IDX: u32 = 7;
pub const GRASS_BORDER_LOWER: &str = "tiles-test/tile_0037.png";
pub const GRASS_BORDER_LOWER_IDX: u32 = 8;
pub const GRASS_BORDER_LOWER_RIGHT: &str = "tiles-test/tile_0038.png";
pub const GRASS_BORDER_LOWER_RIGHT_IDX: u32 = 9;
