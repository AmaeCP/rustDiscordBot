use crate::error::BotError;
use crate::playback;
use crate::types::{Context, Error};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::NotInGuild)?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or(BotError::VoiceNotConnected)?;

    let handler = manager.get(guild_id).ok_or(BotError::VoiceNotConnected)?;

    let queue_lock = ctx.data().get_queue(guild_id);
    let next_track = {
        let mut queue = queue_lock.write().await;
        queue.skip()
    };

    if let Some(track) = next_track {
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
            "⏭️ Skipped! Now playing: **{}** `[{}]`",
            track.title,
            track.duration_string()
        ))
        .await?;
    } else {
        {
            let mut queue = queue_lock.write().await;
            queue.current = None;
            queue.handle = None;
        }
        ctx.say("⏭️ Skipped! Queue is now empty.").await?;
        playback::schedule_idle_disconnect(
            queue_lock.clone(),
            manager,
            guild_id,
            ctx.data().clone(),
        )
        .await;
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
