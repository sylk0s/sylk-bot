use std::net::TcpStream;
use std::sync::Arc;
use crate::utils::mc::interface::{MCInterface, ServerStatus};
use async_trait::async_trait;
use websocket::{ClientBuilder, Message, OwnedMessage};
use websocket::receiver::Reader;
use websocket::sender::Writer;
use crate::utils::error::ConnectionError;

pub struct Taurus {
    password: String,
    address: String,
}

impl Taurus {
    pub fn new() -> Result<Taurus, ConnectionError> {
        let password = String::new();
        let address = "ws://127.0.0.1:7500";

        Ok(Taurus {password, address: address.to_string()})
    }
}

#[async_trait]
impl MCInterface for Taurus {
    // send a message to a server
    async fn send(&self, s: &str, msg: String) {
        let ws = ClientBuilder::new(&*self.address).unwrap()
            .connect_insecure().unwrap();

        let (_, mut sender) = ws.split().unwrap();

        println!("connecting to taurus...");
        sender.send_message(&Message::text(self.password.clone())).unwrap();

        sender.send_message(&Message::text(msg)).expect("MSG failed to send");
    }

    // execute a command on a server
    async fn execute(&self, s: &str, cmd: Vec<&str>) {
        unimplemented!();
    }

    // get the status of a server
    async fn status(&self, s: &str) -> ServerStatus {
        unimplemented!();
    }

    // Stream output
    async fn stream(&self, s: &str) -> Box<dyn tokio_stream::Stream<Item=String>> {

        let ws = ClientBuilder::new(&*self.address).unwrap()
            .connect_insecure().unwrap();

        let (mut receiver, mut sender) = ws.split().unwrap();

        println!("connecting to taurus...");
        sender.send_message(&Message::text(self.password.clone())).unwrap();
        println!("authenticating...");
        sender.send_message(&Message::text("PING")).unwrap();

        Box::new(futures::stream::iter(
            std::iter::from_fn(move || Some(receiver.recv_message().unwrap()))
                .filter_map(|message| {
                    if let OwnedMessage::Text(msg) = message {
                        if let "MSG" = &msg[0..3] {
                            return Some(String::from(&msg[4..]))
                        }
                    }
                    return None;
                })))
    }
}

/*
    let ws = ClientBuilder::new("ws://127.0.0.1:7500").unwrap()
                     .connect_insecure().unwrap();

    let (mut receiver, mut sender) = ws.split().unwrap();
    let (mut s_cache, mut r_cache) = mpsc::channel();

    println!("connecting to taurus...");
    sender.send_message(&websocket::Message::text(config.password.clone())).unwrap();    
    println!("authenticating...");
    sender.send_message(&websocket::Message::text("PING")).unwrap();

    let http3 = client.cache_and_http.http.clone();
    tokio::spawn(async move {
        for message in receiver.incoming_messages() {
            if let OwnedMessage::Text(msg) = message.unwrap() {
                match &msg[0..3] {
                    "MSG" => {
                    http3.get_channel(config.chatbridge_id).await.unwrap().id().send_message(&http3, |m| { m.content(&msg[4..]) }).await.expect("Message failed");
                    },
                    _ => {
                        s_cache.send(msg);
                        println!("Unknown WS message");
                    }
                }
            }
        }
    });

    // "await" or fail function from bot
    r_cache.recv();
*/