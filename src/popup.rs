#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use crate::{
    constants::{
        DIRT, GRASS_BORDER_LOWER, GRASS_BORDER_LOWER_IDX, GRASS_BORDER_LOWER_LEFT,
        GRASS_BORDER_LOWER_LEFT_IDX, GRASS_BORDER_LOWER_RIGHT, GRASS_BORDER_LOWER_RIGHT_IDX,
        GRASS_BORDER_UPPER, GRASS_BORDER_UPPER_IDX, GRASS_IDX, PopupBase,
    },
    tilemaptest::{AlphaPos, CurTilePos, CursorPos, LastTilePos, TileBuddies, TileValues},
};
use bevy::{color::palettes::basic::*, prelude::*};
use bevy_ecs_tilemap::tiles::{TileColor, TilePos, TileStorage, TileTextureIndex};

pub struct Popup;

impl Plugin for Popup {
    fn build(&self, app: &mut App) {
        app.add_event::<PopupEvent>()
            .add_systems(Startup, setup)
            .add_systems(Update, (button_system, pick_and_place, place_tiles));
    }
}

#[derive(Event, Clone)]
pub struct PopupEvent {
    entity: Entity,
    tile_values: TileValues,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let popup_root = commands
        .spawn((
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
        ))
        .id();

    let horizontal_walkway_label = "Horizontal Walkway";
    let grass_border_upper = ImageNode::new(asset_server.load(GRASS_BORDER_UPPER));
    let grass_border_lower = ImageNode::new(asset_server.load(GRASS_BORDER_LOWER));

    let (horizontal_walkway_tile_node, horizontal_walkway_label_node) = four_by_four(
        horizontal_walkway_label,
        &grass_border_lower,
        &grass_border_lower,
        &grass_border_upper,
        &grass_border_upper,
        &mut commands,
    );

    let container = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Percent(12.0),
            ..default()
        })
        .add_children(&[horizontal_walkway_tile_node, horizontal_walkway_label_node])
        .id();

    commands.entity(popup_root).add_child(container);

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
        .add_child(popup_root);
}

#[derive(Component, Default, Clone)]
pub struct PopupItem {
    pub alpha_texture_idx: TileTextureIndex,
    pub relative_pos_and_idx: Vec<(TilePos, TileTextureIndex)>,
}

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
                let picked_item = PopupItem {
                    alpha_texture_idx: TileTextureIndex(GRASS_BORDER_LOWER_IDX),
                    relative_pos_and_idx: vec![
                        (
                            TilePos { x: 1, y: 0 },
                            TileTextureIndex(GRASS_BORDER_LOWER_IDX),
                        ),
                        (
                            TilePos { x: 0, y: 1 },
                            TileTextureIndex(GRASS_BORDER_UPPER_IDX),
                        ),
                        (
                            TilePos { x: 1, y: 1 },
                            TileTextureIndex(GRASS_BORDER_UPPER_IDX),
                        ),
                    ],
                };
                outline.color = RED.into();
                commands.spawn((
                    Sprite::from_image(asset_server.load(GRASS_BORDER_LOWER_LEFT)),
                    picked_item,
                    Transform::from_xyz(50., 50., 1.),
                    GlobalZIndex(5),
                ));

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

enum PlaceableReason {
    None,
    Grass,
}

