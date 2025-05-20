use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage, TileTextureIndex};
use pathfinding::{grid::Grid, prelude::astar};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    bdk_zone::mine_blocks,
    constants::{ImgAsset, WALKABLES},
    tilemaptest::{tilepos_to_transform, translation_to_tilepos, usizes_to_transform},
};

pub struct Tourists;

impl Plugin for Tourists {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentRound>()
            .add_event::<RedrawGrid>()
            .add_event::<RecalcTouristPath>()
            .add_systems(PostStartup, post_startup)
            .add_systems(
                Update,
                (
                    tourist_spawner,
                    move_tourist,
                    redraw_grid,
                    path_recalculator,
                ),
            );
    }
}

#[derive(Event)]
pub enum RedrawGrid {
    Redraw,
    MarkUnWalkable(TilePos),
    MarkWalkable(TilePos),
}

#[derive(Event)]
pub enum RecalcTouristPath {
    NewGoal((Entity, TilePos)),
}

#[derive(Component, Deref, DerefMut)]
struct SpawnTouristTimer(Timer);

#[derive(Component, Deref, DerefMut)]
struct NextRound(pub Timer);

#[derive(Resource, Default)]
struct CurrentRound(pub u32);

#[derive(Component)]
pub struct Tourist {
    status: TouristStatus,
    path: Vec<(usize, usize)>,
}

pub enum TouristStatus {
    Standing,
    Navigating,
    Walking(TilePos),
}

#[derive(Component)]
pub struct SatsToSend {
    pub sats: u64,
    pub iterations: u32,
}

#[derive(Component, Deref, DerefMut)]
pub struct TouristGrid(Grid);

#[derive(Component, Clone, Debug, Default, Serialize, Deserialize)]
pub struct TouristSpawnPoint {}

#[derive(Component, Clone, Debug, Default, Serialize, Deserialize)]
pub struct TouristDespawnPoint {}

fn post_startup(
    mut commands: Commands,
    tilemap_q: Query<&TileStorage>,
    position_q: Query<(&TilePos, &TileTextureIndex)>,
) {
    commands.spawn(SatsToSend {
        sats: 0,
        iterations: 0,
    });
    commands.spawn(SpawnTouristTimer(Timer::from_seconds(2.0, TimerMode::Once)));
    commands.spawn(NextRound(Timer::from_seconds(10.0, TimerMode::Once)));

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
        .filter(|(_, texture_idx)| !WALKABLES.iter().any(|x| x == &texture_idx.0))
        .for_each(|(tile_pos, _)| {
            grid.remove_vertex((tile_pos.x as usize, tile_pos.y as usize));
        });

    commands.spawn(TouristGrid(grid));
}

fn redraw_grid(
    mut redrawgrid_e: EventReader<RedrawGrid>,
    mut grid_q: Query<&mut TouristGrid>,
    tilemap_q: Query<&TileStorage>,
    position_q: Query<(&TilePos, &TileTextureIndex)>,
) {
    for event in redrawgrid_e.read() {
        match event {
            RedrawGrid::Redraw => {
                if let Ok(mut grid) = grid_q.single_mut() {
                    for y in 0..128 {
                        for x in 0..128 {
                            grid.add_vertex((x, y));
                        }
                    }

                    tilemap_q
                        .iter()
                        .flat_map(|tile_storage| tile_storage.iter().filter_map(|e| *e))
                        .filter_map(|entity| position_q.get(entity).ok())
                        .filter(|(_, texture_idx)| !WALKABLES.iter().any(|x| x == &texture_idx.0))
                        .for_each(|(tile_pos, _)| {
                            grid.remove_vertex((tile_pos.x as usize, tile_pos.y as usize));
                        });
                }
            }
            RedrawGrid::MarkUnWalkable(tile_pos) => {
                if let Ok(mut grid) = grid_q.single_mut() {
                    grid.remove_vertex((tile_pos.x as usize, tile_pos.y as usize));
                }
            }
            RedrawGrid::MarkWalkable(tile_pos) => {
                if let Ok(mut grid) = grid_q.single_mut() {
                    grid.add_vertex((tile_pos.x as usize, tile_pos.y as usize));
                }
            }
        }
    }
}

