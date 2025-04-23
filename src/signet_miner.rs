use bitcoin::absolute::LockTime;
use bitcoin::blockdata::script::{Builder, ScriptBuf};
use bitcoin::blockdata::transaction::{OutPoint, Transaction, TxIn, TxOut};
use bitcoin::consensus::encode::serialize;
use bitcoin::consensus::{deserialize, encode::Encodable};
use bitcoin::hashes::{sha256d, Hash};
use bitcoin::opcodes::OP_0;
use bitcoin::psbt::{raw::ProprietaryKey, Psbt};
use bitcoin::script::PushBytes;
use bitcoin::transaction::Version;
use bitcoin::Script;
use bitcoin::{merkle_tree, Block, Txid};

const SIGNET_HEADER: &[u8; 4] = b"\xec\xc7\xda\xa2";

pub fn signet_txs(block: &Block, challenge: bitcoin::ScriptBuf) -> (Transaction, Transaction) {
    // Clone the transactions
    let mut txs: Vec<Transaction> = block.txdata.clone();

    // Append SIGNET_HEADER to the last vout scriptPubKey of coinbase tx
    let mut coinbase = txs[0].clone();
    let mut last_vout = coinbase.output.pop().expect("Coinbase txn needs a txout");
    last_vout.script_pubkey.push_slice(SIGNET_HEADER);
    coinbase.output.push(last_vout);
    txs[0] = coinbase;

    // Recalculate txids (optional, as rust-bitcoin does this lazily)

    let hashes = txs.iter().map(|tx| tx.compute_txid().as_raw_hash().clone());
    // Calculate the Merkle root
    let mroot = merkle_tree::calculate_root(hashes).expect("a merkle root");

    // Construct the signet solution data
    let a: [u8; 4] = block.header.version.to_consensus().to_le_bytes();
    let b: [u8; 32] = block.header.prev_blockhash.to_byte_array();

    let mut sd = Vec::new();
    sd.extend(&block.header.version.to_consensus().to_le_bytes()); // 4
    sd.extend(block.header.prev_blockhash.to_byte_array()); // 32
    sd.extend(mroot.as_byte_array()); // 32
    sd.extend(&block.header.time.to_le_bytes()); // 4

    // Construct to_spend transaction
    let to_spend = Transaction {
        version: Version::non_standard(0), // TODO : Why version zero???
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: Txid::all_zeros(), // 0
                vout: std::u32::MAX,     // 0xffffffff
            },
            script_sig: Builder::new()
                .push_opcode(OP_0)
                .push_slice(&sd)
                .into_script(),
            sequence: 0,
            witness: vec![],
        }],
        output: vec![TxOut {
            value: 0,
            script_pubkey: challenge,
        }],
    };

    // Construct spend transaction
    let spend = Transaction {
        version: 0,
        lock_time: 0,
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: to_spend.txid(),
                vout: 0,
            },
            script_sig: Script::new(),
            sequence: 0,
            witness: vec![],
        }],
        output: vec![TxOut {
            value: 0,
            script_pubkey: Script::new_op_return(&[]),
        }],
    };

    (spend, to_spend)
}

const PSBT_SIGNET_BLOCK: &[u8] = b"\xfc\x06signetb";

/// Decode a base64 PSBT that includes a signet block as a proprietary global field.
/// Returns the deserialized `Block` and the final scriptSig + scriptWitness stack as a single `Vec<u8>`.
fn decode_psbt(b64_psbt: &str) -> Result<(Block, Vec<u8>), Box<dyn std::error::Error>> {
    let psbt: Psbt = bitcoin::base64::decode(b64_psbt)
        .map_err(|e| format!("Base64 decode failed: {e}")) // optional detailed error
        .and_then(|bytes| deserialize(&bytes).map_err(|e| format!("PSBT decode failed: {e}")))?;

    if psbt.global.unsigned_tx.input.len() != 1 || psbt.global.unsigned_tx.output.len() != 1 {
        return Err("PSBT must have exactly 1 input and 1 output".into());
    }

    let signet_block_key = ProprietaryKey {
        prefix: b"\xfc\x06".to_vec(),
        subtype: b"signetb"[0], // only safe if `signetb` is one byte, which it's not
        key: b"signetb"[1..].to_vec(), // slice remaining
    };

    let signet_block_bytes = psbt
        .global
        .proprietary
        .get(&signet_block_key)
        .ok_or("Missing signet block in proprietary global PSBT map")?;

    let block: Block = deserialize(signet_block_bytes)?;

    let input_map = psbt.inputs.get(0).ok_or("Missing input map")?;
    let script_sig = input_map
        .final_script_sig
        .as_ref()
        .map(|s| s.as_bytes())
        .unwrap_or(&[]);

    let witness_bytes = input_map
        .final_script_witness
        .as_ref()
        .map(|w| {
            let mut result = Vec::new();
            w.consensus_encode(&mut result).unwrap(); // Stack encoding
            result
        })
        .unwrap_or_else(|| vec![0x00]); // default witness

    let mut combined = Vec::new();
    script_sig.consensus_encode(&mut combined)?; // Encodes as a "string" with length prefix
    combined.extend(witness_bytes);

    Ok((block, combined))
}

