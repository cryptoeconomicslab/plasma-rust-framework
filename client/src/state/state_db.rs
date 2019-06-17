extern crate ethabi;

use crate::error::{Error, ErrorKind};
use ethabi::Token;
use plasma_core::data_structure::StateUpdate;
use plasma_db::impls::rangedb::RangeDbImpl;
use plasma_db::range::Range;
use plasma_db::traits::{DatabaseTrait, KeyValueStore, RangeStore};

#[derive(Clone, Debug, PartialEq)]
pub struct VerifiedStateUpdate {
    start: u64,
    end: u64,
    verified_block_number: u64,
    state_update: StateUpdate,
}

impl VerifiedStateUpdate {
    pub fn new(
        start: u64,
        end: u64,
        verified_block_number: u64,
        state_update: StateUpdate,
    ) -> Self {
        VerifiedStateUpdate {
            start,
            end,
            verified_block_number,
            state_update,
        }
    }
    pub fn from(verified_block_number: u64, state_update: &StateUpdate) -> Self {
        VerifiedStateUpdate {
            start: state_update.get_start(),
            end: state_update.get_end(),
            verified_block_number,
            state_update: state_update.clone(),
        }
    }
    pub fn get_start(&self) -> u64 {
        self.start
    }
    pub fn get_end(&self) -> u64 {
        self.end
    }
    pub fn get_state_update(&self) -> &StateUpdate {
        &self.state_update
    }
    pub fn to_abi(&self) -> Vec<u8> {
        ethabi::encode(&[
            Token::Uint(self.start.into()),
            Token::Uint(self.end.into()),
            Token::Uint(self.verified_block_number.into()),
            Token::Bytes(self.state_update.to_abi()),
        ])
    }
    pub fn from_abi(data: &[u8]) -> Result<Self, Error> {
        let decoded: Vec<Token> = ethabi::decode(
            &[
                ethabi::ParamType::Uint(8),
                ethabi::ParamType::Uint(8),
                ethabi::ParamType::Uint(8),
                ethabi::ParamType::Bytes,
            ],
            data,
        )
        .map_err(|_e| Error::from(ErrorKind::AbiDecode))?;
        let block_number = decoded[0].clone().to_uint();
        let start = decoded[1].clone().to_uint();
        let end = decoded[2].clone().to_uint();
        let state_update = decoded[3].clone().to_bytes();

        if let (Some(block_number), Some(start), Some(end), Some(state_update)) =
            (block_number, start, end, state_update)
        {
            Ok(VerifiedStateUpdate::new(
                block_number.as_u64(),
                start.as_u64(),
                end.as_u64(),
                StateUpdate::from_abi(&state_update).unwrap(),
            ))
        } else {
            Err(Error::from(ErrorKind::AbiDecode))
        }
    }
}

pub struct StateDb<KVS: KeyValueStore<Range>> {
    db: Box<RangeDbImpl<KVS>>,
}

impl<KVS> Default for StateDb<KVS>
where
    KVS: DatabaseTrait + KeyValueStore<Range>,
{
    fn default() -> Self {
        let base_db = KVS::open("state");
        Self {
            db: Box::new(RangeDbImpl::from(base_db)),
        }
    }
}

impl<KVS> StateDb<KVS>
where
    KVS: DatabaseTrait + KeyValueStore<Range>,
{
    pub fn get_verified_state_updates(
        &self,
        start: u64,
        end: u64,
    ) -> Result<Box<[VerifiedStateUpdate]>, Error> {
        let ranges = self.db.get(start, end).map_err::<Error, _>(Into::into)?;
        ranges
            .iter()
            .map(|range| VerifiedStateUpdate::from_abi(range.get_value()))
            .collect()
    }
    pub fn put_verified_state_update(
        &self,
        verified_state_update: &VerifiedStateUpdate,
    ) -> Result<(), Error> {
        self.db
            .put(
                verified_state_update.get_start(),
                verified_state_update.get_end(),
                &verified_state_update.to_abi(),
            )
            .map_err::<Error, _>(Into::into)
    }
}
