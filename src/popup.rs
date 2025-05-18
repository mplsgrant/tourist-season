#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use crate::{
    constants::{ImgAsset, PopupBase},
    tilemaptest::{AlphaPos, CurTilePos, CursorPos, LastTilePos, TileBuddies, TileValues},
    tourists::{TouristDespawnPoint, TouristSpawnPoint},
};
use bevy::{color::palettes::basic::*, prelude::*};
use bevy_ecs_tilemap::tiles::{TileColor, TilePos, TileStorage, TileTextureIndex};

pub struct Popup;

impl Plugin for Popup {
    fn build(&self, app: &mut App) {
        app.add_event::<PopupEvent>()
            .add_event::<EraserEvent>()
            .add_systems(Startup, startup)
            .add_systems(
                Update,
                (button_system, pick_and_place, place_tiles, erase_tiles),
            );
    }
}

#[derive(Event, Clone, Debug)]
pub struct PopupEvent {
    pub clicked_entity: Entity,
    pub tile_values: TileValues,
}

#[derive(Event, Clone)]
pub struct EraserEvent {
    entities: Vec<Entity>,
}

#[derive(Component, Clone, Copy)]
pub enum PopupMenuTileType {
    HorizontalPath,
    BuildingA,
    Grass,
    Entrypoint,
    DespawnPoint,
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
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

    // GRASS (it's an eraser)
    let building_a_label = "Eraser";

    let grass = ImageNode::new(asset_server.load(ImgAsset::Grass.path()));

    let my_tile = [[&grass]];
    let (grass_tile_node, grass_label_node) = matrix_to_tile_nodes(
        building_a_label,
        my_tile,
        PopupMenuTileType::Grass,
        &mut commands,
    );
    /////////////////

    // BUILDING A
    let building_a_label = "Building A";
    let red_brick_col_upper = ImageNode::new(asset_server.load(ImgAsset::RedBrickColUpper.path()));
    let red_brick_col_lower = ImageNode::new(asset_server.load(ImgAsset::RedBrickColLower.path()));

    let my_tile = [
        [&red_brick_col_upper, &red_brick_col_upper],
        [&red_brick_col_lower, &red_brick_col_lower],
    ];
    let (building_a_tile_node, building_a_label_node) = matrix_to_tile_nodes(
        building_a_label,
        my_tile,
        PopupMenuTileType::BuildingA,
        &mut commands,
    );
    ///////////////

    // WALKWAY
    let horizontal_walkway_label = "Horizontal Walkway";
    let grass_border_upper = ImageNode::new(asset_server.load(ImgAsset::GrassBorderUpper.path()));
    let grass_border_lower = ImageNode::new(asset_server.load(ImgAsset::GrassBorderLower.path()));

    let my_tile = [
        [&grass_border_upper, &grass_border_upper],
        [&grass_border_lower, &grass_border_lower],
    ];
    let (horizontal_walkway_tile_node, horizontal_walkway_label_node) = matrix_to_tile_nodes(
        horizontal_walkway_label,
        my_tile,
        PopupMenuTileType::HorizontalPath,
        &mut commands,
    );
    /////////////////

    // Spawnpoint
    let entrypoint_label = "Entrypoint";
    let sidewalk = ImageNode::new(asset_server.load(ImgAsset::Sidewalk.path()));

    let my_tile = [[&sidewalk, &sidewalk], [&sidewalk, &sidewalk]];
    let (entrypoint_tile_node, entrypoint_label_node) = matrix_to_tile_nodes(
        entrypoint_label,
        my_tile,
        PopupMenuTileType::Entrypoint,
        &mut commands,
    );
    /////////////////

    // Despawn Point
    let despawn_label = "Despawn Point";
    let sidewalk_left = ImageNode::new(asset_server.load(ImgAsset::Sidewalk.path()));
    let sidewalk_right = ImageNode::new(asset_server.load(ImgAsset::Sidewalk.path()));

