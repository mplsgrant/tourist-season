use crate::bdk_zone::launch_bitcoind_process;
use bevy::prelude::*;
use std::{io::ErrorKind, process::Child};

pub struct BitcoindHandler;

impl Plugin for BitcoindHandler {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, insert_bitcoind)
            .add_systems(PostUpdate, cleanup_bitcoind);
    }
}

#[derive(Resource)]
struct BitcoindProcess {
    child: Child,
}

fn insert_bitcoind(mut commands: Commands) {
    let maybe_child = launch_bitcoind_process("put descriptor here");
    if let Ok(child) = maybe_child {
        let bitcoind_process = BitcoindProcess { child };
        commands.insert_resource(bitcoind_process);
    } else {
        warn!("Could not insert the BitcoindProcess Resource")
    }
}

fn cleanup_bitcoind(
    exit_events: EventReader<AppExit>,
    mut maybe_bitcoind: Option<NonSendMut<BitcoindProcess>>,
) {
    if exit_events.is_empty() {
        return;
    }

    let Some(bitcoind) = &mut maybe_bitcoind else {
        warn!("BitcoindProcess not found during cleanup.");
        return;
    };

    match bitcoind.child.try_wait() {
        Ok(Some(status)) => {
            info!("Bitcoind already exited with status: {}", status);
        }
        Ok(None) => {
            info!("Bitcoind still running; sending kill signal...");
            match bitcoind.child.kill() {
                Ok(()) => (),
                Err(e) if e.kind() == ErrorKind::InvalidInput => {
                    info!("Bitcoind process already terminated before kill.");
                }
                Err(e) => {
                    error!("Failed to kill bitcoind: {}", e);
                    return;
                }
            }

            match bitcoind.child.wait() {
                Ok(status) => info!("Bitcoind exited after kill with status: {}", status),
                Err(e) => error!("Failed to wait for bitcoind after kill: {}", e),
            }
        }
        Err(e) => {
            error!("Failed to query bitcoind process status: {}", e);
        }
    }
}
