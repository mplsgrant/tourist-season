use std::io::BufRead;
use std::path::PathBuf;
use std::process::{Child, Stdio};
use std::str::FromStr;

use bdk_electrum::electrum_client::Client;
use bdk_electrum::{BdkElectrumClient, electrum_client};
use bdk_wallet::KeychainKind;
use bdk_wallet::Wallet;
use bdk_wallet::bitcoin::Network;
use bdk_wallet::{AddressInfo, SignOptions};
use bevy::prelude::*;
use bevy::state::commands;
use bitcoin::{Address, Amount, FeeRate};

use crate::bdk_zone::{get_config_dir, get_data_dir};
use crate::bitcoind::log_or_print;
use crate::constants::BITCOIN_DIR;
use crate::tourists::SatsToSend;

pub struct ElectrumWallet;

impl Plugin for ElectrumWallet {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup)
            .add_systems(Update, send_sats);
    }
}

const STOP_GAP: usize = 50;
const BATCH_SIZE: usize = 5;

const EXTERNAL_DESCRIPTOR: &str = "tr(tprv8ZgxMBicQKsPdrjwWCyXqqJ4YqcyG4DmKtjjsRt29v1PtD3r3PuFJAjWytzcvSTKnZAGAkPSmnrdnuHWxCAwy3i1iPhrtKAfXRH7dVCNGp6/86'/1'/0'/0/*)#g9xn7wf9";
const INTERNAL_DESCRIPTOR: &str = "tr(tprv8ZgxMBicQKsPdrjwWCyXqqJ4YqcyG4DmKtjjsRt29v1PtD3r3PuFJAjWytzcvSTKnZAGAkPSmnrdnuHWxCAwy3i1iPhrtKAfXRH7dVCNGp6/86'/1'/0'/1/*)#e3rjrmea";

const EXTERNAL_ABANDON: &str = "tr(tprv8ZgxMBicQKsPe5YMU9gHen4Ez3ApihUfykaqUorj9t6FDqy3nP6eoXiAo2ssvpAjoLroQxHqr3R5nE3a5dU3DHTjTgJDd7zrbniJr6nrCzd/86h/1h/0h/0/*)#vak0p2pv";
const INTERNAL_ABANDON: &str = "tr(tprv8ZgxMBicQKsPe5YMU9gHen4Ez3ApihUfykaqUorj9t6FDqy3nP6eoXiAo2ssvpAjoLroQxHqr3R5nE3a5dU3DHTjTgJDd7zrbniJr6nrCzd/86h/1h/0h/1/*)#afnwul35";

#[derive(Component, Deref, DerefMut)]
pub struct SendSatsTimer(Timer);

#[derive(Component)]
pub struct TouristWallet {
    pub wallet: Wallet,
}

#[derive(Component)]
pub struct PlayerWallet {
    pub wallet: Wallet,
}

pub fn startup(mut commands: Commands) {
    commands.spawn(SendSatsTimer(Timer::from_seconds(3.0, TimerMode::Once)));
}

pub fn send_sats(
    time: Res<Time>,
    mut player_wallet_q: Query<&PlayerWallet>,
    mut sats_to_send_q: Query<&mut SatsToSend>,
    mut send_sats_timer_q: Query<&mut SendSatsTimer>,
    mut wallet_q: Query<&mut TouristWallet>,
) {
    for mut sats_timer in &mut send_sats_timer_q {
        if sats_timer.tick(time.delta()).just_finished() {
            //bcrt1p8wpt9v4frpf3tkn0srd97pksgsxc5hs52lafxwru9kgeephvs7rqjeprhg
            let sats_to_send = sats_to_send_q.single_mut().unwrap().sats;
            if sats_to_send > 0 {
                let base_fee = 4;
                let more_fee = sats_to_send / 4_000;
                let mut wallet = wallet_q.single_mut().unwrap();
                let address = Address::from_str(
                    "bcrt1p8wpt9v4frpf3tkn0srd97pksgsxc5hs52lafxwru9kgeephvs7rqjeprhg",
                )
                .unwrap()
                .require_network(bitcoin::Network::Regtest)
                .unwrap();
                let amount = Amount::from_sat(sats_to_send);
                let fee = FeeRate::from_sat_per_vb(base_fee + more_fee).unwrap();
                let mut builder = wallet.wallet.build_tx();
                builder.fee_rate(fee).add_recipient(address, amount);
                let mut psbt = builder.finish().unwrap();
                let finalized = wallet
                    .wallet
                    .sign(&mut psbt, SignOptions::default())
                    .unwrap();

                let tx = psbt.extract_tx().unwrap();
                let client: BdkElectrumClient<Client> = BdkElectrumClient::new(
                    electrum_client::Client::new("127.0.0.1:60401").unwrap(),
                );
                match client.transaction_broadcast(&tx) {
                    Ok(_) => info!("Transaction broadcast! Txid: {}", tx.compute_txid()),
                    Err(err) => warn!("Broadcast error: {err}"),
                }
            }
            sats_timer.0.reset();
        }
    }
}

