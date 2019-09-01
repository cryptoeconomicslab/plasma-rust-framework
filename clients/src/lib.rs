pub mod plasma_aggregator;
pub mod plasma_client;
pub mod state_db;
pub mod state_manager;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
