use bdk_button::BDKButton;
use bevy::prelude::*;
use bitcoind::BitcoindHandler;

mod bdk_button;
mod bdk_zone;
mod bitcoind;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BDKButton)
        .add_plugins(BitcoindHandler)
        .run();
}
