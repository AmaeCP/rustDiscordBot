use thiserror::Error;

#[derive(Error, Debug)]
pub enum BotError {
    #[error("Failed to join voice channel: {0}")]
    VoiceJoin(String),

    #[error("Bot is not connected to a voice channel")]
    VoiceNotConnected,

    #[error("You must be in a voice channel to use this command")]
    UserNotInVoice,

    #[error("The queue is empty")]
    QueueEmpty,

    #[error("Invalid queue index: {0}")]
    InvalidIndex(usize),

    #[error("Audio source error: {0}")]
    AudioSource(String),

    #[error("Serenity error: {0}")]
    Serenity(#[from] poise::serenity_prelude::Error),

    #[error("Failed to extract metadata: {0}")]
    MetadataExtraction(String),
}
