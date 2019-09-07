use super::block_db::BlockDb;
use super::plasma_block::PlasmaBlock;
use contract_wrapper::commitment_contract_adaptor::CommitmentContractAdaptor;
use ethabi::Contract as ContractABI;
use ethereum_types::Address;
use ovm::types::StateUpdate;
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
    current_block_number: u64,
}

impl<KVS: KeyValueStore + DatabaseTrait> BlockManager<KVS> {
    pub fn new(aggregator_address: Address, commitment_contract_address: Address) -> Self {
        let db = KVS::open("plasma_aggregator_db");
        let db = RangeDbImpl::from(db);

        BlockManager {
            aggregator_address,
            commitment_contract_address,
            db,
            current_block_number: 0,
        }
    }

    pub fn get_queued_state_updates(&self) -> Vec<StateUpdate> {
        let block_db = BlockDb::from(&self.db);
        block_db.get_pending_state_updates().unwrap()
    }

    pub fn enqueue_state_update(&self, state_update: StateUpdate) -> Result<(), Error> {
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
        let mut block = PlasmaBlock::new(self.get_next_block_number(), state_updates);

        block.merkelize();
        let root = block.get_root().unwrap();

        // send root hash to commitment contract
        let f = File::open("CommitmentContract.json").unwrap();
        let reader = BufReader::new(f);
        let contract_abi = ContractABI::load(reader).unwrap();
        let contract = CommitmentContractAdaptor::new(
            "http://127.0.0.1:9545",
            &self.commitment_contract_address.to_string(),
            contract_abi,
        )
        .unwrap();
        let _ = contract.submit_block(self.aggregator_address, root);

        let _ = block_db.save_block(&block);
    }

    pub fn get_next_block_number(&self) -> u64 {
        self.current_block_number + 1
    }
}
