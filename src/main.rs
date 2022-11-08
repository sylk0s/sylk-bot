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
use websocket::{ClientBuilder, OwnedMessage};

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
#[commands(ping, pin, role)]
struct General;

#[derive(Deserialize)]
pub struct Config {
    log_channel_id: u64,
    chatbridge_id: u64,
    password: String,
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
    let config: Arc<Config> = Arc::new(toml::from_str(&file).unwrap());

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

    // 

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
    let data = client.data.clone();
    let http2 = client.cache_and_http.http.clone();

    tokio::spawn(async move { 
        for _ in eventual::Timer::new().interval_ms(1000).iter() { 
            let mut aaa = data.write().await;
            let votes = aaa.get_mut::<VoteContainer>().unwrap();
            Vote::check_votes_over(votes, &http2).await;
        }
    });

    // todo move into a new file

    let ws = ClientBuilder::new("ws://127.0.0.1:7500").unwrap()
                     .connect_insecure().unwrap();

    let (mut receiver, mut sender) = ws.split().unwrap();
    let mut message_cache = Vec::<String>::new();

    println!("connecting to taurus...");
    sender.send_message(&websocket::Message::text(config.password.clone())).unwrap();    
    println!("authenticating...");
    sender.send_message(&websocket::Message::text("PING")).unwrap();

    // TODO await pong message

    let http3 = client.cache_and_http.http.clone();
    tokio::spawn(async move {
        for message in receiver.incoming_messages() {
            if let OwnedMessage::Text(msg) = message.unwrap() {
                if msg[0..3] == *"MSG" {
                    http3.get_channel(config.chatbridge_id).await.unwrap().id().send_message(&http3, |m| { m.content(&msg[4..]) }).await.expect("Message failed");
                } else {
                    message_cache.push(msg);
                }
            }
        }
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
