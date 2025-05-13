#![allow(clippy::type_complexity)]

use crate::{
    constants::{
        GRASS_BORDER_LOWER_IDX, GRASS_BORDER_LOWER_LEFT, GRASS_BORDER_LOWER_LEFT_IDX,
        GRASS_BORDER_LOWER_RIGHT, GRASS_BORDER_LOWER_RIGHT_IDX, GRASS_IDX, PopupBase,
    },
    tilemaptest::{AlphaPos, Buddy, CurTilePos, CursorPos, LastTilePos, TileBuddies},
};
use bevy::{color::palettes::basic::*, platform::collections::HashSet, prelude::*};
use bevy_ecs_tilemap::tiles::{TileColor, TilePos, TileStorage, TileTextureIndex};

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

#[derive(Component, Default, Clone)]
pub struct PickedItem {
    pub relative_pos: Vec<(TilePos, TileTextureIndex)>,
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
                let buddy_a = Buddy {
                    x: 1,
                    y: 0,
                    texture_index: TileTextureIndex(GRASS_BORDER_LOWER_IDX),
                };
                let buddy_b = Buddy {
                    x: 2,
                    y: 0,
                    texture_index: TileTextureIndex(GRASS_BORDER_LOWER_RIGHT_IDX),
                };
                let mut hs = HashSet::new();
                hs.insert(buddy_a);
                hs.insert(buddy_b);
                let tile_buddies = TileBuddies { buddies: Some(hs) };

                let picked_item = PickedItem {
                    relative_pos: vec![
                        (
                            TilePos { x: 0, y: 0 },
                            TileTextureIndex(GRASS_BORDER_LOWER_LEFT_IDX),
                        ),
                        (
                            TilePos { x: 0, y: 1 },
                            TileTextureIndex(GRASS_BORDER_LOWER_IDX),
                        ),
                        (
                            TilePos { x: 0, y: 2 },
                            TileTextureIndex(GRASS_BORDER_LOWER_RIGHT_IDX),
                        ),
                    ],
                };
                outline.color = RED.into();
                let picked_item = commands
                    .spawn((
                        Sprite::from_image(asset_server.load(GRASS_BORDER_LOWER_LEFT)),
                        picked_item,
                        Transform::from_xyz(50., 50., 1.),
                        GlobalZIndex(5),
                        tile_buddies,
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
    mut picked_q: Query<(&mut Transform, &PickedItem)>,
    cursor_pos: Res<CursorPos>,
    //mouse_button_input: Res<ButtonInput<MouseButton>>,
    cur_tile_pos: Res<CurTilePos>,
    last_tile_pos: Res<LastTilePos>,
    tilemap_q: Query<&TileStorage>,
    //actual_buddies_q: Query<(&TileBuddies, &AlphaPos)>,
    //mut buddy_q: Query<(Entity, &mut TileTextureIndex, &TileBuddies)>,
    mut color_q: Query<&mut TileColor>,
    texture_q: Query<&TileTextureIndex>,
) {
    if let Ok((mut transform, picked_item)) = picked_q.single_mut() {
        // Make the PickedItem follow the the mouse
        transform.translation.x = cursor_pos.0.x;
        transform.translation.y = cursor_pos.0.y;

        if cur_tile_pos.0 != last_tile_pos.0 {
            color_q
                .iter_mut()
                .for_each(|mut color| color.0 = Color::default());
            info!("HI")
        } else {
            info!("BYE");
            if let Some(active_tile_pos) = cur_tile_pos.0 {
                if let Ok(tile_storage) = tilemap_q.single() {
                    if let Some(active_tile_entity) = tile_storage.get(&active_tile_pos) {
                        let active_tile_texture = texture_q.get(active_tile_entity).unwrap();

                        let pos: Vec<Option<Entity>> = picked_item
                            .relative_pos
                            .iter()
                            .map(|(rel_pos, texture_index)| TilePos {
                                x: rel_pos.x + active_tile_pos.x,
                                y: rel_pos.y + active_tile_pos.y,
                            })
                            .map(|pos| (pos, tile_storage.get(&pos)))
                            .map(|(pos, maybe_tile_entity)| {
                                if let Some(tile_entity) = maybe_tile_entity {
                                    if let Ok(texture) = texture_q.get(tile_entity) {
                                        (pos, Some(texture), Some(tile_entity))
                                    } else {
                                        (pos, None, None)
                                    }
                                } else {
                                    (pos, None, None)
                                }
                            })
                            .chain(std::iter::once((
                                active_tile_pos,
                                Some(active_tile_texture),
                                Some(active_tile_entity),
                            )))
                            .map(|(pos, texture, entity)| {
                                if let (Some(texture), Some(entity)) = (texture, entity) {
                                    if texture.0 == GRASS_IDX {
                                        let mut color = color_q.get_mut(entity).unwrap();
                                        color.0 = Color::srgba(1.0, 1.0, 0.0, 0.5);
                                        Some(entity)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                            .collect();

                        // if mouse_button_input.just_pressed(MouseButton::Left) {
                        //     let maybe_buddies = if let Ok((buddies, alpha_pos)) =
                        //         actual_buddies_q.get(active_tile_entity)
                        //     {
                        //         let alpha_tile = tilemap_q.get(alpha_pos.0).unwrap();
                        //     } else {
                        //         None
                        //     };

                        //     if let Some(buddies) = maybe_buddies {
                        //         if let Some(buddies) = &buddies.buddies {
                        //             for buddy in buddies {
                        //                 let x = active_tile_pos.x as i32 + buddy.x;
                        //                 let y = active_tile_pos.y as i32 + buddy.y;
                        //                 let buddy_tile_pos = TilePos {
                        //                     x: x as u32,
                        //                     y: y as u32,
                        //                 };
                        //                 let buddy_entity = tile_storage.get(&buddy_tile_pos).unwrap();
                        //                 let (_, mut buddy_tile_texture_index, _) =
                        //                     actual_buddies_q.get_mut(buddy_entity).unwrap();
                        //                 *buddy_tile_texture_index = buddy.texture_index;
                        //             }
                        //         }
                        //     }
                        // }
                    }
                }
            }
        }
    }
}