use std::process::Command;

/// Finish constructing a signet block by appending the solution and possibly grinding.
pub fn finish_block(
    mut block: Block,
    signet_solution: &[u8],
    grind_cmd: Option<&str>,
) -> Result<Block, Box<dyn std::error::Error>> {
    // Append the solution to the coinbase output script
    if let Some(coinbase_tx) = block.txdata.first_mut() {
        if let Some(last_output) = coinbase_tx.output.last_mut() {
            let payload = [SIGNET_HEADER, signet_solution].concat();
            last_output.script_pubkey = Builder::from(last_output.script_pubkey.clone())
                .push_slice(&payload)
                .into_script();
        }
        coinbase_tx.compute_witness_hash(); // equivalent to rehashing
    }

    block.header.merkle_root = Some(block.compute_merkle_root());

    if let Some(cmd_str) = grind_cmd {
        let mut header_bytes = Vec::new();
        block.header.consensus_encode(&mut header_bytes)?;
        let headhex = hex::encode(&header_bytes);

        let mut cmd_parts = cmd_str
            .split_whitespace()
            .map(String::from)
            .collect::<Vec<_>>();
        cmd_parts.push(headhex);

        let output = Command::new(&cmd_parts[0]).args(&cmd_parts[1..]).output()?;

        if !output.status.success() {
            return Err(format!("Grind command failed: {:?}", output).into());
        }

        let new_header_bytes = hex::decode(output.stdout.trim())?;
        let new_header: bitcoin::BlockHeader = deserialize(&new_header_bytes)?;

        block.header.nonce = new_header.nonce;
        block.header.calc_pow_hash(); // optional, if needed
    } else {
        // Simulate solving by brute force? (you'll need to implement or stub this)
        // block.solve();
        // For now, do nothing.
    }

    Ok(block)
}

use bitcoin::block::Header;

/// Solve the block by incrementing the nonce until the header hash meets the PoW target.
pub fn solve(header: &mut Header) {
    let target = compact_to_target(header.bits);

    loop {
        let pow_hash = header.pow_hash().to_byte_array(); // [u8; 32]
        if pow_hash <= target {
            break;
        }
        header.nonce = header.nonce.wrapping_add(1);
    }
}

/// Convert compact `nBits` to a 32-byte target (big-endian)
fn compact_to_target(bits: u32) -> [u8; 32] {
    let exponent = (bits >> 24) as usize;
    let mut mantissa = bits & 0x007fffff;

    if bits & 0x00800000 != 0 {
        // Sign bit set (invalid compact), return max target
        return [0xff; 32];
    }

    let mut target = [0u8; 32];

    if exponent <= 3 {
        mantissa >>= 8 * (3 - exponent);
        target[31] = (mantissa & 0xff) as u8;
        target[30] = ((mantissa >> 8) & 0xff) as u8;
        target[29] = ((mantissa >> 16) & 0xff) as u8;
    } else {
        let byte_index = 32 - exponent;
        if byte_index < 32 {
            target[byte_index] = ((mantissa >> 16) & 0xff) as u8;
        }
        if byte_index + 1 < 32 {
            target[byte_index + 1] = ((mantissa >> 8) & 0xff) as u8;
        }
        if byte_index + 2 < 32 {
            target[byte_index + 2] = (mantissa & 0xff) as u8;
        }
    }

    target
}

// use bitcoin::blockdata::constants::MAX_TARGET;
// use bitcoin::Uint256;

// /// Solve a block header by incrementing the nonce until the proof-of-work target is met.
// pub fn solve(header: &mut BlockHeader) {
//     let target = compact_to_target(header.bits);

//     loop {
//         let pow_hash = header.pow_hash();
//         if Uint256::from_be_bytes(pow_hash.to_byte_array()) <= target {
//             break;
//         }
//         header.nonce = header.nonce.wrapping_add(1); // handle overflow just in case
//     }
// }

// /// Convert compact nBits to a full Uint256 target value.
// fn compact_to_target(bits: u32) -> Uint256 {
//     let exponent = (bits >> 24) as u8;
//     let mantissa = bits & 0x007fffff;

//     let mut target = if bits & 0x00800000 != 0 {
//         // Sign bit set (invalid), return max target
//         return MAX_TARGET;
//     } else {
//         Uint256::from_u64(mantissa as u64).unwrap_or(MAX_TARGET)
//     };

//     if exponent <= 3 {
//         target >>= 8 * (3 - exponent as usize);
//     } else {
//         target <<= 8 * (exponent as usize - 3);
//     }

//     target
// }

use bitcoin::{blockdata::witness::Witness, hashes::hex::FromHex};

use std::collections::BTreeMap;

/// A minimal structure for the JSON template input (you can expand this as needed)
#[derive(Debug, serde::Deserialize)]
struct BlockTemplate {
    version: i32,
    previousblockhash: String,
    bits: String,
    curtime: u32,
    mintime: u32,
    coinbasevalue: u64,
    height: u32,
    transactions: Vec<TransactionTemplate>,
    signet_challenge: String,
}

#[derive(Debug, serde::Deserialize)]
struct TransactionTemplate {
    data: String, // hex
}

