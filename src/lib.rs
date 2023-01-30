use crate::commands::vote::Vote;
use serenity::client::bridge::gateway::ShardManager;
use serenity::prelude::*;
use std::sync::Arc;
use std::collections::HashSet;

use serenity::async_trait;
use serenity::framework::standard::macros::{group, help};
use serenity::framework::standard::{CommandResult, CommandGroup, HelpOptions, Args, help_commands};
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::model::prelude::{Message, UserId};

use serde::Deserialize;
use tracing::info;

use crate::commands::ping::*;
use crate::commands::pin::*;
use crate::commands::role::*;
use crate::commands::mallet::*;

pub mod commands;
pub mod utils;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("Connected as {}", ready.user.name);

        Vote::reload(&ctx).await.expect("Failed to reload the votes");
        println!("Reloaded all cloud votes");
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

    // TODO implement on_message for messages in chatbridge -> taurus
}

#[group]
#[commands(ping, pin, role, mallet)]
pub struct General;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub log_channel_id: u64,
    pub backend: String,
}

#[help]
#[individual_command_tip = "Help"]
#[command_not_found_text = "Could not find: `{}`."]
#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

pub struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct VoteContainer;
impl TypeMapKey for VoteContainer {
    type Value = Vec<Vote>;
}

pub struct WSCache;

/*
impl TypeMapKey for WSCache {
    // fix this issue with strings not wanting to send between threads
    type Value = mpsc::Receiver<String>;
}
*/

/*
pub struct ConfigContainer;
impl TypeMapKey for ConfigContainer {
    type Value = Config;
}
*/
