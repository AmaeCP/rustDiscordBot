use std::time::Duration;

use crate::audio::source::AudioSource;
use crate::audio::ytdl;
use crate::error::BotError;
use crate::types::{Context, Error};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn replay(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::VoiceJoin(
        "Must be used in a server".to_string(),
    ))?;

    let queue_lock = ctx.data().get_queue(guild_id);
    let queue = queue_lock.read().await;

    if let Some(handle) = &queue.handle {
        let _ = handle.seek(Duration::from_secs(0));
        ctx.say("🔄 Replaying current track from the beginning.").await?;
    } else {
        ctx.say("Nothing is playing right now.").await?;
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn previous(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::VoiceJoin(
        "Must be used in a server".to_string(),
    ))?;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or(BotError::VoiceNotConnected)?;

    let handler = manager
        .get(guild_id)
        .ok_or(BotError::VoiceNotConnected)?;

    let queue_lock = ctx.data().get_queue(guild_id);
    let prev_track = {
        let mut queue = queue_lock.write().await;
        queue.previous()
    };

    if let Some(track) = prev_track {
        let input: songbird::input::Input = match &track.source {
            AudioSource::Youtube { url } | AudioSource::Playlist { url } => {
                ytdl::build_source(url.clone(), ctx.data().http_client.clone()).into()
            }
            AudioSource::LocalFile { path } => {
                songbird::input::File::new(path.clone()).into()
            }
        };

        let mut call = handler.lock().await;
        let handle = call.play_input(input);
        drop(call);

        {
            let mut queue = queue_lock.write().await;
            let _ = handle.set_volume(queue.volume);
            queue.current = Some(track.clone());
            queue.handle = Some(handle);
        }

        ctx.say(format!(
            "⏮️ Playing previous track: **{}** `[{}]`",
            track.title,
            track.duration_string()
        ))
        .await?;
    } else {
        ctx.say("No previous track found in history.").await?;
    }

    Ok(())
}
