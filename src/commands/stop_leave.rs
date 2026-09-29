use crate::error::BotError;
use crate::playback;
use crate::types::{Context, Error};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn clear(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::NotInGuild)?;

    let queue_lock = ctx.data().get_queue(guild_id);
    let count = {
        let mut queue = queue_lock.write().await;
        let len = queue.tracks.len();
        queue.tracks.clear();
        len
    };

    ctx.say(format!("🗑️ Cleared **{count}** tracks from the queue."))
        .await?;
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
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::NotInGuild)?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or(BotError::VoiceNotConnected)?;

    if manager.get(guild_id).is_none() {
        return Err(BotError::VoiceNotConnected.into());
    }

    let queue_lock = ctx.data().get_queue(guild_id);
    {
        let mut queue = queue_lock.write().await;
        queue.clear();
    }

    ctx.say("⏹️ Stopped and cleared the entire queue.").await?;
    playback::schedule_idle_disconnect(queue_lock, manager, guild_id, ctx.data().clone()).await;
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
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::NotInGuild)?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or(BotError::VoiceNotConnected)?;

    if manager.get(guild_id).is_none() {
        ctx.say("I'm not in a voice channel.").await?;
        return Ok(());
    }

    manager
        .remove(guild_id)
        .await
        .map_err(|e| BotError::VoiceJoin(format!("Failed to leave: {e}")))?;

    let queue_lock = ctx.data().get_queue(guild_id);
    queue_lock.write().await.reset_after_disconnect();
    crate::panel::disable_panel(&ctx.serenity_context().http, ctx.data(), guild_id).await;

    ctx.say("👋 Left the voice channel.").await?;

    Ok(())
}
