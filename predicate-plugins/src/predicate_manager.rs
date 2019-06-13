use crate::ownership::OwnershipPredicate;
use crate::predicate::PredicatePlugin;
use ethereum_types::Address;

/// Predicate manager to load and get predicate plugin
pub struct PredicateManager {}

impl PredicateManager {
    pub fn get_plugin(_address: Address) -> Box<dyn PredicatePlugin> {
        let predicate: OwnershipPredicate = Default::default();
        Box::new(predicate)
    }
}
