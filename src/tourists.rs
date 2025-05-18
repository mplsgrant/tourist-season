use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage, TileTextureIndex};
use pathfinding::{directed, grid::Grid, prelude::astar};
use serde::{Deserialize, Serialize};

use crate::{
    constants::{ImgAsset, WALKABLES},
    tilemaptest::{tilepos_to_transform, transform_to_tilepos},
};

pub struct Tourists;

impl Plugin for Tourists {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup)
            .add_systems(Update, tourist_spawner);
    }
}

#[derive(Component, Deref, DerefMut)]
struct SpawnTouristTimer(Timer);

#[derive(Component)]
pub struct Tourist {
    status: TouristStatus,
    path: Vec<(usize, usize)>,
}

pub enum TouristStatus {
    Standing,
    Walking,
}

#[derive(Component, Deref, DerefMut)]
pub struct TouristGrid(Grid);

#[derive(Component, Clone, Debug, Default, Serialize, Deserialize)]
pub struct TouristSpawnPoint {}

#[derive(Component, Clone, Debug, Default, Serialize, Deserialize)]
pub struct TouristDespawnPoint {}

fn startup(
    mut commands: Commands,
    tilemap_q: Query<&TileStorage>,
    position_q: Query<(&TilePos, &TileTextureIndex)>,
) {
    commands.spawn(SpawnTouristTimer(Timer::from_seconds(2.0, TimerMode::Once)));

    let mut grid = Grid::new(128, 128);
    for y in 0..128 {
        for x in 0..128 {
            grid.add_vertex((x, y));
        }
    }

    tilemap_q
        .iter()
        .flat_map(|tile_storage| tile_storage.iter().filter_map(|e| *e))
        .filter_map(|entity| position_q.get(entity).ok())
        .filter(|(_, texture_idx)| WALKABLES.iter().any(|x| x == &texture_idx.0))
        .for_each(|(tile_pos, _)| {
            grid.remove_vertex((tile_pos.x as usize, tile_pos.y as usize));
        });

    commands.spawn(TouristGrid(grid));
}

fn tourist_spawner(
    mut commands: Commands,
    mut spawn_tourist_timer: Query<&mut SpawnTouristTimer>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    spawnpoint_q: Query<&TilePos, With<TouristSpawnPoint>>,
    grid_q: Query<&TouristGrid>,
    despawn_pos_q: Query<&TilePos, With<TouristDespawnPoint>>,
) {
    for mut timer in &mut spawn_tourist_timer {
        if timer.tick(time.delta()).just_finished() {
            let grid = grid_q.single().expect("One tourist grid.");
            for spawnpoint_tile_pos in spawnpoint_q.iter() {
                info!("Spawnpoint tile pos: {:?}", spawnpoint_tile_pos);
                let tourist_initial_transform =
                    tilepos_to_transform(spawnpoint_tile_pos, Vec2 { x: 50.0, y: 50.0 }, 6.0);
                let start = (
                    spawnpoint_tile_pos.x as usize,
                    spawnpoint_tile_pos.y as usize,
                );
                if let Some(goal_tile_pos) = despawn_pos_q.iter().next() {
                    let goal = (goal_tile_pos.x as usize, goal_tile_pos.y as usize);
                    let result = astar(
                        &start,
                        |p| {
                            grid.neighbours(*p).into_iter().map(|n| (n, 1)) // cost of 1 per move
                        },
                        |p| {
                            ((p.0 as isize - goal.0 as isize).abs()
                                + (p.1 as isize - goal.1 as isize).abs())
                                as u32
                        }, // Manhattan distance
                        |p| *p == goal,
                    );

                    match result {
                        Some((path, cost)) => {
                            info!("Found path with cost {}: {:?}", cost, path);
                            let speed = 100.0;

                            let _ = commands
                                .spawn((
                                    Sprite::from_image(
                                        asset_server
                                            .load(ImgAsset::GreenTouristStandingFront.path()),
                                    ),
                                    Tourist {
                                        status: TouristStatus::Standing,
                                        path,
                                    },
                                    tourist_initial_transform,
                                    GlobalZIndex(6),
                                ))
                                .id();
                        }
                        None => {
                            info!("No path found!");
                        }
                    }
                }
            }
            timer.reset();
        }
    }
}

fn move_tourist(
    mut tourist_q: Query<(&mut Tourist, &mut Transform, &mut Sprite)>,
    time: Res<Time>,
) {
    for (mut tourist, mut transform, mut sprite) in tourist_q.iter() {
        let tourist_pos = transform_to_tilepos(transform, Vec2::default());
        let start = (tourist_pos.x as usize, tourist_pos.y as usize);
    }
}
