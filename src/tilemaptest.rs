use crate::{
    bdk_zone::get_data_dir,
    constants::{ImgAsset, MAP_DIR, MAP_JSON, PopupBase, Z_TILEMAP},
};
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use bevy_ecs_tilemap::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use strum::IntoEnumIterator;

pub struct GameMap;

impl Plugin for GameMap {
    fn build(&self, app: &mut App) {
        app.add_event::<GameMapEvent>()
            .add_plugins(
                DefaultPlugins
                    .set(WindowPlugin {
                        primary_window: Some(Window {
                            title: String::from("Accessing Tiles Example"),
                            ..Default::default()
                        }),
                        ..default()
                    })
                    .set(ImagePlugin::default_nearest()),
            )
            .init_resource::<CursorPos>()
            .init_resource::<CurTilePos>()
            .init_resource::<LastTilePos>()
            .add_plugins(TilemapPlugin)
            .add_plugins(crate::tiled_thing::TiledMapPlugin)
            .add_systems(Startup, startup_original_tiles)
            .add_systems(
                First,
                (
                    crate::camera::movement,
                    update_cursor_pos,
                    update_cur_tile_pos,
                )
                    .chain(),
            )
            .add_systems(Update, interact_with_tile)
            .add_systems(Last, save_tilemap);
    }
}

#[derive(Event)]
pub enum GameMapEvent {
    Save,
}

#[derive(Component, Clone, Default, Serialize, Deserialize)]
pub struct TileBuddies {
    pub buddies: Option<HashSet<Buddy>>,
}

#[derive(Clone, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Buddy {
    pub x: i32,
    pub y: i32,
    pub texture_index: TileTextureIndex,
}

#[derive(
    Component,
    Reflect,
    Default,
    Clone,
    Copy,
    Debug,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Deserialize,
    Serialize,
)]
pub struct AlphaPos(pub TilePos);

#[derive(Component, Clone, Default, Serialize, Deserialize)]
pub struct TileValues {
    pub pos: TilePos,
    pub alpha_pos: AlphaPos,
    pub texture_index: TileTextureIndex,
    pub buddies: TileBuddies,
}

fn startup_original_tiles(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let image_handles: Vec<Handle<Image>> = ImgAsset::iter()
        .map(|img_asset| asset_server.load(img_asset.path()))
        .collect();

    let textures = TilemapTexture::Vector(image_handles);
    // let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    // Size of the tile map in tiles.
    let map_size = TilemapSize { x: 128, y: 128 };

    // To create a map we use the TileStorage component.
    // This component is a grid of tile entities and is used to help keep track of individual
    // tiles in the world. If you have multiple layers of tiles you would have a Tilemap2dStorage
    // component per layer.
    let mut tile_storage = TileStorage::empty(map_size);

    // For the purposes of this example, we consider a tilemap with rectangular tiles.
    let map_type = TilemapType::Square;

    // Create a tilemap entity a little early
    // We want this entity early because we need to tell each tile which tilemap entity
    // it is associated with. This is done with the TilemapId component on each tile.
    let tilemap_entity = commands.spawn_empty().id();

    // Load map
    let map_json_file = get_data_dir(Some(MAP_DIR.into())).unwrap().join(MAP_JSON);

    let map: Vec<TileValues> = if let Ok(map) = fs::read_to_string(&map_json_file) {
        serde_json::from_str(&map).unwrap()
    } else {
        // Make a map out of whole cloth
        let mut map = vec![];
        for x in 0..map_size.x {
            for y in 0..map_size.y {
                let value = TileValues {
                    pos: TilePos { x, y },
                    alpha_pos: AlphaPos(TilePos { x, y }), // alpha can be self
                    texture_index: TileTextureIndex(ImgAsset::Grass.index()),
                    buddies: TileBuddies::default(),
                };
                map.push(value);
            }
        }
        map
    };

    // Spawn a 32 by 32 tilemap.
    // Alternatively, you can use helpers::fill_tilemap.
    for tile_values in map {
        let tile_entity = commands
            .spawn((
                TileBundle {
                    position: tile_values.pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: tile_values.texture_index,
                    ..Default::default()
                },
                tile_values.buddies,
                tile_values.alpha_pos,
            ))
            .id();
        // Here we let the tile storage component know what tiles we have.
        tile_storage.set(&tile_values.pos, tile_entity);
    }

    // We can grab a list of neighbors.
    let neighbor_positions =
        Neighbors::get_square_neighboring_positions(&TilePos { x: 0, y: 0 }, &map_size, true);
    let neighbor_entities = neighbor_positions.entities(&tile_storage);

    // We can access tiles using:
    assert!(tile_storage.get(&TilePos { x: 0, y: 0 }).is_some());
    assert_eq!(neighbor_entities.iter().count(), 3); // Only 3 neighbors since negative is outside of map.

    // This is the size of each individual tiles in pixels.
    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();

    // Spawns a tilemap.
    // Once the tile storage is inserted onto the tilemap entity it can no longer be accessed.
    commands.entity(tilemap_entity).insert((
        TilemapBundle {
            grid_size,
            size: map_size,
            storage: tile_storage,
            map_type,
            texture: textures, //TilemapTexture::Single(texture_handle),
            tile_size,
            anchor: TilemapAnchor::Center,
            ..Default::default()
        },
        GlobalZIndex(Z_TILEMAP),
    ));
}

