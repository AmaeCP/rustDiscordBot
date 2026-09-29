use crate::error::BotError;
use crate::types::{Context, Error};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn shuffle(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::NotInGuild)?;

    let queue_lock = ctx.data().get_queue(guild_id);
    let count = {
        let mut queue = queue_lock.write().await;
        if queue.tracks.is_empty() {
            None
        } else {
            let count = queue.tracks.len();
            queue.shuffle();
            drop(queue);
            Some(count)
        }
    };

    if let Some(count) = count {
        ctx.say(format!("🔀 Shuffled **{count}** tracks in the queue!"))
            .await?;
    } else {
        ctx.say("Queue is empty. Nothing to shuffle.").await?;
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
