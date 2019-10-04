use super::command::NewTransactionEvent;
use super::error::{Error, ErrorKind};
use abi_utils::{Decodable, Encodable, Error as PlasmaCoreError, ErrorKind as PlasmaCoreErrorKind};
use bytes::Bytes;
use ethabi::{ParamType, Token};
use ethereum_types::Address;
use merkle_interval_tree::{DoubleLayerTree, DoubleLayerTreeLeaf};
use ovm::types::core::Integer;
use ovm::types::StateUpdate;

pub struct PlasmaBlock {
    block_number: Integer,
    state_updates: Vec<StateUpdate>,
    transactions: Vec<NewTransactionEvent>,
    tree: Option<DoubleLayerTree>,
}

impl PlasmaBlock {
    pub fn new(
        block_number: u64,
        state_updates: Vec<StateUpdate>,
        transactions: Vec<NewTransactionEvent>,
    ) -> Self {
        Self {
            block_number: Integer::new(block_number),
            state_updates,
            transactions,
            tree: None,
        }
    }

    pub fn get_block_number(&self) -> u64 {
        self.block_number.0
    }

    pub fn get_state_updates(&self) -> &[StateUpdate] {
        &self.state_updates
    }

    pub fn get_transactions(&self) -> &[NewTransactionEvent] {
        &self.transactions
    }

    pub fn get_root(&self) -> Option<Bytes> {
        if let Some(tree) = &self.tree {
            Some(tree.get_root())
        } else {
            None
        }
    }

    pub fn get_inclusion_proof_with_index(&self, address: Address, index: usize) -> Option<Bytes> {
        if let Some(tree) = &self.tree {
            Some(tree.get_inclusion_proof(address, index))
        } else {
            None
        }
    }

    pub fn get_inclusion_proof(&self, state_update: StateUpdate) -> Option<Bytes> {
        // TODO: we shoud use tree.get_index(data)
        if let Some(tree) = &self.tree {
            let index = tree.get_index(
                state_update.get_deposit_contract_address(),
                &Bytes::from(state_update.to_abi()),
            );
            self.get_inclusion_proof_with_index(state_update.get_deposit_contract_address(), index)
        } else {
            None
        }
    }
    pub fn get_exclusion_proof(&self, state_update: StateUpdate) -> Option<Bytes> {
        // TODO: we shoud use tree.get_index(data)
        if let Some(tree) = &self.tree {
            let index = tree.get_index_by_end(
                state_update.get_deposit_contract_address(),
                state_update.get_range().get_end(),
            );
            self.get_inclusion_proof_with_index(state_update.get_deposit_contract_address(), index)
        } else {
            None
        }
    }

    pub fn merkelize(&mut self) -> Result<Bytes, Error> {
        if self.state_updates.is_empty() {
            return Err(Error::from(ErrorKind::MerkelizingError));
        }
        let mut leaves = vec![];
        for s in self.state_updates.iter() {
            leaves.push(DoubleLayerTreeLeaf {
                address: s.get_deposit_contract_address(),
                end: s.get_range().get_end(),
                data: Bytes::from(s.to_abi()),
            });
        }

        let tree = DoubleLayerTree::generate(&leaves);
        self.tree = Some(tree);
        if let Some(root) = self.get_root() {
            Ok(root)
        } else {
            Err(Error::from(ErrorKind::MerkelizingError))
        }
    }
}

impl Encodable for PlasmaBlock {
    fn to_tuple(&self) -> Vec<Token> {
        vec![
            Token::Uint(self.block_number.0.into()),
            Token::Array(
                self.state_updates
                    .iter()
                    .map(|s| Token::Bytes(s.to_abi()))
                    .collect(),
            ),
            Token::Array(
                self.transactions
                    .iter()
                    .map(|t| Token::Bytes(t.to_abi()))
                    .collect(),
            ),
        ]
    }
}

impl Decodable for PlasmaBlock {
    type Ok = PlasmaBlock;
    fn from_tuple(tuple: &[Token]) -> Result<Self, PlasmaCoreError> {
        let block_number = tuple[0].clone().to_uint();
        let state_updates = tuple[1].clone().to_array();
        let transactions = tuple[2].clone().to_array();
        if let (Some(block_number), Some(state_updates), Some(transactions)) =
            (block_number, state_updates, transactions)
        {
            let state_updates: Result<Vec<_>, _> = state_updates
                .iter()
                .map(|s| {
                    if let Token::Bytes(v) = s {
                        StateUpdate::from_abi(v)
                    } else {
                        Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))
                    }
                })
                .collect();
            let transactions: Result<Vec<_>, _> = transactions
                .iter()
                .map(|tx| {
                    if let Token::Bytes(t) = tx {
                        NewTransactionEvent::from_abi(t)
                    } else {
                        Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))
                    }
                })
                .collect();
            if let (Ok(s), Ok(t)) = (state_updates, transactions) {
                Ok(PlasmaBlock {
                    block_number: Integer(block_number.as_u64()),
                    state_updates: s,
                    transactions: t,
                    tree: None,
                })
            } else {
                Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))
            }
        } else {
            Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))
        }
    }

    fn get_param_types() -> Vec<ParamType> {
        vec![
            ParamType::Uint(64),
            ParamType::Array(Box::new(ParamType::Bytes)),
            ParamType::Array(Box::new(ParamType::Bytes)),
        ]
    }
}
