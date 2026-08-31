use crate::error::BotError;
use crate::types::{Context, Error};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn volume(
    ctx: Context<'_>,
    #[description = "Volume level (1 to 200)"]
    #[min = 1]
    #[max = 200]
    level: u32,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::VoiceJoin(
        "Must be used in a server".to_string(),
    ))?;

    let vol_float = level as f32 / 100.0;

    let queue_lock = ctx.data().get_queue(guild_id);
    let mut queue = queue_lock.write().await;
    queue.set_volume(vol_float);

    let icon = if level == 0 {
        "🔇"
    } else if level < 50 {
        "🔉"
    } else {
        "🔊"
    };

    ctx.say(format!("{icon} Volume set to **{level}%**")).await?;

    Ok(())
}
