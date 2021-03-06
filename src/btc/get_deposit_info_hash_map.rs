use std::collections::HashMap;
use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc::{
        btc_state::BtcState,
        btc_types::{
            DepositInfoList,
            DepositInfoHashMap,
        },
    },
};

pub fn create_hash_map_from_deposit_info_list(
    deposit_info_list: &DepositInfoList
) -> Result<DepositInfoHashMap> {
    let mut hash_map = HashMap::new();
    deposit_info_list
        .iter()
        .map(|deposit_info|
             hash_map.insert(
                 deposit_info.btc_deposit_address.clone(),
                 deposit_info.clone()
             )
         )
        .for_each(drop);
    Ok(hash_map)
}

pub fn get_deposit_info_hash_map_and_put_in_state<D>(
    state: BtcState<D>
) -> Result<BtcState<D>>
    where D: DatabaseInterface
{
    create_hash_map_from_deposit_info_list(
        &state.get_btc_block_and_id()?.deposit_address_list
    )
        .and_then(|hash_map| state.add_deposit_info_hash_map(hash_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc::btc_test_utils::get_sample_btc_block_and_id;

    #[test]
    fn should_create_hash_map_from_deposit_info_list() {
        let address_info_list = get_sample_btc_block_and_id()
            .unwrap()
            .deposit_address_list
            .clone();
        let result = create_hash_map_from_deposit_info_list(&address_info_list)
            .unwrap();
        assert!(!result.is_empty());
        assert!(result.len() == address_info_list.len());
        result
            .iter()
            .map(|(key, value)| assert!(key == &value.btc_deposit_address))
            .for_each(drop);
    }
}
