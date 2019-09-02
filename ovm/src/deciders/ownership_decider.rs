use plasma_core::data_structure::Transaction;
use ovm::types::PlasmaDataBlock;

/// OwnershipInput {
///     pre_state,
///     transaction,
///     post_state
/// }
///
/// 1. verify transaction have prev_state owner's signature.
/// 2. prev_state's block_number is less than origin_block_number
/// 3. ensure post_state's range is same as transaction's range.
/// 4. transaction.parameters.new_state is same as post_state.state
/// OwnershipDecider {
///     decide(self, input) {
///     }
///
/// }
///

struct OwnershipPredicate {}

/// OwnershipDecider property construction
/// Transaction have property which is
/// Input = {
///     prev_state: PlasmaDataBlock,
///     transaction: Transaction,
///     post_state: PlasmaDataBlock,
/// }
/// property.input have to have owner address.
/// PlasmaDataBlock have owner address in property.input
/// Use SignedByDecider's address.

impl OwnershipPredicate {
    /// verify transaction
    /// 1. verify transaction have prev_state owner's signature.
    /// 2. prev_state's block_number is less than origin_block_number
    /// 3. ensure post_state's range is same as transaction's range.
    /// 4. transaction.parameters.new_state is same as post_state.state
    pub fn verify_transaction(prev_state: PlasmaDataBlock, transaction: Transaction, post_state: PlasmaDataBlock) -> bool {
        // verify transaction have valid signature of prev_state.parameters.owner.
        let input = SignedByInput::new(transaction.signature, prev_state.parameters.owner);
        let property = Property::SignedByDecider(input.clone());
        let decider: PropertyExecutor<CoreDbMemoryImpl> = Default::default();
        if !decider.decide(&property).unwrap().get_outcome() {
            return false
        }
        if transaction.get_range() != post_state.get_updated_range() {
            return false
        }

    }

    pub fn execute_state_transition(prev_state: PlasmaDataBlock, transaction: Transaction) -> PlasmaDataBlock {
        PlasmaDataBlock::new()
    }
}

