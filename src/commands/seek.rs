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
                mins.checked_mul(60)?
                    .checked_add(secs)
                    .map(Duration::from_secs)
            }
            3 => {
                let hours: u64 = parts[0].parse().ok()?;
                let mins: u64 = parts[1].parse().ok()?;
                let secs: u64 = parts[2].parse().ok()?;
                hours
                    .checked_mul(3600)?
                    .checked_add(mins.checked_mul(60)?)?
                    .checked_add(secs)
                    .map(Duration::from_secs)
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
    let guild_id = ctx.guild_id().ok_or(BotError::NotInGuild)?;

    let duration =
        parse_time(&position).ok_or_else(|| BotError::InvalidTimestamp(position.clone()))?;

    let queue_lock = ctx.data().get_queue(guild_id);
    let handle = queue_lock.read().await.handle.clone();

    if let Some(handle) = handle {
        let _ = handle.seek(duration);
        ctx.say(format!(
            "⏩ Seeked to **{}**",
            ytdl::format_duration(Some(duration.as_secs_f64()))
        ))
        .await?;
    } else {
        ctx.say("Nothing is playing right now.").await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time_seconds() {
        assert_eq!(parse_time("90"), Some(Duration::from_secs(90)));
    }

    #[test]
    fn test_parse_time_minutes_seconds() {
        assert_eq!(parse_time("1:30"), Some(Duration::from_secs(90)));
    }

    #[test]
    fn test_parse_time_hours_minutes_seconds() {
        assert_eq!(parse_time("1:02:05"), Some(Duration::from_secs(3725)));
    }

    #[test]
    fn test_parse_time_invalid() {
        assert_eq!(parse_time("abc"), None);
        assert_eq!(parse_time("1:2:3:4"), None);
        assert_eq!(parse_time("18446744073709551615:0"), None);
        assert_eq!(parse_time("1:18446744073709551615:0"), None);
    }
}
