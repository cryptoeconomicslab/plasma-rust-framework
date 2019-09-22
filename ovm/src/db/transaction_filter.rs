use ethereum_types::Address;
use plasma_core::data_structure::{Range, Transaction};

#[derive(Debug)]
pub struct TransactionFilter {
    from_block: u64,
    to_block: u64,
    from_address: Option<Address>,
    to_address: Option<Address>,
    range: Range,
}

impl TransactionFilter {
    pub fn get_from_block(&self) -> u64 {
        self.from_block
    }

    pub fn get_to_block(&self) -> u64 {
        self.to_block
    }

    pub fn get_range(&self) -> Range {
        self.range
    }

    fn filter(&self, transaction: &Transaction) -> bool {
        let from_address = transaction.get_metadata().get_from();
        let to_address = transaction.get_metadata().get_to();

        (self.from_address.is_none() || self.from_address.unwrap() == from_address)
            || (self.to_address.is_none() || self.to_address.unwrap() == to_address)
    }

    pub fn query(&self, transactions: Vec<Transaction>) -> Vec<Transaction> {
        transactions
            .into_iter()
            .filter(|t| self.filter(t))
            .collect()
    }
}

#[derive(Debug, Default)]
pub struct TransactionFilterBuilder<FromBlockType, ToBlockType, RangeType> {
    from_block: FromBlockType,
    to_block: ToBlockType,
    from_address: Option<Address>,
    to_address: Option<Address>,
    range: RangeType,
}

impl TransactionFilterBuilder<(), (), ()> {
    pub fn new() -> Self {
        Self {
            from_block: (),
            to_block: (),
            from_address: None,
            to_address: None,
            range: (),
        }
    }
}

impl TransactionFilterBuilder<u64, u64, Range> {
    pub fn build(self) -> TransactionFilter {
        TransactionFilter {
            from_block: self.from_block,
            to_block: self.to_block,
            from_address: self.from_address,
            to_address: self.to_address,
            range: self.range,
        }
    }
}

impl<FromBlockType, ToBlockType, RangeType>
    TransactionFilterBuilder<FromBlockType, ToBlockType, RangeType>
{
    pub fn block_from(
        self,
        from_block: u64,
    ) -> TransactionFilterBuilder<u64, ToBlockType, RangeType> {
        TransactionFilterBuilder {
            from_block,
            to_block: self.to_block,
            from_address: self.from_address,
            to_address: self.to_address,
            range: self.range,
        }
    }

    pub fn block_to(
        self,
        to_block: u64,
    ) -> TransactionFilterBuilder<FromBlockType, u64, RangeType> {
        TransactionFilterBuilder {
            from_block: self.from_block,
            to_block,
            from_address: self.from_address,
            to_address: self.to_address,
            range: self.range,
        }
    }

    pub fn range(
        self,
        range: Range,
    ) -> TransactionFilterBuilder<FromBlockType, ToBlockType, Range> {
        TransactionFilterBuilder {
            from_block: self.from_block,
            to_block: self.to_block,
            from_address: self.from_address,
            to_address: self.to_address,
            range,
        }
    }

    pub fn address_from<A: Into<Address>>(mut self, from_address: A) -> Self {
        self.from_address = Some(from_address.into());
        self
    }

    pub fn address_to<A: Into<Address>>(mut self, to_address: A) -> Self {
        self.to_address = Some(to_address.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use ethereum_types::Address;
    use plasma_core::data_structure::{Metadata, Range};

    #[test]
    /// filters works only for addresses
    /// range and block_numbers are used to check in block_db.
    fn test_filter_from_address() {
        let address =
            Address::from_slice(&hex::decode("2932b7a2355d6fecc4b5c0b6bd44cc31df247a2e").unwrap());
        let transactions = vec![
            Transaction::new(
                Address::zero(),
                Range::new(0, 1),
                Bytes::default(),
                Bytes::default(),
                Metadata::new(address, Address::zero()),
            ),
            Transaction::new(
                Address::zero(),
                Range::new(0, 1),
                Bytes::default(),
                Bytes::default(),
                Metadata::new(Address::zero(), address),
            ),
        ];

        let filter = TransactionFilterBuilder::new()
            .range(Range::new(10, 20))
            .block_from(0)
            .block_to(2)
            .address_from(address)
            .build();

        assert_eq!(filter.query(transactions).len(), 2);
    }

    #[test]
    /// filters works only for addresses
    /// range and block_numbers are used to check in block_db.
    fn test_filter_to_address() {
        let address =
            Address::from_slice(&hex::decode("2932b7a2355d6fecc4b5c0b6bd44cc31df247a2e").unwrap());
        let transactions = vec![
            Transaction::new(
                Address::zero(),
                Range::new(0, 1),
                Bytes::default(),
                Bytes::default(),
                Metadata::new(address, Address::zero()),
            ),
            Transaction::new(
                Address::zero(),
                Range::new(0, 1),
                Bytes::default(),
                Bytes::default(),
                Metadata::new(Address::zero(), address),
            ),
        ];

        let filter = TransactionFilterBuilder::new()
            .range(Range::new(10, 20))
            .block_from(0)
            .block_to(2)
            .address_from(address)
            .build();

        assert_eq!(filter.query(transactions).len(), 2);
    }

}
