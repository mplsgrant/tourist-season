use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use bevy_ecs_tilemap::prelude::*;

pub struct TileMapTest;

impl Plugin for TileMapTest {
    fn build(&self, app: &mut App) {
        app.add_plugins(
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
        .add_plugins(TilemapPlugin)
        .add_plugins(tiled_thing::TiledMapPlugin)
        .add_systems(Startup, startup_tmx)
        .add_systems(First, (camera::movement, update_cursor_pos).chain())
        .add_systems(Update, interact_with_tile);
        //.add_systems(Startup, startup_original_tiles)
        // .add_systems(Update, update_map);
    }
}

fn startup_tmx(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let map_handle = tiled_thing::TiledMapHandle(asset_server.load("tiled_map_example/map.tmx"));

    commands.spawn(tiled_thing::TiledMapBundle {
        tiled_map: map_handle,
        ..Default::default()
    });
}

#[derive(Component)]
struct CurrentColor(u16);

#[derive(Component)]
struct LastUpdate(f64);

fn startup_original_tiles(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

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

    // Spawn a 32 by 32 tilemap.
    // Alternatively, you can use helpers::fill_tilemap.
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    ..Default::default()
                })
                .id();
            // Here we let the tile storage component know what tiles we have.
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    // We can grab a list of neighbors.
    let neighbor_positions =
        Neighbors::get_square_neighboring_positions(&TilePos { x: 0, y: 0 }, &map_size, true);
    let neighbor_entities = neighbor_positions.entities(&tile_storage);

    // We can access tiles using:
    assert!(tile_storage.get(&TilePos { x: 0, y: 0 }).is_some());
    assert_eq!(neighbor_entities.iter().count(), 3); // Only 3 neighbors since negative is outside of map.

    // This changes some of our tiles by looking at neighbors.
    let mut color = 0;
    for x in (2..128).step_by(4) {
        color += 1;
        for y in (2..128).step_by(4) {
            // Grabbing neighbors is easy.

            let neighbors =
                Neighbors::get_square_neighboring_positions(&TilePos { x, y }, &map_size, true);
            for pos in neighbors.iter() {
                // We can replace the tile texture component like so:
                commands
                    .entity(tile_storage.get(pos).unwrap())
                    .insert(TileTextureIndex(color));
            }
        }
    }

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
            texture: TilemapTexture::Single(texture_handle),
            tile_size,
            transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
            ..Default::default()
        },
        LastUpdate(0.0),
        CurrentColor(1),
    ));
}

fn startup_rpg_urban_pack_tiles(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Kenney Game Assets All-in-1 2.9.0/2D assets/RPG Urban Pack/Tilemap
    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

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

    // Spawn a 32 by 32 tilemap.
    // Alternatively, you can use helpers::fill_tilemap.
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    ..Default::default()
                })
                .id();
            // Here we let the tile storage component know what tiles we have.
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    // We can grab a list of neighbors.
    let neighbor_positions =
        Neighbors::get_square_neighboring_positions(&TilePos { x: 0, y: 0 }, &map_size, true);
    let neighbor_entities = neighbor_positions.entities(&tile_storage);

    // We can access tiles using:
    assert!(tile_storage.get(&TilePos { x: 0, y: 0 }).is_some());
    assert_eq!(neighbor_entities.iter().count(), 3); // Only 3 neighbors since negative is outside of map.

    // This changes some of our tiles by looking at neighbors.
    let mut color = 0;
    for x in (2..128).step_by(4) {
        color += 1;
        for y in (2..128).step_by(4) {
            // Grabbing neighbors is easy.

            let neighbors =
                Neighbors::get_square_neighboring_positions(&TilePos { x, y }, &map_size, true);
            for pos in neighbors.iter() {
                // We can replace the tile texture component like so:
                commands
                    .entity(tile_storage.get(pos).unwrap())
                    .insert(TileTextureIndex(color));
            }
        }
    }

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
            texture: TilemapTexture::Single(texture_handle),
            tile_size,
            transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
            ..Default::default()
        },
        LastUpdate(0.0),
        CurrentColor(1),
    ));
}