/// Generate a PSBT for signing a block using signet challenge logic.
pub fn generate_psbt(
    tmpl: BlockTemplate,
    reward_spk: ScriptBuf,
    blocktime: Option<u32>,
    poolid: Option<Vec<u8>>,
) -> Result<String, Box<dyn std::error::Error>> {
    let signet_spk = Vec::from_hex(&tmpl.signet_challenge)?;

    // Coinbase input script
    let mut script_sig = script_bip34_coinbase_height(tmpl.height);
    if let Some(pid) = poolid {
        script_sig = Builder::from(script_sig).push_slice(&pid).into_script();
    }

    let coinbase_tx = Transaction {
        version: 1,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint::null(),
            script_sig: script_sig.clone(),
            sequence: 0xffff_fffe,
            witness: Witness::new(),
        }],
        output: vec![TxOut {
            value: tmpl.coinbasevalue,
            script_pubkey: reward_spk,
        }],
    };

    // Block header setup
    let mut block = Block {
        header: bitcoin::BlockHeader {
            version: tmpl.version,
            prev_blockhash: bitcoin::BlockHash::from_hex(&tmpl.previousblockhash)?,
            merkle_root: None,
            time: blocktime.unwrap_or(tmpl.curtime).max(tmpl.mintime),
            bits: u32::from_str_radix(&tmpl.bits, 16)?,
            nonce: 0,
        },
        txdata: vec![coinbase_tx],
    };

    // Add non-coinbase transactions
    for tx in tmpl.transactions {
        let raw = Vec::from_hex(&tx.data)?;
        let parsed_tx: Transaction = deserialize(&raw)?;
        block.txdata.push(parsed_tx);
    }

    // Coinbase witness commitment (Signet doesn’t need full segwit support, but we mimic)
    let witnonce = 0u32;
    let mut wit_stack = Witness::new();
    wit_stack.push(&witnonce.to_le_bytes());
    block.txdata[0].input[0].witness = wit_stack;

    let witroot = block.witness_merkle_root();
    let witness_script = get_witness_script(&witroot, witnonce);
    block.txdata[0].output.push(TxOut {
        value: 0,
        script_pubkey: ScriptBuf::from(witness_script),
    });

    // Signet-specific double-spend trick
    let (signme, spendme) = signet_txs(&mut block, ScriptBuf::from(signet_spk));

    // Construct PSBT
    let mut psbt = Psbt {
        global: miniscript::psbt::Global {
            unsigned_tx: signme.clone(),
            proprietary: BTreeMap::new(),
            ..Default::default()
        },
        inputs: vec![Input {
            non_witness_utxo: Some(spendme.clone()),
            sighash_type: Some(bitcoin::EcdsaSighashType::All),
            ..Default::default()
        }],
        outputs: vec![Output::default()],
    };

    // Proprietary field to hold the block
    use bitcoin::psbt::raw::ProprietaryKey;
    let key = ProprietaryKey {
        prefix: b"\xfc\x06".to_vec(),
        subtype: b's', // if using b"signetb", you'd need a custom mapping here
        key: b"ignetb".to_vec(),
    };
    psbt.global.proprietary.insert(key, serialize(&block));

    Ok(bitcoin::base64::encode(&serialize(&psbt)?))
}

pub fn get_poolid(poolid: Option<String>, poolnum: Option<u32>) -> Option<Vec<u8>> {
    if let Some(pid) = poolid {
        Some(pid.into_bytes())
    } else if let Some(num) = poolnum {
        Some(format!("/signet:{}/", num).into_bytes())
    } else {
        None
    }
}

fn script_bip34_coinbase_height(height: u32) -> ScriptBuf {
    Builder::new().push_int(height as i64).into_script()
}

use serde_json::Value;

/// Arguments/state used across calls
pub struct MinerArgs {
    pub address: Option<String>,
    pub descriptor: Option<String>,
    pub derived_addresses: BTreeMap<u32, String>,
    pub reward_spk: Option<Vec<u8>>,
    pub bcli: Box<dyn Fn(&str, &[&str]) -> Result<String, Box<dyn std::error::Error>>>,
}

pub fn get_reward_addr_spk(
    args: &mut MinerArgs,
    height: u32,
) -> Result<(String, Vec<u8>), Box<dyn std::error::Error>> {
    assert!(args.address.is_some() || args.descriptor.is_some());

    if let Some(ref reward_spk) = args.reward_spk {
        return Ok((args.address.clone().unwrap(), reward_spk.clone()));
    }

    let reward_addr = if let Some(ref addr) = args.address {
        addr.clone()
    } else if let Some(ref descriptor) = args.descriptor {
        if !descriptor.contains('*') {
            let json_str = (args.bcli)("deriveaddresses", &[descriptor])?;
            let derived: Vec<String> = serde_json::from_str(&json_str)?;
            let first = derived.get(0).ok_or("No address returned")?.clone();
            args.address = Some(first.clone());
            first
        } else {
            // clean out old cache
            let old_keys: Vec<u32> = args
                .derived_addresses
                .keys()
                .copied()
                .filter(|k| *k + 20 <= height)
                .collect();
            for k in old_keys {
                args.derived_addresses.remove(&k);
            }

            // derive new range if needed
            if !args.derived_addresses.contains_key(&height) {
                let range_arg = format!("[{},{}]", height, height + 20);
                let json_str = (args.bcli)("deriveaddresses", &[descriptor, &range_arg])?;
                let derived: Vec<String> = serde_json::from_str(&json_str)?;
                for (i, addr) in derived.into_iter().enumerate() {
                    args.derived_addresses.insert(height + i as u32, addr);
                }
            }

            args.derived_addresses
                .get(&height)
                .ok_or("Failed to get derived address")?
                .clone()
        }
    } else {
        return Err("No address or descriptor provided".into());
    };

    let info_json = (args.bcli)("getaddressinfo", &[&reward_addr])?;
    let info: Value = serde_json::from_str(&info_json)?;
    let spk_hex = info["scriptPubKey"]
        .as_str()
        .ok_or("Missing scriptPubKey in address info")?;
    let reward_spk = hex::decode(spk_hex)?;

    if args.address.is_some() {
        args.reward_spk = Some(reward_spk.clone());
    }

    Ok((reward_addr, reward_spk))
}

