
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{CommandResult, Args};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
async fn mallet(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let ar = args.raw().collect::<Vec<&str>>();
    msg.reply(&ctx.http, format!("Bonked {} with the mallet of loving correction", ar[0])).await?;
    Ok(())
}