#[derive(Resource)]
pub struct CursorPos(Vec2);

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
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    cursor_pos: Res<CursorPos>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &Transform,
    )>,
    mut text_q: Query<&mut TextColor>,
) {
    for (map_size, grid_size, map_type, tile_storage, map_transform) in tilemap_q.iter() {
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
        if let Some(tile_pos) =
            TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type)
        {
            // Highlight the relevant tile's label
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                if mouse_button_input.just_pressed(MouseButton::Left) {
                    info!(
                        "MY TILE: {tile_entity} {cursor_pos} {cursor_in_map_pos} {} {}",
                        tile_pos.x, tile_pos.y
                    );
                }
            }
        }
    }
}

// A system that manipulates tile colors.
fn update_map(
    time: Res<Time>,
    mut tilemap_query: Query<(
        &mut CurrentColor,
        &mut LastUpdate,
        &TileStorage,
        &TilemapSize,
    )>,
    mut tile_query: Query<&mut TileTextureIndex>,
) {
    let current_time = time.elapsed_secs_f64();
    for (mut current_color, mut last_update, tile_storage, map_size) in tilemap_query.iter_mut() {
        if current_time - last_update.0 > 0.1 {
            current_color.0 += 1;
            if current_color.0 > 5 {
                current_color.0 = 1;
            }

            let mut color = current_color.0;

            for x in (2..128).step_by(4) {
                for y in (2..128).step_by(4) {
                    // Grab the neighboring tiles
                    let neighboring_entities = Neighbors::get_square_neighboring_positions(
                        &TilePos { x, y },
                        map_size,
                        true,
                    )
                    .entities(tile_storage);

                    // Iterate over neighbors
                    for neighbor_entity in neighboring_entities.iter() {
                        // Query the tile entities to change the colors
                        if let Ok(mut tile_texture) = tile_query.get_mut(*neighbor_entity) {
                            tile_texture.0 = color as u32;
                        }
                    }
                }
                color += 1;
                if color > 5 {
                    color = 1;
                }
            }
            last_update.0 = current_time;
        }
    }
}

mod camera {
    use bevy::{input::ButtonInput, math::Vec3, prelude::*, render::camera::Camera};

    // A simple camera system for moving and zooming the camera.
    #[allow(dead_code)]
    pub fn movement(
        time: Res<Time>,
        keyboard_input: Res<ButtonInput<KeyCode>>,
        mut query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
    ) {
        for (mut transform, mut ortho) in query.iter_mut() {
            let mut direction = Vec3::ZERO;

            if keyboard_input.pressed(KeyCode::KeyA) {
                direction -= Vec3::new(1.0, 0.0, 0.0);
            }

            if keyboard_input.pressed(KeyCode::KeyD) {
                direction += Vec3::new(1.0, 0.0, 0.0);
            }

            if keyboard_input.pressed(KeyCode::KeyW) {
                direction += Vec3::new(0.0, 1.0, 0.0);
            }

            if keyboard_input.pressed(KeyCode::KeyS) {
                direction -= Vec3::new(0.0, 1.0, 0.0);
            }

            if keyboard_input.pressed(KeyCode::KeyZ) {
                ortho.scale += 0.1;
            }

            if keyboard_input.pressed(KeyCode::KeyX) {
                ortho.scale -= 0.1;
            }

            if ortho.scale < 0.5 {
                ortho.scale = 0.5;
            }

            let z = transform.translation.z;
            transform.translation += time.delta_secs() * direction * 500.;
            // Important! We need to restore the Z values when moving the camera around.
            // Bevy has a specific camera setup and this can mess with how our layers are shown.
            transform.translation.z = z;
        }
    }
}