    let my_tile = [
        [&sidewalk_left, &sidewalk_right],
        [&sidewalk_left, &sidewalk_right],
    ];
    let (despawn_tile_node, despawn_label_node) = matrix_to_tile_nodes(
        despawn_label,
        my_tile,
        PopupMenuTileType::DespawnPoint,
        &mut commands,
    );
    /////////////////

    let container = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Percent(12.0),
            ..default()
        })
        .add_children(&[horizontal_walkway_tile_node, horizontal_walkway_label_node])
        .add_children(&[building_a_tile_node, building_a_label_node])
        .add_children(&[grass_tile_node, grass_label_node])
        .add_children(&[entrypoint_tile_node, entrypoint_label_node])
        .add_children(&[despawn_tile_node, despawn_label_node])
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
    pub spawnpoint: Option<TouristSpawnPoint>,
    pub despawnpoint: Option<TouristDespawnPoint>,
}

fn button_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &mut Outline, &PopupMenuTileType),
        (Changed<Interaction>, With<Button>),
    >,
    asset_server: Res<AssetServer>,
    mut popup_q: Query<&mut Node, With<PopupBase>>,
) {
    for (interaction, mut outline, tile_type) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                outline.color = RED.into();

                match tile_type {
                    PopupMenuTileType::HorizontalPath => {
                        let picked_item = PopupItem {
                            alpha_texture_idx: TileTextureIndex(ImgAsset::GrassBorderLower.index()),
                            relative_pos_and_idx: vec![
                                (
                                    TilePos { x: 1, y: 0 },
                                    TileTextureIndex(ImgAsset::GrassBorderLower.index()),
                                ),
                                (
                                    TilePos { x: 0, y: 1 },
                                    TileTextureIndex(ImgAsset::GrassBorderUpper.index()),
                                ),
                                (
                                    TilePos { x: 1, y: 1 },
                                    TileTextureIndex(ImgAsset::GrassBorderUpper.index()),
                                ),
                            ],
                            spawnpoint: None,
                            despawnpoint: None,
                        };

                        commands.spawn((
                            Sprite::from_image(
                                asset_server.load(ImgAsset::GrassBorderLowerLeft.path()),
                            ),
                            picked_item,
                            Transform::from_xyz(50., 50., 1.),
                            GlobalZIndex(5),
                        ));
                    }
                    PopupMenuTileType::BuildingA => {
                        let picked_item = PopupItem {
                            alpha_texture_idx: TileTextureIndex(ImgAsset::RedBrickColLower.index()),
                            relative_pos_and_idx: vec![
                                (
                                    TilePos { x: 1, y: 0 },
                                    TileTextureIndex(ImgAsset::DoorSingleGlassClosed.index()),
                                ),
                                (
                                    TilePos { x: 2, y: 0 },
                                    TileTextureIndex(ImgAsset::RedBrickColLower.index()),
                                ),
                                (
                                    TilePos { x: 0, y: 1 },
                                    TileTextureIndex(ImgAsset::RedBrickColUpper.index()),
                                ),
                                (
                                    TilePos { x: 1, y: 1 },
                                    TileTextureIndex(ImgAsset::RedBrickMidUpperA.index()),
                                ),
                                (
                                    TilePos { x: 2, y: 1 },
                                    TileTextureIndex(ImgAsset::RedBrickColUpper.index()),
                                ),
                            ],
                            spawnpoint: None,
                            despawnpoint: None,
                        };

                        commands.spawn((
                            Sprite::from_image(asset_server.load(ImgAsset::RedBrickBlankA.path())),
                            picked_item,
                            Transform::from_xyz(50., 50., 1.),
                            GlobalZIndex(5),
                        ));
                    }
                    PopupMenuTileType::Grass => {
                        let picked_item = PopupItem {
                            alpha_texture_idx: TileTextureIndex(ImgAsset::Grass.index()),
                            relative_pos_and_idx: vec![],
                            spawnpoint: None,
                            despawnpoint: None,
                        };

                        commands.spawn((
                            Sprite::from_image(asset_server.load(ImgAsset::Grass.path())),
                            picked_item,
                            Transform::from_xyz(50., 50., 1.),
                            GlobalZIndex(5),
                        ));
                    }
                    PopupMenuTileType::Entrypoint => {
                        let picked_item = PopupItem {
                            alpha_texture_idx: TileTextureIndex(
                                ImgAsset::SidewalkBottomLeft.index(),
                            ),
                            relative_pos_and_idx: vec![
                                (
                                    TilePos { x: 1, y: 0 },
                                    TileTextureIndex(ImgAsset::SidewalkBottom.index()),
                                ),
                                (
                                    TilePos { x: 2, y: 0 },
                                    TileTextureIndex(ImgAsset::SidewalkBottom.index()),
                                ),
                                (
                                    TilePos { x: 0, y: 1 },
                                    TileTextureIndex(ImgAsset::SidewalkLeft.index()),
                                ),
                                (
                                    TilePos { x: 1, y: 1 },
                                    TileTextureIndex(ImgAsset::Sidewalk.index()),
                                ),
                                (
                                    TilePos { x: 2, y: 1 },
                                    TileTextureIndex(ImgAsset::Sidewalk.index()),
                                ),
                                (
                                    TilePos { x: 0, y: 2 },
                                    TileTextureIndex(ImgAsset::SidewalkTopLeft.index()),
                                ),
                                (
                                    TilePos { x: 1, y: 2 },
                                    TileTextureIndex(ImgAsset::SidewalkTop.index()),
                                ),
                                (
                                    TilePos { x: 2, y: 2 },
                                    TileTextureIndex(ImgAsset::SidewalkTop.index()),
                                ),
                            ],
                            spawnpoint: Some(TouristSpawnPoint {}),
                            despawnpoint: None,
                        };

                        commands.spawn((
                            Sprite::from_image(asset_server.load(ImgAsset::SidewalkLeft.path())),
                            picked_item,
                            Transform::from_xyz(50., 50., 1.),
                            GlobalZIndex(5),
                        ));
                    }
                    PopupMenuTileType::DespawnPoint => {
                        let picked_item = PopupItem {
                            alpha_texture_idx: TileTextureIndex(
                                ImgAsset::SidewalkBottomLeft.index(),
                            ),
                            relative_pos_and_idx: vec![
                                (
                                    TilePos { x: 1, y: 0 },
                                    TileTextureIndex(ImgAsset::SidewalkBottom.index()),
                                ),
                                (
                                    TilePos { x: 2, y: 0 },
                                    TileTextureIndex(ImgAsset::SidewalkBottom.index()),
                                ),
                                (
                                    TilePos { x: 0, y: 1 },
                                    TileTextureIndex(ImgAsset::Sidewalk.index()),
                                ),
                                (
                                    TilePos { x: 1, y: 1 },
                                    TileTextureIndex(ImgAsset::Sidewalk.index()),
                                ),
                                (
                                    TilePos { x: 2, y: 1 },
                                    TileTextureIndex(ImgAsset::Sidewalk.index()),
                                ),
                                (
                                    TilePos { x: 0, y: 2 },
                                    TileTextureIndex(ImgAsset::SidewalkTopLeft.index()),
                                ),
                                (
                                    TilePos { x: 1, y: 2 },
                                    TileTextureIndex(ImgAsset::SidewalkTop.index()),
                                ),
                                (
                                    TilePos { x: 2, y: 2 },
                                    TileTextureIndex(ImgAsset::SidewalkTop.index()),
                                ),
                            ],
                            spawnpoint: None,
                            despawnpoint: Some(TouristDespawnPoint {}),
                        };

                        commands.spawn((
                            Sprite::from_image(asset_server.load(ImgAsset::Sidewalk.path())),
                            picked_item,
                            Transform::from_xyz(50., 50., 1.),
                            GlobalZIndex(5),
                        ));
                    }
                }

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
    NotPlaceable,
    Grass,
}

