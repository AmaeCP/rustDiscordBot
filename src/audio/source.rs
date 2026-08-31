use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum AudioSource {
    Youtube { url: String },
    Playlist { url: String },
    LocalFile { path: PathBuf },
}

impl AudioSource {
    pub fn from_input(input: &str) -> Self {
        let input = input.trim();

        if std::path::Path::new(input).exists() {
            return Self::LocalFile {
                path: PathBuf::from(input),
            };
        }

        if input.contains("watch?v=")
            && (input.contains("&list=RD")
                || input.contains("&list=LL")
                || input.contains("&list=WL")
                || input.contains("&start_radio="))
        {
            let cleaned = if let Some((base, _)) = input.split_once("&list=") {
                base.to_string()
            } else if let Some((base, _)) = input.split_once("&start_radio=") {
                base.to_string()
            } else {
                input.to_string()
            };
            return Self::Youtube { url: cleaned };
        }

        if input.contains("youtube.com/playlist") {
            return Self::Playlist {
                url: input.to_string(),
            };
        }

        if input.contains("&list=") && !input.contains("watch?v=") {
            return Self::Playlist {
                url: input.to_string(),
            };
        }

        if input.starts_with("http://") || input.starts_with("https://") {
            return Self::Youtube {
                url: input.to_string(),
            };
        }

        Self::Youtube {
            url: format!("ytsearch1:{input}"),
        }
    }

    pub fn url(&self) -> &str {
        match self {
            Self::Youtube { url } | Self::Playlist { url } => url,
            Self::LocalFile { path } => path.to_str().unwrap_or(""),
        }
    }
}
