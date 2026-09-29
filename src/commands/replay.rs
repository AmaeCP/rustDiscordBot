use std::time::Duration;

use crate::error::BotError;
use crate::playback;
use crate::types::{Context, Error};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn replay(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::NotInGuild)?;

    let queue_lock = ctx.data().get_queue(guild_id);
    let handle = queue_lock.read().await.handle.clone();

    if let Some(handle) = handle {
        let _ = handle.seek(Duration::from_secs(0));
        ctx.say("🔄 Replaying current track from the beginning.")
            .await?;
    } else {
        ctx.say("Nothing is playing right now.").await?;
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn previous(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::NotInGuild)?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or(BotError::VoiceNotConnected)?;

    let handler = manager.get(guild_id).ok_or(BotError::VoiceNotConnected)?;

    let queue_lock = ctx.data().get_queue(guild_id);
    let prev_track = {
        let mut queue = queue_lock.write().await;
        queue.previous()
    };

    if let Some(track) = prev_track {
        queue_lock.write().await.current = Some(track.clone());
        playback::play_track(
            &handler,
            &track,
            queue_lock.clone(),
            ctx.data().clone(),
            manager,
            guild_id,
        )
        .await?;

        ctx.say(format!(
            "⏮️ Playing previous track: **{}** `[{}]`",
            track.title,
            track.duration_string()
        ))
        .await?;
    } else {
        ctx.say("No previous track found in history.").await?;
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
