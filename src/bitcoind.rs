use crate::bdk_zone::launch_bitcoind_process;
use bevy::{prelude::*, utils::OnDrop};
use std::{io::ErrorKind, process::Child};

pub struct BitcoindHandler;

impl Plugin for BitcoindHandler {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, insert_bitcoind);
    }
}

#[derive(Resource)]
struct BitcoindProcess {
    child: Child,
}

impl Drop for BitcoindProcess {
    fn drop(&mut self) {
        match self.child.try_wait() {
            Ok(Some(status)) => {
                log_or_print(
                    &format!("Bitcoind already exited with status: {}", status),
                    log::Level::Info,
                );
            }
            Ok(None) => {
                info!(")Bitcoind still running; sending kill signal...");
                match self.child.kill() {
                    Ok(()) => (),
                    Err(e) if e.kind() == ErrorKind::InvalidInput => {
                        log_or_print(
                            "Bitcoind process already terminated before kill.",
                            log::Level::Info,
                        );
                    }
                    Err(e) => {
                        error!("Failed to kill bitcoind: {}", e);
                        return;
                    }
                }

                match self.child.wait() {
                    Ok(status) => log_or_print(
                        &format!("Bitcoind exited after kill with status: {}", status),
                        log::Level::Info,
                    ),
                    Err(e) => log_or_print(
                        &format!("Failed to wait for bitcoind after kill: {}", e),
                        log::Level::Warn,
                    ),
                }
            }
            Err(e) => {
                log_or_print(
                    &format!("Failed to query bitcoind process status: {}", e),
                    log::Level::Error,
                );
            }
        }
    }
}

fn log_or_print(msg: &str, level: log::Level) {
    if log::log_enabled!(level) {
        log::log!(level, "{}", msg)
    } else {
        println!("{}", msg);
    }
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
