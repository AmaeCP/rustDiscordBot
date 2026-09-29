use poise::serenity_prelude as serenity;

use crate::audio::source::AudioSource;
use crate::audio::ytdl;
use crate::error::BotError;
use crate::queue::track::QueuedTrack;
use crate::types::{Context, Error};

fn split_urls(input: &str) -> Vec<String> {
    let spaced = input
        .replace("https://", " https://")
        .replace("http://", " http://");
    spaced
        .split_whitespace()
        .filter(|t| t.starts_with("http://") || t.starts_with("https://"))
        .map(ToString::to_string)
        .collect()
}

fn get_voice_channel(ctx: Context<'_>) -> Result<(serenity::GuildId, serenity::ChannelId), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::NotInGuild)?;

    let author_id = ctx.author().id;

    if let Some(channel_id) = ctx.data().get_user_voice(guild_id, author_id) {
        return Ok((guild_id, channel_id));
    }

    let cache = &ctx.serenity_context().cache;
    if let Some(guild) = cache.guild(guild_id) {
        if let Some(channel_id) = guild
            .voice_states
            .get(&author_id)
            .and_then(|vs| vs.channel_id)
        {
            return Ok((guild_id, channel_id));
        }
    }

    Err(BotError::UserNotInVoice.into())
}

async fn ensure_voice_connection(
    ctx: Context<'_>,
    guild_id: serenity::GuildId,
    channel_id: serenity::ChannelId,
) -> Result<std::sync::Arc<tokio::sync::Mutex<songbird::Call>>, Error> {
    let manager = songbird::get(ctx.serenity_context())
        .await
        .ok_or(BotError::SongbirdNotInitialized)?;

    let handler = if let Some(handler) = manager.get(guild_id) {
        handler
    } else {
        manager
            .join(guild_id, channel_id)
            .await
            .map_err(|e| BotError::VoiceJoin(format!("{e}")))?
    };

    Ok(handler)
}

