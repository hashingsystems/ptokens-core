use serde_json;
use ethereum_types::Address as EthAddress;
use crate::{
    constants::SAFE_ETH_ADDRESS,
    types::{
        Bytes,
        Result,
    },
    btc::{
        btc_constants::{
            DEFAULT_BTC_SEQUENCE,
            PTOKEN_P2SH_SCRIPT_BYTES,
        },
        btc_types::{
            BtcBlockAndId,
            MintingParams,
            BtcUtxoAndValue,
            BtcUtxosAndValues,
            BtcBlockInDbFormat,
            DepositAddressInfo,
            DepositAddressInfoJson,
        },
    },
    utils::{
        convert_bytes_to_u64,
        convert_u64_to_bytes,
    },
    base58::{
        from as from_base58,
        encode_slice as base58_encode_slice,
    },
};
use bitcoin::{
    network::constants::Network as BtcNetwork,
    consensus::encode::serialize as btc_serialize,
    consensus::encode::deserialize as btc_deserialize,
    hashes::{
        Hash,
        sha256d,
    },
    blockdata::{
        opcodes,
        transaction::{
            TxIn as BtcUtxo,
            TxOut as BtcTxOut,
            OutPoint as BtcOutPoint,
            Transaction as BtcTransaction,
        },
        script::{
            Script as BtcScript,
            Builder as BtcScriptBuilder,
        },
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedBlockAndId {
    pub id: Bytes,
    pub block: Bytes,
    pub height: Bytes,
}

impl SerializedBlockAndId {
    pub fn new(
        serialized_id: Bytes,
        serialized_block: Bytes,
        serialized_height: Bytes,
    ) -> Self {
        SerializedBlockAndId {
            id: serialized_id,
            block: serialized_block,
            height: serialized_height,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedBlockInDbFormat {
    pub id: Bytes,
    pub block: Bytes,
    pub height: Bytes,
    pub extra_data: Bytes,
    pub minting_params: Bytes,
}

impl SerializedBlockInDbFormat {
    pub fn new(
        serialized_id: Bytes,
        serialized_block: Bytes,
        serialized_height: Bytes,
        serialized_extra_data: Bytes,
        serialized_minting_params: Bytes,
    ) -> Self {
        SerializedBlockInDbFormat {
            id: serialized_id,
            block: serialized_block,
            height: serialized_height,
            extra_data: serialized_extra_data,
            minting_params: serialized_minting_params,
        }
    }
}

pub fn get_p2sh_redeem_script_sig(
    utxo_spender_pub_key_slice: &[u8],
    eth_address_and_nonce_hash: &sha256d::Hash,
) -> BtcScript {
    info!("✔ Generating `p2sh`'s redeem `script_sig`");
    debug!(
        "✔ Using `eth_address_and_nonce_hash`: {}",
        hex::encode(eth_address_and_nonce_hash)
    );
    debug!(
        "✔ Using `pub key slice`: {}",
        hex::encode(utxo_spender_pub_key_slice)
    );
    BtcScriptBuilder::new()
        .push_slice(&eth_address_and_nonce_hash[..])
        .push_opcode(opcodes::all::OP_DROP)
        .push_slice(&utxo_spender_pub_key_slice)
        .push_opcode(opcodes::all::OP_CHECKSIG)
        .into_script()
}

pub fn get_p2sh_script_sig_from_redeem_script(
    signature_slice: &[u8],
    redeem_script: &BtcScript,
) -> BtcScript {
    BtcScriptBuilder::new()
        .push_slice(&signature_slice)
        .push_slice(redeem_script.as_bytes())
        .into_script()
}

pub fn get_btc_block_in_db_format(
    btc_block_and_id: BtcBlockAndId,
    minting_params: MintingParams,
    extra_data: Bytes,
) -> Result<BtcBlockInDbFormat> {
    BtcBlockInDbFormat::new(
        btc_block_and_id.height,
        btc_block_and_id.id,
        minting_params,
        btc_block_and_id.block,
        extra_data,
    )
}

pub fn serialize_minting_params(
    minting_params: &MintingParams
) -> Result<Bytes> {
    Ok(serde_json::to_vec(minting_params)?)
}

pub fn deserialize_minting_params(
    serialized_minting_params: Bytes
) -> Result<MintingParams> {
    Ok(serde_json::from_slice(&serialized_minting_params[..])?)
}

pub fn create_op_return_btc_utxo_and_value_from_tx_output(
    tx: &BtcTransaction,
    output_index: u32,
) -> BtcUtxoAndValue {
    BtcUtxoAndValue::new(
        tx.output[output_index as usize].value,
        &create_unsigned_utxo_from_tx(tx, output_index),
        None,
        None,
    )
}

pub fn create_unsigned_utxo_from_tx(
    tx: &BtcTransaction,
    output_index: u32,
) -> BtcUtxo {
    let outpoint = BtcOutPoint {
        txid: tx.txid(),
        vout: output_index,
    };
    BtcUtxo {
        witness: vec![], // NOTE: We don't currently support segwit txs.
        previous_output: outpoint,
        sequence: DEFAULT_BTC_SEQUENCE,
        script_sig: tx
            .output[output_index as usize]
            .script_pubkey
            .clone(),
    }
}

pub fn convert_deposit_info_to_json(
    deposit_info_struct: &DepositAddressInfo
) -> DepositAddressInfoJson {
    DepositAddressInfoJson {
        nonce:
            deposit_info_struct.nonce,
        btc_deposit_address:
            deposit_info_struct.btc_deposit_address.to_string(),
        eth_address:
            hex::encode(deposit_info_struct.eth_address.as_bytes()),
        eth_address_and_nonce_hash:
            hex::encode(deposit_info_struct.eth_address_and_nonce_hash),
    }
}

pub fn convert_btc_network_to_bytes(network: &BtcNetwork) -> Result<Bytes> {
    match network {
        BtcNetwork::Bitcoin => Ok(convert_u64_to_bytes(&0)),
        BtcNetwork::Testnet => Ok(convert_u64_to_bytes(&1)),
        BtcNetwork::Regtest=> Ok(convert_u64_to_bytes(&2)),
    }
}

pub fn convert_bytes_to_btc_network(bytes: &Bytes) -> Result<BtcNetwork> {
    match convert_bytes_to_u64(bytes)? {
        1 => Ok(BtcNetwork::Testnet),
        2 => Ok(BtcNetwork::Regtest),
        _ => Ok(BtcNetwork::Bitcoin),
    }
}

pub fn serialize_btc_block_in_db_format(
    btc_block_in_db_format: &BtcBlockInDbFormat,
) -> Result<(Bytes, Bytes)> {
    let serialized_id = btc_block_in_db_format.id.to_vec();
    Ok(
        (
            serialized_id.clone(),
            serde_json::to_vec(
                &SerializedBlockInDbFormat::new(
                    serialized_id,
                    btc_serialize(&btc_block_in_db_format.block),
                    convert_u64_to_bytes(&btc_block_in_db_format.height),
                    btc_block_in_db_format.extra_data.clone(),
                    serialize_minting_params(
                        &btc_block_in_db_format.minting_params
                    )?,
                )
            )?
        )
    )
}

pub fn deserialize_btc_block_in_db_format(
    serialized_block_in_db_format: &Bytes
) -> Result<BtcBlockInDbFormat> {
    let serialized_struct: SerializedBlockInDbFormat = serde_json::from_slice(
        &serialized_block_in_db_format
    )?;
    BtcBlockInDbFormat::new(
        convert_bytes_to_u64(&serialized_struct.height)?,
        sha256d::Hash::from_slice(&serialized_struct.id)?,
        deserialize_minting_params(
            serialized_struct.minting_params
        )?,
        btc_deserialize(&serialized_struct.block)?,
        serialized_struct.extra_data,
    )
}

pub fn get_safe_eth_address() -> EthAddress {
    EthAddress::from_slice(&SAFE_ETH_ADDRESS)
}

pub fn get_total_value_of_utxos_and_values(
    utxos_and_values: &BtcUtxosAndValues
) -> u64 {
   utxos_and_values
        .iter()
        .map(|utxo_and_value| utxo_and_value.value)
        .sum()
}

pub fn get_tx_id_from_signed_btc_tx(
    signed_btc_tx: &BtcTransaction
) -> String {
    let mut tx_id = signed_btc_tx
        .txid()
        .to_vec();
    tx_id.reverse();
    hex::encode(tx_id)
}

pub fn get_hex_tx_from_signed_btc_tx(
    signed_btc_tx: &BtcTransaction
) -> String {
    hex::encode(btc_serialize(signed_btc_tx))
}

pub fn get_script_sig<'a>(
    signature_slice: &'a[u8],
    utxo_spender_pub_key_slice: &'a[u8]
) -> BtcScript {
    let script_builder = BtcScriptBuilder::new();
    script_builder
        .push_slice(&signature_slice)
        .push_slice(&utxo_spender_pub_key_slice)
        .into_script()
}

pub fn create_new_tx_output(value: u64, script: BtcScript) -> Result<BtcTxOut> {
    Ok(BtcTxOut { value, script_pubkey: script })
}

pub fn create_new_pay_to_pub_key_hash_output(
    value: &u64,
    recipient: &str,
) -> Result<BtcTxOut> {
    create_new_tx_output(*value, get_pay_to_pub_key_hash_script(recipient)?)
}

pub fn calculate_btc_tx_fee(
    num_inputs: usize,
    num_outputs: usize,
    sats_per_byte: u64,
) -> u64 {
    calculate_btc_tx_size(num_inputs, num_outputs) * sats_per_byte
}

// NOTE: Assumes compressed keys and no multi-sigs!
pub fn calculate_btc_tx_size(num_inputs: usize, num_outputs: usize) -> u64 {
    ((num_inputs * (148 + PTOKEN_P2SH_SCRIPT_BYTES)) + (num_outputs * 34) + 10 + num_inputs) as u64
}

pub fn serialize_btc_utxo(btc_utxo: &BtcUtxo) -> Bytes {
    btc_serialize(btc_utxo)
}

pub fn deserialize_btc_utxo(bytes: &Bytes) -> Result<BtcUtxo> {
    Ok(btc_deserialize(bytes)?)
}

pub fn convert_btc_address_to_bytes(
    btc_address: &String
) -> Result<Bytes> {
    Ok(from_base58(btc_address)?)
}

pub fn convert_bytes_to_btc_address(encoded_bytes: Bytes) -> String {
    base58_encode_slice(&encoded_bytes[..])
}

pub fn convert_btc_address_to_pub_key_hash_bytes(
    btc_address: &str
) -> Result<Bytes> {
    Ok(from_base58(btc_address)?[1..21].to_vec())
}

pub fn get_pay_to_pub_key_hash_script(btc_address: &str) -> Result<BtcScript> {
    let script = BtcScriptBuilder::new();
    Ok(
        script
            .push_opcode(opcodes::all::OP_DUP)
            .push_opcode(opcodes::all::OP_HASH160)
            .push_slice(
                &convert_btc_address_to_pub_key_hash_bytes(btc_address)?[..]
            )
            .push_opcode(opcodes::all::OP_EQUALVERIFY)
            .push_opcode(opcodes::all::OP_CHECKSIG)
            .into_script()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use bitcoin::{
        util::address::Address as BtcAddress,
        hashes::{
            Hash,
            sha256d,
        },
    };
    use crate::{
        utils::convert_satoshis_to_ptoken,
        btc::{
            btc_types::MintingParamStruct,
            btc_test_utils::{
                get_sample_btc_utxo,
                SAMPLE_TRANSACTION_INDEX,
                SAMPLE_TARGET_BTC_ADDRESS,
                SAMPLE_SERIALIZED_BTC_UTXO,
                get_sample_btc_private_key,
                SAMPLE_OUTPUT_INDEX_OF_UTXO,
                get_sample_testnet_block_and_txs,
                get_sample_p2sh_redeem_script_sig,
                get_sample_btc_block_in_db_format,
                get_sample_op_return_utxo_and_value_n,
            },
        },
    };

    #[test]
    fn should_create_new_pay_to_pub_key_hash_output() {
        let expected_script = get_pay_to_pub_key_hash_script(
            SAMPLE_TARGET_BTC_ADDRESS
        ).unwrap();
        let value = 1;
        let result = create_new_pay_to_pub_key_hash_output(
            &value,
            SAMPLE_TARGET_BTC_ADDRESS
        ).unwrap();
        assert!(result.value == value);
        assert!(result.script_pubkey == expected_script);
    }

    #[test]
    fn should_create_new_tx_output() {
        let value = 1;
        let script = get_pay_to_pub_key_hash_script(SAMPLE_TARGET_BTC_ADDRESS)
            .unwrap();
        let result = create_new_tx_output(value, script.clone())
            .unwrap();
        assert!(result.value == value);
        assert!(result.script_pubkey == script);
    }

    #[test]
    fn should_calculate_btc_tx_size() {
        let expected_result = 193;
        let result = calculate_btc_tx_size(1, 1);
        assert!(result == expected_result);
    }

    #[test]
    fn should_serialize_btc_utxo() {
        let result = hex::encode(serialize_btc_utxo(&get_sample_btc_utxo()));
        assert!(result == SAMPLE_SERIALIZED_BTC_UTXO);
    }

    #[test]
    fn should_deserialize_btc_utxo() {
        let expected_vout = SAMPLE_OUTPUT_INDEX_OF_UTXO;
        let expected_witness_length = 0;
        let expected_sequence = 4294967295;
        let expected_txid = sha256d::Hash::from_str(
            "04bf43a86a99fca519dbfce42566b78cda0895d78c0a07484162d5888f588d0e"
        ).unwrap();
        let serialized_btc_utxo = hex::decode(SAMPLE_SERIALIZED_BTC_UTXO)
            .unwrap();
        let result = deserialize_btc_utxo(&serialized_btc_utxo)
            .unwrap();
        assert!(result.sequence == expected_sequence);
        assert!(result.previous_output.txid == expected_txid);
        assert!(result.previous_output.vout == expected_vout);
        assert!(result.witness.len() == expected_witness_length);
    }

    #[test]
    fn should_convert_btc_address_to_bytes() {
        let expected_result_hex =
            "6f54102783c8640c5144d039cea53eb7dbb470081462fbafd9";
        let result = convert_btc_address_to_bytes(
            &SAMPLE_TARGET_BTC_ADDRESS.to_string()
        ).unwrap();
        let result_hex = hex::encode(result);
        assert!(result_hex == expected_result_hex);
    }

    #[test]
    fn should_convert_bytes_to_btc_address() {
        let bytes = convert_btc_address_to_bytes(
            &SAMPLE_TARGET_BTC_ADDRESS.to_string()
        ).unwrap();
        let result = convert_bytes_to_btc_address(bytes);
        assert!(result == SAMPLE_TARGET_BTC_ADDRESS);
    }

    #[test]
    fn should_convert_btc_address_to_pub_key_hash_bytes() {
        let expected_result = "54102783c8640c5144d039cea53eb7dbb4700814";
        let result = convert_btc_address_to_pub_key_hash_bytes(
            SAMPLE_TARGET_BTC_ADDRESS
        ).unwrap();
        assert!(hex::encode(result) == expected_result);
    }

    #[test]
    fn should_get_pay_to_pub_key_hash_script() {
        let example_script = get_sample_testnet_block_and_txs()
            .unwrap()
            .block
            .txdata[SAMPLE_TRANSACTION_INDEX as usize]
            .output[SAMPLE_OUTPUT_INDEX_OF_UTXO as usize]
            .script_pubkey
            .clone();
        let expected_result =
            "76a91454102783c8640c5144d039cea53eb7dbb470081488ac";
        let result_script = get_pay_to_pub_key_hash_script(
            SAMPLE_TARGET_BTC_ADDRESS
        ).unwrap();
        let hex_result = hex::encode(result_script.as_bytes());
        assert!(!result_script.is_p2sh());
        assert!(result_script.is_p2pkh());
        assert!(hex_result == expected_result);
        assert!(result_script == example_script);
    }

    #[test]
    fn should_get_script_sig() {
        let expected_result = "4730440220275e800c20aa5096a49e6c36aae8f532093fc3fdc4a1dd6039314b250efd62300220492fe4b7e27bf555648f023811fb2258bbcd057fd54967f96942cf1f606e4fe7012103d2a5e3b162eb580fe2ce023cd5e0dddbb6286923acde77e3e5468314dc9373f7";
        let hash_type = 1;
        let hash = sha256d::Hash::hash(b"a message");
        let btc_pk = get_sample_btc_private_key();
        let signature = btc_pk
            .sign_hash_and_append_btc_hash_type(hash.to_vec(), hash_type)
            .unwrap();
        let pub_key_slice = btc_pk.to_public_key_slice();
        let result_script = get_script_sig(&signature, &pub_key_slice);
        let hex_result = hex::encode(result_script.as_bytes());
        assert!(hex_result == expected_result);
    }

    #[test]
    fn should_get_total_value_of_utxos_and_values() {
        let expected_result = 1942233;
        let utxos = vec![
            get_sample_op_return_utxo_and_value_n(2)
                .unwrap(),
            get_sample_op_return_utxo_and_value_n(3)
                .unwrap(),
            get_sample_op_return_utxo_and_value_n(4)
                .unwrap(),
        ];
        let result = get_total_value_of_utxos_and_values(&utxos);
        assert!(result == expected_result);
    }

    #[test]
    fn should_serde_minting_params() {
        let expected_serialization =  vec![
91, 123, 34, 97, 109, 111, 117, 110, 116, 34, 58, 34, 48, 120, 99, 50, 56, 102, 50, 49, 57, 99, 52, 48, 48, 34, 44, 34, 101, 116, 104, 95, 97, 100, 100, 114, 101, 115, 115, 34, 58, 34, 48, 120, 102, 101, 100, 102, 101, 50, 54, 49, 54, 101, 98, 51, 54, 54, 49, 99, 98, 56, 102, 101, 100, 50, 55, 56, 50, 102, 53, 102, 48, 99, 99, 57, 49, 100, 53, 57, 100, 99, 97, 99, 34, 44, 34, 111, 114, 105, 103, 105, 110, 97, 116, 105, 110, 103, 95, 116, 120, 95, 104, 97, 115, 104, 34, 58, 34, 57, 101, 56, 100, 100, 50, 57, 102, 48, 56, 51, 57, 56, 100, 55, 97, 100, 102, 57, 50, 53, 50, 56, 97, 99, 49, 49, 51, 98, 99, 99, 55, 51, 54, 102, 55, 97, 100, 99, 100, 55, 99, 57, 57, 101, 101, 101, 48, 52, 54, 56, 97, 57, 57, 50, 99, 56, 49, 102, 51, 101, 97, 57, 56, 34, 44, 34, 111, 114, 105, 103, 105, 110, 97, 116, 105, 110, 103, 95, 116, 120, 95, 97, 100, 100, 114, 101, 115, 115, 34, 58, 34, 50, 78, 50, 76, 72, 89, 98, 116, 56, 75, 49, 75, 68, 66, 111, 103, 100, 54, 88, 85, 71, 57, 86, 66, 118, 53, 89, 77, 54, 120, 101, 102, 100, 77, 50, 34, 125, 93
                ];
        let amount = convert_satoshis_to_ptoken(1337);
        let originating_tx_address = BtcAddress::from_str(
            "2N2LHYbt8K1KDBogd6XUG9VBv5YM6xefdM2"
        ).unwrap();
        let eth_address = EthAddress::from_slice(
            &hex::decode("fedfe2616eb3661cb8fed2782f5f0cc91d59dcac").unwrap()
        );
        let originating_tx_hash = sha256d::Hash::from_slice(&hex::decode(
        "98eaf3812c998a46e0ee997ccdadf736c7bc13c18a5292df7a8d39089fd28d9e"
            ).unwrap()
        ).unwrap();
        let minting_param_struct = MintingParamStruct::new(
            amount,
            eth_address,
            originating_tx_hash,
            originating_tx_address,
        );
        let minting_params = vec![minting_param_struct];
        let serialized_minting_params = serialize_minting_params(
            &minting_params
        ).unwrap();
        assert!(serialized_minting_params == expected_serialization);
        let deserialized = deserialize_minting_params(serialized_minting_params)
            .unwrap();
        assert!(deserialized.len() == minting_params.len());
        deserialized
            .iter()
            .enumerate()
            .map(|(i, minting_param_struct)|
                 assert!(minting_param_struct == &minting_params[i])
             )
            .for_each(drop);
    }

    #[test]
    fn should_serde_btc_block_in_db_format() {
        let block = get_sample_btc_block_in_db_format()
            .unwrap();
        let (_db_key, serialized_block)= serialize_btc_block_in_db_format(
            &block
        ).unwrap();
        let deserialized = deserialize_btc_block_in_db_format(&serialized_block)
            .unwrap();
        assert!(deserialized == block);
    }

    #[test]
    fn should_get_p2sh_redeem_script_sig() {
        let result = get_sample_p2sh_redeem_script_sig();
        let result_hex = hex::encode(result.as_bytes());
        let expected_result = "2071a8e55edefe53f703646a679e66799cfef657b98474ff2e4148c3a1ea43169c752103d2a5e3b162eb580fe2ce023cd5e0dddbb6286923acde77e3e5468314dc9373f7ac";
        assert!(result_hex == expected_result);
    }

    #[test]
    fn should_get_p2sh_script_sig_from_redeem_script() {
        let signature_slice = &vec![6u8, 6u8, 6u8][..];
        let redeem_script = get_sample_p2sh_redeem_script_sig();
        let expected_result = "03060606452071a8e55edefe53f703646a679e66799cfef657b98474ff2e4148c3a1ea43169c752103d2a5e3b162eb580fe2ce023cd5e0dddbb6286923acde77e3e5468314dc9373f7ac";
        let result = get_p2sh_script_sig_from_redeem_script(
            &signature_slice,
            &redeem_script,
        );
        let result_hex = hex::encode(result.as_bytes());
        assert!(result_hex == expected_result);
    }

    #[test]
    fn should_create_unsigned_utxo_from_tx() {
        let expected_result = "f80c2f7c35f5df8441a5a5b52e2820793fc7e69f4603d38ba7217be41c20691d0000000016001497cfc76442fe717f2a3f0cc9c175f7561b661997ffffffff";
        let index = 0;
        let tx = get_sample_btc_block_in_db_format()
            .unwrap()
            .block
            .txdata[0]
            .clone();
        let result = create_unsigned_utxo_from_tx(&tx, index);
        let result_hex = hex::encode(btc_serialize(&result));
        assert!(result_hex == expected_result);
    }

    #[test]
    fn should_create_op_return_btc_utxo_and_value_from_tx_output() {
        let expected_value = 1261602424;
        let expected_utxo = "f80c2f7c35f5df8441a5a5b52e2820793fc7e69f4603d38ba7217be41c20691d0000000016001497cfc76442fe717f2a3f0cc9c175f7561b661997ffffffff";
        let index = 0;
        let tx = get_sample_btc_block_in_db_format()
            .unwrap()
            .block
            .txdata[0]
            .clone();
        let result = create_op_return_btc_utxo_and_value_from_tx_output(
            &tx,
            index,
        );
        assert!(result.maybe_pointer == None);
        assert!(result.value == expected_value);
        assert!(result.maybe_extra_data == None);
        assert!(result.maybe_deposit_info_json == None);
        assert!(hex::encode(result.serialized_utxo) == expected_utxo);
    }

    #[test]
    fn should_serde_btc_network_correctly() {
        let network = BtcNetwork::Bitcoin;
        let bytes = convert_btc_network_to_bytes(&network)
            .unwrap();
        let result = convert_bytes_to_btc_network(&bytes)
            .unwrap();
        assert!(result == network);
    }

    #[test]
    fn should_serde_btc_block_in_db_format_correctly() {
        let block_in_db_format = get_sample_btc_block_in_db_format()
            .unwrap();
        let (id, serialized_block) = serialize_btc_block_in_db_format(
            &block_in_db_format
        ).unwrap();
        assert!(id == &block_in_db_format.id[..]);
        let result = deserialize_btc_block_in_db_format(&serialized_block)
            .unwrap();
        assert!(result == block_in_db_format);
    }

    #[test]
    fn should_get_safe_eth_address() {
        let expected_result = "71a440ee9fa7f99fb9a697e96ec7839b8a1643b8";
        let result = get_safe_eth_address();
        assert!(hex::encode(result.as_bytes()) == expected_result);
    }
}
