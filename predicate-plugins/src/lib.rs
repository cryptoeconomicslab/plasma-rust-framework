pub mod ownership;
pub mod parameters;
pub mod predicate;
pub mod predicate_manager;

pub use ownership::{OwnershipPredicate, OwnershipPredicateParameters};
pub use parameters::PredicateParameters;
pub use predicate::PredicatePlugin;
pub use predicate_manager::PredicateManager;
