use crate::{
    errors::AppError,
    traits::DatabaseInterface,
    types::{
        Bytes,
        Result,
    },
    eth::{
        eth_state::EthState,
        eth_types::EthBlockAndReceipts,
        eth_crypto::eth_transaction::get_ptoken_smart_contract_bytecode,
        eth_database_utils::{
            put_eth_chain_id_in_db,
            put_eth_gas_price_in_db,
            put_eth_account_nonce_in_db,
            put_eth_tail_block_hash_in_db,
            put_eth_canon_block_hash_in_db,
            put_eth_anchor_block_hash_in_db,
            put_eth_latest_block_hash_in_db,
            put_eth_block_and_receipts_in_db,
            put_eth_canon_to_tip_length_in_db,
        },
    },

};

pub fn check_for_existence_of_eth_contract_byte_code() -> Result<Bytes> {
    get_ptoken_smart_contract_bytecode()
}

pub fn put_eth_tail_block_hash_in_db_and_return_state<D>(
    state: EthState<D>
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    info!("✔ Putting ETH tail block has in db...");
    put_eth_tail_block_hash_in_db(
        &state.db,
        &state.get_eth_block_and_receipts()?.block.hash
    )
        .map(|_| state)
}

fn set_hash_from_block_in_state<D>(
    state: EthState<D>,
    hash_type: &str,
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    let hash = &state.get_eth_block_and_receipts()?.block.hash;
    match hash_type {
        "canon" => {
            info!("✔ Initializating ETH canon block hash...");
            put_eth_canon_block_hash_in_db(&state.db, hash)
        },
        "latest" => {
            info!("✔ Initializating ETH latest block hash...");
            put_eth_latest_block_hash_in_db(&state.db, hash)
        }
        "anchor" => {
            info!("✔ Initializating ETH anchor block hash...");
            put_eth_anchor_block_hash_in_db(&state.db, hash)
        }
        _ => Err(AppError::Custom("✘ Hash type not recognized!".to_string()))
    }?;
    Ok(state)
}

pub fn set_eth_latest_block_hash_and_return_state<D>(
    state: EthState<D>
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    set_hash_from_block_in_state(state, "latest")
}

pub fn set_eth_anchor_block_hash_and_return_state<D>(
    state: EthState<D>
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    set_hash_from_block_in_state(state, "anchor")
}

pub fn set_eth_canon_block_hash_and_return_state<D>(
    state: EthState<D>
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    set_hash_from_block_in_state(state, "canon")
}

pub fn put_canon_to_tip_length_in_db_and_return_state<D>(
    canon_to_tip_length: u64,
    state: EthState<D>,
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    put_eth_canon_to_tip_length_in_db(&state.db, &canon_to_tip_length)
        .map(|_| state)
}

pub fn put_eth_chain_id_in_db_and_return_state<D>(
    chain_id: u8,
    state: EthState<D>
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    trace!(
        "✔ Putting ETH chain ID of {} in db...",
        chain_id,
    );
    put_eth_chain_id_in_db(&state.db, &chain_id)
        .map(|_| state)
}

pub fn put_eth_gas_price_in_db_and_return_state<D>(
    gas_price: u64,
    state: EthState<D>,
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    trace!(
        "✔ Putting ETH gas price of {} in db...",
        gas_price
    );
    put_eth_gas_price_in_db(&state.db, &gas_price)
        .map(|_| state)
}

pub fn put_eth_account_nonce_in_db_and_return_state<D>(
    state: EthState<D>,
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    trace!("✔ Putting ETH account nonce of 1 in db...");
    put_eth_account_nonce_in_db(&state.db, &1) // NOTE: ∵ of the contract tx!
        .map(|_| state)
}

pub fn remove_receipts_from_block_in_state<D>( // ∵ there shouldn't be relevant txs!
    state: EthState<D>
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    trace!("✔ Removing receipts from ETH block in state...");
    let block_with_no_receipts = EthBlockAndReceipts {
        block: state.get_eth_block_and_receipts()?.block.clone(),
        receipts: vec![],
    };
    state.update_eth_block_and_receipts(block_with_no_receipts)
}

pub fn add_eth_block_to_db_and_return_state<D>(
    state: EthState<D>
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    trace!("✔ Adding ETH block and receipts to db...",);
    put_eth_block_and_receipts_in_db(
        &state.db,
        state.get_eth_block_and_receipts()?
    )
        .map(|_| state)
}
