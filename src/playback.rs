use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use poise::serenity_prelude as serenity;
use songbird::{
    events::{Event, EventContext, EventHandler as SongbirdEventHandler, TrackEvent},
    Songbird,
};
use tokio::sync::{Mutex, RwLock};

use crate::{
    audio::{source::AudioSource, ytdl},
    queue::{
        manager::{GuildQueue, LoopMode},
        track::QueuedTrack,
    },
    types::Data,
};

pub const IDLE_TIMEOUT: Duration = Duration::from_secs(10 * 60);
pub type CallHandle = Arc<Mutex<songbird::Call>>;

pub async fn play_track(
    handler: &CallHandle,
    track: &QueuedTrack,
    queue: Arc<RwLock<GuildQueue>>,
    data: Data,
    manager: Arc<Songbird>,
    guild_id: serenity::GuildId,
) -> Result<(), crate::types::Error> {
    let input: songbird::input::Input = match &track.source {
        AudioSource::Youtube { url } | AudioSource::Playlist { url } => {
            ytdl::build_source(url.clone(), data.http_client.clone()).into()
        }
        AudioSource::LocalFile { path } => songbird::input::File::new(path.clone()).into(),
    };
    let handle = handler.lock().await.play_input(input);
    let generation = {
        let mut state = queue.write().await;
        state.playback_generation = state.playback_generation.wrapping_add(1);
        state.idle_generation = state.idle_generation.wrapping_add(1);
        state.idle_waiting = false;
        state.paused = false;
        let _ = handle.set_volume(state.volume.as_f32());
        state.handle = Some(handle.clone());
        state.playback_generation
    };
    handle.add_event(
        Event::Track(TrackEvent::End),
        TrackFinished {
            queue,
            data: data.clone(),
            handler: Arc::clone(handler),
            manager,
            guild_id,
            generation,
        },
    )?;
    crate::panel::refresh_existing_panel(&data, guild_id).await;
    Ok(())
}

struct TrackFinished {
    queue: Arc<RwLock<GuildQueue>>,
    data: Data,
    handler: CallHandle,
    manager: Arc<Songbird>,
    guild_id: serenity::GuildId,
    generation: u64,
}

#[async_trait]
impl SongbirdEventHandler for TrackFinished {
    async fn act(&self, _: &EventContext<'_>) -> Option<Event> {
        let next = {
            let mut queue = self.queue.write().await;
            if queue.playback_generation != self.generation {
                return Some(Event::Cancel);
            }
            queue.handle = None;
            queue.paused = false;
            if let Some(current) = queue.current.clone() {
                if queue.history.len() == 50 {
                    queue.history.pop_front();
                }
                queue.history.push_back(current.clone());
                match queue.loop_mode {
                    LoopMode::Track => Some(current),
                    LoopMode::Queue => {
                        queue.current.take();
                        queue.tracks.push_back(current);
                        queue.tracks.pop_front()
                    }
                    LoopMode::Off => {
                        queue.current.take();
                        queue.tracks.pop_front()
                    }
                }
            } else {
                None
            }
        };

        if let Some(track) = next {
            self.queue.write().await.current = Some(track.clone());
            if let Err(error) = play_track(
                &self.handler,
                &track,
                Arc::clone(&self.queue),
                self.data.clone(),
                Arc::clone(&self.manager),
                self.guild_id,
            )
            .await
            {
                tracing::error!(%error, guild = %self.guild_id, "failed to start queued track");
                self.queue.write().await.current = None;
                schedule_idle_disconnect(
                    Arc::clone(&self.queue),
                    Arc::clone(&self.manager),
                    self.guild_id,
                    self.data.clone(),
                )
                .await;
            }
        } else {
            crate::panel::refresh_existing_panel(&self.data, self.guild_id).await;
            schedule_idle_disconnect(
                Arc::clone(&self.queue),
                Arc::clone(&self.manager),
                self.guild_id,
                self.data.clone(),
            )
            .await;
        }
        Some(Event::Cancel)
    }
}

pub async fn schedule_idle_disconnect(
    queue: Arc<RwLock<GuildQueue>>,
    manager: Arc<Songbird>,
    guild_id: serenity::GuildId,
    data: Data,
) {
    let generation = {
        let mut state = queue.write().await;
        if !state.is_empty() {
            return;
        }
        state.idle_generation = state.idle_generation.wrapping_add(1);
        state.idle_waiting = true;
        state.idle_generation
    };
    tokio::spawn(async move {
        tokio::time::sleep(IDLE_TIMEOUT).await;
        let disconnected = {
            let mut state = queue.write().await;
            if !idle_disconnect_allowed(&state, generation) {
                false
            } else {
                match manager.remove(guild_id).await {
                    Ok(()) => {
                        state.active_channel = None;
                        state.idle_waiting = false;
                        true
                    }
                    Err(error) => {
                        tracing::warn!(%error, guild = %guild_id, "idle voice disconnect failed");
                        false
                    }
                }
            }
        };
        if disconnected {
            let http = data.http.read().ok().and_then(|slot| slot.clone());
            if let Some(http) = http {
                crate::panel::disable_panel(&http, &data, guild_id).await;
            }
        }
    });
}

fn idle_disconnect_allowed(queue: &GuildQueue, generation: u64) -> bool {
    queue.idle_generation == generation && queue.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn added_track_invalidates_old_idle_timeout() {
        let mut queue = GuildQueue::new();
        queue.idle_generation = 9;
        queue.add_track(QueuedTrack::new(
            "test".into(),
            None,
            serenity::UserId::new(1),
            AudioSource::Youtube {
                url: "https://example.com".into(),
            },
        ));
        assert!(!idle_disconnect_allowed(&queue, 9));
    }

    #[test]
    fn stale_idle_timeout_does_not_disconnect_a_nonempty_queue() {
        let mut queue = GuildQueue::new();
        queue.add_track(QueuedTrack::new(
            "test".into(),
            None,
            serenity::UserId::new(1),
            AudioSource::Youtube {
                url: "https://example.com".into(),
            },
        ));
        assert!(!idle_disconnect_allowed(&queue, queue.idle_generation));
    }
}
