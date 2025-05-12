#![allow(clippy::type_complexity)]

use crate::{
    constants::{GRASS_BORDER_LOWER_RIGHT, GRASS_BORDER_LOWER_RIGHT_IDX, PopupBase},
    tilemaptest::{CurTilePos, CursorPos},
};
use bevy::{color::palettes::basic::*, prelude::*};
use bevy_ecs_tilemap::tiles::{TileStorage, TileTextureIndex};

pub struct Popup;

impl Plugin for Popup {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (button_system, pick_and_place));
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let base_node = (
        Node {
            margin: UiRect::all(Val::Px(25.0)),
            align_self: AlignSelf::Stretch,
            justify_self: JustifySelf::Stretch,
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            align_content: AlignContent::FlexStart,
            ..default()
        },
        BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
    );

    let root = commands.spawn(base_node).id();

    // labels for the different border edges
    let building_labels = ["Lower right"];

    // all the different combinations of border edges
    // these correspond to the labels above
    let lower_right = asset_server.load(GRASS_BORDER_LOWER_RIGHT);
    let building_images = [ImageNode::new(lower_right)];

    for (label, building_image) in building_labels.into_iter().zip(building_images) {
        let building_node = commands
            .spawn((
                building_image,
                Button,
                Outline {
                    width: Val::Px(2.),
                    offset: Val::Px(0.),
                    color: Color::WHITE,
                },
            ))
            .id();
        let label_node = commands
            .spawn((
                Text::new(label),
                TextFont {
                    font_size: 12.0,
                    ..Default::default()
                },
            ))
            .id();
        let container = commands
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Percent(12.0),
                ..default()
            })
            .add_children(&[building_node, label_node])
            .id();
        commands.entity(root).add_child(container);
    }

    let popup_label = commands
        .spawn((
            Node {
                margin: UiRect {
                    left: Val::Px(25.0),
                    right: Val::Px(25.0),
                    top: Val::Px(25.0),
                    bottom: Val::Px(0.0),
                },
                ..default()
            },
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
        ))
        .with_children(|builder| {
            builder.spawn((
                Text::new("Buildings"),
                TextFont {
                    font_size: 20.0,
                    ..Default::default()
                },
            ));
        })
        .id();

    commands
        .spawn((
            Node {
                margin: UiRect::all(Val::Px(25.0)),
                flex_direction: FlexDirection::Column,
                align_self: AlignSelf::Stretch,
                justify_self: JustifySelf::Stretch,
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                align_content: AlignContent::FlexStart,
                ..default()
            },
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
            PopupBase,
        ))
        .add_child(popup_label)
        .add_child(root);
}

#[derive(Component)]
pub struct PickedItem;

fn button_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &mut Outline),
        (Changed<Interaction>, With<Button>),
    >,
    asset_server: Res<AssetServer>,
    mut popup_q: Query<&mut Node, With<PopupBase>>,
) {
    for (interaction, mut outline) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                outline.color = RED.into();
                let picked_item = commands
                    .spawn((
                        Sprite::from_image(asset_server.load(GRASS_BORDER_LOWER_RIGHT)),
                        PickedItem,
                        Transform::from_xyz(50., 50., 1.),
                        GlobalZIndex(5),
                    ))
                    .id();
                info!("{picked_item:?}");

                // Disappear the popup
                for mut node in &mut popup_q {
                    node.display = match node.display {
                        Display::None => Display::Flex,
                        _ => Display::None,
                    };
                }
            }
            Interaction::Hovered => {
                outline.color = Color::WHITE;
            }
            Interaction::None => {
                outline.color = Color::BLACK;
            }
        }
    }
}

fn pick_and_place(
    mut picked_q: Query<&mut Transform, With<PickedItem>>,
    cursor_pos: Res<CursorPos>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    cur_tile_pos: Res<CurTilePos>,
    tilemap_q: Query<&TileStorage>,
    mut tile_q: Query<(Entity, &mut TileTextureIndex)>,
) {
    if let Ok(mut transform) = picked_q.single_mut() {
        // Make the PickedItem follow the the mouse
        transform.translation.x = cursor_pos.0.x;
        transform.translation.y = cursor_pos.0.y;

        // Interact with tile
        if let Some(tile_pos) = cur_tile_pos.0 {
            if let Ok(tile_storage) = tilemap_q.single() {
                if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                    if mouse_button_input.just_pressed(MouseButton::Left) {
                        info!("MY TILE AGAIN: {tile_entity} {} {}", tile_pos.x, tile_pos.y);
                        if let Ok((_, mut tile_texture_index)) = tile_q.get_mut(tile_entity) {
                            *tile_texture_index = TileTextureIndex(GRASS_BORDER_LOWER_RIGHT_IDX);
                        }
                    }
                }
            }
        }
    }
}
