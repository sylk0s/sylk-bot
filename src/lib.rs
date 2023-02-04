pub mod commands;
pub mod utils;

use crate::commands::vote::Vote;

use poise::serenity_prelude as serenity;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

// Types used by all command functions
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, State, Error>;

// State is held both by the bot and by various threads
// State fields are always accessed in the "thread" level code
pub type State = Arc<RwLock<Data>>;

// Custom user data passed to all command functions
pub struct Data {
    pub votes: Vec<Vote>,
}
