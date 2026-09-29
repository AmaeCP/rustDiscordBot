use std::fmt::Write;

use crate::error::BotError;
use crate::types::{Context, Error};

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn queue(
    ctx: Context<'_>,
    #[description = "Page number"] page: Option<usize>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or(BotError::NotInGuild)?;

    let queue_lock = ctx.data().get_queue(guild_id);
    let response = {
        let queue = queue_lock.read().await;
        let mut response = String::new();

        if let Some(current) = &queue.current {
            let _ = writeln!(
                response,
                "🎶 **Now Playing:** {} `[{}]` — <@{}>\n",
                current.title,
                current.duration_string(),
                current.requester
            );
        }

        let list = queue.list_queue(page.unwrap_or(1));
        drop(queue);
        response.push_str(&list);
        response
    };

    ctx.say(response).await?;
    crate::panel::sync_panel(
        &ctx.serenity_context().http,
        ctx.data(),
        guild_id,
        ctx.channel_id(),
    )
    .await;

    Ok(())
}
