use crate::error::BotError;
use crate::queue::manager::LoopMode;
use crate::types::{Context, Error};

#[poise::command(slash_command, prefix_command, guild_only, rename = "loop")]
pub async fn loop_cmd(
    ctx: Context<'_>,
    #[description = "Loop mode: Off, Track, or Queue"] mode: Option<LoopMode>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::VoiceJoin(
        "Must be used in a server".to_string(),
    ))?;

    let queue_lock = ctx.data().get_queue(guild_id);
    let mut queue = queue_lock.write().await;

    let new_mode = if let Some(m) = mode {
        queue.set_loop(m);
        m
    } else {
        *queue.toggle_loop()
    };

    ctx.say(format!("🔁 Loop mode set to: **{}**", new_mode.display())).await?;

    Ok(())
}
