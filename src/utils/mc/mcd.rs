use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use crate::utils::error::ConnectionError;
use serde::{Deserialize, Serialize};
use websocket::OwnedMessage;
use crate::utils::mc::interface::MCInterface;
use async_trait::async_trait;

/*
// fix this
const MCD_PATH: &str = "/home/sylkos/.config/mc-docker";

// literally steals MC-DOCKER's config file\
// make this better pls
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
    servers: Arc<Mutex<HashMap<String, Server>>>,
    config: McdConf,
}

impl Mcd {
    pub fn new() -> Result<Mcd, ConnectionError> {

        // Gets the mc-docker config.toml
        let mcd_file = if let Ok(a) = fs::read_to_string(format!("{MCD_PATH}/config.toml")) { a } else {
            return Err(ConnectionError::from("Failed to find config.toml from mc-docker"))
        };
        
        // parses the mcd config into an object
        let config: McdConf = if let Ok(a) = toml::from_str(&mcd_file) { a } else {
            return Err(ConnectionError::from("Failed to parse mc-docker toml file"))
        };

        // gets servers from server.json
        let svs_file = if let Ok(a) = fs::read_to_string("./servers.json") { a } else {
            return Err(ConnectionError::from("Failed to find servers.json file"))
        };

        // parses the json into an object
        let svs: Servers = if let Ok(a) = serde_json::from_str(&svs_file) { a } else {
            return Err(ConnectionError::from("Failed to parse servers.json file"))
        };

        // puts the servers in a hashmap with the key as the server's name and the value as the server object
        let servers = Arc::new(Mutex::new(HashMap::new()));
        for server in svs.servers.iter() {
            servers.lock().unwrap().insert(server.name.clone(), server.clone());
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

    // Send a text message to the server
    async fn send(&self, s: String, msg: String) {
        /*
        let mut cmd = Vec::new();
        cmd.push("tellraw".to_string());
        cmd.push("@a".to_string());
        cmd.push(format!(r#"{{\"text\":\"[{}] {}\"}}"#, s, msg));
        self.execute(s, cmd).await;
         */
        unimplemented!();
    }

    // Execute a command on the server
    async fn execute(&self, s: String, cmd: Vec<String>) {
        /*
        let servers = self.servers.lock().unwrap();
        let server = servers.get(&s).unwrap(); 
        let path = format!("https://localhost:{}", self.config.ws_port);
        let body = format!(r#"{{"args":[{}]}}"#, 
            cmd.iter().skip(1).fold(format!(r#""{}""#, cmd.iter().next().unwrap().clone()), 
                    |x, acc| format!(r#"{},"{}""#,acc, x)));
        reqwest::Client::new().post(format!("{path}/exec/{}", server.name)).body(body)
            .send().await.expect("Failed to send command to server");
            */
         unimplemented!();
    }

    // Get the status of the server
    async fn status(&self, s: String) -> Status {
        unimplemented!();
    }

    async fn stream(&self, s: &str) -> Box<dyn Iterator<Item=OwnedMessage>> {
        todo!()
    }
}

 */