pub fn do_genpsbt(args: &mut MinerArgs) -> Result<(), Box<dyn std::error::Error>> {
    let poolid = get_poolid(args.poolid.clone(), args.poolnum);
    let tmpl: BlockTemplate = serde_json::from_reader(std::io::stdin())?;
    let (_, reward_spk) = get_reward_addr_spk(args, tmpl.height)?;
    let psbt_b64 = generate_psbt(tmpl, reward_spk, None, poolid)?;
    println!("{}", psbt_b64);
    Ok(())
}

pub fn do_solvepsbt(grind_cmd: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    let (block, signet_solution) = decode_psbt(&input)?;
    let block = finish_block(block, &signet_solution, grind_cmd.as_deref())?;
    println!("{}", hex::encode(serialize(&block)));
    Ok(())
}

pub fn nbits_to_target(nbits: u32) -> [u8; 32] {
    let mut target = [0u8; 32];
    let exponent = (nbits >> 24) as usize;
    let mantissa = nbits & 0x00ffffff;

    if exponent <= 3 {
        let mantissa_shifted = mantissa >> (8 * (3 - exponent));
        target[31] = (mantissa_shifted & 0xff) as u8;
        target[30] = ((mantissa_shifted >> 8) & 0xff) as u8;
        target[29] = ((mantissa_shifted >> 16) & 0xff) as u8;
    } else {
        let byte_index = 32 - exponent;
        if byte_index < 32 {
            target[byte_index] = ((mantissa >> 16) & 0xff) as u8;
        }
        if byte_index + 1 < 32 {
            target[byte_index + 1] = ((mantissa >> 8) & 0xff) as u8;
        }
        if byte_index + 2 < 32 {
            target[byte_index + 2] = (mantissa & 0xff) as u8;
        }
    }

    target
}

pub fn target_to_nbits(target: &[u8; 32]) -> u32 {
    let mut i = 0;
    while i < 32 && target[i] == 0 {
        i += 1;
    }

    let mut nsize = 32 - i;
    let mut compact: u32 = if nsize >= 3 {
        ((target[i] as u32) << 16) | ((target[i + 1] as u32) << 8) | (target[i + 2] as u32)
    } else if nsize == 2 {
        ((target[i] as u32) << 8) | (target[i + 1] as u32)
    } else {
        target[i] as u32
    };

    // Handle sign bit
    if (compact & 0x00800000) != 0 {
        compact >>= 8;
        nsize += 1;
    }

    ((nsize as u32) << 24) | (compact & 0x007fffff)
}

pub fn seconds_to_hms(mut s: i64) -> String {
    if s == 0 {
        return "0s".to_string();
    }

    let neg = s < 0;
    if neg {
        s = -s;
    }

    let hours = s / 3600;
    let minutes = (s % 3600) / 60;
    let seconds = s % 60;

    let mut result = String::new();
    if hours > 0 {
        result += &format!("{}h", hours);
    }
    if minutes > 0 {
        result += &format!("{}m", minutes);
    }
    if seconds > 0 {
        result += &format!("{}s", seconds);
    }

    if neg {
        result.insert(0, '-');
    }

    result
}

use std::{thread::sleep, time::Duration};

const INTERVAL: f64 = 600.0 * 2016.0 / 2015.0; // 10 minutes adjusted for the off-by-one bug

pub struct Generate {
    multi_low: u32,
    multi_high: u32,
    multi_period: u32,
    ultimate_target: f64,
    poisson: bool,
    max_interval: f64,
    standby_delay: i64,
    backup_delay: i64,
    set_block_time: Option<i64>,
    pub poolid: Option<Vec<u8>>,

    // state
    pub mine_time: i64,
    pub action_time: i64,
    pub is_mine: bool,
}

impl Generate {
    pub fn new(
        multiminer: Option<(u32, u32, u32)>,
        ultimate_target: f64,
        poisson: bool,
        max_interval: f64,
        standby_delay: i64,
        backup_delay: i64,
        set_block_time: Option<i64>,
        poolid: Option<Vec<u8>>,
    ) -> Self {
        let (low, high, period) = multiminer.unwrap_or((0, 1, 1));
        Self {
            multi_low: low,
            multi_high: high,
            multi_period: period,
            ultimate_target,
            poisson,
            max_interval,
            standby_delay,
            backup_delay,
            set_block_time,
            poolid,
            mine_time: 0,
            action_time: 0,
            is_mine: false,
        }
    }

