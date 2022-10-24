mod commands;
mod utils;

use std::collections::HashSet;
use std::env;
use std::sync::Arc;
use std::fs;

use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::standard::macros::{group, help};
use serenity::framework::standard::{CommandResult, CommandGroup, HelpOptions, Args, help_commands};
use serenity::framework::StandardFramework;
use serenity::http::Http;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;

use serenity::model::prelude::{Message, UserId};
use serenity::prelude::*;

use tracing::{error, info};
use serde::Deserialize;

use crate::commands::ping::*;
use crate::commands::pin::*;
use crate::commands::role::*;
use crate::commands::vote::*;
use crate::commands::vote::Vote;

pub struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct VoteContainer;
impl TypeMapKey for VoteContainer {
    type Value = Vec<Vote>;
}

/*
pub struct ConfigContainer;
impl TypeMapKey for ConfigContainer {
    type Value = Config;
}
*/

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

#[group]
#[commands(ping, pin, role)]
struct General;

#[derive(Deserialize)]
pub struct Config {
    log_channel: u32,
}

#[help]
#[individual_command_tip = "aaabbb"]
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

#[tokio::main]
async fn main() {
    let file = fs::read_to_string("./config.toml").unwrap();
    //let config: Config = toml::from_str(&file).unwrap();

    // This will load the environment variables located at `./.env`, relative to
    // the CWD. See `./.env.example` for an example on how to structure this.
    dotenv::dotenv().expect("Failed to load .env file");

    // Initialize the logger to use environment variables.
    //
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to `debug`.
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let http = Http::new(&token);

    // We will fetch your bot's owners and id
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Create the framework
    let framework =
        StandardFramework::new().configure(|c| c.owners(owners).prefix("~"))
            .group(&GENERAL_GROUP)
            .group(&VOTING_GROUP)
            .help(&MY_HELP);

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<VoteContainer>(vec![]);
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    // spawn checker thread
    // need to figure out how to reference this globally from list command

    // app handle

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
