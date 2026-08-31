#![allow(dead_code)]

mod audio;
mod commands;
mod error;
mod queue;
mod types;

use poise::serenity_prelude as serenity;
use songbird::SerenityInit;
use tracing_subscriber::EnvFilter;

use types::Data;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set in .env");

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
                        serenity::FullEvent::VoiceStateUpdate { old: _, new } => {
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
                            }
                        }
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
                    let _ = poise::builtins::register_in_guild(ctx, &framework.options().commands, guild.id).await;
                }
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data::new())
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(&token, intents)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Failed to create client");

    if let Err(e) = client.start().await {
        tracing::error!("Client error: {e}");
    }
}
