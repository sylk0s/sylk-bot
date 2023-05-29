pub mod commands;
pub mod utils;

use crate::commands::vote::Vote;
use crate::utils::mc::interface::MCInterface;

use poise::serenity_prelude as serenity;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use crate::utils::mc::bridge::Bridge;

// Types used by all command functions
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, State, Error>;

// State is held both by the bot and by various threads
// State fields are always accessed in the "thread" level code
pub type State = Arc<Mutex<Data>>;

// Custom user data passed to all command functions
pub struct Data {
    pub votes: HashMap<u64, Vote>,
    pub manager: Option<Box<dyn MCInterface>>,
    pub bridges: Vec<Bridge>,
}