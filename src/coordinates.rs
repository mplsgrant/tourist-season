use bevy::prelude::*;

use crate::tilemaptest::{CurTilePos, CursorPos};

pub struct CoordinateIndicator;

impl Plugin for CoordinateIndicator {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
    }
}

#[derive(Component, Default, Clone, Deref, DerefMut)]
pub struct ShowCoordinates(bool);

fn startup(mut commands: Commands) {
    commands.spawn((
        Text::new("Position"),
        TextFont {
            font_size: 20.0,
            ..Default::default()
        },
        ShowCoordinates(false),
    ));
}

fn set_cursor_pos_label(
    cursor_pos: Res<CursorPos>,
    cur_tile: Res<CurTilePos>,
    mut cursor_label_q: Query<(&mut Text, &mut ShowCoordinates)>,
) {
    for (mut text, show_coordinates) in cursor_label_q.iter_mut() {
        text.0 = format!("{:?} {:?}", cursor_pos.0, cur_tile.0);
    }
}