fn pick_and_place(
    mut picked_q: Query<(&mut Transform, &PopupItem)>,
    mut color_q: Query<&mut TileColor>,
    mut popup_e: EventWriter<PopupEvent>,
    mut eraser_e: EventWriter<EraserEvent>,
    alpha_buddies_q: Query<(&AlphaPos, &TileBuddies)>,
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

                    let mut tile_buddies = TileBuddies::default();

                    // Handle the Eraser condition
                    if popup_item.alpha_texture_idx.0 == ImgAsset::Grass.index() {
                        // Main idea: Get the alpha tile and all its buddies and erase them
                        let (alpha_pos, buddies) = alpha_buddies_q
                            .get(active_tile_entity)
                            .expect("alpha pos and buddies");
                        let storage = tilemap_q.single().expect("storage");
                        let entities: Vec<Entity> = if alpha_pos.0 == active_tile_pos {
                            // The active tile is the alpha tile
                            buddies
                                .buddies
                                .iter()
                                .flat_map(|tile_pos| storage.checked_get(tile_pos))
                                .chain(std::iter::once(active_tile_entity))
                                .collect()
                        } else {
                            // The active tile is a buddy, and need to query alpha tile and other buddies
                            let alpha_entity_hack = storage
                                .checked_get(&alpha_pos.0)
                                .expect("alpha entity exists");
                            storage
                                .checked_get(&alpha_pos.0)
                                .iter()
                                .filter_map(|alpha_entity| alpha_buddies_q.get(*alpha_entity).ok())
                                .flat_map(|(_alpha_pos, tile_buddies)| tile_buddies.buddies.clone())
                                .filter_map(|buddy_tile_pos| storage.checked_get(&buddy_tile_pos))
                                .chain(std::iter::once(alpha_entity_hack))
                                .collect()
                        };
                        if mouse_button_input.just_pressed(MouseButton::Left) {
                            let eraser_event = EraserEvent { entities };
                            let _id = eraser_e.write(eraser_event);
                        }
                    } else {
                        // Continue on handling a normal (non-eraser) tile
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
                                tile_buddies.buddies.insert(pos); // Hacky way to handle tile buddies
                                (pos, texture_idx, tile_storage.checked_get(&pos))
                            })
                            .map(|(pos, texture_idx, maybe_tile_entity)| {
                                if let Some(tile_entity) = maybe_tile_entity {
                                    if let Ok(existing_texture) = texture_q.get(tile_entity) {
                                        (
                                            pos,
                                            texture_idx,
                                            Some(existing_texture),
                                            Some(tile_entity),
                                        )
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
                                        if existing_texture_idx.0 == ImgAsset::Grass.index() {
                                            Some((pos, texture_idx, entity, PlaceableReason::Grass))
                                        } else {
                                            Some((
                                                pos,
                                                texture_idx,
                                                entity,
                                                PlaceableReason::NotPlaceable,
                                            ))
                                        }
                                    } else {
                                        None
                                    }
                                },
                            )
                            .collect();

                        let placeables = if tiles_to_highlight.iter().any(|tile| match tile {
                            Some(tile) => match tile.3 {
                                PlaceableReason::NotPlaceable => true,
                                PlaceableReason::Grass => false,
                            },
                            None => true,
                        }) {
                            // NOT PLACEABLE
                            tiles_to_highlight
                                .iter()
                                .flatten()
                                .for_each(|(_, _, entity, _)| {
                                    let mut color = color_q.get_mut(*entity).unwrap();
                                    color.0 = Color::srgba(1.0, 0.0, 0.0, 0.5); // RED
                                });
                            None
                        } else {
                            // YES PLACEABLE
                            let placeables = tiles_to_highlight
                                .iter()
                                .flatten()
                                .map(|(pos, texture_idx, entity, _)| {
                                    let mut color = color_q.get_mut(*entity).unwrap();
                                    color.0 = Color::srgba(0.0, 1.0, 0.5, 0.5); // GREEN
                                    if pos == &active_tile_pos {
                                        // Setting an alpha tile
                                        PopupEvent {
                                            clicked_entity: *entity,
                                            tile_values: TileValues {
                                                pos: *pos,
                                                alpha_pos: AlphaPos(active_tile_pos),
                                                texture_index: **texture_idx,
                                                buddies: tile_buddies.clone(),
                                                spawnpoint: popup_item.spawnpoint.clone(),
                                                despawnpoint: popup_item.despawnpoint.clone(),
                                            },
                                        }
                                    } else {
                                        // Setting a buddy tile
                                        PopupEvent {
                                            clicked_entity: *entity,
                                            tile_values: TileValues {
                                                pos: *pos,
                                                alpha_pos: AlphaPos(active_tile_pos),
                                                texture_index: **texture_idx,
                                                buddies: TileBuddies::default(),
                                                spawnpoint: None,
                                                despawnpoint: None,
                                            },
                                        }
                                    }
                                })
                                .collect::<Vec<PopupEvent>>();
                            Some(placeables)
                        };

                        if mouse_button_input.just_pressed(MouseButton::Left) {
                            placeables.iter().flatten().for_each(|event| {
                                let _id = popup_e.write(event.clone());
                            });
                        }
                    }
                }
            }
        }
    }
}