fn path_recalculator(
    mut commands: Commands,
    mut tourist_q: Query<(Entity, &mut Tourist, &Transform)>,
    mut recalc_er: EventReader<RecalcTouristPath>,
    grid_q: Query<&TouristGrid>,
) {
    for event in recalc_er.read() {
        match event {
            RecalcTouristPath::NewGoal((entity, tile_pos)) => {
                if let Ok((entity, mut tourist, transform)) = tourist_q.get_mut(*entity) {
                    if let Ok(grid) = grid_q.single() {
                        let start = translation_to_tilepos(&transform.translation, Vec2::default());
                        let start = (start.x as usize, start.y as usize);
                        if !grid.has_vertex(start) {
                            warn!("Start position {:?} not in grid", start);
                        }
                        if !grid.has_vertex((tile_pos.x as usize, tile_pos.y as usize)) {
                            warn!("Goal position {:?} not in grid", tile_pos);
                        }
                        if let Some(result) = my_astar(start, tile_pos, grid) {
                            tourist.path = result.0;
                        } else {
                            warn!("Could not get my_astar result");
                            warn!("Killing entity: {entity:?}");
                            commands.entity(entity).despawn();
                        };
                    } else {
                        warn!("Could not get a grid");
                    }
                } else {
                    warn!("Could not get mutable access to a tourist: {entity:?}")
                }
            }
        }
    }
}

fn my_astar(
    start: (usize, usize),
    goal_tile_pos: &TilePos,
    grid: &Grid,
) -> Option<(Vec<(usize, usize)>, u32)> {
    let goal = (goal_tile_pos.x as usize, goal_tile_pos.y as usize);
    astar(
        &start,
        |p| {
            grid.neighbours(*p).into_iter().map(|n| (n, 1)) // cost of 1 per move
        },
        |p| {
            ((p.0 as isize - goal.0 as isize).abs() + (p.1 as isize - goal.1 as isize).abs()) as u32
        }, // Manhattan distance
        |p| *p == goal,
    )
}

fn tourist_spawner(
    mut commands: Commands,
    mut spawn_tourist_timer: Query<&mut SpawnTouristTimer>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    spawnpoint_q: Query<&TilePos, With<TouristSpawnPoint>>,
    grid_q: Query<&TouristGrid>,
    despawn_pos_q: Query<&TilePos, With<TouristDespawnPoint>>,
    mut next_round_timer_q: Query<&mut NextRound>,
    mut current_round_q: ResMut<CurrentRound>,
) {
    for mut timer in &mut spawn_tourist_timer {
        if timer.tick(time.delta()).just_finished() {
            let grid = grid_q.single().expect("One tourist grid.");
            for spawnpoint_tile_pos in spawnpoint_q.iter() {
                let tourist_initial_transform =
                    tilepos_to_transform(spawnpoint_tile_pos, Vec2 { x: 25.0, y: 25.0 }, 6.0);
                let start = (
                    spawnpoint_tile_pos.x as usize,
                    spawnpoint_tile_pos.y as usize,
                );
                if let Some(goal_tile_pos) = despawn_pos_q.iter().next() {
                    let mut goal_tile_pos = *goal_tile_pos;
                    goal_tile_pos.x += 1;
                    goal_tile_pos.y += 1;
                    let maybe_first_leg = match current_round_q.0 {
                        0 => None,
                        1 => my_astar(start, &TilePos { x: 115, y: 90 }, grid).map(|x| x.0),
                        2 => my_astar(start, &TilePos { x: 90, y: 115 }, grid).map(|x| x.0),
                        3 => my_astar(start, &TilePos { x: 15, y: 32 }, grid).map(|x| x.0),
                        4 => my_astar(start, &TilePos { x: 32, y: 15 }, grid).map(|x| x.0),
                        5 => my_astar(start, &TilePos { x: 19, y: 19 }, grid).map(|x| x.0),
                        _ => {
                            let x = rand::rng().random_range(1..=127);
                            let y = rand::rng().random_range(1..=127);
                            my_astar(start, &TilePos { x, y }, grid).map(|x| x.0)
                        }
                    };
                    let maybe_path = if let Some(first_leg) = maybe_first_leg {
                        info!("First leg");
                        if let Some(last) = first_leg.last() {
                            let maybe_second_leg = my_astar(*last, &goal_tile_pos, grid);
                            if let Some(second_leg) = maybe_second_leg {
                                let second_leg = second_leg.0;
                                let second_leg = second_leg[1..].to_vec();
                                let path: Vec<(usize, usize)> =
                                    first_leg.into_iter().chain(second_leg).collect();
                                Some(path)
                            } else {
                                match my_astar(start, &goal_tile_pos, grid) {
                                    Some(path_and_cost) => Some(path_and_cost.0),
                                    None => None,
                                }
                            }
                        } else {
                            None
                        }
                    } else {
                        match my_astar(start, &goal_tile_pos, grid) {
                            Some(path_and_cost) => Some(path_and_cost.0),
                            None => None,
                        }
                    };

                    if let Some(path) = maybe_path {
                        let _ = commands
                            .spawn((
                                Sprite::from_image(
                                    asset_server.load(ImgAsset::GreenTouristStandingFront.path()),
                                ),
                                Tourist {
                                    status: TouristStatus::Standing,
                                    path: path[1..].into(), // skip current tile
                                },
                                tourist_initial_transform,
                                GlobalZIndex(6),
                            ))
                            .id();
                    } else {
                        info!("No path found!");
                    }
                }
            }
            timer.reset();
        }
    }

    for mut timer in &mut next_round_timer_q {
        if timer.0.tick(time.delta()).just_finished() {
            mine_blocks(
                8,
                "bcrt1pkar3gerekw8f9gef9vn9xz0qypytgacp9wa5saelpksdgct33qdqan7c89",
            )
            .unwrap();
            current_round_q.0 += 1;
            info!("current round: {}", current_round_q.0);
            timer.reset();
        }
    }
}