async fn play_next(
    handler: &std::sync::Arc<tokio::sync::Mutex<songbird::Call>>,
    track: &QueuedTrack,
    http_client: reqwest::Client,
) -> Result<songbird::tracks::TrackHandle, Error> {
    let input: songbird::input::Input = match &track.source {
        AudioSource::Youtube { url } | AudioSource::Playlist { url } => {
            let src = ytdl::build_source(url.clone(), http_client);
            src.into()
        }
        AudioSource::LocalFile { path } => songbird::input::File::new(path.clone()).into(),
    };

    let handle = handler.lock().await.play_input(input);
    Ok(handle)
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "YouTube URL(s), search queries, or playlist URL"]
    #[rest]
    query: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let (guild_id, channel_id) = get_voice_channel(ctx)?;
    let handler = ensure_voice_connection(ctx, guild_id, channel_id).await?;

    let url_tokens = split_urls(&query);

    if url_tokens.len() > 1 {
        let mut tracks = Vec::with_capacity(url_tokens.len());

        for url in &url_tokens {
            let src = AudioSource::from_input(url);
            let target_url = src.url();

            if let Ok(metadata) = ytdl::extract_metadata(target_url).await {
                let title = metadata.title.unwrap_or_else(|| "Unknown".to_string());
                let actual_url = metadata
                    .webpage_url
                    .unwrap_or_else(|| target_url.to_string());

                tracks.push(QueuedTrack::new(
                    title,
                    metadata.duration,
                    ctx.author().id,
                    AudioSource::Youtube { url: actual_url },
                ));
            }
        }

        let count = tracks.len();
        if count == 0 {
            ctx.say("❌ Failed to extract audio from the provided URLs.")
                .await?;
            return Ok(());
        }

        let queue_lock = ctx.data().get_queue(guild_id);
        let should_play = {
            let mut queue = queue_lock.write().await;
            let was_empty = queue.is_empty();
            queue.add_playlist(tracks.clone());
            was_empty
        };

        if should_play {
            if let Some(first) = tracks.into_iter().next() {
                {
                    let mut queue = queue_lock.write().await;
                    queue.tracks.pop_front();
                    queue.current = Some(first.clone());
                }
                let handle = play_next(&handler, &first, ctx.data().http_client.clone()).await?;
                {
                    let mut queue = queue_lock.write().await;
                    let _ = handle.set_volume(queue.volume.as_f32());
                    queue.handle = Some(handle);
                }
            }
        }

        ctx.say(format!("📋 Added **{count}** tracks to the queue!"))
            .await?;
        return Ok(());
    }

    let single_query = if url_tokens.len() == 1 {
        &url_tokens[0]
    } else {
        &query
    };

    let source = AudioSource::from_input(single_query);

    match &source {
        AudioSource::Playlist { url } => {
            let entries = ytdl::extract_playlist_entries(url)
                .await
                .unwrap_or_default();
            let count = entries.len();

            if count > 0 {
                let mut tracks = Vec::with_capacity(count);
                for entry in entries {
                    let entry_url = entry.url.unwrap_or_default();
                    let full_url = if entry_url.starts_with("http") {
                        entry_url
                    } else {
                        format!("https://www.youtube.com/watch?v={entry_url}")
                    };

                    tracks.push(QueuedTrack::new(
                        entry.title.unwrap_or_else(|| "Unknown".to_string()),
                        entry.duration,
                        ctx.author().id,
                        AudioSource::Youtube { url: full_url },
                    ));
                }

                let queue_lock = ctx.data().get_queue(guild_id);
                let should_play = {
                    let mut queue = queue_lock.write().await;
                    let was_empty = queue.is_empty();
                    queue.add_playlist(tracks.clone());
                    was_empty
                };

                if should_play {
                    if let Some(first) = tracks.into_iter().next() {
                        {
                            let mut queue = queue_lock.write().await;
                            queue.tracks.pop_front();
                            queue.current = Some(first.clone());
                        }
                        let handle =
                            play_next(&handler, &first, ctx.data().http_client.clone()).await?;
                        {
                            let mut queue = queue_lock.write().await;
                            let _ = handle.set_volume(queue.volume.as_f32());
                            queue.handle = Some(handle);
                        }
                    }
                }

                ctx.say(format!(
                    "📋 Added **{count}** tracks from playlist to the queue!"
                ))
                .await?;
            } else {
                play_single(ctx, guild_id, &handler, url).await?;
            }
        }

        AudioSource::Youtube { url } => {
            play_single(ctx, guild_id, &handler, url).await?;
        }

        AudioSource::LocalFile { .. } => {
            ctx.say("Use `/play_local` for local files.").await?;
        }
    }

    Ok(())
}

async fn play_single(
    ctx: Context<'_>,
    guild_id: serenity::GuildId,
    handler: &std::sync::Arc<tokio::sync::Mutex<songbird::Call>>,
    url: &str,
) -> Result<(), Error> {
    let metadata = ytdl::extract_metadata(url).await?;
    let title = metadata.title.unwrap_or_else(|| "Unknown".to_string());
    let duration = metadata.duration;
    let actual_url = metadata.webpage_url.unwrap_or_else(|| url.to_string());

    let track = QueuedTrack::new(
        title.clone(),
        duration,
        ctx.author().id,
        AudioSource::Youtube { url: actual_url },
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
        let handle = play_next(handler, &track, ctx.data().http_client.clone()).await?;
        {
            let mut queue = queue_lock.write().await;
            let _ = handle.set_volume(queue.volume.as_f32());
            queue.handle = Some(handle);
        }
        ctx.say(format!(
            "🎶 Now playing: **{}** `[{}]`",
            title,
            ytdl::format_duration(duration)
        ))
        .await?;
    } else {
        let pos = queue_lock.read().await.tracks.len();
        ctx.say(format!(
            "📥 Added to queue at position **#{pos}**: **{title}** `[{}]`",
            ytdl::format_duration(duration)
        ))
        .await?;
    }

    Ok(())
}
