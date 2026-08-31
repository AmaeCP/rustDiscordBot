use poise::serenity_prelude as serenity;

use crate::audio::source::AudioSource;

#[derive(Debug, Clone)]
pub struct QueuedTrack {
    pub title: String,
    pub duration: Option<f64>,
    pub requester: serenity::UserId,
    pub source: AudioSource,
    pub url: String,
}

impl QueuedTrack {
    pub fn new(
        title: String,
        duration: Option<f64>,
        requester: serenity::UserId,
        source: AudioSource,
        url: String,
    ) -> Self {
        Self {
            title,
            duration,
            requester,
            source,
            url,
        }
    }

    pub fn duration_string(&self) -> String {
        crate::audio::ytdl::format_duration(self.duration)
    }
}
