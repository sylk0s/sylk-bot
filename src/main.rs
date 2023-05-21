use sylk_bot::commands::{*, vote::Vote};
use sylk_bot::{State, Error, Context, Data};
use poise::serenity_prelude as serenity;
use std::{env::var, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use dotenv;
use websocket::OwnedMessage;

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

//    env_logger::init();

    let options = poise::FrameworkOptions {
        commands: vec![
            help(),
            register(),
            general::boop(),
            //general::voiceinfo(),
            //general::echo(),
            //#[cfg(feature = "cache")]
            //general::servers(),
            //general::reply(),
            //general::bonk(),
            //general::pin(),
            //general::name(),
            /*
            context_menu::user_info(),
            context_menu::echo(),
            autocomplete::greet(),
            checks::shutdown(),
            checks::modonly(),
            checks::delete(),
            checks::ferrisparty(),
            checks::cooldowns(),
            checks::minmax(),
            checks::get_guild_name(),
            checks::only_in_dms(),
            checks::lennyface(),
            checks::permissions_v2(),
            subcommands::parent(),
            localization::welcome(),
            */
            vote::vote(),
        ],
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
        event_handler: |_ctx, event, _framework, _data| {
            Box::pin(async move {
                println!("Got an event in event handler: {:?}", event.name());
                Ok(())
            })
        },
        ..Default::default()
    };

    let token = var("DISCORD_TOKEN").expect("Missing `DISCORD_TOKEN` env var, see README for more information.");
    let state = Arc::new(RwLock::new( Data { 
        votes: Vote::reload(&serenity::Http::new(&token)).await.expect("Loading votes failed :(") ,
        //manager: None,
    }));

    // Handles voting thread in the bot for checking and reacting to votes ending
    let state1 = Arc::clone(&state);
    let http = serenity::Http::new(&token);
    // thread for checking votes incrementally
    tokio::spawn(async move { 
        for _ in eventual::Timer::new().interval_ms(10000).iter() {
            let votes = state1.read().await.votes.clone();
            state1.write().await.votes = Vote::end_finished_votes(votes, &http).await;
        }
    });

    // Handles voting thread in the bot for checking and reacting to votes ending
    let state2 = Arc::clone(&state);
    let http2 = serenity::Http::new(&token);
    // thread for checking votes incrementally
    tokio::spawn(async move { 
        let manager = state2.read().await.manager.clone();
        // tldr send the messages in the channel
        if let Some(man) = manager {
            manager.stream("unused").map(|message| async move {
                if let OwnedMessage::Text(msg) = message {
                    match &msg[0..3] {
                        "MSG" => {
                        http2.get_channel(0).await.unwrap()
                        .id().send_message(&http2, |m| { m.content(&msg[4..]) })
                        .await.expect("Message failed");
                        },
                        _ => {
                            // what is this and why did i include it
                            //s_cache.send(msg);
                            println!("Unknown WS message");
                        }
                    }
                }
            });
        }
    });

    // Starts all of the bridges on the server
    // Each bridge spawns a thread forwarding the stream from that docker container to the specified channel for the obj
    

    poise::Framework::builder()
        .token(&token)
        .setup(move |_ctx, _ready, _framework| {
            // Initialize the data held globally by the bot.
            Box::pin(async move {
                Ok(state)
            })
        })
        .options(options)
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .run()
        .await
        .unwrap();
}