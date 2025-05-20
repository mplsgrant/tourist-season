use std::io::BufRead;
use std::path::PathBuf;
use std::process::{Child, Stdio};
use std::str::FromStr;

use bdk_electrum::{
    BdkElectrumClient,
    electrum_client::{self, Client},
};
use bdk_wallet::KeychainKind;
use bdk_wallet::Wallet;
use bdk_wallet::bitcoin::Network;
use bdk_wallet::chain::spk_client::{SyncRequest, SyncResponse};
use bdk_wallet::{AddressInfo, SignOptions};
use bevy::prelude::*;
use bitcoin::{Address, Amount, FeeRate};
use crossbeam_channel::{Receiver, Sender, bounded};
use num_format::{Locale, ToFormattedString};

use crate::bdk_zone::{get_config_dir, get_data_dir};
use crate::bitcoind::log_or_print;
use crate::constants::BITCOIN_DIR;
use crate::tourists::SatsToSend;

pub struct ElectrumWallet;

impl Plugin for ElectrumWallet {
    fn build(&self, app: &mut App) {
        app.add_event::<StreamEvent>()
            .add_systems(Startup, startup)
            .add_systems(Update, (send_sats, request_balance))
            .add_systems(FixedUpdate, read_stream)
            .insert_resource(Time::<Fixed>::from_seconds(0.5));
    }
}

const STOP_GAP: usize = 50;
const BATCH_SIZE: usize = 5;

const EXTERNAL_DESCRIPTOR: &str = "tr(tprv8ZgxMBicQKsPdrjwWCyXqqJ4YqcyG4DmKtjjsRt29v1PtD3r3PuFJAjWytzcvSTKnZAGAkPSmnrdnuHWxCAwy3i1iPhrtKAfXRH7dVCNGp6/86'/1'/0'/0/*)#g9xn7wf9";
const INTERNAL_DESCRIPTOR: &str = "tr(tprv8ZgxMBicQKsPdrjwWCyXqqJ4YqcyG4DmKtjjsRt29v1PtD3r3PuFJAjWytzcvSTKnZAGAkPSmnrdnuHWxCAwy3i1iPhrtKAfXRH7dVCNGp6/86'/1'/0'/1/*)#e3rjrmea";

const EXTERNAL_ABANDON: &str = "tr(tprv8ZgxMBicQKsPe5YMU9gHen4Ez3ApihUfykaqUorj9t6FDqy3nP6eoXiAo2ssvpAjoLroQxHqr3R5nE3a5dU3DHTjTgJDd7zrbniJr6nrCzd/86h/1h/0h/0/*)#vak0p2pv";
const INTERNAL_ABANDON: &str = "tr(tprv8ZgxMBicQKsPe5YMU9gHen4Ez3ApihUfykaqUorj9t6FDqy3nP6eoXiAo2ssvpAjoLroQxHqr3R5nE3a5dU3DHTjTgJDd7zrbniJr6nrCzd/86h/1h/0h/1/*)#afnwul35";

#[derive(Resource, Deref)]
struct StreamReceiver(Receiver<SyncResponse>);

#[derive(Resource, Deref)]
struct StreamSender(Sender<SyncRequest<(KeychainKind, u32)>>);

#[derive(Event)]
struct StreamEvent(Box<SyncResponse>);

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

#[derive(Component)]
pub struct WalletBalanceLabel;

#[derive(Component)]
pub struct BalanceTimer(pub Timer);

pub fn startup(mut commands: Commands) {
    commands.spawn(BalanceTimer(Timer::from_seconds(7.0, TimerMode::Once)));
    commands.spawn(SendSatsTimer(Timer::from_seconds(4.0, TimerMode::Once)));
    commands.spawn((
        Text::new("Sats: 0"),
        TextFont {
            font_size: 20.0,
            ..Default::default()
        },
        WalletBalanceLabel,
    ));

    let (sender_tx, sender_rx) = bounded::<SyncRequest<(KeychainKind, u32)>>(1);
    let (electrs_tx, electrs_rx) = bounded::<SyncResponse>(1);
    std::thread::spawn(move || {
        loop {
            match sender_rx.recv() {
                Ok(request) => {
                    let client: BdkElectrumClient<Client> = BdkElectrumClient::new(
                        electrum_client::Client::new("127.0.0.1:60401").unwrap(),
                    );
                    let update = client.sync(request, 25, true).unwrap();
                    electrs_tx.send(update).unwrap();
                }
                Err(err) => warn!("Sender recv did not get anything: {}", err),
            };
        }
    });

    commands.insert_resource(StreamReceiver(electrs_rx));
    commands.insert_resource(StreamSender(sender_tx));
}

pub fn request_balance(
    player_wallet_q: Query<&PlayerWallet>,
    mut balance_timer: Query<&mut BalanceTimer>,
    time: Res<Time>,
    sender: ResMut<StreamSender>,
) {
    for mut timer in &mut balance_timer {
        if timer.0.tick(time.delta()).just_finished() {
            for player in player_wallet_q {
                let request = player.wallet.start_sync_with_revealed_spks().build();

                match sender.0.send(request) {
                    Ok(val) => (),
                    Err(err) => warn!("Err sending value: {err}"),
                };
            }
            timer.0.reset();
        }
    }
}

fn read_stream(
    mut player_wallet_q: Query<&mut PlayerWallet>,
    mut balance_label_q: Query<&mut Text, With<WalletBalanceLabel>>,
    receiver: Res<StreamReceiver>,
) {
    for response in receiver.try_iter() {
        let mut player = player_wallet_q.single_mut().unwrap();
        player.wallet.apply_update(response).unwrap();
        let balance = player.wallet.balance();
        let amount = balance.total();
        let sat = amount.to_sat();
        balance_label_q.single_mut().unwrap().0 =
            format!("Sats: {}", sat.to_formatted_string(&Locale::en));
    }
}

pub fn send_sats(
    time: Res<Time>,
    mut sats_to_send_q: Query<&mut SatsToSend>,
    mut send_sats_timer_q: Query<&mut SendSatsTimer>,
    mut wallet_q: Query<&mut TouristWallet>,
) {
    for mut sats_timer in &mut send_sats_timer_q {
        if sats_timer.tick(time.delta()).just_finished() {
            //bcrt1p8wpt9v4frpf3tkn0srd97pksgsxc5hs52lafxwru9kgeephvs7rqjeprhg
            let mut wallet = wallet_q.single_mut().unwrap();

            // let client: BdkElectrumClient<Client> =
            //     BdkElectrumClient::new(electrum_client::Client::new("127.0.0.1:60401").unwrap());

            // let request = wallet.wallet.start_sync_with_revealed_spks().build();
            // let update = client.sync(request, 25, true).unwrap();
            // wallet.wallet.apply_update(update).unwrap();

            let sats_to_send = sats_to_send_q.single_mut().unwrap().sats;
            if sats_to_send > 0 {
                let base_fee = 4;
                let more_fee = sats_to_send / 4_000;

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

                let _is_finalized = wallet
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
    Ok((child, data_dir, conf_path))
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
                        &format!("Electrs exited after kill with status: {status}"),
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
