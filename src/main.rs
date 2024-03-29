use sylk_bot::commands::{*, vote::Vote};
use sylk_bot::{State, Error, Context, Data};
use poise::serenity_prelude as serenity;
use std::{env::var, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use dotenv;
use poise::Event::Message;
use futures::stream::StreamExt;
use sylk_bot::utils::mc::bridge::Bridge;

/* TODO
    - Move away from ENV vars, move everything I can into one TOML
    - Reimplement the good logger that i yeeted because it was trash
    - More vertitle config file using optionals 
 */

/// Show this help menu
#[poise::command(track_edits, slash_command)]
async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "\n
This is an example bot made to showcase features of my custom Discord bot framework",
            show_context_menu_commands: true,
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

/// Registers or unregisters application commands in this guild or globally
#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;

    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, State, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

// Figure out on_msg
#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let token = var("DISCORD_TOKEN").expect("Missing `DISCORD_TOKEN` env var, see README for more information.");
    let state = Arc::new(Mutex::new( Data {
        votes: Vote::reload(&serenity::Http::new(&token)).await.expect("Loading votes failed :(") ,
        manager: None,
        bridges: Vec::new(),
    }));

//    env_logger::init();

    let options = poise::FrameworkOptions {

        // The commands to register for this bot
        commands: vec![
            help(),
            register(),
            general::boop(),
            vote::vote(),
        ],

        // The options for the prefix to normal commands in this bot
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("~".into()),
            edit_tracker: Some(poise::EditTracker::for_timespan(Duration::from_secs(3600))),
            additional_prefixes: vec![
                poise::Prefix::Literal("hey bot"),
                poise::Prefix::Literal("hey bot,"),
            ],
            ..Default::default()
        },

        /// The global error handler for all error cases that may occur
        on_error: |error| Box::pin(on_error(error)),

        /// This code is run before every command
        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },

        /// This code is run after a command if it was successful (returned Ok)
        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },

        /// Every command invocation must pass this check to continue execution
        command_check: Some(|ctx| {
            Box::pin(async move {
                if ctx.author().id == 123456789 {
                    return Ok(false);
                }
                Ok(true)
            })
        }),

        /// Enforce command checks even for owners (enforced by default)
        /// Set to true to bypass checks, which is useful for testing
        skip_checks_for_owners: false,

        // This code will handle events and event types
        event_handler: |_ctx, event, _framework, state| {
            Box::pin(async move {
                // On Message
                if let Message { new_message: msg } = event {
                    let bridges = &state.lock().await.bridges;

                    // If the message is in a channel that is registered to forward...
                    for bridge in bridges {
                        if bridge.is_channel(msg.channel_id.into()) {
                            println!("Sent: {}",msg.content);

                            // Sends the message back to the server
                            bridge.send_message(msg.content.clone()).await;
                        }
                    }
                }
                Ok(())
            })
        },
        ..Default::default()
    };

    // Handles voting thread in the bot for checking and reacting to votes ending
    let state1 = Arc::clone(&state);
    let http = serenity::Http::new(&token);
    // thread for checking votes incrementally
    tokio::spawn(async move { 
        for _ in eventual::Timer::new().interval_ms(10000).iter() {
            let votes = state1.lock().await.votes.clone();
            state1.lock().await.votes = Vote::end_finished_votes(votes, &http).await;
        }
    });

    let mut bridges: Vec<Bridge> = Vec::new();
    for bridge in bridges {
        let mut stream = bridge.stream();
        let http = serenity::Http::new(&token);
        tokio::spawn(async move {
            while let Some(msg) = stream.next().await {
                let _ = &http.get_channel(bridge.channel).await.unwrap().id()
                    .send_message(&http, |m| m.content(msg))
                    .await.expect("failed to get msg");
            }
        });
    }

    poise::Framework::builder()
        .token(&token)
        .setup(move |_ctx, _ready, _framework| {
            // Initialize the data held globally by the bot.
            Box::pin(async move {
                Ok(state)
            })
        }).options(options)
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .run().await.unwrap();
}