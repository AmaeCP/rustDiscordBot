use std::sync::Arc;

use dashmap::DashMap;
use poise::serenity_prelude as serenity;
use tokio::sync::RwLock;

use crate::queue::manager::GuildQueue;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    pub guild_queues: Arc<DashMap<serenity::GuildId, Arc<RwLock<GuildQueue>>>>,
    pub voice_states: Arc<DashMap<serenity::GuildId, DashMap<serenity::UserId, serenity::ChannelId>>>,
    pub http_client: reqwest::Client,
}

impl Data {
    pub fn new() -> Self {
        Self {
            guild_queues: Arc::new(DashMap::new()),
            voice_states: Arc::new(DashMap::new()),
            http_client: reqwest::Client::new(),
        }
    }

    pub fn get_queue(&self, guild_id: serenity::GuildId) -> Arc<RwLock<GuildQueue>> {
        self.guild_queues
            .entry(guild_id)
            .or_insert_with(|| Arc::new(RwLock::new(GuildQueue::new())))
            .clone()
    }

    pub fn set_user_voice(&self, guild_id: serenity::GuildId, user_id: serenity::UserId, channel_id: serenity::ChannelId) {
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

    pub fn get_user_voice(&self, guild_id: serenity::GuildId, user_id: serenity::UserId) -> Option<serenity::ChannelId> {
        self.voice_states
            .get(&guild_id)
            .and_then(|guild_map| guild_map.get(&user_id).map(|ch| *ch))
    }
}
