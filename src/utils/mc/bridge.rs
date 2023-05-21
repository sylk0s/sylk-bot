use crate::utils::mc::interface::MCInterface;
//use crate::utils::mc::mcd::Mcd;
//use crate::utils::mc::taurus::Taurus;

/*
// Each bridge represents a single interface between a server and a discord channel
// To create multiple links, multiple bridge objects are created and started
struct Bridge<T> where T: MCInterface {
    interface: T,
    channel: u64,
}

impl Bridge<Mcd> {
    fn new(channel: u64, server: &str) -> Bridge<Mcd> {
        Bridge {
            interface: Mcd::new().unwrap(),
            channel,
        }
    }
}

impl Bridge<Taurus> {
    fn new(channel: u64, server: &str) -> Bridge<Taurus> {
        Bridge {
            interface: Taurus::new().unwrap(),
            channel,
        }
    }
}

impl Bridge<T> where T: MCInterface {
    // Starts the bridge
    // This will start both directions of message forwarding
    async fn start(&self) {
        // Somehow will need to register this destination in the handler for the onMessageRecieved
    }
}
*/