fn save_tilemap(
    mut tilemap_e: EventReader<GameMapEvent>,
    tilemap_q: Query<(&TilePos, &AlphaPos, &TileTextureIndex, &TileBuddies)>,
) {
    for tilemap_event in tilemap_e.read() {
        match tilemap_event {
            GameMapEvent::Save => {
                let items: Vec<TileValues> = tilemap_q
                    .iter()
                    .map(|(pos, alpha_pos, idx, buddies)| TileValues {
                        pos: *pos,
                        alpha_pos: *alpha_pos,
                        texture_index: *idx,
                        buddies: buddies.clone(),
                    })
                    .collect();
                let json_items = serde_json::to_string(&items).unwrap();
                let map_json_file = get_data_dir(Some(MAP_DIR.into())).unwrap().join(MAP_JSON);
                fs::write(map_json_file, json_items).unwrap();
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct CurTilePos(pub Option<TilePos>);

#[derive(Resource, Default)]
pub struct LastTilePos(pub Option<TilePos>);

fn update_cur_tile_pos(
    cursor_pos: Res<CursorPos>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapTileSize,
        &TilemapType,
        &Transform,
        &TilemapAnchor,
    )>,
    mut cur_tile_pos: ResMut<CurTilePos>,
    mut last_tile_pos: ResMut<LastTilePos>,
) {
    last_tile_pos.0 = cur_tile_pos.0;
    for (map_size, grid_size, tile_size, map_type, map_transform, anchor) in tilemap_q.iter() {
        // Grab the cursor position from the `Res<CursorPos>`
        let cursor_pos: Vec2 = cursor_pos.0;
        // We need to make sure that the cursor's world position is correct relative to the map
        // due to any map transformation.
        let cursor_in_map_pos: Vec2 = {
            // Extend the cursor_pos vec3 by 0.0 and 1.0
            let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.xy()
        };
        // Once we have a world position we can transform it into a possible tile position.
        cur_tile_pos.0 = TilePos::from_world_pos(
            &cursor_in_map_pos,
            map_size,
            grid_size,
            tile_size,
            map_type,
            anchor,
        );
    }
}

/// TODO: Move this to its own mod
#[derive(Resource)]
pub struct CursorPos(pub Vec2);

impl Default for CursorPos {
    fn default() -> Self {
        // Initialize the cursor pos at some far away place. It will get updated
        // correctly when the cursor moves.
        Self(Vec2::new(-1000.0, -1000.0))
    }
}

// We need to keep the cursor position updated based on any `CursorMoved` events.
pub fn update_cursor_pos(
    camera_q: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    for cursor_moved in cursor_moved_events.read() {
        // To get the mouse's world position, we have to transform its window position by
        // any transforms on the camera. This is done by projecting the cursor position into
        // camera space (world space).
        for (cam_t, cam) in camera_q.iter() {
            if let Ok(pos) = cam.viewport_to_world_2d(cam_t, cursor_moved.position) {
                *cursor_pos = CursorPos(pos);
            }
        }
    }
}

fn interact_with_tile(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    cur_tile_pos: Res<CurTilePos>,
    tilemap_q: Query<&TileStorage>,
    popup_q: Query<&Node, With<PopupBase>>,
) {
    let popup_is_visible = popup_q
        .iter()
        .any(|node| !matches!(node.display, Display::None));
    if popup_is_visible {
        return;
    }

    if let Some(tile_pos) = cur_tile_pos.0 {
        if let Ok(tile_storage) = tilemap_q.single() {
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                if mouse_button_input.just_pressed(MouseButton::Left) {
                    info!("MY TILE: {tile_entity} {} {}", tile_pos.x, tile_pos.y);
                }
            }
        }
    }
}
