use serde::Deserialize;
use std::fs;
use std::collections::HashMap;
use futures::StreamExt;
use crate::utils::error::ConnectionError;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

// Each bridge represents a single interface between a server and a discord channel
// To create multiple links, multiple bridge objects are created and started
struct Bridge<T> where T: MCInterface {
    interface: T,
}

impl<T> Bridge<T> where T: MCInterface {
    // This will read from the servers.json file to create all the neccecary bridges
    fn init() {
        unimplemented!();
    }

    // Creates a new bridge
    fn new(kind: String, channel: u64, server: String) {
        unimplemented!();
    }

    // Starts the bridge
    // This will start forwarding the stream of message output to the server from the 
    async fn start() {

    }
}

struct Taurus;

struct TauConf {
    chatbridge_id: u64,
    password: String,
}

// todo move into a new file
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

// fix this
const MCD_PATH: &str = "/home/sylkos/.config/mc-docker";

// literally steals MC-DOCKER's config file
#[derive(Debug, Deserialize)]
struct McdConf {
    ws_port: u16,
}

// servers.json
#[derive(Debug, Deserialize, Clone)]
struct Servers {
    servers: Vec<Server>
}

#[derive(Debug, Deserialize, Clone)]
struct Server {
    name: String,
    channel: u64
}

pub struct Mcd {
    servers: Arc<Mutex<HashMap<String, Arc<Mutex<Server>>>>>,
    config: McdConf,
}

impl Mcd {
    pub fn new() -> Result<Mcd, ConnectionError> {
        let mcd_file = if let Ok(a) = fs::read_to_string(format!("{MCD_PATH}/config.toml")) { a } else {
            return Err(ConnectionError::from("Failed to find config.toml from mc-docker"))
        };
        
        let config: McdConf = if let Ok(a) = toml::from_str(&mcd_file) { a } else {
            return Err(ConnectionError::from("Failed to parse mc-docker toml file"))
        };

        let svs_file = if let Ok(a) = fs::read_to_string("./servers.json") { a } else {
            return Err(ConnectionError::from("Failed to find servers.json file"))
        };

        let svs: Servers = if let Ok(a) = serde_json::from_str(&svs_file) { a } else {
            return Err(ConnectionError::from("Failed to parse servers.json file"))
        };

        let servers = Arc::new(Mutex::new(HashMap::new()));
        for server in svs.servers.iter() {
            servers.lock().unwrap().insert(server.name.clone(), Arc::new(Mutex::new(server.clone())));
        }

        Ok(Mcd{ servers, config })
    }

    /*
    pub fn init(&self, _config: Config, http: Arc<Http>) {
        let path = format!("http://localhost:{}", self.config.ws_port);
        for s in self.servers.lock().unwrap().values() {
            let server = s.lock().unwrap().clone();
            let p = path.clone();
            let http2 = http.clone();
            tokio::spawn(async move {
                println!("Streaming output from server@{p}");
                let mut stream = reqwest::get(format!("{p}/out/{}", server.name)).await.unwrap().bytes_stream();
                while let Some(msg) = stream.next().await {
                    let msg = std::str::from_utf8(&msg.unwrap()).unwrap().to_string();
                    println!("{}: {msg}", server.name);
                    // send to discord channel
                    http2.get_channel(server.channel).await.unwrap().guild().unwrap()
                        .send_message(&http2, |m| m.content(format!("[{}] {msg}", server.name))).await.unwrap();
                }
            });
        }
    }
    */
}

#[async_trait]
impl MCInterface for Mcd {
    async fn send(&self, s: String, msg: String) {
        let server = self.servers.lock().unwrap().get(&s).unwrap().lock().unwrap().clone(); 
        let mut cmd = Vec::new();
        cmd.push("tellraw".to_string());
        cmd.push("@a".to_string());
        cmd.push(format!(r#"{{\"text\":\"[{}] {}\"}}"#, server.name, msg));
        self.execute(s, cmd).await;
    }

    async fn execute(&self, s: String, cmd: Vec<String>) {
        let server = self.servers.lock().unwrap().get(&s).unwrap().lock().unwrap().clone(); 
        let path = format!("https://localhost:{}", self.config.ws_port);
        let body = format!(r#"{{"args":[{}]}}"#, 
            cmd.iter().skip(1).fold(format!(r#""{}""#, cmd.iter().next().unwrap().clone()), 
                    |x, acc| format!(r#"{},"{}""#,acc, x)));
        reqwest::Client::new().post(format!("{path}/exec/{}", server.name)).body(body)
            .send().await.expect("Failed to send command to server");
    }

    async fn status(&self, s: String) -> String {
        unimplemented!();
    }
}

// An interface to interact with Minecraft  server management systems
#[async_trait]
trait MCInterface {
    // send a message to a server
    async fn send(&self, s: String, msg: String);

    // execute a command on a server
    async fn execute(&self, s: String, cmd: Vec<String>);

    // get the status of a server
    async fn status(&self, s: String) -> String; 

    // Stream output

    /*
        let mut stream = reqwest::get(format!("{ADDR}/{}/{}", self.name, self.id)).await.unwrap().bytes_stream();

        while let Some(msg) = stream.next().await {
            println!("{:?}", msg);
        }
     */

    // Start

    // Stop


}