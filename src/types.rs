use std::sync::Arc;

use dashmap::DashMap;
use poise::serenity_prelude as serenity;
use tokio::sync::RwLock;

use crate::queue::manager::GuildQueue;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Debug, Default)]
pub struct Data {
    pub guild_queues: Arc<DashMap<serenity::GuildId, Arc<RwLock<GuildQueue>>>>,
    pub voice_states:
        Arc<DashMap<serenity::GuildId, DashMap<serenity::UserId, serenity::ChannelId>>>,
    pub http_client: reqwest::Client,
}

impl Data {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_queue(&self, guild_id: serenity::GuildId) -> Arc<RwLock<GuildQueue>> {
        Arc::clone(
            &self
                .guild_queues
                .entry(guild_id)
                .or_insert_with(|| Arc::new(RwLock::new(GuildQueue::new()))),
        )
    }

    pub fn set_user_voice(
        &self,
        guild_id: serenity::GuildId,
        user_id: serenity::UserId,
        channel_id: serenity::ChannelId,
    ) {
        self.voice_states
            .entry(guild_id)
            .or_default()
            .insert(user_id, channel_id);
    }

    pub fn remove_user_voice(&self, guild_id: serenity::GuildId, user_id: serenity::UserId) {
        if let Some(guild_map) = self.voice_states.get(&guild_id) {
            guild_map.remove(&user_id);
        }
    }

    pub fn get_user_voice(
        &self,
        guild_id: serenity::GuildId,
        user_id: serenity::UserId,
    ) -> Option<serenity::ChannelId> {
        self.voice_states
            .get(&guild_id)
            .and_then(|guild_map| guild_map.get(&user_id).map(|r| *r))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_voice_state_tracking() {
        let data = Data::new();
        let guild_id = serenity::GuildId::new(100);
        let user_id = serenity::UserId::new(200);
        let channel_id = serenity::ChannelId::new(300);

        assert_eq!(data.get_user_voice(guild_id, user_id), None);

        data.set_user_voice(guild_id, user_id, channel_id);
        assert_eq!(data.get_user_voice(guild_id, user_id), Some(channel_id));

        data.remove_user_voice(guild_id, user_id);
        assert_eq!(data.get_user_voice(guild_id, user_id), None);
    }

    #[test]
    fn test_data_get_queue_creates_default() {
        let data = Data::new();
        let guild_id = serenity::GuildId::new(100);

        let queue_arc = data.get_queue(guild_id);
        let same_arc = data.get_queue(guild_id);

        assert!(Arc::ptr_eq(&queue_arc, &same_arc));
    }
}
