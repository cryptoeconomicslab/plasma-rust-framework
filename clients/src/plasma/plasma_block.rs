use bytes::Bytes;
use ethabi::{ParamType, Token};
use merkle_interval_tree::{MerkleIntervalNode, MerkleIntervalTree};
use ovm::types::core::Integer;
use ovm::types::StateUpdate;
use plasma_core::data_structure::abi::{Decodable, Encodable};
use plasma_core::data_structure::error::{
    Error as PlasmaCoreError, ErrorKind as PlasmaCoreErrorKind,
};

pub struct PlasmaBlock {
    block_number: Integer,
    state_updates: Vec<StateUpdate>,
    tree: Option<MerkleIntervalTree<u64>>,
}

impl PlasmaBlock {
    pub fn new(block_number: u64, state_updates: Vec<StateUpdate>) -> Self {
        Self {
            block_number: Integer::new(block_number),
            state_updates,
            tree: None,
        }
    }

    pub fn get_block_number(&self) -> u64 {
        self.block_number.0
    }

    pub fn get_root(&self) -> Option<Bytes> {
        if let Some(tree) = &self.tree {
            Some(Bytes::from(tree.get_root()))
        } else {
            None
        }
    }

    pub fn get_inclusion_proof_with_index(&self, index: usize) -> Option<Bytes> {
        if let Some(tree) = &self.tree {
            Some(tree.get_inclusion_proof(index, 2))
        } else {
            None
        }
    }

    pub fn get_inclusion_proof(&self, state_update: StateUpdate) -> Option<Bytes> {
        if let Some(index) = self
            .state_updates
            .iter()
            .position(|s| s.get_hash() == state_update.get_hash())
        {
            self.get_inclusion_proof_with_index(index)
        } else {
            None
        }
    }

    pub fn merkelize(&mut self) {
        let mut leaves = vec![];
        for s in self.state_updates.iter() {
            leaves.push(MerkleIntervalNode::Leaf {
                end: s.get_range().get_end(),
                data: Bytes::from(s.to_abi()),
            });
        }

        let tree = MerkleIntervalTree::generate(&leaves);
        self.tree = Some(tree);
    }
}

impl Encodable for PlasmaBlock {
    fn to_tuple(&self) -> Vec<Token> {
        vec![
            Token::Uint(self.block_number.0.into()),
            Token::Array(
                self.state_updates
                    .iter()
                    .map(|s| Token::Tuple(s.to_tuple()))
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
        if let (Some(block_number), Some(state_updates)) = (block_number, state_updates) {
            let state_updates: Result<Vec<_>, _> = state_updates
                .iter()
                .map(|s| {
                    if let Token::Tuple(v) = s {
                        StateUpdate::from_tuple(v)
                    } else {
                        Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))
                    }
                })
                .collect();
            if let Ok(s) = state_updates {
                Ok(PlasmaBlock {
                    block_number: Integer(block_number.as_u64()),
                    state_updates: s,
                    tree: None,
                })
            } else {
                return Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode));
            }
        } else {
            Err(PlasmaCoreError::from(PlasmaCoreErrorKind::AbiDecode))
        }
    }

    fn get_param_types() -> Vec<ParamType> {
        vec![
            ParamType::Uint(64),
            ParamType::Array(Box::new(ParamType::Tuple(StateUpdate::get_param_types()))),
        ]
    }
}
