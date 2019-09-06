use super::message::Message;
use ws::Sender;

/// Trait to implement server event handlers
///
pub trait Handler: Clone {
    fn handle_message(&mut self, msg: Message, sender: Sender);
    fn handle_open(&mut self, _sender: Sender) {}
    fn handle_close(&mut self) {}
}
