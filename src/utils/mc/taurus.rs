use std::sync::mpsc;
use crate::utils::mc::interface::MCInterface;
use async_trait::async_trait;
use websocket::OwnedMessage;
use crate::utils::error::ConnectionError;
/*
pub struct Taurus {
    receiver: Option<mpsc::Sender<OwnedMessage>>,
    sender: Option<mpsc::Receiver<OwnedMessage>>,
}

impl Taurus {
    pub fn new() -> Result<Taurus, ConnectionError> {
        Ok(Taurus { receiver: None, sender: None })
    }
}

#[async_trait]
impl MCInterface for Taurus {
    // send a message to a server
    async fn send(&self, s: &str, msg: &str) {
        unimplemented!();
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
    // TODO returns something useful
    async fn stream(&self, s: &str) -> Box<dyn Iterator<Item=OwnedMessage>> {
        unimplemented!();
    }
}

struct TauConf {
    chatbridge_id: u64,
    password: String,
}

// todo move into a new file suggesting
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
*/