mod ldtk_thing {
    use bevy_ecs_tilemap::{
        TilemapBundle,
        helpers::geometry::get_tilemap_center_transform,
        map::{TilemapId, TilemapSize, TilemapTexture, TilemapTileSize},
        tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex},
    };
    use std::{collections::HashMap, io::ErrorKind};
    use thiserror::Error;

    use bevy::{asset::io::Reader, reflect::TypePath};
    use bevy::{
        asset::{AssetLoader, AssetPath, LoadContext},
        prelude::*,
    };
    use bevy_ecs_tilemap::map::TilemapType;

    #[derive(Default)]
    pub struct LdtkPlugin;

    impl Plugin for LdtkPlugin {
        fn build(&self, app: &mut App) {
            app.init_asset::<LdtkMap>()
                .register_asset_loader(LdtkLoader)
                .add_systems(Update, process_loaded_tile_maps);
        }
    }

    #[derive(TypePath, Asset)]
    pub struct LdtkMap {
        pub project: ldtk_rust::Project,
        pub tilesets: HashMap<i64, Handle<Image>>,
    }

    #[derive(Default, Component)]
    pub struct LdtkMapConfig {
        pub selected_level: usize,
    }

    #[derive(Default, Component)]
    pub struct LdtkMapHandle(pub Handle<LdtkMap>);

    #[derive(Default, Bundle)]
    pub struct LdtkMapBundle {
        pub ldtk_map: LdtkMapHandle,
        pub ldtk_map_config: LdtkMapConfig,
        pub transform: Transform,
        pub global_transform: GlobalTransform,
    }

    pub struct LdtkLoader;

    #[derive(Debug, Error)]
    pub enum LdtkAssetLoaderError {
        /// An [IO](std::io) Error
        #[error("Could not load LDTk file: {0}")]
        Io(#[from] std::io::Error),
    }

    impl AssetLoader for LdtkLoader {
        type Asset = LdtkMap;
        type Settings = ();
        type Error = LdtkAssetLoaderError;

        async fn load(
            &self,
            reader: &mut dyn Reader,
            _settings: &Self::Settings,
            load_context: &mut LoadContext<'_>,
        ) -> Result<Self::Asset, Self::Error> {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let project: ldtk_rust::Project = serde_json::from_slice(&bytes).map_err(|e| {
                std::io::Error::new(
                    ErrorKind::Other,
                    format!("Could not read contents of Ldtk map: {e}"),
                )
            })?;
            let dependencies: Vec<(i64, AssetPath)> = project
                .defs
                .tilesets
                .iter()
                .filter_map(|tileset| {
                    tileset.rel_path.as_ref().map(|rel_path| {
                        (
                            tileset.uid,
                            load_context.path().parent().unwrap().join(rel_path).into(),
                        )
                    })
                })
                .collect();

            let ldtk_map = LdtkMap {
                project,
                tilesets: dependencies
                    .iter()
                    .map(|dep| (dep.0, load_context.load(dep.1.clone())))
                    .collect(),
            };
            Ok(ldtk_map)
        }

        fn extensions(&self) -> &[&str] {
            static EXTENSIONS: &[&str] = &["ldtk"];
            EXTENSIONS
        }
    }

    pub fn process_loaded_tile_maps(
        mut commands: Commands,
        mut map_events: EventReader<AssetEvent<LdtkMap>>,
        maps: Res<Assets<LdtkMap>>,
        mut query: Query<(Entity, &LdtkMapHandle, &LdtkMapConfig)>,
        new_maps: Query<&LdtkMapHandle, Added<LdtkMapHandle>>,
    ) {
        let mut changed_maps = Vec::<AssetId<LdtkMap>>::default();
        for event in map_events.read() {
            match event {
                AssetEvent::Added { id } => {
                    log::info!("Map added!");
                    changed_maps.push(*id);
                }
                AssetEvent::Modified { id } => {
                    log::info!("Map changed!");
                    changed_maps.push(*id);
                }
                AssetEvent::Removed { id } => {
                    log::info!("Map removed!");
                    // if mesh was modified and removed in the same update, ignore the modification
                    // events are ordered so future modification events are ok
                    changed_maps.retain(|changed_handle| changed_handle == id);
                }
                _ => continue,
            }
        }

        // If we have new map entities, add them to the changed_maps list
        for new_map_handle in new_maps.iter() {
            changed_maps.push(new_map_handle.0.id());
        }

        for changed_map in changed_maps.iter() {
            for (entity, map_handle, map_config) in query.iter_mut() {
                // only deal with currently changed map
                if map_handle.0.id() != *changed_map {
                    continue;
                }
                if let Some(ldtk_map) = maps.get(&map_handle.0) {
                    // Despawn all existing tilemaps for this LdtkMap
                    commands.entity(entity).despawn_descendants();

                    // Pull out tilesets and their definitions into a new hashmap
                    let mut tilesets = HashMap::new();
                    ldtk_map.project.defs.tilesets.iter().for_each(|tileset| {
                        tilesets.insert(
                            tileset.uid,
                            (
                                ldtk_map.tilesets.get(&tileset.uid).unwrap().clone(),
                                tileset,
                            ),
                        );
                    });

                    let default_grid_size = ldtk_map.project.default_grid_size;
                    let level = &ldtk_map.project.levels[map_config.selected_level];

                    let map_tile_count_x = (level.px_wid / default_grid_size) as u32;
                    let map_tile_count_y = (level.px_hei / default_grid_size) as u32;

                    let size = TilemapSize {
                        x: map_tile_count_x,
                        y: map_tile_count_y,
                    };

                    // We will create a tilemap for each layer in the following loop
                    for (layer_id, layer) in level
                        .layer_instances
                        .as_ref()
                        .unwrap()
                        .iter()
                        .rev()
                        .enumerate()
                    {
                        if let Some(uid) = layer.tileset_def_uid {
                            let (texture, tileset) = tilesets.get(&uid).unwrap().clone();

                            // Tileset-specific tilemap settings
                            let tile_size = TilemapTileSize {
                                x: tileset.tile_grid_size as f32,
                                y: tileset.tile_grid_size as f32,
                            };

                            // Pre-emptively create a map entity for tile creation
                            let map_entity = commands.spawn_empty().id();

                            // Create tiles for this layer from LDtk's grid_tiles and auto_layer_tiles
                            let mut storage = TileStorage::empty(size);

                            for tile in layer.grid_tiles.iter().chain(layer.auto_layer_tiles.iter())
                            {
                                let mut position = TilePos {
                                    x: (tile.px[0] / default_grid_size) as u32,
                                    y: (tile.px[1] / default_grid_size) as u32,
                                };

                                position.y = map_tile_count_y - position.y - 1;

                                let tile_entity = commands
                                    .spawn(TileBundle {
                                        position,
                                        tilemap_id: TilemapId(map_entity),
                                        texture_index: TileTextureIndex(tile.t as u32),
                                        ..default()
                                    })
                                    .id();

                                storage.set(&position, tile_entity);
                            }

                            let grid_size = tile_size.into();
                            let map_type = TilemapType::default();

                            // Create the tilemap
                            commands.entity(map_entity).insert(TilemapBundle {
                                grid_size,
                                map_type,
                                size,
                                storage,
                                texture: TilemapTexture::Single(texture),
                                tile_size,
                                transform: get_tilemap_center_transform(
                                    &size,
                                    &grid_size,
                                    &map_type,
                                    layer_id as f32,
                                ),
                                ..default()
                            });
                        }
                    }
                }
            }
        }
    }
}

