use crate::{
    bdk_zone::{
        get_data_dir, launch_bitcoind_process, load_descriptor, mine_blocks, read_cookie_auth,
        wait_for_rpc_ready, xpriv_key_from_abandon, xpriv_to_descriptor,
    },
    constants::BITCOIN_DIR,
    electrum_wallet::{
        ElectrsProcess, PlayerWallet, TouristWallet, activate_wallet, spawn_electrs,
    },
};
use bevy::prelude::*;
use std::{
    path::PathBuf,
    process::{Child, Stdio},
};

pub struct BitcoindHandler;

impl Plugin for BitcoindHandler {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, insert_bitcoind);
    }
}

#[derive(Resource)]
struct BitcoindProcess {
    child: Child,
    data_dir: PathBuf,
    conf_path: PathBuf,
}

impl Drop for BitcoindProcess {
    fn drop(&mut self) {
        match self.child.try_wait() {
            Ok(Some(status)) => {
                log_or_print(
                    &format!("Bitcoind already exited with status: {status}"),
                    log::Level::Info,
                );
            }
            Ok(None) => {
                info!("Bitcoind still running; trying `bitcoin-cli -conf=... stop`");

                let cli_result = std::process::Command::new("bitcoin-cli")
                    .arg(format!("-datadir={}", self.data_dir.display()))
                    .arg(format!("-conf={}", self.conf_path.display()))
                    .arg("stop")
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status();

                match cli_result {
                    Ok(status) if status.success() => {
                        info!("Sent `bitcoin-cli stop`; waiting for process to exit...");

                        for _ in 0..10 {
                            std::thread::sleep(std::time::Duration::from_secs(1));
                            if let Ok(Some(status)) = self.child.try_wait() {
                                log_or_print(
                                    &format!("Bitcoind exited gracefully with status: {}", status),
                                    log::Level::Info,
                                );
                                return;
                            }
                        }

                        warn!("Bitcoind did not exit in time; force killing...");
                    }
                    Ok(status) => {
                        error!("`bitcoin-cli stop` exited with status code: {}", status);
                    }
                    Err(e) => {
                        error!("Failed to run `bitcoin-cli stop`: {}", e);
                    }
                }

                // Fallback to hard kill
                if let Err(e) = self.child.kill() {
                    error!("Failed to force kill bitcoind: {}", e);
                    return;
                }

                if let Ok(status) = self.child.wait() {
                    log_or_print(
                        &format!("Bitcoind exited after kill with status: {}", status),
                        log::Level::Info,
                    );
                }
            }
            Err(e) => {
                error!("Failed to check bitcoind status: {}", e);
            }
        }
    }
}

pub fn log_or_print(msg: &str, level: log::Level) {
    if log::log_enabled!(level) {
        log::log!(level, "{}", msg)
    } else {
        println!("{}", msg);
    }
}

fn insert_bitcoind(mut commands: Commands) {
    let maybe_child_etal = launch_bitcoind_process();
    let xpriv = xpriv_key_from_abandon().unwrap();
    let descriptor = xpriv_to_descriptor(xpriv);
    info!("LETS TRY insert bitcoind: {}", descriptor);
    if let Ok((child, data_dir, conf_path)) = maybe_child_etal {
        load_descriptor("SOME DESCRIPTOR").expect("LOADED!");
        let bitcoind_process = BitcoindProcess {
            child,
            data_dir,
            conf_path,
        };
        commands.insert_resource(bitcoind_process);
    } else {
        warn!("Could not insert the BitcoindProcess Resource")
    }

    let datadir = get_data_dir(Some(BITCOIN_DIR.into())).expect("A datadir");
    let (user, pass) = read_cookie_auth(&datadir).expect("A user/pass");
    let block_count = wait_for_rpc_ready(&user, &pass).expect("bitcoind is ready");
    if block_count < 50 {
        mine_blocks(
            101,
            "bcrt1pkar3gerekw8f9gef9vn9xz0qypytgacp9wa5saelpksdgct33qdqan7c89",
        )
        .expect("Blocks");
    }

    let (child, _, _) = spawn_electrs().expect("Need to have electrs installed on your machine");
    commands.insert_resource(ElectrsProcess { child });

    let (tourist_wallet, player_wallet) = activate_wallet();

    commands.spawn(TouristWallet {
        wallet: tourist_wallet,
    });

    commands.spawn(PlayerWallet {
        wallet: player_wallet,
    });
}
