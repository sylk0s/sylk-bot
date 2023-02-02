use std::collections::HashSet;
use std::env;
use std::fs;
use std::sync::Arc;

use serenity::framework::StandardFramework; 
use serenity::http::Http;
use serenity::prelude::*;

use tracing::error;

use sylk_bot::commands::vote::*;
use sylk_bot::*;

#[tokio::main]
async fn main() {
    // read in bot wide config file
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

    // TOKEN from environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let http = Http::new(&token);

    // get the bot's owner and ID
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

    // gateway intents
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // make the client for the bot
    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(sylk_bot::Handler)
        .await
        .expect("Err creating client");

    // cloned data for the checker thread
    let data = client.data.clone();
    let http2 = client.cache_and_http.http.clone();

    // thread for checking votes incrementally
    tokio::spawn(async move { 
        for _ in eventual::Timer::new().interval_ms(1000).iter() { 
            let mut aaa = data.write().await;
            let votes = aaa.get_mut::<VoteContainer>().unwrap();
            Vote::check_votes_over(votes, &http2).await;
        }
    });

    // get the minecraft backend from the config file
    let mc = match config.backend.as_str() {
        "mc-docker" => Some(()),
        "taurus" => Some(()),
        _ => None,
    };

    // writes to the client's data
    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<VoteContainer>(vec![]);
        //data.insert::<WSCache>(r_cache);
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
