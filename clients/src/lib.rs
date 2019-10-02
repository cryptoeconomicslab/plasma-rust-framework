#[macro_use]
extern crate lazy_static;

pub mod android;
pub mod plasma;
pub mod state_channel;

pub use android::AndroidClient;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
