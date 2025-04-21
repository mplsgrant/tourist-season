use bevy::prelude::*;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::ScriptBuf;
use bitcoin::{
    hex::DisplayHex,
    key::Secp256k1,
    opcodes::all::{OP_CHECKMULTISIG, OP_PUSHNUM_1},
    Network, Script,
};
use directories::ProjectDirs;
use eyre::{eyre, Result};
use reqwest::blocking::Client;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::str::FromStr;
use std::{thread, time::Duration};

pub fn launch_bitcoind_process() -> Result<Child> {
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

    let child = spawn_bitcoind(bitcoind_path, &datadir, &config_path)?;
    info!("bitcoind launched with PID {}", child.id());

    Ok(child)
}

pub fn load_descriptor(descriptor: &str) -> Result<()> {
    let datadir = get_data_dir()?.join("bitcoind");
    let (user, pass) = read_cookie_auth(&datadir)?;
    info!("user {} pass {}", &user, pass);
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

fn xpriv_key_from_abandon() -> Result<Xpriv> {
    let mnemonic = Mnemonic::parse(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        )
        .map_err(|_| eyre!("Mnemonic key parsing error."))?;
    let seed = mnemonic.to_seed("");
    let xpriv = Xpriv::new_master(Network::Signet, &seed)?;
    Ok(xpriv)
}

pub fn read_cookie_auth(datadir: &Path) -> Result<(String, String)> {
    let subdir = datadir.join("signet");
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
    info!("Config dir: {:?}", dir);
    Ok(dir.to_path_buf())
}

pub fn get_data_dir() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("", "", env!("CARGO_PKG_NAME"))
        .ok_or_else(|| eyre!("Couldn't get data dir"))?;
    let data_dir = proj_dirs.data_dir();
    fs::create_dir_all(data_dir)?;
    info!("Data dir: {:?}", data_dir);
    Ok(proj_dirs.data_dir().to_path_buf())
}

pub fn write_bitcoin_conf(path: &Path, challenge: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "signet=1")?;
    writeln!(file, "server=1")?;
    writeln!(file, "txindex=1")?;
    writeln!(file, "fallbackfee=0.0001")?;
    writeln!(file, "signetchallenge={}", challenge)?;
    //writeln!(file, "debug=1")?;
    Ok(())
}

fn wait_for_rpc_ready(user: &str, pass: &str) -> Result<()> {
    let client = Client::new();
    let url = "http://127.0.0.1:38332";

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
    let url = format!("http://127.0.0.1:38332/wallet/{wallet_name}");

    // Try to create the wallet first (ignore if already exists)
    let _ = client
        .post("http://127.0.0.1:38332/")
        .basic_auth(user, Some(pass))
        .json(&serde_json::json!({
            "jsonrpc": "1.0",
            "id": "create",
            "method": "createwallet",
            "params": [wallet_name, true, true, "", false, true, true],
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

fn spawn_bitcoind(bitcoind_path: &str, data_dir: &Path, conf_path: &Path) -> Result<Child> {
    let mut child = Command::new(bitcoind_path)
        .arg(format!("-datadir={}", data_dir.display()))
        .arg(format!("-conf={}", conf_path.display()))
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
    Ok(child)
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
