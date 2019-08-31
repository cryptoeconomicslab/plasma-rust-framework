pub mod plasma_aggregator;
pub mod plasma_client;
mod state_db;
mod state_manager;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
