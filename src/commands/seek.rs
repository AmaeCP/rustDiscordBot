use std::time::Duration;

use crate::audio::ytdl;
use crate::error::BotError;
use crate::types::{Context, Error};

fn parse_time(input: &str) -> Option<Duration> {
    let input = input.trim();
    if input.contains(':') {
        let parts: Vec<&str> = input.split(':').collect();
        match parts.len() {
            2 => {
                let mins: u64 = parts[0].parse().ok()?;
                let secs: u64 = parts[1].parse().ok()?;
                Some(Duration::from_secs(mins * 60 + secs))
            }
            3 => {
                let hours: u64 = parts[0].parse().ok()?;
                let mins: u64 = parts[1].parse().ok()?;
                let secs: u64 = parts[2].parse().ok()?;
                Some(Duration::from_secs(hours * 3600 + mins * 60 + secs))
            }
            _ => None,
        }
    } else {
        let secs: u64 = input.parse().ok()?;
        Some(Duration::from_secs(secs))
    }
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn seek(
    ctx: Context<'_>,
    #[description = "Timestamp to seek to (e.g. 1:30 or 90)"] position: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::VoiceJoin(
        "Must be used in a server".to_string(),
    ))?;

    let duration = parse_time(&position)
        .ok_or_else(|| BotError::AudioSource("Invalid timestamp format. Use mm:ss or seconds (e.g. 1:30 or 90)".to_string()))?;

    let queue_lock = ctx.data().get_queue(guild_id);
    let queue = queue_lock.read().await;

    if let Some(handle) = &queue.handle {
        let _ = handle.seek(duration);
        ctx.say(format!("⏩ Seeked to **{}**", ytdl::format_duration(Some(duration.as_secs_f64())))).await?;
    } else {
        ctx.say("Nothing is playing right now.").await?;
    }

    Ok(())
}
