use songbird::input::YoutubeDl;
use std::time::Duration;

use serde::Deserialize;

use crate::error::BotError;

#[derive(Debug, Deserialize)]
pub struct VideoMetadata {
    pub title: Option<String>,
    pub duration: Option<f64>,
    pub webpage_url: Option<String>,
    pub uploader: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistMetadata {
    pub title: Option<String>,
    pub entries: Option<Vec<PlaylistEntry>>,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistEntry {
    pub url: Option<String>,
    pub title: Option<String>,
    pub duration: Option<f64>,
}

pub async fn extract_metadata(url: &str) -> Result<VideoMetadata, BotError> {
    let output = tokio::process::Command::new("yt-dlp")
        .args([
            "--no-playlist",
            "--no-download",
            "--print-json",
            "--skip-download",
            "--no-warnings",
            "-f",
            "bestaudio/best",
            url,
        ])
        .output()
        .await
        .map_err(|e| BotError::MetadataExtraction(format!("Failed to run yt-dlp: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(BotError::MetadataExtraction(format!(
            "yt-dlp failed: {stderr}"
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().next().unwrap_or(&stdout);
    let metadata: VideoMetadata = serde_json::from_str(first_line).map_err(|e| {
        BotError::MetadataExtraction(format!("Failed to parse yt-dlp output: {e}"))
    })?;

    Ok(metadata)
}

pub async fn extract_playlist_entries(url: &str) -> Result<Vec<PlaylistEntry>, BotError> {
    let output = tokio::process::Command::new("yt-dlp")
        .args(["--flat-playlist", "-J", "--no-warnings", url])
        .output()
        .await
        .map_err(|e| BotError::MetadataExtraction(format!("Failed to run yt-dlp: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(BotError::MetadataExtraction(format!(
            "yt-dlp playlist failed: {stderr}"
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let playlist: PlaylistMetadata = serde_json::from_str(&stdout).map_err(|e| {
        BotError::MetadataExtraction(format!("Failed to parse playlist output: {e}"))
    })?;

    Ok(playlist.entries.unwrap_or_default())
}

pub fn build_source(url: String, http_client: reqwest::Client) -> YoutubeDl<'static> {
    YoutubeDl::new(http_client, url)
}

pub fn format_duration(duration: Option<f64>) -> String {
    match duration {
        Some(secs) => {
            let d = Duration::from_secs_f64(secs);
            let total_secs = d.as_secs();
            let mins = total_secs / 60;
            let secs = total_secs % 60;
            if mins >= 60 {
                let hours = mins / 60;
                let mins = mins % 60;
                format!("{hours}:{mins:02}:{secs:02}")
            } else {
                format!("{mins}:{secs:02}")
            }
        }
        None => "??:??".to_string(),
    }
}
