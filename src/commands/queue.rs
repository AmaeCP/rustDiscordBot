use crate::error::BotError;
use crate::types::{Context, Error};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn queue(
    ctx: Context<'_>,
    #[description = "Page number"] page: Option<usize>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::VoiceJoin(
        "Must be used in a server".to_string(),
    ))?;

    let queue_lock = ctx.data().get_queue(guild_id);
    let queue = queue_lock.read().await;

    let mut response = String::new();

    if let Some(current) = &queue.current {
        response.push_str(&format!(
            "🎶 **Now Playing:** {} `[{}]` — <@{}>\n\n",
            current.title,
            current.duration_string(),
            current.requester
        ));
    }

    let list = queue.list_queue(page.unwrap_or(1));
    response.push_str(&list);

    ctx.say(response).await?;

    Ok(())
}
