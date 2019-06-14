extern crate ethabi;
extern crate ethereum_types;

use super::error::{Error, ErrorKind};
use super::state_update::StateUpdate;
use ethabi::Token;
use ethereum_types::H256;

#[derive(Clone, Debug)]
/// ## struct Block
/// - has many `state_updates`
/// - has a `merkle root hash`
/// - Traits
///   - Encodable
///   - Decodable
pub struct Block {
    state_updates: Vec<StateUpdate>,
    root: H256,
}

impl Block {
    /// ### Block.new
    /// A constructor of a Block struct
    /// ```ignore
    /// let block = Block.new(&txs, root)
    /// ```
    pub fn new(state_updates: &[StateUpdate], root: H256) -> Block {
        Block {
            state_updates: state_updates.to_vec(),
            root,
        }
    }

    pub fn to_abi(&self) -> Vec<u8> {
        let state_update_tokens = self
            .state_updates
            .iter()
            .map(|state_update| Token::Bytes(state_update.to_abi()))
            .collect();
        ethabi::encode(&[
            Token::Array(state_update_tokens),
            Token::Bytes(self.root.as_bytes().to_vec()),
        ])
    }
    pub fn from_abi(data: &[u8]) -> Result<Self, Error> {
        let decoded: Vec<Token> = ethabi::decode(
            &[
                ethabi::ParamType::Array(Box::new(ethabi::ParamType::Bytes)),
                ethabi::ParamType::Bytes,
            ],
            data,
        )
        .map_err::<Error, _>(Into::into)?;
        let decoded_array = decoded[0].clone().to_array();
        let root = decoded[1].clone().to_bytes();

        if let (Some(decoded_array), Some(root)) = (decoded_array, root) {
            let state_updates = decoded_array
                .to_vec()
                .iter()
                .map(|token| {
                    StateUpdate::from_abi(&token.clone().to_bytes().unwrap())
                        .ok()
                        .unwrap()
                })
                .collect::<Vec<_>>();
            Ok(Block::new(&state_updates, H256::from_slice(&root)))
        } else {
            Err(Error::from(ErrorKind::AbiDecode))
        }
    }
}
