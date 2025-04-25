use bevy::prelude::*;
use bip39::Mnemonic;
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv};
use bitcoin::key::UntweakedPublicKey;
use bitcoin::{
    hex::DisplayHex,
    key::Secp256k1,
    opcodes::all::{OP_CHECKMULTISIG, OP_PUSHNUM_1},
    Network, Script,
};
use bitcoin::{Address, CompressedPublicKey, KnownHrp, PrivateKey, ScriptBuf};
use directories::ProjectDirs;
use eyre::{eyre, Result};
use miniscript::descriptor::{DescriptorSecretKey, DescriptorXKey};
use miniscript::{Descriptor, Segwitv0};
use reqwest::blocking::Client;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::str::FromStr;
use std::{thread, time::Duration};

pub fn launch_bitcoind_process() -> Result<(Child, PathBuf, PathBuf)> {
    let challenge = get_segwit_challenge()?;
    let challenge = format!("{}", challenge.as_bytes().as_hex());
    let bitcoind_path = "bitcoind";

    let config_path = get_config_dir()?.join("bitcoin.conf");
    let datadir = get_data_dir()?.join("bitcoind");

    fs::create_dir_all(&datadir)?;

    if !config_path.exists() {
        write_bitcoin_conf(&config_path, &challenge)?;
        info!("Wrote config to {}", config_path.display());
    }

    let (child, data_dir, conf_path) = spawn_bitcoind(bitcoind_path, &datadir, &config_path)?;
    info!("bitcoind launched with PID {}", child.id());

    Ok((child, data_dir, conf_path))
}

pub fn load_descriptor(descriptor: &str) -> Result<()> {
    let datadir = get_data_dir()?.join("bitcoind");
    let (user, pass) = read_cookie_auth(&datadir)?;

    wait_for_rpc_ready(&user, &pass)?;

    load_descriptor_wallet("default", descriptor, &user, &pass)?;
    Ok(())
}

pub fn get_segwit_challenge() -> Result<ScriptBuf> {
    let master_xpriv = xpriv_key_from_abandon()?;

    let secp = Secp256k1::new();

    // purpose (bip 86) / coin type (1=testnet) / account / change (0=external, 1=internal) / addy index
    let deriv_path = DerivationPath::from_str("m/86h/1h/0h/0/0")?;
    let xpriv = master_xpriv.derive_priv(&secp, &deriv_path)?;

    let public_key = xpriv.to_priv().public_key(&secp);
    let segwit_challenge = Script::builder()
        .push_opcode(OP_PUSHNUM_1)
        .push_key(&public_key)
        .push_opcode(OP_PUSHNUM_1)
        .push_opcode(OP_CHECKMULTISIG)
        .into_script();

    Ok(segwit_challenge)
}

pub fn xpriv_key_from_abandon() -> Result<Xpriv> {
    let mnemonic = Mnemonic::parse(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        )
        .map_err(|_| eyre!("Mnemonic key parsing error."))?;
    let seed = mnemonic.to_seed("");
    let xpriv = Xpriv::new_master(Network::Regtest, &seed)?;
    Ok(xpriv)
}

