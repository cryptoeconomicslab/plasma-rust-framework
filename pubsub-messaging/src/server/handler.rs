use super::message::Message;
use ws::Sender;

/// Trait to implement server event handlers
///
pub trait Handler: Clone {
    fn handle_message(&self, msg: Message, sender: Sender);
}
