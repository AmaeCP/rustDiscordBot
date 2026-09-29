use crate::error::BotError;
use crate::queue::manager::Volume;
use crate::types::{Context, Error};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn volume(
    ctx: Context<'_>,
    #[description = "Volume level (0 to 200)"]
    #[min = 0]
    #[max = 200]
    level: u32,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::NotInGuild)?;

    let level = level.min(200);
    let vol = Volume::from_percent(level);

    let queue_lock = ctx.data().get_queue(guild_id);
    queue_lock.write().await.set_volume(vol);

    let icon = if level == 0 {
        "🔇"
    } else if level < 50 {
        "🔉"
    } else {
        "🔊"
    };

    ctx.say(format!("{icon} Volume set to **{level}%**"))
        .await?;
    crate::panel::sync_panel(
        &ctx.serenity_context().http,
        ctx.data(),
        guild_id,
        ctx.channel_id(),
    )
    .await;

    Ok(())
}