fn move_tourist(
    mut commands: Commands,
    mut tourist_q: Query<(Entity, &mut Tourist, &mut Transform, &mut Sprite)>,
    mut recalc_ew: EventWriter<RecalcTouristPath>,
    texture_q: Query<&TileTextureIndex>,
    storage_q: Query<&TileStorage>,
    time: Res<Time>,
    mut sats_to_send_q: Query<&mut SatsToSend>,
) {
    for (entity, mut tourist, mut transform, sprite) in tourist_q.iter_mut() {
        let tile_pos = translation_to_tilepos(&transform.translation, Vec2::default());
        let storage = storage_q.single().expect("One tile storage");
        let is_walkable = if let Some(tile_entity) = storage.checked_get(&tile_pos) {
            if let Ok(texture_idx) = texture_q.get(tile_entity) {
                if texture_idx.0 == ImgAsset::SidewalkSpecial.index() {
                    let mut sats_to_send = sats_to_send_q.single_mut().unwrap();
                    sats_to_send.sats += 4_000;
                    sats_to_send.iterations += 1;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        match &tourist.status {
            TouristStatus::Standing => tourist.status = TouristStatus::Navigating,
            TouristStatus::Walking(x) => {
                info!("Walking");
            }
            TouristStatus::Navigating => {
                if let Some(usizes) = tourist.path.first() {
                    if tourist.path.len() <= 1 {
                        commands.entity(entity).despawn();
                    }
                    let speed = 100.0;
                    let a_little_buffer = Vec2 { x: 8.0, y: 8.0 }; // let them walk in the middle of the tile, not on edge
                    let next_stop = usizes_to_transform(usizes, a_little_buffer, 6.0);
                    let travel_vector = next_stop.translation - transform.translation;
                    let distance = travel_vector.length();
                    if distance < 1.0 {
                        tourist.path.remove(0);
                    } else {
                        let step = travel_vector.normalize() * speed * time.delta_secs();
                        if step.length() >= distance {
                            transform.translation = next_stop.translation;
                            tourist.path.remove(0);
                        } else {
                            let next_step = transform.translation + step;
                            let tile_pos = translation_to_tilepos(&next_step, Vec2::default());
                            let storage = storage_q.single().expect("One tile storage");
                            let is_walkable =
                                if let Some(tile_entity) = storage.checked_get(&tile_pos) {
                                    if let Ok(texture_idx) = texture_q.get(tile_entity) {
                                        WALKABLES.iter().any(|x| x == &texture_idx.0)
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                };
                            if is_walkable {
                                transform.translation += step;
                            } else {
                                let goal = tourist.path.last().expect("A path");
                                let goal = TilePos {
                                    x: goal.0 as u32,
                                    y: goal.1 as u32,
                                };
                                let event = RecalcTouristPath::NewGoal((entity, goal));
                                recalc_ew.write(event);
                            }
                        }
                    }
                }
            }
        }
    }
}
