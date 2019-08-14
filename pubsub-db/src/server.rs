use ws::{listen, Handler, Sender, Result, Message, CloseCode};

pub struct Server {
    out: Sender,
}

impl Handler for Server {
    fn on_message(&mut self, msg: Message) -> Result<()> {
        self.out.send(msg)
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        match code {
            CloseCode::Normal => println!("The client is done with the connection."),
            CloseCode::Away => println!("The client is leaving the site."),
            _ => println!("The client encountered an error: {}", reason),
        }
    }
}


pub fn start_server(host: &str) {
    listen(host, |out| {
        Server { out }
    }).unwrap();
}

