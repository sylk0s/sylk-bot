use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::framework::standard::Args;

#[command]
async fn name(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let ar = args.raw().collect::<Vec<&str>>();
    for person in &msg.mentions {
        msg.guild_id.unwrap().member(&ctx.http, person.id).await.unwrap().edit(&ctx.http, |m| m.nickname(ar[0])).await.unwrap();
    }
    Ok(())
}
