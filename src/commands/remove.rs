use crate::error::BotError;
use crate::queue::manager::QueueIndex;
use crate::types::{Context, Error};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "Track number to remove (1-based)"] index: usize,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::NotInGuild)?;

    let queue_idx = QueueIndex::new(index);
    let queue_lock = ctx.data().get_queue(guild_id);
    let removed = {
        let mut queue = queue_lock.write().await;
        queue.remove_track(queue_idx)?
    };

    ctx.say(format!("🗑️ Removed **{}** from the queue.", removed.title))
        .await?;

    Ok(())
}