pub fn xpriv_to_descriptor(xpriv: Xpriv) -> String {
    let secp = Secp256k1::new();
    info!("Xpriv: {}", xpriv);
    let deriv_path = DerivationPath::from_str("m/86h/1h/0h/0/0").expect("a derivation path");

    // tprv comes from "abandon abandon..." seed
    // bitcoin-cli -datadir=$HOME/.local/share/bird-view/bitcoind -conf=$HOME/.config/bird-view/bitcoin.conf -rpcwallet=default getdescriptorinfo "tr(tprv8ZgxMBicQKsPe5YMU9gHen4Ez3ApihUfykaqUorj9t6FDqy3nP6eoXiAo2ssvpAjoLroQxHqr3R5nE3a5dU3DHTjTgJDd7zrbniJr6nrCzd/86h/1h/0h/0/*)"

    // bitcoin-cli -datadir=$HOME/.local/share/bird-view/bitcoind -conf=$HOME/.config/bird-view/bitcoin.conf -rpcwallet=default importdescriptors '[{"desc":"tr(tprv8ZgxMBicQKsPe5YMU9gHen4Ez3ApihUfykaqUorj9t6FDqy3nP6eoXiAo2ssvpAjoLroQxHqr3R5nE3a5dU3DHTjTgJDd7zrbniJr6nrCzd/86h/1h/0h/0/*)#vak0p2pv","timestamp":"now", "active": true}]'

    // bitcoin-cli -datadir=$HOME/.local/share/bird-view/bitcoind -conf=$HOME/.config/bird-view/bitcoin.conf -rpcwallet=default getnewaddress "" bech32m
    // -> bcrt1p8wpt9v4frpf3tkn0srd97pksgsxc5hs52lafxwru9kgeephvs7rqjeprhg

    let xpriv_desc = format!("{}/{}", xpriv, "86h/1h/0h/0/0");
    let desc = DescriptorSecretKey::from_str(&xpriv_desc).unwrap();
    let xkey = match desc {
        DescriptorSecretKey::Single(_) => panic!("not a single!"),
        DescriptorSecretKey::XPrv(descriptor_xkey) => descriptor_xkey,
        DescriptorSecretKey::MultiXPrv(_) => panic!("not a multi!"),
    };

    let derived_xpriv = xkey.xkey.derive_priv(&secp, &deriv_path).unwrap();
    let kp = derived_xpriv.to_keypair(&secp);
    let pk = UntweakedPublicKey::from_keypair(&kp).0;
    let addy = Address::p2tr(&secp, pk, None, KnownHrp::Regtest);

    assert_eq!(
        format!("{}", addy),
        "bcrt1p8wpt9v4frpf3tkn0srd97pksgsxc5hs52lafxwru9kgeephvs7rqjeprhg"
    );

    // Sanity check (main net)
    let private_key = PrivateKey::new(kp.secret_key(), Network::Bitcoin);
    let cpk = CompressedPublicKey::from_private_key(&secp, &private_key).unwrap();
    let addy2 = Address::p2wpkh(&cpk, KnownHrp::Mainnet);
    assert_eq!(
        format!("{}", addy2),
        "bc1qvjvarw3yz8xmhcrt54qkaafry5r072pg4c5nmy"
    );

    format!("{}", addy)
}

pub fn read_cookie_auth(datadir: &Path) -> Result<(String, String)> {
    let subdir = datadir.join("regtest");
    let cookie_path = subdir.join(".cookie");

    wait_for_file(&cookie_path, Duration::from_secs(30))?;

    let contents = fs::read_to_string(&cookie_path).map_err(|e| {
        eyre!(
            "Failed to read cookie file {}: {}",
            cookie_path.display(),
            e
        )
    })?;

    let mut parts = contents.trim().splitn(2, ':');
    let user = parts
        .next()
        .ok_or_else(|| eyre!("Malformed cookie file"))?
        .to_string();
    let pass = parts
        .next()
        .ok_or_else(|| eyre!("Malformed cookie file"))?
        .to_string();
    Ok((user, pass))
}

pub fn get_config_dir() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("", "", env!("CARGO_PKG_NAME"))
        .ok_or_else(|| eyre!("Couldn't get config dir"))?;
    let dir = proj_dirs.config_dir();
    fs::create_dir_all(dir)?;
    info!("config dir: {:?}", dir);
    Ok(dir.to_path_buf())
}

pub fn get_data_dir() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("", "", env!("CARGO_PKG_NAME"))
        .ok_or_else(|| eyre!("Couldn't get data dir"))?;
    let data_dir = proj_dirs.data_dir();
    fs::create_dir_all(data_dir)?;
    info!("data dir: {:?}", data_dir);
    Ok(proj_dirs.data_dir().to_path_buf())
}