mod tiled_thing {
    // How to use this:
    //   You should copy/paste this into your project and use it much like examples/tiles.rs uses this
    //   file. When you do so you will need to adjust the code based on whether you're using the
    //   'atlas` feature in bevy_ecs_tilemap. The bevy_ecs_tilemap uses this as an example of how to
    //   use both single image tilesets and image collection tilesets. Since your project won't have
    //   the 'atlas' feature defined in your Cargo config, the expressions prefixed by the #[cfg(...)]
    //   macro will not compile in your project as-is. If your project depends on the bevy_ecs_tilemap
    //   'atlas' feature then move all of the expressions prefixed by #[cfg(not(feature = "atlas"))].
    //   Otherwise remove all of the expressions prefixed by #[cfg(feature = "atlas")].
    //
    // Functional limitations:
    //   * When the 'atlas' feature is enabled tilesets using a collection of images will be skipped.
    //   * Only finite tile layers are loaded. Infinite tile layers and object layers will be skipped.

    use std::io::{Cursor, ErrorKind};
    use std::path::Path;
    use std::sync::Arc;

    use bevy::{
        asset::{AssetLoader, AssetPath, io::Reader},
        log,
        prelude::{
            Added, Asset, AssetApp, AssetEvent, AssetId, Assets, Bundle, Commands, Component,
            DespawnRecursiveExt, Entity, EventReader, GlobalTransform, Handle, Image, Plugin,
            Query, Res, Transform, Update,
        },
        reflect::TypePath,
        utils::HashMap,
    };
    use bevy_ecs_tilemap::prelude::*;