fn pick_and_place(
    mut picked_q: Query<(&mut Transform, &PopupItem)>,
    mut color_q: Query<&mut TileColor>,
    mut popup_e: EventWriter<PopupEvent>,
    cursor_pos: Res<CursorPos>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    cur_tile_pos: Res<CurTilePos>,
    last_tile_pos: Res<LastTilePos>,
    tilemap_q: Query<&TileStorage>,
    texture_q: Query<&TileTextureIndex>,
) {
    if let Ok((mut transform, popup_item)) = picked_q.single_mut() {
        // Make the PickedItem follow the the mouse
        transform.translation.x = cursor_pos.0.x;
        transform.translation.y = cursor_pos.0.y;

        if cur_tile_pos.0 != last_tile_pos.0 {
            color_q
                .iter_mut()
                .for_each(|mut color| color.0 = Color::default());
        } else if let Some(active_tile_pos) = cur_tile_pos.0 {
            if let Ok(tile_storage) = tilemap_q.single() {
                if let Some(active_tile_entity) = tile_storage.get(&active_tile_pos) {
                    let active_tile_texture = texture_q
                        .get(active_tile_entity)
                        .expect("Texture query did not find active tile.");

                    let tiles_to_highlight: Vec<
                        Option<(TilePos, &TileTextureIndex, Entity, PlaceableReason)>,
                    > = popup_item
                        .relative_pos_and_idx
                        .iter()
                        .map(|(rel_pos, texture_idx)| {
                            (
                                TilePos {
                                    x: rel_pos.x + active_tile_pos.x,
                                    y: rel_pos.y + active_tile_pos.y,
                                },
                                texture_idx,
                            )
                        })
                        .map(|(pos, texture_idx)| {
                            (pos, texture_idx, tile_storage.checked_get(&pos))
                        })
                        .map(|(pos, texture_idx, maybe_tile_entity)| {
                            if let Some(tile_entity) = maybe_tile_entity {
                                if let Ok(existing_texture) = texture_q.get(tile_entity) {
                                    (pos, texture_idx, Some(existing_texture), Some(tile_entity))
                                } else {
                                    (pos, texture_idx, None, None)
                                }
                            } else {
                                (pos, texture_idx, None, None)
                            }
                        })
                        .chain(std::iter::once((
                            active_tile_pos,
                            &popup_item.alpha_texture_idx,
                            Some(active_tile_texture),
                            Some(active_tile_entity),
                        )))
                        .map(
                            |(pos, texture_idx, maybe_existing_texture_idx, maybe_entity)| {
                                if let (Some(existing_texture_idx), Some(entity)) =
                                    (maybe_existing_texture_idx, maybe_entity)
                                {
                                    if existing_texture_idx.0 == GRASS_IDX {
                                        Some((pos, texture_idx, entity, PlaceableReason::Grass))
                                    } else {
                                        Some((pos, texture_idx, entity, PlaceableReason::None))
                                    }
                                } else {
                                    None
                                }
                            },
                        )
                        .collect();

                    let is_placeable = if tiles_to_highlight.iter().any(|tile| match tile {
                        Some(tile) => match tile.3 {
                            PlaceableReason::None => true,
                            PlaceableReason::Grass => false,
                        },
                        None => true,
                    }) {
                        tiles_to_highlight
                            .iter()
                            .flatten()
                            .for_each(|(_, _, entity, _)| {
                                let mut color = color_q.get_mut(*entity).unwrap();
                                color.0 = Color::srgba(1.0, 0.0, 0.0, 0.5);
                            });
                        None
                    } else {
                        let placeables = tiles_to_highlight
                            .iter()
                            .flatten()
                            .map(|(pos, texture_idx, entity, _)| {
                                let mut color = color_q.get_mut(*entity).unwrap();
                                color.0 = Color::srgba(0.0, 1.0, 0.5, 0.5);
                                PopupEvent {
                                    entity: *entity,
                                    tile_values: TileValues {
                                        pos: *pos,
                                        alpha_pos: AlphaPos(active_tile_pos),
                                        texture_index: **texture_idx,
                                        buddies: TileBuddies { buddies: None },
                                    },
                                }
                            })
                            .collect::<Vec<PopupEvent>>();
                        Some(placeables)
                    };

                    if mouse_button_input.just_pressed(MouseButton::Left) {
                        is_placeable.iter().flatten().for_each(|event| {
                            let _id = popup_e.write(event.clone());
                        });
                    }
                }
            }
        }
    }
}

fn place_tiles(
    mut popup_e: EventReader<PopupEvent>,
    mut tiles_q: Query<(&mut TileBuddies, &mut AlphaPos, &mut TileTextureIndex)>,
) {
    for event in popup_e.read() {
        if let Ok((mut buddies, mut alpha_pos, mut texture_idx)) = tiles_q.get_mut(event.entity) {
            *buddies = event.tile_values.buddies.clone();
            *alpha_pos = event.tile_values.alpha_pos;
            *texture_idx = event.tile_values.texture_index;
        }
    }
}

fn four_by_four(
    label: &str,
    lower_left: &ImageNode,
    lower_right: &ImageNode,
    upper_left: &ImageNode,
    upper_right: &ImageNode,
    commands: &mut Commands,
) -> (Entity, Entity) {
    let horizontal_walkway_tile_node = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            Button,
            Outline {
                width: Val::Px(2.),
                offset: Val::Px(0.),
                color: Color::WHITE,
            },
        ))
        .with_children(|main_tile| {
            main_tile.spawn(Node::default()).with_children(|top_row| {
                top_row.spawn((upper_left.clone(),));
                top_row.spawn((upper_right.clone(),));
            });

            main_tile.spawn(Node::default()).with_children(|top_row| {
                top_row.spawn((lower_left.clone(),));
                top_row.spawn((lower_right.clone(),));
            });
        })
        .id();
    let horizontal_walkway_label_node = commands
        .spawn((
            Text::new(label),
            TextFont {
                font_size: 12.0,
                ..Default::default()
            },
        ))
        .id();

    (horizontal_walkway_tile_node, horizontal_walkway_label_node)
}
