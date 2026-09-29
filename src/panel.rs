use poise::serenity_prelude as serenity;
use serenity::{
    builder::{
        CreateActionRow, CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage,
        CreateMessage, EditMessage,
    },
    ButtonStyle,
};

use crate::{playback, queue::manager::Volume, types::Data};

fn controls(disabled: bool) -> Vec<CreateActionRow> {
    let button = |id: &str, label: &str, style| {
        let mut button = CreateButton::new(id).label(label).style(style);
        if disabled {
            button = button.disabled(true);
        }
        button
    };
    vec![
        CreateActionRow::Buttons(vec![
            button("music:previous", "⏮", ButtonStyle::Secondary),
            button("music:pause", "⏯", ButtonStyle::Primary),
            button("music:skip", "⏭", ButtonStyle::Primary),
            button("music:stop", "⏹", ButtonStyle::Danger),
            button("music:queue", "📃 Queue", ButtonStyle::Secondary),
        ]),
        CreateActionRow::Buttons(vec![
            button("music:loop", "🔁 Repeat", ButtonStyle::Secondary),
            button("music:shuffle", "🔀 Shuffle", ButtonStyle::Secondary),
            button("music:vol_down", "🔉 −", ButtonStyle::Secondary),
            button("music:vol_up", "🔊 +", ButtonStyle::Secondary),
            button("music:clear", "🗑 Clear", ButtonStyle::Danger),
        ]),
    ]
}

fn is_current_panel(
    stored: Option<(serenity::ChannelId, serenity::MessageId)>,
    clicked_channel: serenity::ChannelId,
    clicked_message: serenity::MessageId,
) -> bool {
    stored == Some((clicked_channel, clicked_message))
}

async fn panel_text(data: &Data, guild_id: serenity::GuildId) -> String {
    let queue = data.get_queue(guild_id);
    let state = queue.read().await;
    match &state.current {
        Some(track) => format!(
            "🎶 **Сейчас играет:** {} `[{}]`\nВ очереди: {} · Повтор: {}",
            track.title,
            track.duration_string(),
            state.tracks.len(),
            state.loop_mode.display()
        ),
        None => "🎵 **Музыкальное управление**\nСейчас ничего не играет.".to_string(),
    }
}

pub async fn sync_panel(
    http: &serenity::Http,
    data: &Data,
    guild_id: serenity::GuildId,
    channel_id: serenity::ChannelId,
) {
    let panel_lock = data.get_panel_lock(guild_id);
    let _guard = panel_lock.lock().await;
    let queue = data.get_queue(guild_id);
    let (old_panel, active_channel) = {
        let state = queue.read().await;
        (state.control_panel, state.active_channel)
    };
    let message_content = panel_text(data, guild_id).await;
    if let Some((old_channel, old_message)) = old_panel {
        if old_channel == channel_id {
            if let Ok(mut message) = old_channel.message(http, old_message).await {
                if message
                    .edit(
                        http,
                        EditMessage::new()
                            .content(&message_content)
                            .components(controls(false)),
                    )
                    .await
                    .is_ok()
                {
                    return;
                }
            }
        } else if let Ok(mut message) = old_channel.message(http, old_message).await {
            let _ = message
                .edit(http, EditMessage::new().components(controls(true)))
                .await;
        }
    }
    if active_channel.is_none() {
        return;
    }
    if let Ok(message) = channel_id
        .send_message(
            http,
            CreateMessage::new()
                .content(message_content)
                .components(controls(false)),
        )
        .await
    {
        queue.write().await.control_panel = Some((channel_id, message.id));
    }
}

pub async fn refresh_existing_panel(data: &Data, guild_id: serenity::GuildId) {
    let queue = data.get_queue(guild_id);
    let channel = queue.read().await.control_panel.map(|panel| panel.0);
    if let (Some(channel), Some(http)) =
        (channel, data.http.read().ok().and_then(|slot| slot.clone()))
    {
        sync_panel(&http, data, guild_id, channel).await;
    }
}

pub async fn disable_panel(http: &serenity::Http, data: &Data, guild_id: serenity::GuildId) {
    let panel_lock = data.get_panel_lock(guild_id);
    let _guard = panel_lock.lock().await;
    let queue = data.get_queue(guild_id);
    let panel = queue.write().await.control_panel.take();
    if let Some((channel, message_id)) = panel {
        if let Ok(mut message) = channel.message(http, message_id).await {
            let _ = message
                .edit(http, EditMessage::new().components(controls(true)))
                .await;
        }
    }
}

