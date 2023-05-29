use async_trait::async_trait;
use tokio_stream::Stream;
use tokio_stream::StreamExt;

// The server stores a list of bridges and then checks for each's channel each time
// this is for forwarding to the server
// bridge is specialized for handling interactions with discord
// the bridge uses an interface to handle communicating with the underlying backend
// this way any backend can be implemented for this object (for example, MCDaemon)
// the backend bridge also spawn the thread that is sending the messages to the channel 
// the backed just provides the ability to get a stream

// An interface to interact with Minecraft  server management systems
#[async_trait]
pub trait MCInterface: Send + Sync {
    // send a message to a server
    async fn send(&self, s: &str, msg: String);

    // execute a command on a server
    async fn execute(&self, s: &str, cmd: Vec<&str>);

    // get the status of a server
    async fn status(&self, s: &str) -> ServerStatus;

    // Stream output
    async fn stream(&self, s: &str) -> Box<dyn Stream<Item=String>>;

    /*
        let mut stream = reqwest::get(format!("{ADDR}/{}/{}", self.name, self.id)).await.unwrap().bytes_stream();
        while let Some(msg) = stream.next().await {
            println!("{:?}", msg);
        }
     */

    // Start

    // Stop

    // Pipe output to ?
}

pub struct ServerStatus;

/*
// The result of pinging a minecraft server's status
struct ServerStatus<'a> {
    online: bool,
    onserver: Option<u32>,
    max: Option<u32>,
    name: Option<&'a str>,
    motd: Option<&'a str>,
    players: Option<Vec<&'a str>>,
}

impl<'a> ServerStatus<'a> {
    fn formatted(&self) -> String {
        if !self.online {
            format!("The server did not respond. It is either offline, or not accessible to this machine.")
        } else {
            format!("{} is online with {}/{} players", self.name.clone().unwrap(), self.onserver.unwrap(), self.max.unwrap())
        }
    }
}
*/