fn place_tiles(
    mut commands: Commands,
    mut popup_e: EventReader<PopupEvent>,
    mut tiles_q: Query<(&mut TileBuddies, &mut AlphaPos, &mut TileTextureIndex)>,
) {
    for event in popup_e.read() {
        if let Ok((mut buddies, mut alpha_pos, mut texture_idx)) =
            tiles_q.get_mut(event.clicked_entity)
        {
            // Update the tile
            *buddies = event.tile_values.buddies.clone();
            *alpha_pos = event.tile_values.alpha_pos;
            *texture_idx = event.tile_values.texture_index;

            if event.tile_values.spawnpoint.is_some() {
                commands
                    .entity(event.clicked_entity)
                    .insert(TouristSpawnPoint {});
                info!("Placed spawnpoint");
            } else {
                commands
                    .entity(event.clicked_entity)
                    .remove::<TouristSpawnPoint>();
            }

            if event.tile_values.despawnpoint.is_some() {
                commands
                    .entity(event.clicked_entity)
                    .insert(TouristDespawnPoint {});
                info!("Placed despawnpoint");
            } else {
                commands
                    .entity(event.clicked_entity)
                    .remove::<TouristDespawnPoint>();
            }
        }
    }
}

fn erase_tiles(
    mut commands: Commands,
    mut eraser_e: EventReader<EraserEvent>,
    mut tiles_q: Query<(
        &TilePos,
        &mut TileBuddies,
        &mut AlphaPos,
        &mut TileTextureIndex,
    )>,
) {
    for event in eraser_e.read() {
        for entity in &event.entities {
            if let Ok((tile_pos, mut buddies, mut alpha_pos, mut texture_idx)) =
                tiles_q.get_mut(*entity)
            {
                *buddies = TileBuddies::default();
                *alpha_pos = AlphaPos(*tile_pos);
                *texture_idx = TileTextureIndex(ImgAsset::Grass.index());

                commands.entity(*entity).remove::<TouristSpawnPoint>();
                commands.entity(*entity).remove::<TouristDespawnPoint>();
            }
        }
    }
}

fn matrix_to_tile_nodes<'a, I, J>(
    label: &str,
    matrix: I,
    tile_type: PopupMenuTileType,
    commands: &mut Commands,
) -> (Entity, Entity)
where
    I: IntoIterator<Item = J>,
    J: IntoIterator<Item = &'a ImageNode>,
{
    let horizontal_walkway_tile_node = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            Button,
            tile_type,
            Outline {
                width: Val::Px(2.),
                offset: Val::Px(0.),
                color: Color::WHITE,
            },
        ))
        .with_children(|main_tile| {
            for matrix_row in matrix {
                main_tile.spawn(Node::default()).with_children(|tile_row| {
                    for item in matrix_row {
                        tile_row.spawn((item.clone(),));
                    }
                });
            }
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
