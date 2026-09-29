mod audio;
mod commands;
mod error;
mod panel;
mod playback;
mod queue;
mod types;

use anyhow::Context;
use poise::serenity_prelude as serenity;
use songbird::SerenityInit;
use tracing_subscriber::EnvFilter;

use types::Data;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let token = std::env::var("DISCORD_TOKEN").context("DISCORD_TOKEN must be set in .env")?;

    let intents = serenity::GatewayIntents::all();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::all_commands(),
            event_handler: |_ctx, event, _framework, user_data| {
                Box::pin(async move {
                    tracing::info!("Gateway event: {:?}", event.snake_case_name());
                    match event {
                        serenity::FullEvent::GuildCreate { guild, .. } => {
                            tracing::info!(
                                "📥 GuildCreate: {} ({}) [voice states: {}]",
                                guild.name,
                                guild.id,
                                guild.voice_states.len()
                            );
                            for (user_id, vs) in &guild.voice_states {
                                if let Some(channel_id) = vs.channel_id {
                                    user_data.set_user_voice(guild.id, *user_id, channel_id);
                                }
                            }
                        }
                        serenity::FullEvent::VoiceStateUpdate { old, new } => {
                            if let Some(guild_id) = new.guild_id {
                                if let Some(channel_id) = new.channel_id {
                                    tracing::info!(
                                        "🎤 User {} joined voice channel {} in guild {}",
                                        new.user_id,
                                        channel_id,
                                        guild_id
                                    );
                                    user_data.set_user_voice(guild_id, new.user_id, channel_id);
                                } else {
                                    tracing::info!(
                                        "🎤 User {} left voice in guild {}",
                                        new.user_id,
                                        guild_id
                                    );
                                    user_data.remove_user_voice(guild_id, new.user_id);
                                }

                                if new.user_id == _ctx.cache.current_user().id {
                                    if new.channel_id.is_none()
                                        && (old
                                            .as_ref()
                                            .and_then(|state| state.channel_id)
                                            .is_some()
                                            || user_data
                                                .get_queue(guild_id)
                                                .read()
                                                .await
                                                .active_channel
                                                .is_some())
                                    {
                                        if let Some(manager) = songbird::get(_ctx).await {
                                            if manager.get(guild_id).is_some() {
                                                let _ = manager.remove(guild_id).await;
                                            }
                                        }
                                        let queue = user_data.get_queue(guild_id);
                                        queue.write().await.reset_after_disconnect();
                                        crate::panel::disable_panel(
                                            &_ctx.http, user_data, guild_id,
                                        )
                                        .await;
                                    } else if let Some(channel_id) = new.channel_id {
                                        user_data
                                            .get_queue(guild_id)
                                            .write()
                                            .await
                                            .active_channel = Some(channel_id);
                                    }
                                }
                            }
                        }
                        serenity::FullEvent::InteractionCreate {
                            interaction: serenity::Interaction::Component(component),
                        } => crate::panel::handle_component(_ctx, user_data, component).await,
                        _ => {}
                    }
                    Ok(())
                })
            },
            on_error: |error| {
                Box::pin(async move {
                    match error {
                        poise::FrameworkError::Command { error, ctx, .. } => {
                            let msg = format!("❌ {error}");
                            let _ = ctx.say(&msg).await;
                            if let Some(guild_id) = ctx.guild_id() {
                                if let Some(manager) = songbird::get(ctx.serenity_context()).await {
                                    crate::playback::schedule_idle_disconnect(
                                        ctx.data().get_queue(guild_id),
                                        manager,
                                        guild_id,
                                        ctx.data().clone(),
                                    )
                                    .await;
                                }
                            }
                            tracing::error!("Command error: {error}");
                        }
                        other => {
                            if let Err(e) = poise::builtins::on_error(other).await {
                                tracing::error!("Fatal error handler error: {e}");
                            }
                        }
                    }
                })
            },
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                tracing::info!(
                    "🤖 {} is connected! Bot is present in {} server(s): {:?}",
                    ready.user.name,
                    ready.guilds.len(),
                    ready.guilds.iter().map(|g| g.id).collect::<Vec<_>>()
                );
                for guild in &ready.guilds {
                    let _ = poise::builtins::register_in_guild(
                        ctx,
                        &framework.options().commands,
                        guild.id,
                    )
                    .await;
                }
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                let data = Data::new();
                data.set_http(ctx.http.clone());
                Ok(data)
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(&token, intents)
        .framework(framework)
        .register_songbird()
        .await
        .context("Failed to create serenity client")?;

    client
        .start()
        .await
        .context("Client encountered a runtime error")?;
    Ok(())
}
