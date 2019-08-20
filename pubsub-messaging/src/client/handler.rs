use super::message::Message;
use ws::Sender;

pub trait Handler {
    fn handle_open(&self, _sender: Sender) {}
    fn handle_message(&self, msg: Message, sender: Sender);
}
