use bdk_button::BDKButton;
use bevy::prelude::*;
use bitcoind::BitcoindHandler;
use popup::Popup;
use tilemaptest::GameMap;
use tourists::Tourists;

mod bdk_button;
mod bdk_zone;
mod bitcoind;
mod borders;
mod camera;
mod constants;
mod popup;
mod tiled_thing;
mod tilemaptest;
mod tourists;

fn main() {
    App::new()
        .add_plugins(GameMap)
        .add_plugins(BDKButton)
        .add_plugins(BitcoindHandler)
        .add_plugins(Popup)
        .add_plugins(Tourists)
        .run();
}
