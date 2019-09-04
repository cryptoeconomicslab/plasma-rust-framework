use super::block_db::BlockDb;
use bytes::Bytes;
use contract_wrapper::commitment_contract_adaptor::CommitmentContractAdaptor;
use ethabi::Contract as ContractABI;
use ethereum_types::Address;
use merkle_interval_tree::{MerkleIntervalNode, MerkleIntervalTree};
use ovm::types::PlasmaDataBlock;
use plasma_core::data_structure::abi::Encodable;
use plasma_db::error::Error;
use plasma_db::traits::db::DatabaseTrait;
use plasma_db::traits::kvs::KeyValueStore;
use plasma_db::RangeDbImpl;
use std::fs::File;
use std::io::BufReader;

pub struct BlockManager<KVS: KeyValueStore> {
    db: RangeDbImpl<KVS>,
    commitment_contract_address: Address,
    aggregator_address: Address,
}

impl<KVS: KeyValueStore + DatabaseTrait> BlockManager<KVS> {
    pub fn new(aggregator_address: Address, commitment_contract_address: Address) -> Self {
        let db = KVS::open("plasma_aggregator_db");
        let db = RangeDbImpl::from(db);

        BlockManager {
            aggregator_address,
            commitment_contract_address,
            db,
        }
    }

    pub fn enqueue_state_update(&self, state_update: PlasmaDataBlock) -> Result<(), Error> {
        let block_db = BlockDb::from(&self.db);
        block_db.enqueue_state_update(state_update)
    }

    /// generate block from queued state updates
    /// save block in block_db, submit to CommitmentContract
    /// return generated block
    pub fn submit_next_block(&self) {
        let block_db = BlockDb::from(&self.db);
        let result = block_db.get_pending_state_updates();
        if result.is_err() {
            return;
        }
        let state_updates = result.unwrap();

        let mut leaves = vec![];
        for s in state_updates.iter() {
            leaves.push(MerkleIntervalNode::Leaf {
                end: s.get_updated_range().get_end(),
                data: Bytes::from(s.to_abi()),
            });
        }

        let tree = MerkleIntervalTree::generate(&leaves);
        let root = tree.get_root();

        // send root hash to commitment contract
        let f = File::open("CommitmentContract.json").unwrap();
        let reader = BufReader::new(f);
        let contract_abi = ContractABI::load(reader).unwrap();
        let contract = CommitmentContractAdaptor::new(
            "http://127.0.0.1:9545",
            &self.commitment_contract_address.clone().to_string(),
            contract_abi,
        )
        .unwrap();
        let _ = contract.submit_block(self.aggregator_address, root);

        // TODO: return included state_updates and proofs.
    }
}

#[cfg(test)]
mod tests {
    use super::BlockManager;
    use bytes::Bytes;
    use ethereum_types::Address;
    use ovm::types::{Integer, PlasmaDataBlock, Property, SignedByInput};
    use plasma_core::data_structure::Range;
    use plasma_db::impls::kvs::memory::CoreDbMemoryImpl;

    #[test]
    fn test_submit_next_block() {
        let block_manager = BlockManager::<CoreDbMemoryImpl>::new();
        let address: Address = Address::zero();
        let plasma_data_block = PlasmaDataBlock::new(
            Integer::new(1),
            Range::new(0, 100),
            true,
            Property::SignedByDecider(SignedByInput::new(Bytes::from(&b"hi"[..]), address)),
            Bytes::from(&b"root"[..]),
        );

        block_manager.enqueue_state_update(plasma_data_block);
        block_manager.submit_next_block();
        assert!(false);
    }

}
