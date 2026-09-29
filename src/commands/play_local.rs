use poise::serenity_prelude as serenity;

use crate::audio::source::AudioSource;
use crate::error::BotError;
use crate::playback;
use crate::queue::track::QueuedTrack;
use crate::types::{Context, Error};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn play_local(
    ctx: Context<'_>,
    #[description = "Upload an audio file directly from your computer"] file: Option<
        serenity::Attachment,
    >,
    #[description = "Or path to local audio file on host/VPS"] path: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;

    if file.is_none() && path.is_none() {
        ctx.say("❌ Please attach an audio `file` or specify a disk `path`.")
            .await?;
        return Ok(());
    }

    let (guild_id, channel_id) = crate::commands::play::get_voice_channel(ctx)?;
    let (handler, manager) =
        crate::commands::play::ensure_voice_connection(ctx, guild_id, channel_id).await?;
    {
        let queue = ctx.data().get_queue(guild_id);
        let mut state = queue.write().await;
        state.idle_generation = state.idle_generation.wrapping_add(1);
        state.idle_waiting = false;
    }

    if let Some(attachment) = file {
        let file_name = attachment.filename.clone();
        let url = attachment.url.clone();

        let track = QueuedTrack::new(
            file_name.clone(),
            None,
            ctx.author().id,
            AudioSource::Youtube { url: url.clone() },
        );

        let queue_lock = ctx.data().get_queue(guild_id);
        let should_play = {
            let mut queue = queue_lock.write().await;
            let was_empty = queue.is_empty();
            queue.add_track(track.clone());
            was_empty
        };

        if should_play {
            queue_lock.write().await.tracks.pop_front();
            queue_lock.write().await.current = Some(track.clone());
            playback::play_track(
                &handler,
                &track,
                queue_lock.clone(),
                ctx.data().clone(),
                manager.clone(),
                guild_id,
            )
            .await?;

            ctx.say(format!("🎵 Now playing uploaded file: **{file_name}**"))
                .await?;
        } else {
            let pos = queue_lock.read().await.tracks.len();
            ctx.say(format!(
                "📥 Added uploaded file to queue at position **#{pos}**: **{file_name}**"
            ))
            .await?;
        }

        crate::panel::sync_panel(
            &ctx.serenity_context().http,
            ctx.data(),
            guild_id,
            ctx.channel_id(),
        )
        .await;

        return Ok(());
    }

    if let Some(path_str) = path {
        let file_path = std::path::PathBuf::from(&path_str);
        if !file_path.exists() {
            return Err(BotError::FileNotFound(file_path).into());
        }

        let file_name = file_path.file_name().map_or_else(
            || "Local File".to_string(),
            |n| n.to_string_lossy().into_owned(),
        );

        let source = AudioSource::LocalFile {
            path: file_path.clone(),
        };
        let track = QueuedTrack::new(file_name.clone(), None, ctx.author().id, source);

        let queue_lock = ctx.data().get_queue(guild_id);
        let should_play = {
            let mut queue = queue_lock.write().await;
            let was_empty = queue.is_empty();
            queue.add_track(track.clone());
            was_empty
        };

        if should_play {
            {
                let mut queue = queue_lock.write().await;
                queue.tracks.pop_front();
                queue.current = Some(track.clone());
            }

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

            ctx.say(format!("🎵 Now playing local file: **{file_name}**"))
                .await?;
        } else {
            let pos = queue_lock.read().await.tracks.len();
            ctx.say(format!(
                "📥 Added local file to queue at position **#{pos}**: **{file_name}**"
            ))
            .await?;
        }
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
