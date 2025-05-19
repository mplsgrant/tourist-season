use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bitcoind::BitcoindHandler;
use button_row::ButtonRow;
use coordinates::CoordinateIndicator;
use electrum_wallet::ElectrumWallet;
use popup::Popup;
use tilemaptest::GameMap;
use tourists::Tourists;

mod bdk_zone;
mod bitcoind;
mod borders;
mod button_row;
mod camera;
mod constants;
mod coordinates;
mod electrum_wallet;
mod popup;
mod tiled_thing;
mod tilemaptest;
mod tourists;

fn main() {
    App::new()
        .add_plugins(GameMap)
        .add_plugins(ButtonRow)
        .add_plugins(BitcoindHandler)
        .add_plugins(Popup)
        .add_plugins(Tourists)
        .add_plugins(ElectrumWallet)
        //.add_plugins(CoordinateIndicator)
        // .add_plugins(EguiPlugin {
        //     enable_multipass_for_primary_context: true,
        // })
        // .add_plugins(WorldInspectorPlugin::new())
        .run();
}
