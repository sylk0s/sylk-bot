use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use std::fmt::Write as _;

/// Boop the bot!
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn boop(ctx: Context<'_>) -> Result<(), Error> {
    let uuid_boop = ctx.id();

    ctx.send(|m| {
        m.content("I want some boops!").components(|c| {
            c.create_action_row(|ar| {
                ar.create_button(|b| {
                    b.style(serenity::ButtonStyle::Primary)
                        .label("Boop me!")
                        .custom_id(uuid_boop)
                })
            })
        })
    })
    .await?;

    let mut boop_count = 0;
    while let Some(mci) = serenity::CollectComponentInteraction::new(ctx)
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(120))
        .filter(move |mci| mci.data.custom_id == uuid_boop.to_string())
        .await
    {
        boop_count += 1;

        let mut msg = mci.message.clone();
        msg.edit(ctx, |m| m.content(format!("Boop count: {}", boop_count)))
            .await?;

        mci.create_interaction_response(ctx, |ir| {
            ir.kind(serenity::InteractionResponseType::DeferredUpdateMessage)
        })
        .await?;
    }

    Ok(())
}

/// info about a voice channel
#[poise::command(slash_command)]
pub async fn voiceinfo(
    ctx: Context<'_>,
    #[description = "Information about a server voice channel"]
    #[channel_types("Voice")]
    channel: serenity::GuildChannel,
) -> Result<(), Error> {
    let response = format!(
        "\
**Name**: {}
**Bitrate**: {}
**User limit**: {}
**RTC region**: {}
**Video quality mode**: {:?}",
        channel.name,
        channel.bitrate.unwrap_or_default(),
        channel.user_limit.unwrap_or_default(),
        channel.rtc_region.unwrap_or_default(),
        channel
            .video_quality_mode
            .unwrap_or(serenity::VideoQualityMode::Unknown)
    );

    ctx.say(response).await?;
    Ok(())
}

/// basically bash echo
#[poise::command(prefix_command, slash_command)]
pub async fn echo(
    ctx: Context<'_>,
    #[rest]
    #[description = "Text to say"]
    msg: String,
) -> Result<(), Error> {
    ctx.say(msg).await?;
    Ok(())
}

/// lists the servers a bot is in
#[cfg(feature = "cache")]
#[poise::command(slash_command, prefix_command)]
pub async fn servers(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::servers(ctx).await?;
    Ok(())
}

/// replies to the command
#[poise::command(slash_command, prefix_command)]
pub async fn reply(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(|b| {
        b.content(format!("Hello {}!", ctx.author().name))
            .reply(true)
    }).await?;
    Ok(())
}

/// Mallet of loving correction
#[poise::command(slash_command, prefix_command)]
pub async fn bonk(ctx: Context<'_>, user: serenity::User) -> Result<(), Error> {
    ctx.send(|m| {
        m.content(format!("Bonked {} with THE mallet", serenity::Mention::from(user.id))).reply(false)
    }).await?;
    Ok(())
}

/// Pin commands
#[poise::command(slash_command, prefix_command)]
pub async fn pin(ctx: Context<'_>, 
    #[description = "Message to echo (enter a link or ID)"] msg: serenity::Message
) -> Result<(), Error> {
    msg.pin(ctx).await?;
    ctx.send(|b| {
        b.content(format!("Pinned {}!", msg.id))
            .reply(true)
    }).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn name(ctx: Context<'_>,
    #[description = "Person to rename"] member: serenity::Member,
    #[description = "Name to name the person"] name: String
) -> Result<(), Error> {
    member.edit(ctx, |m| m.nickname(name.clone())).await?;
    ctx.send(|b| {
        b.content(format!("Renamed {} to {}!", member.display_name(), name))
            .reply(true)
    }).await?;
    Ok(())
}