    pub fn next_block_delta(&self, last_nbits: u32, last_hash: &str) -> f64 {
        let current_target = self.nbits_to_target_f64(last_nbits);
        let mut retarget_factor = self.ultimate_target / current_target;
        retarget_factor = retarget_factor.clamp(0.25, 4.0);

        let mut avg_interval = INTERVAL * retarget_factor;

        let interval_variance = if self.poisson {
            let det_rand = u32::from_str_radix(&last_hash[last_hash.len() - 8..], 16).unwrap()
                as f64
                * 2f64.powi(-32);
            -((1.0 - det_rand).ln_1p())
        } else {
            1.0
        };

        let this_interval = (avg_interval * interval_variance).clamp(1.0, self.max_interval);
        this_interval
    }

    pub fn next_block_is_mine(&self, last_hash: &str) -> bool {
        let det_rand =
            u32::from_str_radix(&last_hash[last_hash.len() - 16..last_hash.len() - 8], 16).unwrap();
        let slot = det_rand % self.multi_period;
        self.multi_low <= slot && slot < self.multi_high
    }

    pub fn next_block_time(&mut self, now: i64, bestheader: &Value, is_first_block: bool) {
        if let Some(set_time) = self.set_block_time {
            self.mine_time = set_time;
            self.action_time = now;
            self.is_mine = true;
        } else if bestheader["height"].as_u64() == Some(0) {
            let time_delta = (INTERVAL * 100.0) as i64;
            self.mine_time = now - time_delta;
            self.action_time = now;
            self.is_mine = true;
        } else {
            let bits = u32::from_str_radix(bestheader["bits"].as_str().unwrap(), 16).unwrap();
            let last_hash = bestheader["hash"].as_str().unwrap();
            let delta = self.next_block_delta(bits, last_hash);
            self.mine_time = bestheader["time"].as_i64().unwrap() + delta as i64;

            self.is_mine = self.next_block_is_mine(last_hash);
            self.action_time = self.mine_time;

            if !self.is_mine {
                self.action_time += self.backup_delay;
            }

            if self.standby_delay > 0 {
                self.action_time += self.standby_delay;
            } else if is_first_block {
                self.action_time = now;
            }
        }

        self.mine_time = self.mine_time.clamp(i64::MIN, i64::MAX);
        self.action_time = self.action_time.clamp(i64::MIN, self.mine_time - 6900);
    }

    fn nbits_to_target_f64(&self, nbits: u32) -> f64 {
        let shift = (nbits >> 24) & 0xff;
        let mant = (nbits & 0x00ff_ffff) as f64;
        mant * 2f64.powi(8 * (shift as i32 - 3))
    }

    pub fn gbt<F>(&self, bcli: &F, bestblockhash: &str, now: i64) -> Option<Value>
    where
        F: Fn(&str, &[&str]) -> Result<String, Box<dyn std::error::Error>>,
    {
        let tmpl: Value = serde_json::from_str(
            &bcli("getblocktemplate", &["{\"rules\":[\"signet\",\"segwit\"]}"]).ok()?,
        )
        .ok()?;

        if tmpl["previousblockhash"] != bestblockhash {
            eprintln!(
                "GBT based off unexpected block ({} not {}), retrying",
                tmpl["previousblockhash"], bestblockhash
            );
            sleep(Duration::from_secs(1));
            return None;
        }

        if tmpl["mintime"].as_i64()? > self.mine_time {
            eprintln!(
                "Updating block time from {} to {}",
                self.mine_time, tmpl["mintime"]
            );
            if tmpl["mintime"].as_i64()? > now {
                eprintln!(
                    "GBT mintime is in the future: {} is {} seconds later than {}",
                    tmpl["mintime"],
                    tmpl["mintime"].as_i64()? - now,
                    now
                );
                return None;
            }
        }

        Some(tmpl)
    }

    pub fn mine<F>(
        &self,
        bcli: &F,
        grind_cmd: Option<&str>,
        tmpl: &BlockTemplate,
        reward_spk: ScriptBuf,
    ) -> Result<Option<Block>, Box<dyn std::error::Error>>
    where
        F: Fn(&str, &[&str], Option<&[u8]>) -> Result<String, Box<dyn std::error::Error>>,
    {
        let psbt = generate_psbt(
            tmpl.clone(),
            reward_spk,
            Some(self.mine_time as u32),
            self.poolid.clone(),
        )?;
        let input = format!("{}\ntrue\nALL\n", psbt);

        let response_str = bcli("walletprocesspsbt", &["-stdin"], Some(input.as_bytes()))?;
        let response: serde_json::Value = serde_json::from_str(&response_str)?;

        if !response
            .get("complete")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            eprintln!("Generated PSBT: {}", psbt);
            eprintln!("PSBT signing failed");
            return Ok(None);
        }

        let signed_psbt = response
            .get("psbt")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'psbt' field in walletprocesspsbt result")?;

