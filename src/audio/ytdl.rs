use songbird::input::YoutubeDl;
use std::time::Duration;

use serde::Deserialize;

use crate::error::BotError;

#[derive(Debug, Deserialize)]
pub struct VideoMetadata {
    pub title: Option<String>,
    pub duration: Option<f64>,
    pub webpage_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistMetadata {
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
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(BotError::MetadataExtraction(format!(
            "yt-dlp failed: {stderr}"
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().next().unwrap_or(&stdout);
    let metadata: VideoMetadata = serde_json::from_str(first_line)?;

    Ok(metadata)
}

pub async fn extract_playlist_entries(url: &str) -> Result<Vec<PlaylistEntry>, BotError> {
    let output = tokio::process::Command::new("yt-dlp")
        .args(["--flat-playlist", "-J", "--no-warnings", url])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(BotError::MetadataExtraction(format!(
            "yt-dlp playlist failed: {stderr}"
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let playlist: PlaylistMetadata = serde_json::from_str(&stdout)?;

    Ok(playlist.entries.unwrap_or_default())
}

pub fn build_source(url: String, http_client: reqwest::Client) -> YoutubeDl<'static> {
    YoutubeDl::new(http_client, url)
}

pub fn format_duration(duration: Option<f64>) -> String {
    let Some(secs) = duration.filter(|secs| secs.is_finite() && *secs >= 0.0) else {
        return "??:??".to_string();
    };
    let Ok(duration) = Duration::try_from_secs_f64(secs) else {
        return "??:??".to_string();
    };

    let total_secs = duration.as_secs();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_none() {
        assert_eq!(format_duration(None), "??:??");
    }

    #[test]
    fn test_format_duration_seconds_only() {
        assert_eq!(format_duration(Some(45.0)), "0:45");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(Some(215.0)), "3:35");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration(Some(3725.0)), "1:02:05");
    }

    #[test]
    fn test_format_duration_rejects_invalid_values() {
        assert_eq!(format_duration(Some(-1.0)), "??:??");
        assert_eq!(format_duration(Some(f64::NAN)), "??:??");
        assert_eq!(format_duration(Some(f64::INFINITY)), "??:??");
    }
}
