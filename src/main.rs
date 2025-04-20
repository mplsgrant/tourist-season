use bdk_key::BDKButton;
use bevy::prelude::*;
use bitcoind::BitcoindHandler;

mod bdk_key;
pub mod bdk_zone;
mod bitcoind;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BDKButton)
        .add_plugins(BitcoindHandler)
        .run();
}