        let (block, signet_solution) = decode_psbt(signed_psbt)?;
        let block = finish_block(block, &signet_solution, grind_cmd)?;
        Ok(Some(block))
    }
}

fn bcli(
    cmd: &str,
    args: &[&str],
    input: Option<&[u8]>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut command = std::process::Command::new("bitcoin-cli");
    command.arg("-signet").arg(cmd).args(args);

    if let Some(stdin) = input {
        command.stdin(std::process::Stdio::piped());
        command.stdout(std::process::Stdio::piped());
        let mut child = command.spawn()?;
        if let Some(mut child_stdin) = child.stdin.take() {
            use std::io::Write;
            child_stdin.write_all(stdin)?;
        }
        let output = child.wait_with_output()?;
        Ok(String::from_utf8(output.stdout)?)
    } else {
        let output = command.output()?;
        Ok(String::from_utf8(output.stdout)?)
    }
}

use std::time::{SystemTime, UNIX_EPOCH};

pub fn do_generate(args: &mut MinerArgs) -> Result<(), Box<dyn std::error::Error>> {
    let now = || {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    };

    let max_blocks = if let Some(t) = args.set_block_time {
        if t < 0 {
            args.set_block_time = Some(now());
            eprintln!(
                "Treating negative block time as current time ({})",
                args.set_block_time.unwrap()
            );
        }
        Some(1)
    } else if let Some(max) = args.max_blocks {
        if max < 1 {
            eprintln!("--max_blocks must specify a positive integer");
            return Ok(());
        }
        Some(max)
    } else if args.ongoing {
        None
    } else {
        Some(1)
    };

    if args.min_nbits {
        args.nbits = Some("1e0377ae".to_string());
        eprintln!("Using nbits={}", args.nbits.as_ref().unwrap());
    }

    if args.set_block_time.is_none() {
        if args.nbits.as_ref().map(|s| s.len() != 8).unwrap_or(true) {
            eprintln!("Must specify --nbits (use calibrate command to determine value)");
            return Ok(());
        }
    }

    let my_blocks = if let Some(ref mstr) = args.multiminer {
        if !args.ongoing {
            eprintln!("Cannot specify --multiminer without --ongoing");
            return Ok(());
        }
        let re = regex::Regex::new(r"^(\d+)(?:-(\d+))?/(\d+)$").unwrap();
        let caps = re.captures(mstr).ok_or("--multiminer format invalid")?;
        let start = caps.get(1).unwrap().as_str().parse::<u32>()?;
        let stop = caps
            .get(2)
            .map_or(start, |m| m.as_str().parse::<u32>().unwrap());
        let total = caps.get(3).unwrap().as_str().parse::<u32>()?;
        if stop < start || start == 0 || total < stop || total == 0 {
            eprintln!("Inconsistent values for --multiminer");
            return Ok(());
        }
        (start - 1, stop, total)
    } else {
        (0, 1, 1)
    };

    if args.max_interval < 960 {
        eprintln!("--max-interval must be at least 960 (16 minutes)");
        return Ok(());
    }

    let poolid = get_poolid(args.poolid.clone(), args.poolnum);
    let nbits = u32::from_str_radix(args.nbits.as_ref().unwrap(), 16)?;
    let ultimate_target = nbits_to_target_f64(nbits);

    let mut generate = Generate::new(
        Some(my_blocks),
        ultimate_target,
        args.poisson,
        args.max_interval as f64,
        args.standby_delay,
        args.backup_delay,
        args.set_block_time,
        poolid,
    );

    let mut mined_blocks = 0;
    let mut bestheader: Option<Value> = None;
    let mut lastheader: Option<String> = None;

    while max_blocks.map_or(true, |max| mined_blocks < max) {
        let bci: Value = serde_json::from_str(&args.bcli("getblockchaininfo", &[], None)?)?;
        let best_hash = bci["bestblockhash"].as_str().unwrap();

        if bestheader.as_ref().map_or(true, |b| b["hash"] != best_hash) {
            let hdr: Value =
                serde_json::from_str(&args.bcli("getblockheader", &[best_hash], None)?)?;
            bestheader = Some(hdr);
        }

        let bh = bestheader.as_ref().unwrap();
        let bh_hash = bh["hash"].as_str().unwrap().to_string();

        if lastheader.is_none() {
            lastheader = Some(bh_hash.clone());
        } else if lastheader.as_ref().unwrap() != &bh_hash {
            let delta = generate.next_block_delta(
                u32::from_str_radix(bh["bits"].as_str().unwrap(), 16)?,
                bh_hash.as_str(),
            );
            let remaining = delta + (bh["time"].as_i64().unwrap() - now()) as f64;
            let next_is_mine = generate.next_block_is_mine(bh_hash.as_str());
            eprintln!(
                "Received new block at height {}; next in {} ({})",
                bh["height"],
                seconds_to_hms(remaining as i64),
                if next_is_mine { "mine" } else { "backup" }
            );
            lastheader = Some(bh_hash);
        }

        let cur = now();
        generate.next_block_time(cur, bh, mined_blocks == 0);

        if cur < generate.action_time {
            let mut sleep_for = generate.action_time - cur;
            if generate.mine_time < cur {
                sleep_for = sleep_for.min(20);
            } else {
                sleep_for = sleep_for.min(60);
            }

            let mode = if generate.is_mine { "mine" } else { "backup" };
            eprintln!(
                "Sleeping for {}, next block due in {} ({})",
                seconds_to_hms(sleep_for),
                seconds_to_hms(generate.mine_time - cur),
                mode
            );
            sleep(Duration::from_secs(sleep_for as u64));
            continue;
        }

        let tmpl_json = generate.gbt(&args.bcli, best_hash, cur);
        let tmpl = match tmpl_json {
            Some(t) => t,
            None => continue,
        };

        let tmpl_block: BlockTemplate = serde_json::from_value(tmpl.clone())?;

        let (reward_addr, reward_spk) = get_reward_addr_spk(args, tmpl_block.height)?;
        eprintln!(
            "Mining block delta={} start={} mine={}",
            seconds_to_hms(generate.mine_time - bh["time"].as_i64().unwrap()),
            generate.mine_time,
            generate.is_mine
        );

        mined_blocks += 1;
        let block_opt = generate.mine(
            &args.bcli,
            args.grind_cmd.as_deref(),
            &tmpl_block,
            reward_spk,
        )?;

        let block = match block_opt {
            Some(b) => b,
            None => return Ok(()),
        };

        let hex_block = hex::encode(bitcoin::consensus::serialize(&block));
        let submit_result = args.bcli("submitblock", &["-stdin"], Some(hex_block.as_bytes()))?;

        let mode = if generate.is_mine {
            "block"
        } else {
            "backup block"
        };

        let delta = generate.next_block_delta(block.header.bits, &block.block_hash().to_string());
        let next_is_mine = generate.next_block_is_mine(&block.block_hash().to_string());

        eprintln!(
            "Mined {} at height {}; next in {} ({})",
            mode,
            tmpl["height"],
            seconds_to_hms(delta as i64 + (block.header.time as i64 - now())),
            if next_is_mine { "mine" } else { "backup" }
        );

        if !submit_result.trim().is_empty() {
            eprintln!(
                "submitblock returned {} for height {} hash {}",
                submit_result,
                tmpl["height"],
                block.block_hash()
            );
        }

        lastheader = Some(block.block_hash().to_string());
    }

    Ok(())
}

