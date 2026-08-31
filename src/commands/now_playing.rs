use crate::audio::ytdl;
use crate::error::BotError;
use crate::types::{Context, Error};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn nowplaying(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::VoiceJoin(
        "Must be used in a server".to_string(),
    ))?;

    let queue_lock = ctx.data().get_queue(guild_id);
    let queue = queue_lock.read().await;

    let current = queue.current.as_ref().ok_or(BotError::QueueEmpty)?;

    let (position_secs, total_secs) = if let Some(handle) = &queue.handle {
        let info = handle.get_info().await;
        let pos = info
            .map(|i| i.position.as_secs_f64())
            .unwrap_or(0.0);
        (pos, current.duration.unwrap_or(0.0))
    } else {
        (0.0, current.duration.unwrap_or(0.0))
    };

    let progress_bar = build_progress_bar(position_secs, total_secs);

    let response = format!(
        "🎶 **Now Playing**\n\n**{}**\n\nRequested by: <@{}>\n\n{} `{}` / `{}`",
        current.title,
        current.requester,
        progress_bar,
        ytdl::format_duration(Some(position_secs)),
        ytdl::format_duration(Some(total_secs)),
    );

    ctx.say(response).await?;

    Ok(())
}

fn build_progress_bar(position: f64, total: f64) -> String {
    let bar_length = 20;

    if total <= 0.0 {
        let empty: String = "─".repeat(bar_length);
        return format!("[🔘{empty}]");
    }

    let progress = (position / total).clamp(0.0, 1.0);
    let filled = (progress * bar_length as f64) as usize;
    let remaining = bar_length - filled;

    let before = "▓".repeat(filled);
    let after = "─".repeat(remaining);

    format!("[{before}🔘{after}]")
}
