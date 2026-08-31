use crate::error::BotError;
use crate::types::{Context, Error};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn shuffle(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::VoiceJoin(
        "Must be used in a server".to_string(),
    ))?;

    let queue_lock = ctx.data().get_queue(guild_id);
    let mut queue = queue_lock.write().await;

    if queue.tracks.is_empty() {
        ctx.say("Queue is empty. Nothing to shuffle.").await?;
        return Ok(());
    }

    let count = queue.tracks.len();
    queue.shuffle();

    ctx.say(format!("🔀 Shuffled **{count}** tracks in the queue!")).await?;

    Ok(())
}
