use std::pin::Pin;
use poise::futures_util::Stream;
use crate::Error;
use crate::utils::mc::interface::MCInterface;
use crate::utils::mc::taurus::Taurus;
//use crate::utils::mc::mcd::Mcd;
//use crate::utils::mc::taurus::Taurus;

// Each bridge represents a single interface between a server and a discord channel
// To create multiple links, multiple bridge objects are created and started
pub struct Bridge {
    interface: Box<dyn MCInterface>,
    pub channel: u64,
}

impl Bridge {
    pub fn taurus(channel: u64) -> Result<Self, Error> {
        Ok(Bridge {
            interface: Box::new(Taurus::new().expect("Taurus failed to connect")),
            channel
        })
    }

    pub fn stream(&self) -> Pin<Box<dyn Stream<Item=String> + Send>> {
        unimplemented!();
    }

    pub async fn send_message(&self, msg: String) {
        self.interface.send("server", msg).await;
    }

    pub fn is_channel(&self, other: u64) -> bool {
        other == self.channel
    }
}