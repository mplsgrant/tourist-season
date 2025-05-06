use bdk_button::BDKButton;
use bevy::prelude::*;
use bitcoind::BitcoindHandler;
use tilemaptest::TileMapTest;

mod bdk_button;
mod bdk_zone;
mod bitcoind;
mod tilemaptest;

fn main() {
    App::new()
        //.add_plugins(DefaultPlugins)
        .add_plugins(TileMapTest)
        .add_plugins(BDKButton)
        .add_plugins(BitcoindHandler)
        .run();
}