pub fn write_bitcoin_conf(path: &Path, challenge: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    //writeln!(file, "signet=1")?;
    writeln!(file, "regtest=1")?;
    writeln!(file, "server=1")?;
    writeln!(file, "txindex=1")?;
    writeln!(file, "fallbackfee=0.0001")?;
    //writeln!(file, "signetchallenge={}", challenge)?;
    //writeln!(file, "debug=1")?;
    Ok(())
}

fn wait_for_rpc_ready(user: &str, pass: &str) -> Result<()> {
    let client = Client::new();
    //let url = "http://127.0.0.1:38332"; // Signet
    let url = "http://127.0.0.1:18443"; // Regtest

    for _ in 0..30 {
        let res = client
            .post(url)
            .basic_auth(user, Some(pass))
            .json(&serde_json::json!({
                "jsonrpc": "1.0",
                "id": "ready",
                "method": "getblockchaininfo",
                "params": []
            }))
            .send();

        if let Ok(resp) = res {
            if resp.status().is_success() {
                info!("wait for rpc: {}", resp.text()?);
                return Ok(());
            }
        }
        thread::sleep(Duration::from_secs(1));
    }

    Err(eyre!("bitcoind did not become ready in time"))
}

fn load_descriptor_wallet(
    wallet_name: &str,
    descriptor: &str,
    user: &str,
    pass: &str,
) -> Result<()> {
    let client = Client::new();
    let url = format!("http://127.0.0.1:18443/wallet/{wallet_name}");

    // Try to create the wallet first (ignore if already exists)
    let _ = client
        .post("http://127.0.0.1:18443/")
        .basic_auth(user, Some(pass))
        .json(&serde_json::json!({
            "jsonrpc": "1.0",
            "id": "create",
            "method": "createwallet",
            "params": [wallet_name, false, true, "", false, true, true], // wallet_name disable_private_keys blank "passphrase" avoid_reuse descriptors load_on_startup
        }))
        .send();

    // Import the descriptor
    let resp = client
        .post(&url)
        .basic_auth(user, Some(pass))
        .json(&serde_json::json!({
            "jsonrpc": "1.0",
            "id": "importdesc",
            "method": "importdescriptors",
            "params": [[{
                "desc": descriptor,
                "timestamp": "now",
                "active": true,
                "range": [0, 1000]
            }]]
        }))
        .send()?;

    if resp.status().is_success() {
        log::info!("Loaded descriptor into wallet `{}`", wallet_name);
        Ok(())
    } else {
        Err(eyre!("Failed to import descriptor: {}", resp.text()?))
    }
}

fn spawn_bitcoind(
    bitcoind_path: &str,
    data_dir: &Path,
    conf_path: &Path,
) -> Result<(Child, PathBuf, PathBuf)> {
    let data_dir_arg = format!("-datadir={}", data_dir.display());
    let conf_path_arg = format!("-conf={}", conf_path.display());
    info!("{} {} {}", bitcoind_path, data_dir_arg, conf_path_arg);
    let mut child = Command::new(bitcoind_path)
        .arg(data_dir_arg)
        .arg(conf_path_arg)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start bitcoind");

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    // Log stdout
    if let Some(out) = stdout {
        thread::spawn(move || {
            let reader = BufReader::new(out);
            for line in reader.lines().map_while(Result::ok) {
                info!("[bitcoind stdout] {}", line);
            }
        });
    }

    // Log stderr
    if let Some(err) = stderr {
        thread::spawn(move || {
            let reader = BufReader::new(err);
            for line in reader.lines().map_while(Result::ok) {
                warn!("[bitcoind stderr] {}", line);
            }
        });
    }

    info!("{bitcoind_path} process started with PID: {}", child.id());
    Ok((child, PathBuf::from(data_dir), PathBuf::from(conf_path)))
}

fn wait_for_file<P: AsRef<Path>>(path: P, timeout: Duration) -> std::io::Result<()> {
    let start = std::time::Instant::now();

    while !path.as_ref().exists() {
        if start.elapsed() > timeout {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Timed out waiting for file",
            ));
        }

        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
