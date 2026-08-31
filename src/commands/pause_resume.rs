use crate::error::BotError;
use crate::types::{Context, Error};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn pause(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::VoiceJoin(
        "Must be used in a server".to_string(),
    ))?;

    let queue_lock = ctx.data().get_queue(guild_id);
    let queue = queue_lock.read().await;

    if let Some(handle) = &queue.handle {
        let _ = handle.pause();
        ctx.say("⏸️ Paused.").await?;
    } else {
        ctx.say("Nothing is playing right now.").await?;
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn resume(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::VoiceJoin(
        "Must be used in a server".to_string(),
    ))?;

    let queue_lock = ctx.data().get_queue(guild_id);
    let queue = queue_lock.read().await;

    if let Some(handle) = &queue.handle {
        let _ = handle.play();
        ctx.say("▶️ Resumed.").await?;
    } else {
        ctx.say("Nothing is paused right now.").await?;
    }

    Ok(())
}
