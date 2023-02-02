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
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::id::GuildId;

use serde::Deserialize;
use tracing::info;

use crate::commands::ping::*;
use crate::commands::pin::*;
use crate::commands::role::*;
use crate::commands::mallet::*;
use crate::commands::name::*;

pub mod commands;
pub mod utils;

const GUILD_ID: u64 = 1022919692152737802;

// handler struct for commands
pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // When the discord bot is "ready"
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("Connected as {}", ready.user.name);

        // loads in all of the votes from the web server
        Vote::reload(&ctx).await.expect("Failed to reload the votes");
        println!("Reloaded all cloud votes");

        let guild_id = GuildId(GUILD_ID);

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| commands::ping::register(command))
                /*
                .create_application_command(|command| commands::id::register(command))
                .create_application_command(|command| commands::welcome::register(command))
                .create_application_command(|command| commands::numberinput::register(command))
                .create_application_command(|command| commands::attachmentinput::register(command))
                */
        })
        .await.unwrap();

        println!("I now have the following guild slash commands: {:#?}", commands);
    }

    // I dont actually know what this is doing.
    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);

            let content = match command.data.name.as_str() {
                "ping" => commands::ping::run(&command.data.options),
                //"id" => commands::id::run(&command.data.options),
                //"attachmentinput" => commands::attachmentinput::run(&command.data.options),
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    // TODO implement on_message for messages in chatbridge -> taurus
}

// General command group for *most* commands
#[group]
#[commands(ping, pin, role, mallet, name)]
pub struct General;

// Bot specific command
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub log_channel_id: u64,
    pub backend: String,
}

// Config for help command
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

// Holds the shard manager for the bot
pub struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

// Holds the list of votes for the bot
pub struct VoteContainer;
impl TypeMapKey for VoteContainer {
    type Value = Vec<Vote>;
}

// Possibly the future cache of chatbridge commands for taurus
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
