use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BotError {
    #[error("You must be in a server to use this command")]
    NotInGuild,

    #[error("Bot is not connected to a voice channel")]
    VoiceNotConnected,

    #[error("You must be in a voice channel to use this command")]
    UserNotInVoice,

    #[error("Failed to join voice channel: {0}")]
    VoiceJoin(String),

    #[error("Songbird voice engine is not initialized")]
    SongbirdNotInitialized,

    #[error("The queue is empty")]
    QueueEmpty,

    #[error("Invalid queue index: {0}")]
    InvalidIndex(usize),

    #[error("Invalid timestamp: '{0}'. Expected mm:ss or seconds (e.g. 1:30 or 90)")]
    InvalidTimestamp(String),

    #[error("Audio file not found at path: {0}")]
    FileNotFound(PathBuf),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Serenity error: {0}")]
    Serenity(#[from] Box<poise::serenity_prelude::Error>),

    #[error("Failed to extract metadata: {0}")]
    MetadataExtraction(String),
}

impl From<poise::serenity_prelude::Error> for BotError {
    fn from(err: poise::serenity_prelude::Error) -> Self {
        Self::Serenity(Box::new(err))
    }
}
