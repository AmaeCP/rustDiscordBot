use poise::serenity_prelude as serenity;

use crate::audio::source::AudioSource;
use crate::audio::ytdl;
use crate::error::BotError;
use crate::queue::track::QueuedTrack;
use crate::types::{Context, Error};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn play_local(
    ctx: Context<'_>,
    #[description = "Upload an audio file directly from your computer"] file: Option<serenity::Attachment>,
    #[description = "Or path to local audio file on host/VPS"] path: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;

    if file.is_none() && path.is_none() {
        ctx.say("❌ Please attach an audio `file` or specify a disk `path`.").await?;
        return Ok(());
    }

    let guild_id = ctx.guild_id().ok_or(BotError::VoiceJoin(
        "Must be used in a server".to_string(),
    ))?;

    let author_id = ctx.author().id;

    let channel_id = if let Some(ch) = ctx.data().get_user_voice(guild_id, author_id) {
        ch
    } else {
        let cache = &ctx.serenity_context().cache;
        cache
            .guild(guild_id)
            .and_then(|g| {
                g.voice_states
                    .get(&author_id)
                    .and_then(|vs| vs.channel_id)
            })
            .ok_or(BotError::UserNotInVoice)?
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or(BotError::VoiceJoin("Songbird not initialized".to_string()))?;

    let handler = if let Some(handler) = manager.get(guild_id) {
        handler
    } else {
        manager
            .join(guild_id, channel_id)
            .await
            .map_err(|e| BotError::VoiceJoin(format!("{e}")))?
    };

    if let Some(attachment) = file {
        let file_name = attachment.filename.clone();
        let url = attachment.url.clone();

        let track = QueuedTrack::new(
            file_name.clone(),
            None,
            ctx.author().id,
            AudioSource::Youtube { url: url.clone() },
            url.clone(),
        );

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

            let input: songbird::input::Input = ytdl::build_source(url, ctx.data().http_client.clone()).into();
            let mut call = handler.lock().await;
            let handle = call.play_input(input);
            drop(call);

            {
                let mut queue = queue_lock.write().await;
                queue.handle = Some(handle);
            }

            ctx.say(format!("🎵 Now playing uploaded file: **{file_name}**")).await?;
        } else {
            let queue = queue_lock.read().await;
            let pos = queue.tracks.len();
            ctx.say(format!("📥 Added uploaded file to queue at position **#{pos}**: **{file_name}**")).await?;
        }

        return Ok(());
    }

    if let Some(path_str) = path {
        let file_path = std::path::PathBuf::from(&path_str);
        if !file_path.exists() {
            return Err(BotError::AudioSource(format!("File not found: {path_str}")).into());
        }

        let file_name = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Local File".to_string());

        let source = AudioSource::LocalFile {
            path: file_path.clone(),
        };
        let track = QueuedTrack::new(
            file_name.clone(),
            None,
            ctx.author().id,
            source,
            path_str.clone(),
        );

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

            let input: songbird::input::Input =
                songbird::input::File::new(file_path).into();
            let mut call = handler.lock().await;
            let handle = call.play_input(input);
            drop(call);

            {
                let mut queue = queue_lock.write().await;
                queue.handle = Some(handle);
            }

            ctx.say(format!("🎵 Now playing local file: **{file_name}**"))
                .await?;
        } else {
            let queue = queue_lock.read().await;
            let pos = queue.tracks.len();
            ctx.say(format!(
                "📥 Added local file to queue at position **#{pos}**: **{file_name}**"
            ))
            .await?;
        }
    }

    Ok(())
}
