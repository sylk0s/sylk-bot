use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
async fn info(_ctx: &Context, _msg: &Message) -> CommandResult {
    unimplemented!();
}
