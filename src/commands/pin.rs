use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
async fn pin(ctx: &Context, msg: &Message) -> CommandResult {
    if let Some(r) = &msg.referenced_message {
        r.pin(&ctx.http).await?; 
    } else {
        msg.reply(&ctx.http, "Didn't reply to a message!").await?;
    };

    Ok(())
}
