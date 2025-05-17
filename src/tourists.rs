use bevy::prelude::*;

use crate::constants::ImgAsset;

pub struct Tourists;

impl Plugin for Tourists {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup)
            .add_systems(Update, (tourist_spawner, tourist_spawner));
    }
}

#[derive(Component, Deref, DerefMut)]
struct SpawnTouristTimer(Timer);

#[derive(Component)]
pub struct Tourist;

fn startup(mut commands: Commands) {
    commands.spawn(SpawnTouristTimer(Timer::from_seconds(2.0, TimerMode::Once)));
}

fn tourist_spawner(
    mut commands: Commands,
    mut query: Query<&mut SpawnTouristTimer>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
) {
    for mut timer in &mut query {
        if timer.tick(time.delta()).just_finished() {
            let _ = commands
                .spawn((
                    Sprite::from_image(
                        asset_server.load(ImgAsset::GreenTouristStandingFront.path()),
                    ),
                    Tourist,
                    Transform {
                        translation: Vec3 {
                            x: 100.0,
                            y: 100.0,
                            z: 3.0,
                        },
                        ..Default::default()
                    },
                    GlobalZIndex(6),
                ))
                .id();
        }
    }
}