use std::io::Write;
use std::time::Instant;

pub fn do_calibrate(
    grind_cmd: &str,
    nbits: Option<String>,
    seconds: Option<u32>,
) -> Result<(), Box<dyn std::error::Error>> {
    if nbits.is_some() && seconds.is_some() {
        eprintln!("Can only specify one of --nbits or --seconds");
        return Ok(());
    }

    if let Some(ref s) = nbits {
        if s.len() != 8 {
            eprintln!("Must specify 8 hex digits for --nbits");
            return Ok(());
        }
    }

    const TRIALS: usize = 600;
    const TRIAL_BITS: u32 = 0x1e3ea75f;

    let mut header = Header {
        version: 0,
        prev_blockhash: Default::default(),
        merkle_root: None,
        time: 0,
        bits: TRIAL_BITS,
        nonce: 0,
    };

    let targ = nbits_to_target_f64(TRIAL_BITS);

    let start = Instant::now();
    for i in 0..TRIALS {
        header.time = i as u32;
        header.nonce = 0;

        let mut encoded = Vec::new();
        header.consensus_encode(&mut encoded)?;

        let hex_header = hex::encode(&encoded);
        let mut parts = grind_cmd
            .split_whitespace()
            .map(str::to_string)
            .collect::<Vec<_>>();
        parts.push(hex_header);

        let output = Command::new(&parts[0])
            .args(&parts[1..])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .output()?;

        if !output.status.success() {
            eprintln!("grind_cmd failed on trial {}", i);
            return Ok(());
        }
    }

    let elapsed_secs = start.elapsed().as_secs_f64();
    let avg_time = elapsed_secs / TRIALS as f64;

    let (want_time, want_targ): (f64, f64) = if let Some(ref hexbits) = nbits {
        let want_targ = nbits_to_target_f64(u32::from_str_radix(hexbits, 16)?);
        let want_time = avg_time * targ / want_targ;
        (want_time, want_targ)
    } else {
        let want_time = seconds.unwrap_or(25) as f64;
        let want_targ = targ * (avg_time / want_time);
        (want_time, want_targ)
    };

    let nbits = target_to_nbits_f64(want_targ);
    println!(
        "nbits={:08x} for {}s average mining time",
        nbits,
        want_time.round() as u32
    );

    Ok(())
}

pub fn nbits_to_target_f64(nbits: u32) -> f64 {
    let shift = (nbits >> 24) & 0xff;
    let mant = (nbits & 0x00ff_ffff) as f64;
    mant * 2f64.powi(8 * (shift as i32 - 3))
}

pub fn target_to_nbits_f64(target: f64) -> u32 {
    let mut t = format!("{:x}", target.round() as u64);
    while t.len() < 6 {
        t = format!("0{}", t);
    }
    if t.len() % 2 != 0 {
        t = format!("0{}", t);
    }
    if u8::from_str_radix(&t[0..2], 16).unwrap() >= 0x80 {
        t = format!("00{}", t);
    }

    let sz = t.len() / 2;
    let mant = u32::from_str_radix(&t[0..6], 16).unwrap();
    let rest = &t[6..];
    let fix = if rest.chars().all(|c| c == '0') {
        mant
    } else {
        mant + 1
    };

    ((sz as u32) << 24) | (fix & 0x007fffff)
}