    use thiserror::Error;

    #[derive(Default)]
    pub struct TiledMapPlugin;

    impl Plugin for TiledMapPlugin {
        fn build(&self, app: &mut bevy::prelude::App) {
            app.init_asset::<TiledMap>()
                .register_asset_loader(TiledLoader)
                .add_systems(Update, process_loaded_maps);
        }
    }

    #[derive(TypePath, Asset)]
    pub struct TiledMap {
        pub map: tiled::Map,

        pub tilemap_textures: HashMap<usize, TilemapTexture>,

        // The offset into the tileset_images for each tile id within each tileset.
        #[cfg(not(feature = "atlas"))]
        pub tile_image_offsets: HashMap<(usize, tiled::TileId), u32>,
    }

    // Stores a list of tiled layers.
    #[derive(Component, Default)]
    pub struct TiledLayersStorage {
        pub storage: HashMap<u32, Entity>,
    }

    #[derive(Component, Default)]
    pub struct TiledMapHandle(pub Handle<TiledMap>);

    #[derive(Default, Bundle)]
    pub struct TiledMapBundle {
        pub tiled_map: TiledMapHandle,
        pub storage: TiledLayersStorage,
        pub transform: Transform,
        pub global_transform: GlobalTransform,
        pub render_settings: TilemapRenderSettings,
    }

    struct BytesResourceReader {
        bytes: Arc<[u8]>,
    }

    impl BytesResourceReader {
        fn new(bytes: &[u8]) -> Self {
            Self {
                bytes: Arc::from(bytes),
            }
        }
    }

    impl tiled::ResourceReader for BytesResourceReader {
        type Resource = Cursor<Arc<[u8]>>;
        type Error = std::io::Error;

        fn read_from(&mut self, _path: &Path) -> std::result::Result<Self::Resource, Self::Error> {
            // In this case, the path is ignored because the byte data is already provided.
            Ok(Cursor::new(self.bytes.clone()))
        }
    }

    pub struct TiledLoader;

    #[derive(Debug, Error)]
    pub enum TiledAssetLoaderError {
        /// An [IO](std::io) Error
        #[error("Could not load Tiled file: {0}")]
        Io(#[from] std::io::Error),
    }

    impl AssetLoader for TiledLoader {
        type Asset = TiledMap;
        type Settings = ();
        type Error = TiledAssetLoaderError;

        async fn load(
            &self,
            reader: &mut dyn Reader,
            _settings: &Self::Settings,
            load_context: &mut bevy::asset::LoadContext<'_>,
        ) -> Result<Self::Asset, Self::Error> {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let mut loader = tiled::Loader::with_cache_and_reader(
                tiled::DefaultResourceCache::new(),
                BytesResourceReader::new(&bytes),
            );
            let map = loader.load_tmx_map(load_context.path()).map_err(|e| {
                std::io::Error::new(ErrorKind::Other, format!("Could not load TMX map: {e}"))
            })?;

            let mut tilemap_textures = HashMap::default();
            #[cfg(not(feature = "atlas"))]
            let mut tile_image_offsets = HashMap::default();

            for (tileset_index, tileset) in map.tilesets().iter().enumerate() {
                let tilemap_texture = match &tileset.image {
                    None => {
                        #[cfg(feature = "atlas")]
                        {
                            log::info!(
                                "Skipping image collection tileset '{}' which is incompatible with atlas feature",
                                tileset.name
                            );
                            continue;
                        }

                        #[cfg(not(feature = "atlas"))]
                        {
                            let mut tile_images: Vec<Handle<Image>> = Vec::new();
                            for (tile_id, tile) in tileset.tiles() {
                                if let Some(img) = &tile.image {
                                    // The load context path is the TMX file itself. If the file is at the root of the
                                    // assets/ directory structure then the tmx_dir will be empty, which is fine.
                                    let tmx_dir = load_context
                                        .path()
                                        .parent()
                                        .expect("The asset load context was empty.");
                                    let tile_path = tmx_dir.join(&img.source);
                                    let asset_path = AssetPath::from(tile_path);
                                    log::info!(
                                        "Loading tile image from {asset_path:?} as image ({tileset_index}, {tile_id})"
                                    );
                                    let texture: Handle<Image> =
                                        load_context.load(asset_path.clone());
                                    tile_image_offsets
                                        .insert((tileset_index, tile_id), tile_images.len() as u32);
                                    tile_images.push(texture.clone());
                                }
                            }

                            TilemapTexture::Vector(tile_images)
                        }
                    }
                    Some(img) => {
                        // The load context path is the TMX file itself. If the file is at the root of the
                        // assets/ directory structure then the tmx_dir will be empty, which is fine.
                        let tmx_dir = load_context
                            .path()
                            .parent()
                            .expect("The asset load context was empty.");
                        let tile_path = tmx_dir.join(&img.source);
                        let asset_path = AssetPath::from(tile_path);
                        let texture: Handle<Image> = load_context.load(asset_path.clone());

                        TilemapTexture::Single(texture.clone())
                    }
                };

                tilemap_textures.insert(tileset_index, tilemap_texture);
            }

            let asset_map = TiledMap {
                map,
                tilemap_textures,
                #[cfg(not(feature = "atlas"))]
                tile_image_offsets,
            };

            log::info!("Loaded map: {}", load_context.path().display());
            Ok(asset_map)
        }

