use bevy::prelude::*;

pub const Z_TILEMAP: i32 = 0;

/// Marks an entity as being a Popup.
/// Current use: tilemap interactions query to see if the node with this marker is displayed and if it is displayed, the system disables tilemap interaction.
/// In other words, don't register clicks to the tilemap if the Popup is visible to the user.
#[derive(Component, Clone)]
pub struct PopupBase;
