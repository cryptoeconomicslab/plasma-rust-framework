// plasma_clients::plasma::query is Examples of query to StateUpdate list.
use abi_utils::Integer;
use ethereum_types::Address;
use ovm::types::{PropertyInput, StateUpdate};
use std::collections::HashMap;

/// Filters all ownership properties and compute balance
pub fn query_balance(
    state_updates: Vec<StateUpdate>,
    my_address: Address,
) -> HashMap<Address, u64> {
    let balances: HashMap<Address, u64> = state_updates
        .iter()
        .filter(|s| {
            let p = &s.get_property().inputs[2];
            if let PropertyInput::ConstantProperty(signed_by) = p {
                if let PropertyInput::ConstantAddress(address) = signed_by.inputs[0] {
                    return address == my_address;
                }
            }
            false
        })
        .fold(HashMap::new(), |mut acc, s| {
            let deposit_contract = s.get_deposit_contract_address();
            let b = acc.get(&deposit_contract).unwrap_or(&0);
            let new_balance = b + s.get_range().get_end() - s.get_range().get_start();
            acc.insert(deposit_contract, new_balance);
            acc
        });
    balances
}

/// Filters all making order properties
pub fn query_orders(
    state_updates: Vec<StateUpdate>,
) -> Vec<(StateUpdate, Address, Integer, Address)> {
    state_updates
        .iter()
        .filter_map(|s| {
            let or = &s.get_property().inputs[2];
            if let PropertyInput::ConstantProperty(or) = or {
                if let PropertyInput::ConstantProperty(verify_tx) = &or.inputs[0] {
                    if let PropertyInput::ConstantAddress(token_address) = verify_tx.inputs[1] {
                        if let PropertyInput::ConstantInteger(amount) = verify_tx.inputs[2] {
                            if let PropertyInput::ConstantAddress(maker_address) =
                                verify_tx.inputs[3]
                            {
                                return Some((s.clone(), token_address, amount, maker_address));
                            }
                        }
                    }
                }
            }
            None
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use plasma_core::data_structure::Range;

    #[test]
    fn test_query_orders() {
        let property = ovm::statements::plasma::create_making_order_state_object(
            Address::zero(),
            Address::zero(),
            Integer(100),
        );
        let state_update_list = vec![StateUpdate::new(
            Integer::new(7),
            Address::zero(),
            Range::new(5, 7),
            property,
        )];
        let result = query_orders(state_update_list);
        assert_eq!(result.len(), 1);
    }
}
