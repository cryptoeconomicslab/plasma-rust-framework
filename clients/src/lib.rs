#[macro_use]
extern crate futures;

pub mod plasma;
pub mod state_channel;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