pub async fn handle_component(
    ctx: &serenity::Context,
    data: &Data,
    interaction: &serenity::ComponentInteraction,
) {
    let Some(guild_id) = interaction.guild_id else {
        return;
    };
    let queue = data.get_queue(guild_id);
    let (panel, active_channel) = {
        let state = queue.read().await;
        (state.control_panel, state.active_channel)
    };
    if !is_current_panel(panel, interaction.channel_id, interaction.message.id) {
        let _ = interaction
            .create_response(
                ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("Эта панель уже устарела.")
                        .ephemeral(true),
                ),
            )
            .await;
        return;
    }
    if active_channel.is_none()
        || data.get_user_voice(guild_id, interaction.user.id) != active_channel
    {
        let _ = interaction
            .create_response(
                ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("Чтобы управлять музыкой, зайдите в голосовой канал бота.")
                        .ephemeral(true),
                ),
            )
            .await;
        return;
    }
    let custom_id = interaction.data.custom_id.as_str();
    if custom_id == "music:queue" {
        let response = {
            let state = queue.read().await;
            let mut response = state
                .current
                .as_ref()
                .map(|track| format!("🎶 **Сейчас играет:** {}\n\n", track.title))
                .unwrap_or_default();
            response.push_str(&state.list_queue(1));
            response
        };
        let _ = interaction
            .create_response(
                ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(response)
                        .ephemeral(true),
                ),
            )
            .await;
        return;
    }
    if custom_id == "music:pause" {
        let handle = queue.read().await.handle.clone();
        if let Some(handle) = handle {
            let paused = {
                let mut state = queue.write().await;
                state.paused = !state.paused;
                state.paused
            };
            if paused {
                let _ = handle.pause();
            } else {
                let _ = handle.play();
            }
        }
    } else if custom_id == "music:stop" || custom_id == "music:clear" {
        let mut state = queue.write().await;
        if custom_id == "music:stop" {
            state.clear();
        } else {
            state.tracks.clear();
        }
    } else if custom_id == "music:loop" {
        queue.write().await.toggle_loop();
    } else if custom_id == "music:shuffle" {
        queue.write().await.shuffle();
    } else if custom_id == "music:vol_down" || custom_id == "music:vol_up" {
        let mut state = queue.write().await;
        let percent = ((state.volume.as_f32() * 100.0) as i32
            + if custom_id.ends_with("up") { 10 } else { -10 })
        .clamp(0, 200) as u32;
        state.set_volume(Volume::from_percent(percent));
    } else if custom_id == "music:skip" || custom_id == "music:previous" {
        let manager = match songbird::get(ctx).await {
            Some(manager) => manager,
            None => return,
        };
        let handler = match manager.get(guild_id) {
            Some(handler) => handler,
            None => return,
        };
        let track = {
            let mut state = queue.write().await;
            if custom_id == "music:skip" {
                state.skip()
            } else {
                state.previous()
            }
        };
        if let Some(track) = track {
            queue.write().await.current = Some(track.clone());
            if let Err(error) = playback::play_track(
                &handler,
                &track,
                queue.clone(),
                data.clone(),
                manager,
                guild_id,
            )
            .await
            {
                tracing::error!(%error, guild = %guild_id, "failed to start track from control panel");
                queue.write().await.current = None;
                let manager = songbird::get(ctx).await;
                if let Some(manager) = manager {
                    playback::schedule_idle_disconnect(
                        queue.clone(),
                        manager,
                        guild_id,
                        data.clone(),
                    )
                    .await;
                }
            }
        } else {
            if custom_id == "music:previous" {
                let _ = interaction
                    .create_response(
                        ctx,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("В истории нет предыдущего трека.")
                                .ephemeral(true),
                        ),
                    )
                    .await;
                return;
            }
            queue.write().await.current = None;
            playback::schedule_idle_disconnect(queue.clone(), manager, guild_id, data.clone())
                .await;
        }
    }
    if custom_id == "music:stop" {
        if let Some(manager) = songbird::get(ctx).await {
            playback::schedule_idle_disconnect(queue.clone(), manager, guild_id, data.clone())
                .await;
        }
    }
    let _ = interaction
        .create_response(ctx, CreateInteractionResponse::Acknowledge)
        .await;
    sync_panel(&ctx.http, data, guild_id, interaction.channel_id).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_latest_panel_message_is_current() {
        let channel = serenity::ChannelId::new(10);
        let current = serenity::MessageId::new(20);
        assert!(is_current_panel(Some((channel, current)), channel, current));
        assert!(!is_current_panel(
            Some((channel, current)),
            channel,
            serenity::MessageId::new(21)
        ));
        assert!(!is_current_panel(None, channel, current));
    }
}