        fn extensions(&self) -> &[&str] {
            static EXTENSIONS: &[&str] = &["tmx"];
            EXTENSIONS
        }
    }

    pub fn process_loaded_maps(
        mut commands: Commands,
        mut map_events: EventReader<AssetEvent<TiledMap>>,
        maps: Res<Assets<TiledMap>>,
        tile_storage_query: Query<(Entity, &TileStorage)>,
        mut map_query: Query<(
            &TiledMapHandle,
            &mut TiledLayersStorage,
            &TilemapRenderSettings,
        )>,
        new_maps: Query<&TiledMapHandle, Added<TiledMapHandle>>,
    ) {
        let mut changed_maps = Vec::<AssetId<TiledMap>>::default();
        for event in map_events.read() {
            match event {
                AssetEvent::Added { id } => {
                    log::info!("Map added!");
                    changed_maps.push(*id);
                }
                AssetEvent::Modified { id } => {
                    log::info!("Map changed!");
                    changed_maps.push(*id);
                }
                AssetEvent::Removed { id } => {
                    log::info!("Map removed!");
                    // if mesh was modified and removed in the same update, ignore the modification
                    // events are ordered so future modification events are ok
                    changed_maps.retain(|changed_handle| changed_handle == id);
                }
                _ => continue,
            }
        }

        // If we have new map entities add them to the changed_maps list.
        for new_map_handle in new_maps.iter() {
            changed_maps.push(new_map_handle.0.id());
        }

        for changed_map in changed_maps.iter() {
            for (map_handle, mut layer_storage, render_settings) in map_query.iter_mut() {
                // only deal with currently changed map
                if map_handle.0.id() != *changed_map {
                    continue;
                }
                if let Some(tiled_map) = maps.get(&map_handle.0) {
                    // TODO: Create a RemoveMap component..
                    for layer_entity in layer_storage.storage.values() {
                        if let Ok((_, layer_tile_storage)) = tile_storage_query.get(*layer_entity) {
                            for tile in layer_tile_storage.iter().flatten() {
                                commands.entity(*tile).despawn_recursive()
                            }
                        }
                        // commands.entity(*layer_entity).despawn_recursive();
                    }

                    // The TilemapBundle requires that all tile images come exclusively from a single
                    // tiled texture or from a Vec of independent per-tile images. Furthermore, all of
                    // the per-tile images must be the same size. Since Tiled allows tiles of mixed
                    // tilesets on each layer and allows differently-sized tile images in each tileset,
                    // this means we need to load each combination of tileset and layer separately.
                    for (tileset_index, tileset) in tiled_map.map.tilesets().iter().enumerate() {
                        let Some(tilemap_texture) = tiled_map.tilemap_textures.get(&tileset_index)
                        else {
                            log::warn!("Skipped creating layer with missing tilemap textures.");
                            continue;
                        };

                        let tile_size = TilemapTileSize {
                            x: tileset.tile_width as f32,
                            y: tileset.tile_height as f32,
                        };

                        let tile_spacing = TilemapSpacing {
                            x: tileset.spacing as f32,
                            y: tileset.spacing as f32,
                        };

                        // Once materials have been created/added we need to then create the layers.
                        for (layer_index, layer) in tiled_map.map.layers().enumerate() {
                            let offset_x = layer.offset_x;
                            let offset_y = layer.offset_y;

                            let tiled::LayerType::Tiles(tile_layer) = layer.layer_type() else {
                                log::info!(
                                    "Skipping layer {} because only tile layers are supported.",
                                    layer.id()
                                );
                                continue;
                            };

                            let tiled::TileLayer::Finite(layer_data) = tile_layer else {
                                log::info!(
                                    "Skipping layer {} because only finite layers are supported.",
                                    layer.id()
                                );
                                continue;
                            };

                            let map_size = TilemapSize {
                                x: tiled_map.map.width,
                                y: tiled_map.map.height,
                            };

                            let grid_size = TilemapGridSize {
                                x: tiled_map.map.tile_width as f32,
                                y: tiled_map.map.tile_height as f32,
                            };

                            let map_type = match tiled_map.map.orientation {
                                tiled::Orientation::Hexagonal => {
                                    TilemapType::Hexagon(HexCoordSystem::Row)
                                }
                                tiled::Orientation::Isometric => {
                                    TilemapType::Isometric(IsoCoordSystem::Diamond)
                                }
                                tiled::Orientation::Staggered => {
                                    TilemapType::Isometric(IsoCoordSystem::Staggered)
                                }
                                tiled::Orientation::Orthogonal => TilemapType::Square,
                            };

                            let mut tile_storage = TileStorage::empty(map_size);
                            let layer_entity = commands.spawn_empty().id();

                            for x in 0..map_size.x {
                                for y in 0..map_size.y {
                                    // Transform TMX coords into bevy coords.
                                    let mapped_y = tiled_map.map.height - 1 - y;

                                    let mapped_x = x as i32;
                                    let mapped_y = mapped_y as i32;

                                    let layer_tile = match layer_data.get_tile(mapped_x, mapped_y) {
                                        Some(t) => t,
                                        None => {
                                            continue;
                                        }
                                    };
                                    if tileset_index != layer_tile.tileset_index() {
                                        continue;
                                    }
                                    let layer_tile_data =
                                        match layer_data.get_tile_data(mapped_x, mapped_y) {
                                            Some(d) => d,
                                            None => {
                                                continue;
                                            }
                                        };

                                    let texture_index = match tilemap_texture {
                                    TilemapTexture::Single(_) => layer_tile.id(),
                                    #[cfg(not(feature = "atlas"))]
                                    TilemapTexture::Vector(_) =>
                                        *tiled_map.tile_image_offsets.get(&(tileset_index, layer_tile.id()))
                                        .expect("The offset into to image vector should have been saved during the initial load."),
                                    #[cfg(not(feature = "atlas"))]
                                    _ => unreachable!()
                                };

                                    let tile_pos = TilePos { x, y };
                                    let tile_entity = commands
                                        .spawn(TileBundle {
                                            position: tile_pos,
                                            tilemap_id: TilemapId(layer_entity),
                                            texture_index: TileTextureIndex(texture_index),
                                            flip: TileFlip {
                                                x: layer_tile_data.flip_h,
                                                y: layer_tile_data.flip_v,
                                                d: layer_tile_data.flip_d,
                                            },
                                            ..Default::default()
                                        })
                                        .id();
                                    tile_storage.set(&tile_pos, tile_entity);
                                }
                            }

                            commands.entity(layer_entity).insert(TilemapBundle {
                                grid_size,
                                size: map_size,
                                storage: tile_storage,
                                texture: tilemap_texture.clone(),
                                tile_size,
                                spacing: tile_spacing,
                                transform: get_tilemap_center_transform(
                                    &map_size,
                                    &grid_size,
                                    &map_type,
                                    layer_index as f32,
                                ) * Transform::from_xyz(offset_x, -offset_y, 0.0),
                                map_type,
                                render_settings: *render_settings,
                                ..Default::default()
                            });

                            layer_storage
                                .storage
                                .insert(layer_index as u32, layer_entity);
                        }
                    }
                }
            }
        }
    }
}
