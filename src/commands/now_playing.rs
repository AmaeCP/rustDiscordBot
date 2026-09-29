use crate::audio::source::AudioSource;
use crate::audio::ytdl;
use crate::error::BotError;
use crate::types::{Context, Error};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn nowplaying(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::NotInGuild)?;

    let queue_lock = ctx.data().get_queue(guild_id);
    let (title, source, requester, total_secs, handle) = {
        let queue = queue_lock.read().await;
        let current = queue.current.as_ref().ok_or(BotError::QueueEmpty)?;
        (
            current.title.clone(),
            current.source.clone(),
            current.requester,
            current.duration.unwrap_or(0.0),
            queue.handle.clone(),
        )
    };

    let position_secs = if let Some(handle) = &handle {
        handle
            .get_info()
            .await
            .map_or(0.0, |i| i.position.as_secs_f64())
    } else {
        0.0
    };

    let progress_bar = build_progress_bar(position_secs, total_secs);

    let display_title = format_title(&title, &source);

    let response = format!(
        "🎶 **Now Playing**\n\n{}\n\nRequested by: <@{}>\n\n{} `{}` / `{}`",
        display_title,
        requester,
        progress_bar,
        ytdl::format_duration(Some(position_secs)),
        ytdl::format_duration(Some(total_secs)),
    );

    ctx.say(response).await?;

    Ok(())
}

fn format_title(title: &str, source: &AudioSource) -> String {
    match source {
        AudioSource::Youtube { url } | AudioSource::Playlist { url }
            if url.starts_with("https://") || url.starts_with("http://") =>
        {
            format!("[**{title}**]({url})")
        }
        AudioSource::Youtube { .. }
        | AudioSource::Playlist { .. }
        | AudioSource::LocalFile { .. } => format!("**{title}**"),
    }
}

fn build_progress_bar(position: f64, total: f64) -> String {
    const BAR_LENGTH: usize = 20;

    if total <= 0.0 {
        let empty = "─".repeat(BAR_LENGTH);
        return format!("[🔘{empty}]");
    }

    let progress = (position / total).clamp(0.0, 1.0);
    let filled = (progress * BAR_LENGTH as f64).round() as usize;
    let filled = filled.min(BAR_LENGTH);
    let remaining = BAR_LENGTH.saturating_sub(filled);

    let before = "▓".repeat(filled);
    let after = "─".repeat(remaining);

    format!("[{before}🔘{after}]")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn local_track_title_is_not_rendered_as_a_link() {
        let source = AudioSource::LocalFile {
            path: PathBuf::from("/tmp/song.mp3"),
        };

        assert_eq!(format_title("Song", &source), "**Song**");
    }

    #[test]
    fn online_track_title_links_to_its_source() {
        let source = AudioSource::Youtube {
            url: "https://youtube.com/watch?v=id".to_string(),
        };

        assert_eq!(
            format_title("Song", &source),
            "[**Song**](https://youtube.com/watch?v=id)"
        );
    }
}
