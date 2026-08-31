use crate::error::BotError;
use crate::types::{Context, Error};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn clear(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::VoiceJoin(
        "Must be used in a server".to_string(),
    ))?;

    let queue_lock = ctx.data().get_queue(guild_id);
    let count = {
        let mut queue = queue_lock.write().await;
        let len = queue.tracks.len();
        queue.tracks.clear();
        len
    };

    ctx.say(format!("🗑️ Cleared **{count}** tracks from the queue.")).await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::VoiceJoin(
        "Must be used in a server".to_string(),
    ))?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or(BotError::VoiceNotConnected)?;

    let handler = manager
        .get(guild_id)
        .ok_or(BotError::VoiceNotConnected)?;

    {
        let mut call = handler.lock().await;
        call.stop();
    }

    let queue_lock = ctx.data().get_queue(guild_id);
    {
        let mut queue = queue_lock.write().await;
        queue.clear();
    }

    ctx.say("⏹️ Stopped and cleared the entire queue.").await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::VoiceJoin(
        "Must be used in a server".to_string(),
    ))?;

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
    {
        let mut queue = queue_lock.write().await;
        queue.clear();
    }

    ctx.say("👋 Left the voice channel.").await?;

    Ok(())
}
