use crate::audio::source::AudioSource;
use crate::audio::ytdl;
use crate::error::BotError;
use crate::types::{Context, Error};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn skip(ctx: Context<'_>) -> Result<(), Error> {
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
    let next_track = {
        let mut queue = queue_lock.write().await;
        queue.skip()
    };

    if let Some(track) = next_track {
        {
            let mut queue = queue_lock.write().await;
            queue.current = Some(track.clone());
        }

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
            queue.handle = Some(handle);
        }

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
    }

    Ok(())
}