pub fn signet_txs2(
    mut block: Block,
    challenge: ScriptBuf,
    signet_header: &[u8],
) -> (Transaction, Transaction) {
    // Copy and modify transactions
    let mut txs = block.txdata.clone();

    // Replace the first transaction with a mutable clone
    let mut coinbase = txs[0].clone();

    // Append SIGNET_HEADER to the last output scriptPubKey
    let mut last_vout = coinbase.output.pop().expect("Coinbase has no outputs");
    let mut new_script = last_vout.script_pubkey.clone().into_bytes();

    let pushdata = ScriptBuilder::new()
        .push_slice(signet_header)
        .into_script()
        .into_bytes();

    new_script.extend(pushdata);
    last_vout.script_pubkey = Script::from(new_script);
    coinbase.output.push(last_vout);
    txs[0] = coinbase;

    // Compute Merkle root of tx hashes
    let tx_hashes: Vec<sha256d::Hash> = txs.iter().map(|tx| tx.txid().as_hash()).collect();

    let merkle_root = bitcoin::blockdata::block::compute_merkle_root(&tx_hashes);

    // Build signet solution data
    let mut solution_data = Vec::new();
    block
        .header
        .version
        .consensus_encode(&mut solution_data)
        .unwrap();
    block
        .header
        .prev_blockhash
        .consensus_encode(&mut solution_data)
        .unwrap();
    merkle_root.consensus_encode(&mut solution_data).unwrap();
    block
        .header
        .time
        .consensus_encode(&mut solution_data)
        .unwrap();

    // Create to_spend transaction
    let to_spend = Transaction {
        version: 0,
        lock_time: 0,
        input: vec![TxIn {
            previous_output: OutPoint::new(Txid::all_zeros(), 0xFFFFFFFF),
            script_sig: ScriptBuilder::new()
                .push_opcode(bitcoin::blockdata::opcodes::all::OP_FALSE)
                .push_slice(&solution_data)
                .into_script(),
            sequence: 0,
            witness: vec![],
        }],
        output: vec![TxOut {
            value: 0,
            script_pubkey: challenge,
        }],
    };

    let to_spend_txid = to_spend.txid();

    // Create spend transaction
    let spend = Transaction {
        version: 0,
        lock_time: 0,
        input: vec![TxIn {
            previous_output: OutPoint::new(to_spend_txid, 0),
            script_sig: Script::new(),
            sequence: 0,
            witness: vec![],
        }],
        output: vec![TxOut {
            value: 0,
            script_pubkey: ScriptBuilder::new()
                .push_opcode(bitcoin::blockdata::opcodes::all::OP_RETURN)
                .into_script(),
        }],
    };

    (spend, to_spend)
}

mod more_signet {
    use bitcoin::consensus::encode::{serialize, Encodable};
    use bitcoin::consensus::Decodable;
    use bitcoin::hashes::sha256d::Hash as Sha256dHash;
    use bitcoin::{Block, OutPoint, Script, ScriptBuf, Transaction, TxIn, TxOut};

    const SIGNET_HEADER: &[u8] = b"\xec\xc7\xda\xa2";

    fn signet_txs(block: &mut Block, challenge: ScriptBuf) -> (Transaction, Transaction) {
        let mut coinbase_tx = block.txdata[0].clone();
        let mut script = coinbase_tx.output.last_mut().unwrap().script_pubkey.clone();
        script = script.append_slice(&[SIGNET_HEADER.len() as u8]); // simulate OP_PUSH
        script = script.append_slice(SIGNET_HEADER);
        coinbase_tx.output.last_mut().unwrap().script_pubkey = script;

        let hashes: Vec<Sha256dHash> = block.txdata.iter().map(|tx| tx.txid().as_hash()).collect();
        let mroot = block.compute_merkle_root();

        let mut sd = Vec::new();
        block.header.version.consensus_encode(&mut sd).unwrap();
        block
            .header
            .prev_blockhash
            .consensus_encode(&mut sd)
            .unwrap();
        mroot.consensus_encode(&mut sd).unwrap();
        block.header.time.consensus_encode(&mut sd).unwrap();

        let to_spend = Transaction {
            version: 0,
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint::new(Default::default(), 0xffff_ffff),
                script_sig: ScriptBuf::from(vec![0x00, (sd.len() as u8)]).append_slice(&sd),
                sequence: 0,
                witness: vec![],
            }],
            output: vec![TxOut {
                value: 0,
                script_pubkey: challenge,
            }],
        };

        let spend = Transaction {
            version: 0,
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint::new(to_spend.txid(), 0),
                script_sig: ScriptBuf::new(),
                sequence: 0,
                witness: vec![],
            }],
            output: vec![TxOut {
                value: 0,
                script_pubkey: ScriptBuf::new_p2null(), // OP_RETURN
            }],
        };

        (spend, to_spend)
    }
}