pub fn activate_wallet() -> (Wallet, Wallet) {
    let tourist_wallet = create_wallet(EXTERNAL_DESCRIPTOR, INTERNAL_DESCRIPTOR);
    let player_wallet = create_wallet(EXTERNAL_ABANDON, INTERNAL_ABANDON);
    (tourist_wallet, player_wallet)
}

pub fn create_wallet(external: &str, internal: &str) -> Wallet {
    let mut wallet: Wallet = Wallet::create(external.to_string(), internal.to_string())
        .network(Network::Regtest)
        .create_wallet_no_persist()
        .unwrap();

    let address: AddressInfo = wallet.reveal_next_address(KeychainKind::External);
    info!(
        "Generated address {} at index {}",
        address.address, address.index
    );

    // Create the Electrum client
    let client: BdkElectrumClient<Client> =
        BdkElectrumClient::new(electrum_client::Client::new("127.0.0.1:60401").unwrap());

    // Perform the initial full scan on the wallet
    let full_scan_request = wallet.start_full_scan();
    let update = client
        .full_scan(full_scan_request, STOP_GAP, BATCH_SIZE, true)
        .unwrap();

    wallet.apply_update(update).unwrap();
    let balance = wallet.balance();
    println!("Wallet balance: {} sat", balance.total().to_sat());

    wallet
}

pub fn spawn_electrs() -> Result<(Child, PathBuf, PathBuf)> {
    let electrs_path = "electrs";
    let conf_path = get_config_dir()?.join("bitcoin.conf");
    let data_dir = get_data_dir(Some(BITCOIN_DIR.into()))?;
    let db_dir = get_data_dir(Some("electrs_db".into()))?;

    let data_dir_arg = format!("{}", data_dir.display());
    let conf_path_arg = format!("{}", conf_path.display());
    let db_dir_arg = format!("{}", db_dir.display());

    let mut child = std::process::Command::new(electrs_path)
        .arg("--network")
        .arg("regtest")
        .arg("--db-dir")
        .arg(db_dir_arg)
        .arg("--daemon-dir")
        .arg(data_dir_arg)
        .arg("--conf")
        .arg(conf_path_arg)
        .arg("--electrum-rpc-addr")
        .arg("127.0.0.1:60401")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start electrs");

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    // Log stdout
    if let Some(out) = stdout {
        std::thread::spawn(move || {
            let reader = std::io::BufReader::new(out);
            for line in reader.lines().map_while(Result::ok) {
                info!("[bitcoind stdout] {}", line);
            }
        });
    }

    // Log stderr
    if let Some(err) = stderr {
        std::thread::spawn(move || {
            let reader = std::io::BufReader::new(err);
            for line in reader.lines().map_while(Result::ok) {
                warn!("[bitcoind stderr] {}", line);
            }
        });
    }

    info!("{electrs_path} process started with PID: {}", child.id());
    Ok((child, PathBuf::from(data_dir), PathBuf::from(conf_path)))
}

#[derive(Resource)]
pub struct ElectrsProcess {
    pub child: Child,
}

impl Drop for ElectrsProcess {
    fn drop(&mut self) {
        match self.child.try_wait() {
            Ok(Some(status)) => {
                log_or_print(
                    &format!("Electrs already exited with status: {status}"),
                    log::Level::Info,
                );
            }
            Ok(None) => {
                info!("Electrs still running");

                // Fallback to hard kill
                if let Err(e) = self.child.kill() {
                    error!("Failed to force kill electrs: {}", e);
                    return;
                }

                if let Ok(status) = self.child.wait() {
                    log_or_print(
                        &format!("Electrs exited after kill with status: {}", status),
                        log::Level::Info,
                    );
                }
            }
            Err(e) => {
                error!("Failed to check electrs status: {}", e);
            }
        }
    }
}
