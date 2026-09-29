use crate::error::BotError;
use crate::types::{Context, Error};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn pause(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::NotInGuild)?;

    let queue_lock = ctx.data().get_queue(guild_id);
    let handle = queue_lock.read().await.handle.clone();
    queue_lock.write().await.paused = true;

    if let Some(handle) = handle {
        let _ = handle.pause();
        ctx.say("⏸️ Paused.").await?;
    } else {
        ctx.say("Nothing is playing right now.").await?;
    }

    crate::panel::sync_panel(
        &ctx.serenity_context().http,
        ctx.data(),
        guild_id,
        ctx.channel_id(),
    )
    .await;

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn resume(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::NotInGuild)?;

    let queue_lock = ctx.data().get_queue(guild_id);
    let handle = queue_lock.read().await.handle.clone();
    queue_lock.write().await.paused = false;

    if let Some(handle) = handle {
        let _ = handle.play();
        ctx.say("▶️ Resumed.").await?;
    } else {
        ctx.say("Nothing is paused right now.").await?;
    }

    crate::panel::sync_panel(
        &ctx.serenity_context().http,
        ctx.data(),
        guild_id,
        ctx.channel_id(),
    )
    .await;

    Ok(())
}
